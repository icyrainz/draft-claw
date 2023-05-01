use std::fs::File;
use std::collections::{HashMap, HashSet};
use std::io::{BufReader, BufRead};

use crate::models::card::Card;
use crate::models::card_rating::CardRating;

const CARD_DATA_PATH: &str = "./resource/eternal-cards.json";

const CARD_RATING_PATH: &str = "./resource/card_rating.txt";
const CARD_RATING_FORMAT: &str = "14.0";

pub fn load_card_data() -> Vec<Card> {
    let card_data = std::fs::read_to_string(CARD_DATA_PATH).expect("failed to read card data");
    let cards: Vec<Card> = serde_json::from_str(&card_data).expect("failed to parse card data");

    cards
}

pub fn load_card_hashmap_by_name() -> HashMap<String, Card> {
    let cards = load_card_data();
    let mut card_hashmap = HashMap::new();

    for card in cards {
        card_hashmap.insert(card.name.clone(), card);
    }

    card_hashmap
}

pub fn load_card_rating() -> HashMap<String, String> {
    let mut card_ratings: HashMap<String, String> = HashMap::new();

    let file = File::open(CARD_RATING_PATH).unwrap();
    let reader = BufReader::new(file);

    let rating_remap: HashMap<&str, &str> =
        HashMap::from([("4 deliveries", "D+"), ("10 cylices", "D")]);

    reader.lines().skip(1).for_each(|line| {
        let line = line.unwrap();
        let mut iter = line.split("\t");
        let rating = iter.next().unwrap();

        while let Some(name) = iter.next() {
            if name.is_empty() {
                continue;
            }
            let rating = rating_remap.get(rating).unwrap_or(&rating);
            card_ratings.insert(name.to_string(), rating.to_string());
        }
    });

    let all_ratings = card_ratings.values().collect::<HashSet<_>>();

    card_ratings
}

pub fn get_card_rating_list() -> Vec<CardRating> {
    load_card_rating()
        .iter()
        .map(|(name, rating)| CardRating {
            format: CARD_RATING_FORMAT.to_string(),
            name: name.to_string(),
            rating: rating.to_string(),
        })
        .collect()
}
