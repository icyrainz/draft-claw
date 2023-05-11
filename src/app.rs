use std::{collections::HashMap, time};

use indicium::simple::SearchIndex;

use crate::app_context::*;
use crate::models::draft_data::{DraftPick, DraftRecord};

use super::*;
use crate::models::card::*;

use terminal_menu::{back_button, button, label, menu, scroll};

mod card_matcher;
mod ocr_engine;
mod screen;

const GAME_ID_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const GAME_ID_LENGTH: usize = 8;
const CURRENT_GAME_ID_KEY: &str = "current_game_id";

const ACTION_SELECT_CARD: &str = "Select";

const LABEL_NEW_GAME: &str = "New Game";
const LABEL_EXISTING_GAME: &str = "Existing Game";

const LABEL_CONFIRM: &str = "Confirm";
const LABEL_EXIT: &str = "Exit";

const LABEL_SELECT_CARD: &str = "Select Card";

const APP_NAME: &str = "Draft Claw";

pub async fn main(context: &AppContext) {
    let runtime_data = initialize_runtime_data();

    let game_menu = menu(vec![
        label(
            std::iter::repeat('-')
                .take(APP_NAME.len())
                .collect::<String>(),
        ),
        label(APP_NAME),
        label(
            std::iter::repeat('-')
                .take(APP_NAME.len())
                .collect::<String>(),
        ),
        button(LABEL_NEW_GAME),
        button(LABEL_EXISTING_GAME),
        back_button(LABEL_EXIT),
    ]);

    let mut game_id = context.read_data(CURRENT_GAME_ID_KEY).unwrap_or_default();
    terminal_menu::run(&game_menu);

    loop {
        {
            let menu_selection = terminal_menu::mut_menu(&game_menu);
            match menu_selection.selected_item_name() {
                LABEL_NEW_GAME => {
                    game_id = create_new_game(context);
                }
                LABEL_EXISTING_GAME => {
                    if game_id.is_empty() {
                        println!("No existing game found!");
                        continue;
                    }
                }
                _ => {
                    println!("Exiting...");
                    break;
                }
            }
        }
        println!("Game ID: {}", game_id);

        match db_access::get_draft_game(&game_id).await {
            // Insert if current game does not exist in the db
            Ok(result) if result.is_none() => {
                db_access::insert_draft_game(&game_id).await.unwrap();
            }
            Ok(result) => {
                println!(
                    "Game [{}] already exists! It belongs to [{}].",
                    game_id,
                    result
                        .unwrap()
                        .user_id
                        .unwrap_or("unregistered".to_string())
                );
            }
            Err(e) => {
                println!("Unable to get draft game: {}", e);
            }
        }

        let mut card_selections: Vec<String> = Vec::new();

        match capture_draft_record(&runtime_data, &game_id) {
            Ok(record) => {
                // insert draft record into db
                db_access::upsert_draft_record(&record)
                    .await
                    .unwrap_or_else(|err| {
                        println!("Unable to insert draft record: {}", err);
                    });

                card_selections = record.selection_vec.clone();
            }
            Err(e) => {
                println!("Unable to capture draft record: {}", e);
            }
        }

        let select_card_menu = menu(vec![
            label(LABEL_SELECT_CARD),
            label(
                std::iter::repeat('-')
                    .take(LABEL_SELECT_CARD.len())
                    .collect::<String>(),
            ),
            scroll(
                ACTION_SELECT_CARD,
                card_selections
                    .iter()
                    .enumerate()
                    .filter_map(|(i, card)| {
                        if card.is_empty() {
                            None
                        } else {
                            Some(format!("{}: {}", i + 1, card))
                        }
                    })
                    .collect::<Vec<String>>(),
            ),
            button(LABEL_CONFIRM),
        ]);
        terminal_menu::run(&select_card_menu);

        let input: String;
        {
            let select_card_menu_instance = terminal_menu::mut_menu(&select_card_menu);

            if select_card_menu_instance.canceled() {
                return;
            }
            input = select_card_menu_instance
                .selection_value(ACTION_SELECT_CARD)
                .split(":")
                .next()
                .unwrap_or("invalid")
                .to_string();
            dbg!(&input);
        }

        match input.as_str() {
            other if other.parse::<u8>().is_ok() => {
                let card_index = input.parse::<u8>().unwrap();
                screen::select_card(card_index - 1).expect("unable to select card");
            }
            _ => {
                println!("continue");
            }
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

    RuntimeData {
        card_map,
        card_index,
        card_ratings,
    }
}

pub struct ScreenMatchedData {
    pub pick_num: u8,
    pub selection_text: String,
    pub selection_vec: Vec<String>,
    pub deck: Vec<String>,
}

pub fn get_draft_selection_text(data: &RuntimeData) -> Result<ScreenMatchedData, String> {
    let screen_data = screen::capture_raw_text_on_screen()?;

    let card_texts = screen_data
        .cards
        .iter()
        .map(|card| card.as_str())
        .collect::<Vec<&str>>();
    let matched_card_names = card_matcher::find_card_name_matches(&data.card_index, &card_texts);

    let pick_numer = screen_data
        .pick_num
        .split_whitespace()
        .nth(1)
        .ok_or("unable to capture pick number")?
        .parse::<u8>()
        .map_err(|err| err.to_string())?;

    let matched_cards = matched_card_names
        .iter()
        .filter_map(|name| data.card_map.get(name))
        .cloned()
        .collect::<Vec<Card>>();

    println!("Found {} cards on screen.", matched_cards.len());

    let mut draft_selection_text = String::new();
    let mut draft_selection_vec = Vec::new();
    // draft_selection_text.push_str(format!("Card {} of 48\n", pick_numer).as_str());
    for (idx, card) in matched_cards.iter().enumerate() {
        let card_text = format!(
            "{:<2} [{:<2}] {}",
            idx + 1,
            data.card_ratings
                .get(&card.name)
                .unwrap_or(&"NA".to_string()),
            &card.to_text(),
        ) + &"\n";
        draft_selection_text.push_str(&card_text);
        draft_selection_vec.push(card.name.to_string());
    }

    let mut deck = Vec::new();
    let matched_deck_names = card_matcher::find_card_name_matches(
        &data.card_index,
        &screen_data
            .deck
            .iter()
            .map(|item| item.0.as_str())
            .collect::<Vec<&str>>(),
    );

    let matched_deck_cards = matched_deck_names
        .iter()
        .filter_map(|name| data.card_map.get(name))
        .cloned()
        .collect::<Vec<Card>>();
    let matched_deck_cards_with_count =
        matched_deck_cards
            .iter()
            .zip(screen_data.deck.iter().map(|item| {
                item.1
                    .chars()
                    .filter(|c| c.is_digit(10))
                    .collect::<String>()
            }));
    for card_row in matched_deck_cards_with_count {
        let card_text = format!(
            "{}{} {}",
            card_row.0.cost, card_row.0.influence, card_row.0.name
        );
        let deck_text = format!("{}x {:30}", card_row.1, card_text);
        deck.push(deck_text);
    }
    dbg!(&deck);

    Ok(ScreenMatchedData {
        pick_num: pick_numer,
        selection_text: draft_selection_text,
        selection_vec: draft_selection_vec,
        deck,
    })
}

pub fn capture_draft_record(
    runtime_data: &RuntimeData,
    game_id: &str,
) -> Result<DraftRecord, String> {
    println!("Capturing draft record...");

    let screen_matched_data = get_draft_selection_text(&runtime_data)?;

    println!("Pick number: {}", screen_matched_data.pick_num);
    println!(
        "Draft selection text:\n{}",
        screen_matched_data.selection_text
    );

    if screen_matched_data.selection_text.is_empty() {
        return Err(format!(
            "Unable to capture draft record for pick {}",
            screen_matched_data.pick_num
        ));
    }

    let mut draft_record = DraftRecord::new(
        game_id.to_string(),
        DraftPick::new(screen_matched_data.pick_num),
    );
    draft_record.set_selection_text(&screen_matched_data.selection_text);
    draft_record.set_selection_vec(
        &screen_matched_data
            .selection_vec
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
    );
    draft_record.set_decklist_text(
        &screen_matched_data
            .deck
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
    );
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
