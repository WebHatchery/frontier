//! Card data loading from JSON

use crate::combat::{Card, CardClass, CardEffect};
use serde::{Deserialize, Serialize};

/// Raw card data from JSON (matches assets/cards.json structure)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardData {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub description: String,
    pub effects: Vec<CardEffect>,
    #[serde(default)]
    pub image_path: Option<String>,
    #[serde(default)]
    pub class: CardClass,
    #[serde(default)]
    pub required_knowledge: i32,
}

impl CardData {
    /// Load all cards from the cards.json asset file
    pub fn load_all() -> Result<Vec<CardData>, String> {
        macroquad_toolkit::data_loader::load_json_file_with_fallback_sync::<Vec<CardData>>(
            "assets/cards.json",
            macroquad_toolkit::include_json_str!("../../assets/cards.json"),
            macroquad_toolkit::data_loader::JsonFallbackPolicy::ReadError,
        )
    }

    /// Check if this card can be used by the given class name
    pub fn class_matches(&self, class_name: &str) -> bool {
        self.class.matches(class_name)
    }

    /// Convert to a playable Card
    pub fn to_card(&self) -> Card {
        Card {
            id: self.id.clone(),
            name: self.name.clone(),
            cost: self.cost,
            description: self.description.clone(),
            effects: self.effects.clone(),
            image_path: self.image_path.clone(),
            class: self.class.clone(),
            required_knowledge: self.required_knowledge,
        }
    }

    /// Advanced cards are learned in the Foundry by spending Knowledge.
    pub fn is_unlockable(&self) -> bool {
        self.required_knowledge > 0
    }
}

/// Load starter deck from JSON and convert to playable cards
#[allow(dead_code)]
pub fn load_starter_deck() -> Result<Vec<Card>, String> {
    let data = CardData::load_all()?;
    Ok(data
        .iter()
        .filter(|c| !c.is_unlockable())
        .map(|c| c.to_card())
        .collect())
}
