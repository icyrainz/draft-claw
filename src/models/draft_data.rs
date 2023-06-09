use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRecord {
    pub game_id: String,
    pub pick: DraftPick,
    pub selection_text: String,
    pub selection_vec: Vec<String>,
    pub decklist_text: Vec<String>,
    pub selected_card: Option<u8>,
    pub image_url: Option<String>,
}

impl DraftRecord {
    pub fn new(game_id: String, pick: DraftPick) -> Self {
        DraftRecord {
            game_id,
            pick,
            selection_text: String::new(),
            selection_vec: Vec::new(),
            decklist_text: Vec::new(),
            selected_card: None,
            image_url: None,
        }
    }
    pub fn get_id(&self) -> Vec<String> {
        vec![self.game_id.to_string(), self.pick.pick_id.to_string()]
    }

    pub fn generate_id(game_id: &str, pick: &DraftPick) -> Vec<String> {
        vec![game_id.to_string(), pick.pick_id.to_string()]
    }

    pub fn set_selection_vec(&mut self, selections: &[&str]) {
        self.selection_vec = selections
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
    }

    pub fn set_selection_text(&mut self, text: &str) {
        self.selection_text = text.to_string();
    }

    pub fn set_decklist_text(&mut self, text: &[&str]) {
        self.decklist_text = text.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    }

    pub fn pick_card(&mut self, selected_card: u8) {
        self.selected_card = Some(selected_card);
    }

    pub fn set_image_url(&mut self, image_url: &str) {
        self.image_url = Some(image_url.to_string());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPick {
    pub pick_id: u8,
    pub pick_str: String,
}

impl DraftPick {
    pub fn new(id: u8) -> Self {
        if id < 1 || id > 48 {
            panic!("Invalid draft pick id: {}", id);
        }

        let pack = (id - 1) / 12 + 1;
        let pick = (id - 1) % 12 + 1;

        DraftPick {
            pick_id: id,
            pick_str: format!("p{}p{}", pack, pick),
        }
    }

    pub fn to_string(&self) -> String {
        format!("{} ({})", self.pick_id, self.pick_str)
    }

    pub fn get_expected_card_selection_count(&self) -> u8 {
        12 - (self.pick_id - 1) % 12
    }
}

fn get_next_draft_pick(pick: &DraftPick) -> DraftPick {
    DraftPick::new(pick.pick_id + 1)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_draft_pick() {
        let pick1 = DraftPick::new(1);
        assert_eq!(pick1.pick_id, 1);
        assert_eq!(pick1.pick_str, "p1p1");

        let pick2 = DraftPick::new(12);
        assert_eq!(pick2.pick_id, 12);
        assert_eq!(pick2.pick_str, "p1p12");

        let pick3 = DraftPick::new(13);
        assert_eq!(pick3.pick_id, 13);
        assert_eq!(pick3.pick_str, "p2p1");

        let pick4 = DraftPick::new(48);
        assert_eq!(pick4.pick_id, 48);
        assert_eq!(pick4.pick_str, "p4p12");
    }

    #[test]
    fn test_get_next_draft_pick() {
        let pick1 = DraftPick {
            pick_str: "p1p1".to_string(),
            pick_id: 1,
        };
        let next_pick1 = get_next_draft_pick(&pick1);
        assert_eq!(next_pick1.pick_id, 2);
        assert_eq!(next_pick1.pick_str, "p1p2");

        let pick2 = DraftPick {
            pick_str: "p1p12".to_string(),
            pick_id: 12,
        };
        let next_pick2 = get_next_draft_pick(&pick2);
        assert_eq!(next_pick2.pick_id, 13);
        assert_eq!(next_pick2.pick_str, "p2p1");
    }

    #[test]
    fn test_expected_card_selection_count() {
        assert_eq!(DraftPick::new(1).get_expected_card_selection_count(), 12);
        assert_eq!(DraftPick::new(2).get_expected_card_selection_count(), 11);
        assert_eq!(DraftPick::new(3).get_expected_card_selection_count(), 10);
        assert_eq!(DraftPick::new(11).get_expected_card_selection_count(), 2);
        assert_eq!(DraftPick::new(12).get_expected_card_selection_count(), 1);
        assert_eq!(DraftPick::new(13).get_expected_card_selection_count(), 12);
        assert_eq!(DraftPick::new(14).get_expected_card_selection_count(), 11);
        assert_eq!(DraftPick::new(47).get_expected_card_selection_count(), 2);
        assert_eq!(DraftPick::new(48).get_expected_card_selection_count(), 1);
    }
}
