use std::fs::{read, write};
use std::{collections::{HashSet, HashMap}, fs::File, time};
use std::io::{BufReader, BufRead};

use crate::context::*;

use super::*;
use crate::models::card::*;
use crate::models::card_rating::*;

mod card_matcher;
mod ocr_engine;
mod screen;
mod card_loader;

const GAME_ID_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const GAME_ID_LENGTH: usize = 8;
const CURRENT_GAME_ID_KEY: &str = "current_game_id";

pub async fn main(context: &Context) {
    let mut game_id = context.read_data(CURRENT_GAME_ID_KEY).unwrap_or_default();

    if game_id.is_empty() {
        game_id = create_new_game(context);
    }

    println!("Game ID: {}", game_id);
}

fn create_new_game(context: &Context) -> String {
    let game_id = nanoid::nanoid!(
        GAME_ID_LENGTH,
        &GAME_ID_ALPHABET.to_string().chars().collect::<Vec<char>>()
    );
    context.write_data(CURRENT_GAME_ID_KEY, &game_id);

    game_id
}

pub async fn upload_card_rating() {
    let card_ratings = get_card_rating_list();

    db_access::insert_card_rating(&card_ratings).await.unwrap();
}

pub fn get_rating_card_onscreen(card_names: &[String]) -> Vec<String> {
    let cards_on_screen = screen::capture_cards_on_screen();
    card_matcher::find_matches(&cards_on_screen, card_names)
}

pub fn get_draft_selection_text() -> String {
    let cards = card_loader::load_card_data();

    let card_map = cards.iter().fold(HashMap::new(), |mut acc, card| {
        acc.insert(card.name.clone(), card);
        acc
    });

    let card_ratings = load_card_rating();
    let draft_card_names = card_ratings.keys().cloned().collect::<Vec<String>>();

    let matched_card_names = get_rating_card_onscreen(&draft_card_names);
    let mut matched_cards = cards
        .iter()
        .filter(|card| matched_card_names.contains(&card.name))
        .collect::<Vec<_>>();

    matched_cards.sort_unstable_by(|a, b| match a.rarity.cmp(&b.rarity) {
        std::cmp::Ordering::Equal => a.name.cmp(&b.name),
        other => other,
    });

    let mut draft_selection_text = String::new();
    for card in matched_cards.iter() {
        let card_text = format!(
            "[{}] {:30}: {}",
            card.rarity,
            card.name,
            card_ratings.get(&card.name).unwrap()
        );
        draft_selection_text.push_str(&card_text);
    }

    dbg!(&draft_selection_text);
    draft_selection_text
}

pub fn load_card_hashmap_by_name() -> HashMap<String, Card> {
    let cards = card_loader::load_card_data();
    let mut card_hashmap = HashMap::new();

    for card in cards {
        card_hashmap.insert(card.name.clone(), card);
    }

    card_hashmap
}

