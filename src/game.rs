//! Global game loop and state switching
//!
//! Only one GameState is active at a time. Transitions are explicit.

use crate::kingdom::{KingdomState, Party, Roster};
use crate::save::{ensure_save_directory, SaveData};
use crate::state::*;
use macroquad::prelude::*;
use macroquad_toolkit::assets::{load_texture_from_pack_or_file, AssetPack};
use macroquad_toolkit::ui::draw_ui_text;
use std::collections::HashMap;

const ASSET_PACK_PATH: &str = "assets.zip";

/// Top-level game state enum - explicit state machine
pub enum GameState {
    /// Kingdom base management
    Base(BaseState),
    /// Selecting a mission to embark on
    MissionSelect(MissionSelectState),
    /// Active mission/expedition
    Mission(MissionState),
    /// Turn-based card combat
    Combat(CombatState),
    /// Post-mission results and consequences
    Results(ResultState),
    /// Narrative event with choices
    Event(EventState),
    /// Recruit new adventurers
    Recruit(RecruitState),
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Base(BaseState::default())
    }
}

/// Main game struct holding all state
pub struct Game {
    pub state: GameState,
    pub kingdom: KingdomState,
    pub roster: Roster,
    pub message: Option<(String, f32)>, // (message, time remaining)
    pub textures: HashMap<String, Texture2D>,
}

impl Game {
    pub async fn new() -> Self {
        // Try to load existing save
        let (mut kingdom, roster) = if SaveData::exists(&SaveData::default_path()) {
            match SaveData::load(&SaveData::default_path()) {
                Ok(save) => {
                    eprintln!("Loaded save file");
                    (save.kingdom, save.roster)
                }
                Err(e) => {
                    eprintln!("Failed to load save: {}", e);
                    (KingdomState::default(), Roster::starter())
                }
            }
        } else {
            (KingdomState::default(), Roster::starter())
        };
        kingdom.ensure_current_buildings();

        let asset_pack = AssetPack::load(ASSET_PACK_PATH).await.ok();

        // Load textures
        let mut textures = HashMap::new();

        // Helper to load texture if file exists
        async fn load_tex(asset_pack: Option<&AssetPack>, path: &str) -> Option<Texture2D> {
            load_texture_from_pack_or_file(asset_pack, path, FilterMode::Linear)
                .await
                .ok()
        }

        // Helper to parse textures from JSON string
        async fn parse_textures_from_json(
            json_str: &str,
            field_name: &str,
            textures: &mut HashMap<String, Texture2D>,
            asset_pack: Option<&AssetPack>,
        ) {
            if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                for item in items {
                    if let Some(path) = item.get(field_name).and_then(|v| v.as_str()) {
                        if let Some(tex) = load_tex(asset_pack, path).await {
                            textures.insert(path.to_string(), tex);
                        }
                    }
                }
            }
        }

        macro_rules! load_textures {
            ($path:literal, $field:expr, $textures:expr) => {{
                #[cfg(target_arch = "wasm32")]
                let content = macroquad_toolkit::include_json_str!(concat!("../", $path));

                #[cfg(not(target_arch = "wasm32"))]
                let content = match std::fs::read_to_string($path) {
                    Ok(c) => c,
                    Err(_) => {
                        macroquad_toolkit::include_json_str!(concat!("../", $path)).to_string()
                    }
                };

                parse_textures_from_json(&content, $field, $textures, asset_pack.as_ref()).await;
            }};
        }

        // Load all textures from JSON data files
        load_textures!("assets/cards.json", "image_path", &mut textures);
        load_textures!("assets/enemies.json", "image_path", &mut textures);

        // Character images (adventurers are generated, not from JSON)
        let char_images = [
            "soldier_male",
            "soldier_female",
            "scout_male",
            "scout_female",
            "healer_male",
            "healer_female",
            "mystic_male",
            "mystic_female",
        ];
        for name in char_images {
            let path = format!("assets/images/characters/{}.png", name);
            if let Some(tex) = load_tex(asset_pack.as_ref(), &path).await {
                textures.insert(path, tex);
            }
        }

        // Region images
        let region_images = ["dark_woods", "ruined_outpost", "sunken_valley"];
        for name in region_images {
            let path = format!("assets/images/regions/{}.png", name);
            if let Some(tex) = load_tex(asset_pack.as_ref(), &path).await {
                textures.insert(path, tex);
            }
        }

        // UI backdrops
        let ui_images = ["command_table"];
        for name in ui_images {
            let path = format!("assets/images/ui/{}.png", name);
            if let Some(tex) = load_tex(asset_pack.as_ref(), &path).await {
                textures.insert(path, tex);
            }
        }

        Self {
            state: GameState::default(),
            kingdom,
            roster,
            message: None,
            textures,
        }
    }

    /// Update game logic based on current state
    pub fn update(&mut self) {
        // Update message timer
        if let Some((_, ref mut time)) = self.message {
            *time -= get_frame_time();
            if *time <= 0.0 {
                self.message = None;
            }
        }

        // Handle save/load only in base state
        if matches!(self.state, GameState::Base(_)) {
            if is_key_pressed(KeyCode::F5) {
                self.save_game();
            }
            if is_key_pressed(KeyCode::F9) {
                self.load_game();
            }
        }

        match &mut self.state {
            GameState::Base(state) => {
                if let Some(transition) = state.update(&mut self.kingdom, &mut self.roster) {
                    self.transition(transition);
                }
            }
            GameState::MissionSelect(state) => {
                if let Some(transition) = state.update(&self.roster, &self.kingdom) {
                    self.transition(transition);
                }
            }
            GameState::Mission(state) => {
                if let Some(transition) = state.update() {
                    self.transition(transition);
                }
            }
            GameState::Combat(state) => {
                if let Some(transition) = state.update() {
                    self.transition(transition);
                }
            }
            GameState::Results(state) => {
                if let Some(transition) = state.update(&mut self.kingdom, &mut self.roster) {
                    self.transition(transition);
                }
            }
            GameState::Event(state) => {
                if let Some(transition) = state.update() {
                    self.transition(transition);
                }
            }
            GameState::Recruit(state) => {
                if let Some(transition) = state.update(&mut self.kingdom, &mut self.roster) {
                    self.transition(transition);
                }
            }
        }
    }

    /// Draw current state
    pub fn draw(&self) {
        match &self.state {
            GameState::Base(state) => state.draw(&self.kingdom, &self.roster, &self.textures),
            GameState::MissionSelect(state) => state.draw(&self.kingdom, &self.textures),
            GameState::Mission(state) => state.draw(&self.textures),
            GameState::Combat(state) => state.draw(&self.textures),
            GameState::Results(state) => state.draw(&self.textures),
            GameState::Event(state) => state.draw(&self.textures),
            GameState::Recruit(state) => state.draw(&self.kingdom, &self.textures),
        }

        // Draw message if any
        if let Some((msg, _)) = &self.message {
            let x = screen_width() / 2.0 - 100.0;
            draw_rectangle(x - 10.0, 10.0, 220.0, 35.0, Color::from_rgba(0, 0, 0, 200));
            draw_ui_text(msg, x, 35.0, 24.0, YELLOW);
        }
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "recruit" => self.state = GameState::Recruit(RecruitState::new()),
            "missions" => {
                let party = self
                    .roster
                    .adventurers
                    .first()
                    .map(|adv| Party::with_leader(&adv.id))
                    .unwrap_or_default();
                self.state =
                    GameState::MissionSelect(MissionSelectState::for_party(party, &self.roster));
            }
            _ => {
                // Default: the kingdom base screen, same as the boot state.
                self.state = GameState::Base(BaseState::default());
            }
        }
    }

    /// Handle explicit state transitions
    fn transition(&mut self, transition: StateTransition) {
        self.state = match transition {
            StateTransition::ToBase => GameState::Base(BaseState::default()),
            StateTransition::ToMissionSelect(select) => GameState::MissionSelect(select),
            StateTransition::ToMission(mission) => GameState::Mission(mission),
            StateTransition::ToCombat(combat) => GameState::Combat(combat),
            StateTransition::ToResults(results) => GameState::Results(results),
            StateTransition::ToEvent(event) => GameState::Event(event),
            StateTransition::ToRecruit => GameState::Recruit(RecruitState::new()),
        };
    }

    fn save_game(&mut self) {
        if let Err(e) = ensure_save_directory() {
            self.message = Some((format!("Save failed: {}", e), 3.0));
            return;
        }

        let save = SaveData::new(self.kingdom.clone(), self.roster.clone());
        match save.save(&SaveData::default_path()) {
            Ok(()) => {
                self.message = Some(("Game Saved!".to_string(), 2.0));
            }
            Err(e) => {
                self.message = Some((format!("Save failed: {}", e), 3.0));
            }
        }
    }

    fn load_game(&mut self) {
        match SaveData::load(&SaveData::default_path()) {
            Ok(save) => {
                self.kingdom = save.kingdom;
                self.roster = save.roster;
                self.message = Some(("Game Loaded!".to_string(), 2.0));
            }
            Err(e) => {
                self.message = Some((format!("Load failed: {}", e), 3.0));
            }
        }
    }
}
