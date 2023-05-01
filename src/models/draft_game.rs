use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftGame {
    pub game_id: String,
    pub user_id: Option<String>,
    pub time: Option<String>,
}

impl DraftGame {
    pub fn new(game_id: &str) -> Self {
        DraftGame {
            game_id: game_id.to_string(),
            user_id: None,
            time: None,
        }
    }
}
