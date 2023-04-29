use serde::{Serialize, Deserialize};

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
}
