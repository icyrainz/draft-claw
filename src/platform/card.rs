use std::fmt::{Display, Formatter, self};

use serde::{de, Deserialize, Deserializer};
use serde_json;

const CARD_DATA_PATH: &str = "./resource/eternal-cards.json";

#[derive(Debug, PartialEq)]
pub enum Influence {
    Fire,
    Time,
    Justice,
    Primal,
    Shadow,
    Multi,
    None,
}

#[derive(Debug)]
pub struct CardInfluence {
    influences: Vec<Influence>,
}

impl<'de> Deserialize<'de> for CardInfluence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let influences: Vec<Influence> = s
            .chars()
            .filter(|c| *c == '{')
            .zip(s.chars().skip(1))
            .filter_map(|(open_brace, influence_char)| {
                if open_brace == '{' {
                    match influence_char {
                        'F' => Some(Influence::Fire),
                        'T' => Some(Influence::Time),
                        'J' => Some(Influence::Justice),
                        'P' => Some(Influence::Primal),
                        'S' => Some(Influence::Shadow),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect();

        Ok(CardInfluence { influences })
    }
}

#[derive(Debug, PartialEq)]
pub enum CardTypeEnum {
    Unit,
    Spell,
    Relic,
    Power,
    Site,
    Curse,
    None,
}

#[derive(Debug)]
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
            match *card_type_str {
                "Unit" => CardTypeEnum::Unit,
                "Spell" => CardTypeEnum::Spell,
                "Relic" => CardTypeEnum::Relic,
                "Power" => CardTypeEnum::Power,
                "Site" => CardTypeEnum::Site,
                "Curse" => CardTypeEnum::Curse,
                _ => CardTypeEnum::None,
            }
        } else {
            CardTypeEnum::None
        };

        Ok(CardType { card_type, is_fast })
    }
}

#[derive(Debug, Deserialize, PartialOrd, Ord, PartialEq, Eq)]
pub enum CardRarity {
    Legendary,
    Rare,
    Uncommon,
    Common,
    Promo,
    None,
}

impl Display for CardRarity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CardRarity::Legendary => write!(f, "{:^10}", "Legendary"),
            CardRarity::Rare => write!(f, "{:^10}", "Rare"),
            CardRarity::Uncommon => write!(f, "{:^10}", "Uncommon"),
            CardRarity::Common => write!(f, "{:^10}", "Common"),
            CardRarity::Promo => write!(f, "{:^10}", "Promo"),
            CardRarity::None => write!(f, "{:^10}", "None"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Card {
    set_number: u32,
    pub name: String,
    #[serde(default = "no_card_text")]
    card_text: String,
    cost: u32,
    influence: CardInfluence,
    attack: i32,
    health: i32,
    pub rarity: CardRarity,
    #[serde(rename = "Type")]
    card_type: CardType,
    image_url: String,
    details_url: String,
    deck_buildable: bool,
    set_name: String,
}

fn no_card_text() -> String {
    String::from("No card text")
}

pub fn load_card_data() -> Vec<Card> {
    let card_data = std::fs::read_to_string(CARD_DATA_PATH).expect("failed to read card data");
    let cards: Vec<Card> = serde_json::from_str(&card_data).expect("failed to parse card data");

    cards
}
