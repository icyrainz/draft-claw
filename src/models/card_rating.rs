use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CardRating {
    pub format: String,
    pub name: String,
    pub rating: String,
}
