use crate::app::screen;

use std::collections::{HashMap, HashSet};

fn build_inverse_lookup(tokens: &[&str]) -> HashMap<String, String> {
    let mut lookup = HashMap::new();
    for token in tokens {
        let preprocessed_token = preprocess_text(token);
        lookup.insert(preprocessed_token, token.to_string());
    }
    lookup
}

fn preprocess_text(text: &str) -> String {
    let cleaned_text: String = text
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .map(|c| if c.is_whitespace() { ' ' } else { c })
        .collect();
    cleaned_text
}

pub fn find_matches(text: &str, tokens: &[&str]) -> Vec<String> {
    let lookup = build_inverse_lookup(tokens);
    let preprocessed_text = preprocess_text(text);

    let mut results = Vec::new();
    let mut match_start_index = 0;

    while match_start_index < preprocessed_text.chars().count() {
        let found_match = lookup.iter().find(|(preprocessed_token, _)| {
            let token_len = preprocessed_token.chars().count();
            
            if match_start_index + token_len <= preprocessed_text.chars().count() {
                let window: String = preprocessed_text.chars().skip(match_start_index).take(token_len).collect();
                window == **preprocessed_token
            } else {
                false
            }
        });

        if let Some((_, original_token)) = found_match {
            results.push(original_token.clone());
            match_start_index += original_token.chars().count();
        } else {
            match_start_index += 1;
        }
    }

    results
}

mod test {
    use super::*;

    #[test]
    fn tesc_find_matches() {
        let ocr_text = "This is a test card a string card b";
        let card_names = vec!["card b".to_string(), "card a".to_string(), "card c".to_string()];
        let matches = find_matches(ocr_text, &card_names.iter().map(|s| s.as_str()).collect::<Vec<&str>>());
        assert_eq!(matches, vec!["card a".to_string(), "card b".to_string()]);
    }
}
