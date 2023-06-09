use serde::{Deserialize, Serialize};
use super::draft_data::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftGame {
    pub game_id: String,
    pub time: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftVote {
    pub game_id: String,
    pub user_id: String,
    pub draft_pick: DraftPick,
    pub vote_idx: u8,
}

impl DraftVote {
    pub fn new(game_id: &str, user_id: &str, draft_pick: &DraftPick, vote_idx: u8) -> Self {
        DraftVote {
            game_id: game_id.to_string(),
            user_id: user_id.to_string(),
            draft_pick: draft_pick.clone(),
            vote_idx,
        }
    }
    pub fn get_id(&self) -> Vec<String> {
        vec![
            self.game_id.to_string(),
            self.user_id.to_string(),
            self.draft_pick.pick_id.to_string(),
        ]
    }
}
