use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRecord {
    pub game_id: String,
    pub pick: DraftPick,
    pub selection_text: String,
    pub decklist_text: Vec<String>,
}

impl DraftRecord {
    pub fn new(game_id: String, pick: DraftPick) -> Self {
        DraftRecord {
            game_id,
            pick,
            selection_text: String::new(),
            decklist_text: Vec::new(),
        }
    }

    pub fn set_selection_text(&mut self, text: &str) {
        self.selection_text = text.to_string();
    }

    pub fn set_decklist_text(&mut self, text: &[&str]) {
        self.decklist_text = text.iter().map(|s| s.to_string()).collect::<Vec<String>>();
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
}
