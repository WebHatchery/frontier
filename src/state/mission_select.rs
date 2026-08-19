//! Mission selection state: choose a route after assembling the expedition.

mod briefing;
mod cards;
mod input;
mod layout;

use super::{MissionState, StateTransition};
use crate::kingdom::{KingdomState, Party, PartyMemberState, Roster};
use crate::missions::{load_missions, Mission};
use macroquad::prelude::*;

/// State for selecting a mission before departure.
pub struct MissionSelectState {
    pub missions: Vec<Mission>,
    pub selected_mission: usize,
    /// Party members going on this mission (leader is first).
    pub party_members: Vec<PartyMemberState>,
}

impl MissionSelectState {
    /// Create mission select with a pre-selected adventurer.
    #[allow(dead_code)]
    pub fn new(
        adventurer_id: String,
        adventurer_name: String,
        hp: i32,
        max_hp: i32,
        stress: i32,
        image: Option<String>,
    ) -> Self {
        Self {
            missions: load_missions(),
            selected_mission: 0,
            party_members: vec![PartyMemberState {
                id: adventurer_id,
                name: adventurer_name,
                hp,
                max_hp,
                stress,
                image_path: image,
                class_name: "Soldier".to_string(),
                deck_additions: vec![],
                traumas: vec![],
                resolve_state: None,
            }],
        }
    }

    /// Create mission select from a party and roster.
    pub fn for_party(party: Party, roster: &Roster) -> Self {
        let party_members = party
            .member_ids
            .iter()
            .filter_map(|id| roster.get(id))
            .map(PartyMemberState::from_adventurer)
            .collect();
        Self {
            missions: load_missions(),
            selected_mission: 0,
            party_members,
        }
    }

    pub fn leader(&self) -> Option<&PartyMemberState> {
        self.party_members.first()
    }

    pub fn is_mission_unlocked(&self, mission: &Mission, kingdom: &KingdomState) -> bool {
        mission.unlock_requirement.is_met(kingdom)
    }

    pub fn update(&mut self, _roster: &Roster, kingdom: &KingdomState) -> Option<StateTransition> {
        input::update(self, kingdom)
    }

    pub(super) fn start_selected_mission(&self, kingdom: &KingdomState) -> Option<StateTransition> {
        let mission = self.missions.get(self.selected_mission)?;
        if !self.is_mission_unlocked(mission, kingdom) || self.leader().is_none() {
            return None;
        }
        let scaled_mission = mission.scaled_for_kingdom(kingdom);
        Some(StateTransition::ToMission(
            MissionState::from_mission_with_party(scaled_mission, self.party_members.clone()),
        ))
    }

    pub(super) fn selected_mission(&self) -> Option<&Mission> {
        self.missions.get(self.selected_mission)
    }

    pub fn draw(
        &self,
        kingdom: &KingdomState,
        textures: &std::collections::HashMap<String, Texture2D>,
    ) {
        layout::draw_background(textures);
        briefing::draw_header(kingdom);
        briefing::draw_party_panel(self, textures);
        briefing::draw_mission_board(self, kingdom, textures);
        briefing::draw_detail_panel(self, kingdom);
        briefing::draw_optional_hint(self, kingdom);
    }
}

#[cfg(test)]
mod tests;
