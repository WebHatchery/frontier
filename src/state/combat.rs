//! Combat state - turn-based card combat

mod draw;
mod helpers;
mod turn;

use crate::combat::{Card, CombatResolver, Unit};
use crate::data::{enemy_by_id_or_region, random_enemy_for_region_and_difficulty};
use crate::kingdom::PartyMemberState;
use crate::missions::{MapNode, Mission};
use macroquad::prelude::*;

/// Turn-based combat state with party support
pub struct CombatState {
    /// All player units (party members)
    pub players: Vec<Unit>,
    /// Index of the currently active player
    pub current_player_idx: usize,
    pub enemy: Unit,
    pub hand: Vec<Card>,
    pub energy: i32,
    pub max_energy: i32,
    pub turn: usize,
    pub selected_card: Option<usize>,
    pub resolver: CombatResolver,
    /// Mission to return to after combat victory
    pub return_mission: Option<MissionContext>,
    /// Track damage/stress per player for applying after combat
    pub damage_taken: Vec<i32>,
    pub stress_gained: Vec<i32>,
    /// Short-lived UI feedback for clicks and keyboard actions
    pub feedback: Option<(String, f32)>,
}

/// Context needed to return to a mission after combat
#[derive(Clone)]
pub struct MissionContext {
    pub mission: Mission,
    pub current_node: usize,
    pub party_members: Vec<PartyMemberState>,
    /// The generated map nodes for this mission run
    pub map_nodes: Vec<MapNode>,
    /// Nodes that have been visited
    pub visited_nodes: Vec<usize>,
}

impl MissionContext {
    /// Get the leader
    #[allow(dead_code)]
    pub fn leader(&self) -> Option<&PartyMemberState> {
        self.party_members.first()
    }
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            players: vec![Unit::new_player("Adventurer", 50)],
            current_player_idx: 0,
            enemy: Unit::new_enemy("Forest Beast", 30, None),
            hand: Card::starter_hand(),
            energy: 3,
            max_energy: 3,
            turn: 1,
            selected_card: None,
            resolver: CombatResolver::new(),
            return_mission: None,
            damage_taken: vec![0],
            stress_gained: vec![0],
            feedback: None,
        }
    }
}

impl CombatState {
    /// Get the currently active player
    #[allow(dead_code)]
    pub fn current_player(&self) -> Option<&Unit> {
        self.players.get(self.current_player_idx)
    }

    /// Get the currently active player mutably
    #[allow(dead_code)]
    pub fn current_player_mut(&mut self) -> Option<&mut Unit> {
        self.players.get_mut(self.current_player_idx)
    }

    /// Create combat state for a specific adventurer (backwards compat)
    #[allow(dead_code)]
    pub fn for_adventurer(_adventurer_id: &str, adventurer_name: &str) -> Self {
        let player = Unit::new_player(adventurer_name, 50);
        Self {
            players: vec![player],
            ..Default::default()
        }
    }

    /// Create combat that returns to mission on victory, using party stats
    pub fn for_mission(context: MissionContext) -> Self {
        Self::for_mission_with_enemy(context, None)
    }

    pub fn for_mission_with_enemy(context: MissionContext, enemy_id: Option<&str>) -> Self {
        // Create Unit for each party member
        let players: Vec<Unit> = context
            .party_members
            .iter()
            .map(|m| {
                let mut unit = Unit::new_player(&m.name, m.max_hp);
                unit.hp = m.hp;
                unit.stress = m.stress;
                unit.image_path = m.image_path.clone();
                unit.traumas = m.traumas.clone();
                unit.resolve_state = m.resolve_state.clone();
                unit
            })
            .collect();

        let party_size = players.len();

        // Get the current player's class to load appropriate cards
        let class_name = context
            .party_members
            .first()
            .map(|m| m.class_name.as_str())
            .unwrap_or("Soldier");
        let deck_additions = context
            .party_members
            .first()
            .map(|m| m.deck_additions.as_slice())
            .unwrap_or(&[]);
        let hand = Card::load_deck_for_class(class_name, deck_additions)
            .into_iter()
            .take(5)
            .collect();

        // Get random enemy based on mission region and difficulty.
        let enemy = enemy_id
            .map(|id| {
                enemy_by_id_or_region(
                    id,
                    &context.mission.region_id,
                    context.mission.combat_difficulty(),
                )
            })
            .unwrap_or_else(|| {
                random_enemy_for_region_and_difficulty(
                    &context.mission.region_id,
                    context.mission.combat_difficulty(),
                )
            });

        Self {
            players,
            current_player_idx: 0,
            enemy,
            hand,
            return_mission: Some(context),
            damage_taken: vec![0; party_size],
            stress_gained: vec![0; party_size],
            ..Default::default()
        }
    }
}
