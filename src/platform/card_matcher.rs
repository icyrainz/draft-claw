use crate::platform::{card_rating, screen};

use std::collections::{HashMap, HashSet};

pub fn find_matches(ocr_text: &str, card_names: &[String]) -> Vec<String> {
    let threshold = 0.8;
    let mut matches = Vec::new();
    let mut matched_names = HashSet::new();
    let ocr_chars = ocr_text.chars().collect::<Vec<_>>();

    for card_name in card_names {
        if ocr_text.len() < card_name.len() {
            continue;
        }
        for (i, window) in ocr_chars.windows(card_name.len()).enumerate() {
            let ocr_substring = window.iter().collect::<String>();
            let similarity = strsim::jaro_winkler(&ocr_substring, card_name);

            if similarity > threshold && !matched_names.contains(card_name) {
                matches.push((i, card_name.to_string()));
                matched_names.insert(card_name.to_string());
                break; // Break the loop early when a match is found.
            }
        }
    }

    matches.sort_by_key(|(i, _)| *i);
    matches
        .into_iter()
        .map(|(_, card_name)| card_name)
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn tesc_find_matches() {
        let ocr_text = "This is a test carda string card b";
        let card_names = vec!["card a".to_string(), "card b".to_string()];
        let matches = find_matches(ocr_text, &card_names);
        assert_eq!(matches, vec!["card a".to_string(), "card b".to_string()]);
    }
}
