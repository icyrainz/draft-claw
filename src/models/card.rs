use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use indicium::simple::Indexable;
use itertools::Itertools;
use serde::{Deserialize, Deserializer};

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

impl CardInfluence {
    pub fn count(&self) -> u8 {
        self.influences.len() as u8
    }
    pub fn to_text(&self) -> String {
        self.influences.iter().map(|i| format!("{}", i)).join("")
    }
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
    Weapon,
    Power,
    Site,
    Curse,
    None,
}

#[derive(Debug, Clone)]
pub struct CardType {
    card_type: Vec<CardTypeEnum>,
    is_fast: bool,
}

impl<'de> Deserialize<'de> for CardType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let tokens: Vec<&str> = s.split_whitespace().collect();

        let is_fast = tokens.contains(&"Fast");

        let card_type = tokens
            .iter()
            .filter_map(|token| {
                if token == &"Fast" {
                    None
                } else {
                    CardTypeEnum::from_str(token).ok()
                }
            })
            .collect();

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

pub struct CardToTextOpt {
    pub inclued_rarity: bool,
}

pub const CARD_TO_TEXT_OPT_DEFAULT: CardToTextOpt = CardToTextOpt {
    inclued_rarity: true,
};

pub const CARD_TO_TEXT_OPT_NO_RARITY: CardToTextOpt = CardToTextOpt {
    inclued_rarity: false,
};

impl Card {
    pub fn to_text(&self, opt: CardToTextOpt) -> String {
        let extra = match self
            .card_type
            .card_type
            .last()
            .unwrap_or(&CardTypeEnum::None)
        {
            CardTypeEnum::Unit | CardTypeEnum::Weapon => {
                format!("{:>2}/{:>2}", self.attack, self.health)
            }
            CardTypeEnum::Spell => {
                if self.card_type.is_fast {
                    format!("{}", "FSpell")
                } else {
                    format!("{}", "Spell")
                }
            }
            CardTypeEnum::Relic => format!("{}", "Relic"),
            CardTypeEnum::Site => format!("{}", "Site"),
            CardTypeEnum::Curse => format!("{}", "Curse"),
            _ => String::new(),
        };
        let mut result = String::new();
        if opt.inclued_rarity {
            result.push_str(&format!("{} ", self.rarity));
        }
        result.push_str(&format!(
            "{}{:<6} {:30}{:>7}",
            self.cost, self.influence, self.name, extra
        ));
        result
    }
}

impl Indexable for Card {
    fn strings(&self) -> Vec<String> {
        vec![self.name.clone()]
    }
}
