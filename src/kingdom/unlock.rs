//! Unlock requirements for regions and missions

use crate::kingdom::KingdomState;
use serde::{Deserialize, Serialize};

/// Requirement to unlock a region or mission
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnlockRequirement {
    /// No requirement - always available
    #[default]
    None,
    /// Requires a specific building to be built
    Building { building: String },
    /// Requires a minimum knowledge level
    Knowledge { amount: i32 },
    /// Requires a specific mission to be completed
    MissionComplete { mission_id: String },
}

impl UnlockRequirement {
    /// Check if this requirement is met given the current kingdom state
    pub fn is_met(&self, kingdom: &KingdomState) -> bool {
        match self {
            UnlockRequirement::None => true,
            UnlockRequirement::Building { building } => kingdom
                .buildings
                .iter()
                .any(|b| &b.id == building && b.built),
            UnlockRequirement::Knowledge { amount } => kingdom.stats.knowledge >= *amount,
            UnlockRequirement::MissionComplete { mission_id } => {
                kingdom.completed_missions.contains(mission_id)
            }
        }
    }

    /// Get a human-readable description of the requirement
    pub fn description(&self) -> String {
        match self {
            UnlockRequirement::None => "Available".to_string(),
            UnlockRequirement::Building { building } => {
                format!("Requires: {} (Building)", building.replace('_', " "))
            }
            UnlockRequirement::Knowledge { amount } => {
                format!("Requires: {} Knowledge", amount)
            }
            UnlockRequirement::MissionComplete { mission_id } => {
                format!("Requires: Complete \"{}\"", mission_id.replace('_', " "))
            }
        }
    }
}

#[cfg(test)]
mod tests;
