use std::collections::HashSet;
use std::io::prelude::*;
use std::{collections::HashMap, fs::File, io::BufReader};

use serde::{Serialize, Deserialize};

const CARD_RATING_PATH: &str = "./resource/card_rating.txt";
const CARD_RATING_FORMAT: &str = "14.0";

#[derive(Debug, Serialize, Deserialize)]
pub struct CardRating {
    pub format: String,
    pub name: String,
    pub rating: String,
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
