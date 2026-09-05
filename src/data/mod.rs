//! Typed game content loaded through the shared toolkit.

pub mod cards;
pub mod enemies;

pub use enemies::{enemy_by_id_or_region, random_enemy_for_region_and_difficulty};
