use std::ops::IndexMut;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftRecord {
    pub game_id: String,
    pub pick: DraftPick,
    pub selection_cards: Vec<String>,
    pub decklist_cards: Vec<String>,
    pub selection_text: String,
    pub decklist_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPick {
    pub pick: String,
    id: u8,
}

fn get_next_draft_pick(pick: &DraftPick) -> DraftPick {
    let id = pick.id + 1;
    get_draft_pick(id)
}

fn get_draft_pick(id: u8) -> DraftPick {
    let pack = (id - 1) / 12 + 1;
    let pick = (id - 1) % 12 + 1;
    DraftPick {
        pick: format!("p{}p{}", pack, pick),
        id,
    }
}

fn get_draft_pick_from_str(pick: &str) -> DraftPick {
    let mut split = pick.split("p");
    let pack = split.nth(1).unwrap().parse::<u8>().unwrap();
    let pick = split.next().unwrap().parse::<u8>().unwrap();

    let id = (pack - 1) * 12 + pick;

    if id > 48 || id < 1 {
        panic!("Invalid draft pick: {}", pick);
    }

    DraftPick {
        pick: pick.to_string(),
        id,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_draft_pick() {
        assert_eq!(get_draft_pick(1).pick, "p1p1");
        assert_eq!(get_draft_pick(2).pick, "p1p2");
        assert_eq!(get_draft_pick(12).pick, "p1p12");
        assert_eq!(get_draft_pick(13).pick, "p2p1");
        assert_eq!(get_draft_pick(14).pick, "p2p2");
        assert_eq!(get_draft_pick(24).pick, "p2p12");
        assert_eq!(get_draft_pick(48).pick, "p4p12");
    }

    #[test]
    fn test_get_draft_pick_from_str() {
        assert_eq!(super::get_draft_pick_from_str("p1p1").id, 1);
        assert_eq!(super::get_draft_pick_from_str("p1p2").id, 2);
        assert_eq!(super::get_draft_pick_from_str("p1p12").id, 12);
        assert_eq!(super::get_draft_pick_from_str("p2p1").id, 13);
    }

    #[test]
    #[should_panic]
    fn test_bad_draft_pick_ids() {
        get_draft_pick_from_str("p0p1");
        get_draft_pick_from_str("p1p13");
    }

    #[test]
    fn test_get_next_draft_pick() {
        let pick1 = DraftPick {
            pick: "p1p1".to_string(),
            id: 1,
        };
        let next_pick1 = get_next_draft_pick(&pick1);
        assert_eq!(next_pick1.id, 2);
        assert_eq!(next_pick1.pick, "p1p2");

        let pick2 = DraftPick {
            pick: "p1p12".to_string(),
            id: 12,
        };
        let next_pick2 = get_next_draft_pick(&pick2);
        assert_eq!(next_pick2.id, 13);
        assert_eq!(next_pick2.pick, "p2p1");
    }
}
