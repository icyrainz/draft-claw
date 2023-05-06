use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use indicium::simple::Indexable;
use itertools::Itertools;
use serde::{de, Deserialize, Deserializer};
use serde_json;

use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Display, EnumString)]
pub enum Influence {
    #[strum(serialize = "F")]
    Fire,
    #[strum(serialize = "T")]
    Time,
    #[strum(serialize = "J")]
    Justice,
    #[strum(serialize = "P")]
    Primal,
    #[strum(serialize = "S")]
    Shadow,
    #[strum(serialize = "x")]
    None,
}

#[derive(Debug, Clone)]
pub struct CardInfluence {
    influences: Vec<Influence>,
}

impl Display for CardInfluence {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut influences_str = String::new();
        for influence in &self.influences {
            influences_str.push_str(&format!("{}", influence));
        }
        write!(f, "{:<6}", influences_str)
    }
}

impl<'de> Deserialize<'de> for CardInfluence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let influences: Vec<Influence> = s
            .chars()
            .chunks(3)
            .into_iter()
            .filter_map(|chunk| chunk.into_iter().nth(1))
            .map(|c| Influence::from_str(&c.to_string()).unwrap_or(Influence::None))
            .collect();
        Ok(CardInfluence { influences })
    }
}

#[derive(Debug, Clone, PartialEq, EnumString)]
pub enum CardTypeEnum {
    Unit,
    Spell,
    Relic,
    Power,
    Site,
    Curse,
    None,
}

#[derive(Debug, Clone)]
pub struct CardType {
    card_type: CardTypeEnum,
    is_fast: bool,
}

impl<'de> Deserialize<'de> for CardType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut tokens: Vec<&str> = s.split_whitespace().collect();

        let is_fast = tokens.contains(&"Fast");
        if is_fast {
            tokens.retain(|&token| token != "Fast");
        }

        let card_type = if let Some(card_type_str) = tokens.first() {
            CardTypeEnum::from_str(card_type_str).unwrap_or(CardTypeEnum::None)
        } else {
            CardTypeEnum::None
        };

        Ok(CardType { card_type, is_fast })
    }
}

#[derive(Debug, Clone, Deserialize, PartialOrd, Ord, PartialEq, Eq, Display, EnumString)]
pub enum CardRarity {
    #[strum(serialize = "L")]
    Legendary,
    #[strum(serialize = "R")]
    Rare,
    #[strum(serialize = "U")]
    Uncommon,
    #[strum(serialize = "C")]
    Common,
    #[strum(serialize = "P")]
    Promo,
    #[strum(serialize = "x")]
    None,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Card {
    pub set_number: u32,
    pub name: String,
    #[serde(default = "no_card_text")]
    pub card_text: String,
    pub cost: u32,
    pub influence: CardInfluence,
    pub attack: i32,
    pub health: i32,
    pub rarity: CardRarity,
    #[serde(rename = "Type")]
    pub card_type: CardType,
    pub image_url: String,
    pub details_url: String,
    pub deck_buildable: bool,
    pub set_name: String,
}

fn no_card_text() -> String {
    String::from("No card text")
}

impl Indexable for Card {
    fn strings(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
}
