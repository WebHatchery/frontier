//! Enemy data loading from JSON

use crate::combat::{EnemyAiPattern, Unit};
use serde::{Deserialize, Serialize};

/// Enemy template from data file
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyData {
    pub id: String,
    pub name: String,
    pub max_hp: i32,
    pub base_damage: i32,
    pub threat_level: i32,
    pub region: String,
    #[serde(default)]
    pub ai_pattern: EnemyAiPattern,
    #[serde(default)]
    pub image_path: Option<String>,
}

impl EnemyData {
    /// Load all enemies from the enemies.json asset file
    pub fn load_all() -> Result<Vec<EnemyData>, String> {
        macroquad_toolkit::data_loader::load_json_file_with_fallback_sync::<Vec<EnemyData>>(
            "assets/enemies.json",
            macroquad_toolkit::include_json_str!("../../assets/enemies.json"),
            macroquad_toolkit::data_loader::JsonFallbackPolicy::ReadError,
        )
    }

    /// Convert to a combat Unit
    pub fn to_unit(&self) -> Unit {
        let mut unit = Unit::new_enemy_with_pattern(
            &self.name,
            self.max_hp,
            self.base_damage,
            self.image_path.clone(),
            self.ai_pattern.clone(),
        );
        unit.roll_intent(1);
        unit
    }
}

/// Get a random enemy appropriate for the given difficulty
#[allow(dead_code)]
pub fn random_enemy_for_difficulty(difficulty: i32) -> Unit {
    random_enemy_for_region_and_difficulty("", difficulty)
}

/// Get a random enemy appropriate for region and difficulty.
pub fn random_enemy_for_region_and_difficulty(region_id: &str, difficulty: i32) -> Unit {
    match EnemyData::load_all() {
        Ok(enemies) => {
            let mut suitable: Vec<_> = enemies
                .iter()
                .filter(|e| {
                    e.threat_level <= difficulty && (region_id.is_empty() || e.region == region_id)
                })
                .collect();

            if suitable.is_empty() && !region_id.is_empty() {
                suitable = enemies
                    .iter()
                    .filter(|e| e.threat_level <= difficulty)
                    .collect();
            }

            if suitable.is_empty() {
                // Fallback to any enemy
                if let Some(enemy) = enemies.first() {
                    return enemy.to_unit();
                }
            } else {
                // Pick a random one
                if let Some(enemy) = macroquad_toolkit::rng::choose(&suitable) {
                    return enemy.to_unit();
                }
            }

            // Ultimate fallback
            Unit::new_enemy("Forest Beast", 30, None)
        }
        Err(_) => Unit::new_enemy("Forest Beast", 30, None),
    }
}

/// Resolve a named event enemy, falling back to the normal regional table.
pub fn enemy_by_id_or_region(enemy_id: &str, region_id: &str, difficulty: i32) -> Unit {
    if let Ok(enemies) = EnemyData::load_all() {
        if let Some(enemy) = enemies.iter().find(|enemy| enemy.id == enemy_id) {
            return enemy.to_unit();
        }
    }
    random_enemy_for_region_and_difficulty(region_id, difficulty)
}
