use crate::opt::*;

use std::{collections::HashMap, time};

use indicium::simple::SearchIndex;

use crate::app_context::*;
use crate::models::draft_data::{DraftPick, DraftRecord};

use super::*;
use crate::models::card::*;

mod card_matcher;
mod ocr_engine;
mod screen;

const GAME_ID_ALPHABET: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const GAME_ID_LENGTH: usize = 8;
const CURRENT_GAME_ID_KEY: &str = "current_game_id";

const LABEL_ACTION_SELECT: &str = "Select";

const LABEL_NEW_GAME: &str = "New Game";
const LABEL_EXISTING_GAME: &str = "Existing Game";

const LABEL_CONFIRM_AUTO: &str = "Confirm Auto";
const LABEL_CONFIRM_MANUAL: &str = "Confirm Manual";
const LABEL_EXIT: &str = "Exit";
const LABEL_CONTINUE: &str = "Continue";
const LABEL_MATCH: &str = "Match";

const LABEL_MENU_SELECT_CARD: &str = "Select Card";
const LABEL_MENU_ACTION: &str = "Action";

const APP_NAME: &str = "Draft Claw";

fn log(s: String) {
    log_if(s.as_str(), DbgFlg::Capture);
}

pub async fn main(context: &AppContext) {
    let runtime_data = initialize_runtime_data();

    let game_menu = terminal_menu::menu(vec![
        terminal_menu::label(
            std::iter::repeat('-')
                .take(APP_NAME.len())
                .collect::<String>(),
        ),
        terminal_menu::label(APP_NAME),
        terminal_menu::label(
            std::iter::repeat('-')
                .take(APP_NAME.len())
                .collect::<String>(),
        ),
        terminal_menu::button(LABEL_EXISTING_GAME),
        terminal_menu::button(LABEL_NEW_GAME),
        terminal_menu::back_button(LABEL_EXIT),
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
                        log("No existing game found!".to_string());
                        continue;
                    }
                }
                _ => {
                    log("Exiting...".to_string());
                    break;
                }
            }
        }
        log(format!("Game ID: {}", game_id));

        match db_access::get_draft_game(&game_id).await {
            // Insert if current game does not exist in the db
            Ok(result) if result.is_none() => {
                db_access::insert_draft_game(&game_id).await.unwrap();
            }
            Ok(result) => {
                log(format!(
                    "Game [{}] already exists! It belongs to [{}].",
                    game_id,
                    result
                        .unwrap()
                        .user_id
                        .unwrap_or("unregistered".to_string())
                ));
            }
            Err(e) => {
                log(format!("Unable to get draft game: {}", e));
                continue;
            }
        }

        // let action_menu = terminal_menu::menu(vec![
        //     terminal_menu::label(LABEL_MENU_ACTION),
        //     terminal_menu::label(
        //         std::iter::repeat('-')
        //             .take(LABEL_MENU_ACTION.len())
        //             .collect::<String>(),
        //     ),
        //     terminal_menu::button(LABEL_MATCH),
        //     terminal_menu::button(LABEL_CONTINUE),
        // ]);
        // terminal_menu::run(&action_menu);
        //
        // let input: String;
        // {
        //     let action_menu_instance = terminal_menu::mut_menu(&action_menu);
        //
        //     if action_menu_instance.canceled() {
        //         return;
        //     }
        //     match action_menu_instance.selected_item_name() {
        //         LABEL_CONTINUE => {
        //             continue;
        //         }
        //         LABEL_MATCH | _ => {}
        //     }
        // }

        let draft_record: DraftRecord;

        match capture_draft_record(&runtime_data, &game_id) {
            Ok(mut record) => {
                match db_access::get_draft_record(&game_id, &record.pick).await {
                    Ok(Some(record_in_db)) => {
                        log(format!(
                            "Record of {} in game {} exists in db",
                            record.pick.to_string(),
                            game_id
                        ));

                        if record_in_db.selected_card.is_some() {
                            record.selected_card = record_in_db.selected_card;
                        }

                        dbg!(&record);
                    }
                    _ => {
                        log(format!("Unable to get existing draft record. Overwriting with new captured data."));
                    }
                }

                // insert draft record into db
                db_access::upsert_draft_record(&record)
                    .await
                    .unwrap_or_else(|err| {
                        log(format!("Unable to insert draft record: {}", err));
                    });

                draft_record = record.clone();
            }
            Err(e) => {
                log(format!("Unable to capture draft record: {}", e));
                continue;
            }
        }

        let auto_selected_card = match draft_record.selected_card {
            Some(idx) => draft_record.selection_vec[idx as usize].to_owned(),
            None => "No voted card".to_string(),
        };

        let select_card_menu = terminal_menu::menu(vec![
            terminal_menu::label(LABEL_MENU_SELECT_CARD),
            terminal_menu::label(
                std::iter::repeat('-')
                    .take(LABEL_MENU_SELECT_CARD.len())
                    .collect::<String>(),
            ),
            terminal_menu::label(format!("Commited card: {}", auto_selected_card)),
            terminal_menu::button(LABEL_CONFIRM_AUTO),
            terminal_menu::scroll(
                LABEL_ACTION_SELECT,
                draft_record
                    .selection_vec
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
            terminal_menu::button(LABEL_CONFIRM_MANUAL),
            terminal_menu::button(LABEL_CONTINUE),
        ]);
        terminal_menu::run(&select_card_menu);

        let input: String;
        {
            let select_card_menu_instance = terminal_menu::mut_menu(&select_card_menu);

            if select_card_menu_instance.canceled() {
                return;
            }
            match select_card_menu_instance.selected_item_name() {
                LABEL_CONFIRM_MANUAL => {
                    input = select_card_menu_instance
                        .selection_value(LABEL_ACTION_SELECT)
                        .split(":")
                        .next()
                        .unwrap_or("invalid")
                        .to_string();
                    log(format!("Manually selected: {}", input));
                }
                LABEL_CONFIRM_AUTO => {
                    input = draft_record
                        .selected_card
                        .map_or(String::new(), |item| item.to_string());
                    log(format!("Auto selected: {}", input));
                }
                LABEL_CONTINUE => {
                    continue;
                }
                _ => {
                    log("Invalid menu option".to_string());
                    return;
                }
            }
        }

        match input.as_str() {
            other if other.parse::<u8>().is_ok() => {
                let card_index = input.parse::<u8>().unwrap() - 1;
                screen::select_card(card_index).expect("unable to select card");
            }
            _ => {
                log("continue".to_string());
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

    log(format!("Found {} cards on screen.", matched_cards.len()));

    let mut draft_selection_text = String::new();
    let mut draft_selection_vec = Vec::new();
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
    log("Capturing draft record...".to_string());

    let screen_matched_data = get_draft_selection_text(&runtime_data)?;

    log(format!("Pick number: {}", screen_matched_data.pick_num));
    log(format!(
        "Draft selection text:\n{}",
        screen_matched_data.selection_text,
    ));

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
