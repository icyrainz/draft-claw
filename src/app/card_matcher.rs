use indicium::simple::SearchIndex;

fn preprocess_text(text: &str) -> String {
    let cleaned_text: String = text
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == ',' || *c == '\'')
        .collect();
    cleaned_text
}

pub fn find_card_name_matches(card_index: &SearchIndex<String>, texts: &[&str]) -> Vec<String> {
    texts.iter().filter_map(|text| {
        let text = preprocess_text(text);
        let find_result = card_index.search(&text);

        match find_result.len() {
            1 => {
                Some(find_result[0].clone())
            },
            0 => {
                // println!("No matches found for text: {}", text);
                None
            },
            _ => {
                // println!("Multiple matches found for text: {}", text);
                None
            },
        }
    }).collect::<Vec<String>>()
}


mod test {
    use super::*;

    #[test]
    fn tesc_find_matches() {
        let ocr_text = vec!["card a", "card b"];
        let card_names = vec!["card b", "card a", "card c"];
        let mut card_indexes: SearchIndex<String> = SearchIndex::default();

        card_names.iter().for_each(|card_name| {
            card_indexes.insert(&card_name.to_string(), card_name);
        });

        let matches = find_card_name_matches(&card_indexes, &ocr_text);
        assert_eq!(matches, vec!["card a".to_string(), "card b".to_string()]);
    }
}
