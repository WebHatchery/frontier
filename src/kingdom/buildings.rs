//! Buildings - unlock options, not raw power

use super::stats::KingdomStats;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstructionEvaluation {
    pub cost_gold: i32,
    pub cost_supplies: i32,
}

pub fn evaluate_construction(
    building: &Building,
    stats: &KingdomStats,
) -> Option<ConstructionEvaluation> {
    if building.built || stats.gold < building.cost_gold || stats.supplies < building.cost_supplies
    {
        return None;
    }
    Some(ConstructionEvaluation {
        cost_gold: building.cost_gold,
        cost_supplies: building.cost_supplies,
    })
}

/// A building in the kingdom base
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub id: String,
    pub name: String,
    pub description: String,
    pub built: bool,
    pub level: i32,
    pub cost_gold: i32,
    pub cost_supplies: i32,
}

#[cfg(test)]
mod tests;

impl Building {
    pub fn all_starter() -> Vec<Self> {
        vec![
            Self::infirmary(),
            Self::chapel(),
            Self::foundry(),
            Self::guild_hall(),
            Self::watchtowers(),
            Self::citadel(),
        ]
    }

    pub fn infirmary() -> Self {
        Self {
            id: "infirmary".to_string(),
            name: "Infirmary".to_string(),
            description: "Heal injuries. Unlocks 'Heal' action.".to_string(),
            built: false,
            level: 0,
            cost_gold: 50,
            cost_supplies: 20,
        }
    }

    pub fn chapel() -> Self {
        Self {
            id: "chapel".to_string(),
            name: "Chapel".to_string(),
            description: "Reduce stress. Unlocks 'Tavern/Prayer' action.".to_string(),
            built: false,
            level: 0,
            cost_gold: 50,
            cost_supplies: 10,
        }
    }

    pub fn foundry() -> Self {
        Self {
            id: "foundry".to_string(),
            name: "Foundry".to_string(),
            description: "Upgrade cards and gear via crafting.".to_string(),
            built: false,
            level: 0,
            cost_gold: 100,
            cost_supplies: 50,
        }
    }

    pub fn guild_hall() -> Self {
        Self {
            id: "guild_hall".to_string(),
            name: "Guild Hall".to_string(),
            description: "Recruit specialists and better adventurers.".to_string(),
            built: true, // Basic tent exists
            level: 1,
            cost_gold: 150,
            cost_supplies: 50,
        }
    }

    pub fn watchtowers() -> Self {
        Self {
            id: "watchtowers".to_string(),
            name: "Watchtowers".to_string(),
            description: "Safer routes, but stronger enemies attracted.".to_string(),
            built: false,
            level: 0,
            cost_gold: 80,
            cost_supplies: 40,
        }
    }

    pub fn citadel() -> Self {
        Self {
            id: "citadel".to_string(),
            name: "Permanent Citadel".to_string(),
            description: "A lasting stronghold that secures the frontier and wins the campaign."
                .to_string(),
            built: false,
            level: 0,
            cost_gold: 250,
            cost_supplies: 140,
        }
    }
}
