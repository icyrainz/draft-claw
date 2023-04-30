use std::{collections::HashMap, time};

pub mod card;
mod screen;
mod ocr_engine;
mod db_access;
mod card_rating;
mod card_matcher;
pub mod draft_data;

pub async fn run_loop() {
    // loop {
    //     get_draft_selection_text();
    //     std::thread::sleep(time::Duration::from_secs(5));
    // }
}

pub async fn upload_card_rating() {
    let card_ratings = card_rating::get_card_rating_list();

    db_access::insert_card_rating(&card_ratings).await.unwrap();
}

pub fn get_rating_card_onscreen(card_names: &[String]) -> Vec<String> {
    let cards_on_screen = screen::capture_cards_on_screen();
    card_matcher::find_matches(&cards_on_screen, card_names)
}

pub fn get_draft_selection_text() -> String {
    let cards = card::load_card_data();

    let card_map = cards.iter().fold(
        HashMap::new(),
        |mut acc, card| {
            acc.insert(card.name.clone(), card);
            acc
        }
    );

    let card_ratings = card_rating::load_card_rating();
    let draft_card_names = card_ratings.keys().cloned().collect::<Vec<String>>();

    let matched_card_names = get_rating_card_onscreen(&draft_card_names);
    let mut matched_cards = cards.iter().filter(|card| {
        matched_card_names.contains(&card.name)
    }).collect::<Vec<_>>();

    matched_cards.sort_unstable_by(|a, b| {
        match a.rarity.cmp(&b.rarity) {
            std::cmp::Ordering::Equal => a.name.cmp(&b.name),
            other => other,
        }
    });

    let mut draft_selection_text = String::new();
    for card in matched_cards.iter() {
        let card_text = format!("[{}] {:30}: {}", card.rarity, card.name, card_ratings.get(&card.name).unwrap());
        draft_selection_text.push_str(&card_text);
    }

    dbg!(&draft_selection_text);
    draft_selection_text
}

