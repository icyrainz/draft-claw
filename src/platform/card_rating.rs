use std::collections::HashSet;
use std::io::prelude::*;
use std::{collections::HashMap, fs::File, io::BufReader};

const CARD_RATING_PATH: &str = "./resource/card_rating.txt";

#[derive(Debug)]
struct CardRating {
    name: String,
    rating: String,
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
    println!("all_ratings: {:?}", all_ratings);

    card_ratings
}
