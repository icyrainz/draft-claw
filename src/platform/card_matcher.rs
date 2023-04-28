use crate::platform::{card_rating, screen};

use std::collections::{HashSet, HashMap};

pub fn get_rating_card_onscreen(card_names: &Vec<String>) -> Vec<String> {
    let cards_on_screen = screen::capture_cards_on_screen();

    let matched_cards = 
        find_match(&cards_on_screen, card_names)
        .into_iter().collect::<HashSet<_>>();

    matched_cards.iter().cloned().collect::<Vec<_>>()
}

fn find_match(ocr_text: &str, card_names: &Vec<String>) -> Vec<String> {
    let threshold = 0.8;
    let mut matches = Vec::new();

    for card_name in card_names {
        if ocr_text.len() < card_name.len() {
            continue;
        }
        for i in 0..ocr_text.len() - card_name.len() + 1 {
            let ocr_substring = &ocr_text[i..i + card_name.len()];
            let similarity = 
                strsim::jaro_winkler(
                    ocr_substring,
                    card_name
                );

            if similarity > threshold {
                matches.push(card_name.to_string());
            }
        }
    }

    matches
}
