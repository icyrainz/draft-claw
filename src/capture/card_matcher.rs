use std::collections::HashSet;

use indicium::simple::SearchIndex;
use strsim::levenshtein;

fn preprocess_text(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == ',' || *c == '\'')
        .collect()
}

fn correct_word(word: &str, dictionary: &HashSet<String>) -> String {
    if dictionary.contains(word) {
        return word.to_string();
    }

    dictionary
        .iter()
        .map(|dict_word| (levenshtein(word, dict_word), dict_word))
        .min_by_key(|(dist, _)| *dist)
        .map(|(_, word)| word.to_string())
        .unwrap_or_default()
}

fn find_card(text: &str, card_index: &SearchIndex<String>) -> Option<String> {
    let find_result = card_index.search(&text);

    dbg!(&find_result);

    match find_result.len() {
        multiple if multiple > 1 => {
            // If there are multiple results, we want to find the one that is closest to the text
            let nearest_result = find_result
                .iter()
                .map(|res| (res, levenshtein(text, res)))
                .min_by_key(|(_, dist)| *dist)
                .map(|(res, _)| res.clone());
            nearest_result.cloned()
        }
        1 => Some(find_result[0].clone()),
        _ => None,
    }
}

pub fn find_card_name_matches(
    card_index: &SearchIndex<String>,
    card_name_tokens: &HashSet<String>,
    texts: &[&str],
) -> Vec<String> {
    texts
        .iter()
        .filter_map(|text| {
            let text = preprocess_text(text);

            dbg!(&text);

            find_card(&text, card_index).or_else(|| {
                let corrected_text: Vec<String> = text
                    .split_whitespace()
                    .map(|word| correct_word(word, card_name_tokens))
                    .collect();

                let corrected_text = corrected_text.join(" ");

                dbg!(&corrected_text);

                find_card(corrected_text.as_str(), card_index)
            })
        })
        .collect::<Vec<String>>()
}

mod test {
    use super::*;

    #[test]
    fn tesc_find_matches() {
        const ALPHA: &str = "card alpha";
        const BETA: &str = "card beta";
        const OMEGA: &str = "card omega";

        let card_names = vec![ALPHA, BETA, OMEGA];

        let card_indexes = card_names
            .iter()
            .fold(SearchIndex::default(), |mut acc, card_name| {
                acc.insert(&card_name.to_string(), card_name);
                acc
            });

        let card_name_tokens = card_names
            .iter()
            .fold(HashSet::new(), |mut acc, card_name| {
                card_name.split_whitespace().for_each(|token| {
                    acc.insert(token.to_string());
                });
                acc
            });

        assert_eq!(
            find_card_name_matches(
                &card_indexes,
                &card_name_tokens,
                &vec!["caru alpha", "cerd 0mega"],
            ),
            vec![ALPHA, OMEGA]
        );
    }
}
