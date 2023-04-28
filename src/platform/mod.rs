use dotenv::dotenv;
use std::collections::HashMap;

mod card;
mod screen;
mod ocr_engine;
mod db_access;
mod card_rating;
mod card_matcher;
mod discord_bot;

pub fn run() {
    dotenv().ok();

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

    let matched_card_names = card_matcher::get_rating_card_onscreen(&draft_card_names);
    let mut matched_cards = cards.iter().filter(|card| {
        matched_card_names.contains(&card.name)
    }).collect::<Vec<_>>();

    matched_cards.sort_unstable_by(|a, b| {
        a.rarity.cmp(&b.rarity)
    });

    for card in matched_cards.iter() {
        println!("[{}] {:30}: {}", card.rarity, card.name, card_ratings.get(&card.name).unwrap());
    }

    // discord_bot::run();
}

