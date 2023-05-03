use std::borrow::{BorrowMut, Borrow};
use std::fs::{read, write};
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    time,
};

use indicium::simple::SearchIndex;

use crate::app_context::*;
use crate::models::draft_data::{DraftPick, DraftRecord};
use crate::models::draft_game::DraftGame;

use super::*;
use crate::models::card::*;
use crate::models::card_rating::*;

mod card_loader;
mod card_matcher;
mod ocr_engine;
mod screen;

const GAME_ID_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const GAME_ID_LENGTH: usize = 8;
const CURRENT_GAME_ID_KEY: &str = "current_game_id";

pub async fn main(context: &AppContext) {
    let mut game_id = context.read_data(CURRENT_GAME_ID_KEY).unwrap_or_default();

    if game_id.is_empty() {
        game_id = create_new_game(context);
    }

    println!("Game ID: {}", game_id);

    loop {
        match db_access::get_draft_game(&game_id).await {
            // Insert if current game does not exist in the db
            Ok(result) if result.is_none() => {
                db_access::insert_draft_game(&game_id).await.unwrap();
            }
            Ok(result) => {
                println!(
                    "Game [{}] already exists! It belongs to [{}].",
                    game_id,
                    result.unwrap().user_id.unwrap_or("unregistered".to_string())
                );
            }
            Err(e) => {
                println!("Unable to get draft game: {}", e);
            }
        }

        if let Ok(current_draft_record) = capture_draft_record(&game_id) {
            // insert draft record into db
            db_access::upsert_draft_record(&vec![current_draft_record]).await.unwrap();
        } else {
            println!("Unable to capture draft record.");
        }

        tokio::time::sleep(time::Duration::from_secs(1)).await;
    }
}

fn create_new_game(context: &AppContext) -> String {
    let game_id = nanoid::nanoid!(
        GAME_ID_LENGTH,
        &GAME_ID_ALPHABET.to_string().chars().collect::<Vec<char>>()
    );
    context.write_data(CURRENT_GAME_ID_KEY, &game_id);

    game_id
}

pub async fn upload_card_rating() {
    let card_ratings = card_loader::get_card_rating_list();

    db_access::insert_card_rating(&card_ratings).await.unwrap();
}

pub struct RuntimeData {
    card_map: HashMap<String, Card>,
    card_index: SearchIndex<String>,
    card_ratings: HashMap<String, String>,
}

fn initialize_runtime_data() -> RuntimeData {
    let cards = card_loader::load_card_data();

    let card_index = cards.iter().fold(SearchIndex::default(), |mut acc, card| {
        acc.insert(&card.name, card);
        acc
    });

    let card_map = cards.iter().fold(HashMap::new(), |mut acc, card| {
        acc.insert(card.name.clone(), card.clone());
        acc
    });

    let card_ratings = card_loader::load_card_rating();
    let draft_card_names = card_ratings.keys().map(|c| c.as_str()).collect::<Vec<&str>>();

    RuntimeData {
        card_map,
        card_index,
        card_ratings,
    }
}

pub fn get_draft_selection_text(data: &RuntimeData) -> Result<(String, String), String> {

    let cards_on_screen = screen::capture_raw_text_on_screen()?;

    let card_texts = cards_on_screen.0.iter().map(|card| card.as_str()).collect::<Vec<&str>>();
    let matched_card_names = card_matcher::find_card_name_matches(&data.card_index, &card_texts);

    let pick_numer = cards_on_screen.1.split_whitespace().nth(1).ok_or("unable to parse pick number")?;

    let matched_cards = matched_card_names
        .iter()
        .filter_map(|name| 
            data.card_map.get(name))
        .cloned()
        .collect::<Vec<Card>>();

    dbg!(&matched_cards);
    println!("Found {} cards on screen.", matched_cards.len());

    let mut draft_selection_text = String::new();
    draft_selection_text.push_str(format!("Card {} of 48\n", pick_numer).as_str());
    for card in matched_cards.iter() {
        let rating = match data.card_ratings.get(&card.name) {
            Some(rating) => rating,
            None => "NA",
        };

        let card_text = format!(
            "[{}] [{:<2}] {:30}",
            card.rarity,
            rating,
            card.name,
        ) + &"\n";
        draft_selection_text.push_str(&card_text);
    }

    Ok((pick_numer.to_string(), draft_selection_text))
}

pub fn capture_draft_record(game_id: &str) -> Result<DraftRecord, String> {
    println!("Capturing draft record...");

    let data = initialize_runtime_data();

    let (pick_number, draft_selection_text) = get_draft_selection_text(&data)?;
    let pick_number = pick_number.parse::<u8>().map_err(|e| "unable to parse pick number".to_string())?;
    println!("Pick number: {}", pick_number);
    println!("Draft selection text:\n{}", draft_selection_text);

    if draft_selection_text.is_empty() {
        return Err(format!("Unable to capture draft record for pick {}", pick_number));
    }

    let mut draft_record = DraftRecord::new(game_id.to_string(), DraftPick::new(pick_number));
    draft_record.set_selection_text(draft_selection_text);
    Ok(draft_record)
}

pub fn load_card_hashmap_by_name() -> HashMap<String, Card> {
    let cards = card_loader::load_card_data();
    let mut card_hashmap = HashMap::new();

    for card in cards {
        card_hashmap.insert(card.name.clone(), card);
    }

    card_hashmap
}
