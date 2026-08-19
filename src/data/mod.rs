//! Data loading - JSON content for cards, enemies, regions
//!
//! All game content is data-driven. This module handles loading from JSON files.

// use std::fs;
// Still used in public signature? Yes load_asset macro uses generic type but maybe not? Warning said it was unused?
// Warning said: warning: unused import: `serde::de::DeserializeOwned`
// Ah, `load_json` and `load_asset` functions were removed/commented out effectively?
// Wait, I replaced `load_json` and `load_asset` with a macro. The macro does NOT use `DeserializeOwned` trait bound on a function. It just calls `serde_json::from_str::<$type>`.
// So the import IS unused.

pub mod cards;
pub mod enemies;

pub use enemies::{enemy_by_id_or_region, random_enemy_for_region_and_difficulty};
// CardData and EnemyData are used internally

/// Macro to load JSON from assets with WASM support
#[macro_export]
macro_rules! load_asset {
    ($filename:literal, $type:ty) => {{
        #[cfg(target_arch = "wasm32")]
        let content = include_str!(concat!("../../assets/", $filename));

        #[cfg(not(target_arch = "wasm32"))]
        let content = match std::fs::read_to_string(concat!("assets/", $filename)) {
            Ok(c) => c,
            Err(_) => include_str!(concat!("../../assets/", $filename)).to_string(),
        };

        serde_json::from_str::<$type>(&content)
            .map_err(|e| format!("Failed to parse {}: {}", $filename, e))
    }};
}
