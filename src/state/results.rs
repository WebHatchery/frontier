//! Results state - post-mission consequences and resolution

use super::StateTransition;
use crate::kingdom::{Injury, KingdomState, PartyMemberState, Roster};
use crate::missions::Mission;
use crate::ui::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

/// Post-mission results state
pub struct ResultState {
    pub victory: bool,
    pub stress_gained: i32,
    pub hp_lost: i32,
    pub injuries: Vec<String>,
    pub rewards: Vec<String>,
    pub adventurer_id: String,
    pub mission_id: Option<String>,
    pub mission_difficulty: i32,
    pub reward_gold: i32,
    pub reward_supplies: i32,
    pub reward_knowledge: i32,
    pub reward_influence: i32,
    /// All party member IDs and their final states (for future party-wide results)
    #[allow(dead_code)]
    pub party_member_states: Vec<PartyMemberState>,
    /// Final HP after mission (if set, overrides hp_lost calculation)
    pub final_hp: Option<i32>,
    /// Final stress after mission
    pub final_stress: Option<i32>,
}

impl Default for ResultState {
    fn default() -> Self {
        Self::victory_for("")
    }
}

impl ResultState {
    pub fn victory_for(adventurer_id: &str) -> Self {
        Self {
            victory: true,
            stress_gained: 5,
            hp_lost: 0,
            injuries: vec![],
            rewards: vec![
                "20 Gold".to_string(),
                "10 Supplies".to_string(),
                "+5 Knowledge".to_string(),
            ],
            adventurer_id: adventurer_id.to_string(),
            mission_id: None,
            mission_difficulty: 1,
            reward_gold: 20,
            reward_supplies: 10,
            reward_knowledge: 5,
            reward_influence: 0,
            party_member_states: vec![],
            final_hp: None,
            final_stress: None,
        }
    }

    /// Create victory result for a full party
    pub fn victory_for_party(party_members: &[PartyMemberState]) -> Self {
        let adventurer_id = party_members
            .first()
            .map(|m| m.id.clone())
            .unwrap_or_default();
        Self {
            victory: true,
            stress_gained: 5,
            hp_lost: 0,
            injuries: vec![],
            rewards: vec![
                "20 Gold".to_string(),
                "10 Supplies".to_string(),
                "+5 Knowledge".to_string(),
            ],
            adventurer_id,
            mission_id: None,
            mission_difficulty: 1,
            reward_gold: 20,
            reward_supplies: 10,
            reward_knowledge: 5,
            reward_influence: 0,
            party_member_states: party_members.to_vec(),
            final_hp: None,
            final_stress: None,
        }
    }

    pub fn victory_for_mission(mission: &Mission, party_members: &[PartyMemberState]) -> Self {
        let mut result = Self::victory_for_party(party_members);
        result.mission_id = Some(mission.id.clone());
        result.mission_difficulty = mission.difficulty;
        result.stress_gained = mission.base_stress;
        result.reward_gold = mission.reward_gold;
        result.reward_supplies = mission.reward_supplies;
        result.reward_knowledge = mission.reward_knowledge;
        result.reward_influence = mission.reward_influence;
        result.rewards = vec![
            format!("{} Gold", mission.reward_gold),
            format!("{} Supplies", mission.reward_supplies),
            format!("{} Knowledge", mission.reward_knowledge),
        ];
        if mission.reward_influence > 0 {
            result
                .rewards
                .push(format!("{} Influence", mission.reward_influence));
        }
        result
    }

    pub fn defeat_for(adventurer_id: &str) -> Self {
        Self {
            victory: false,
            stress_gained: 15,
            hp_lost: 20,
            injuries: vec!["Wounded Leg".to_string()],
            rewards: vec![],
            adventurer_id: adventurer_id.to_string(),
            mission_id: None,
            mission_difficulty: 1,
            reward_gold: 0,
            reward_supplies: 0,
            reward_knowledge: 0,
            reward_influence: 0,
            party_member_states: vec![],
            final_hp: None,
            final_stress: None,
        }
    }

    /// Create defeat result for a full party
    pub fn defeat_for_party(party_members: &[PartyMemberState]) -> Self {
        let adventurer_id = party_members
            .first()
            .map(|m| m.id.clone())
            .unwrap_or_default();
        Self {
            victory: false,
            stress_gained: 15,
            hp_lost: 20,
            injuries: vec!["Wounded Leg".to_string()],
            rewards: vec![],
            adventurer_id,
            mission_id: None,
            mission_difficulty: 1,
            reward_gold: 0,
            reward_supplies: 0,
            reward_knowledge: 0,
            reward_influence: 0,
            party_member_states: party_members.to_vec(),
            final_hp: None,
            final_stress: None,
        }
    }

    pub fn defeat_for_mission(mission: &Mission, party_members: &[PartyMemberState]) -> Self {
        let mut result = Self::defeat_for_party(party_members);
        result.mission_id = Some(mission.id.clone());
        result.mission_difficulty = mission.difficulty;
        result.stress_gained = mission.base_stress + 10;
        result
    }

    pub fn update(
        &mut self,
        kingdom: &mut KingdomState,
        roster: &mut Roster,
    ) -> Option<StateTransition> {
        if is_key_pressed(KeyCode::Enter) {
            // Apply consequences to kingdom
            if self.victory {
                kingdom.stats.gold += self.reward_gold;
                kingdom.stats.supplies += self.reward_supplies;
                kingdom.stats.knowledge += self.reward_knowledge;
                kingdom.stats.influence += self.reward_influence;
                kingdom.stats.security = (kingdom.stats.security + 3).min(100);
                if let Some(mission_id) = &self.mission_id {
                    kingdom.record_mission_complete(mission_id);
                }
            } else {
                kingdom.stats.morale = (kingdom.stats.morale - 10).max(0);
                kingdom.stats.security = (kingdom.stats.security - 5).max(0);
            }

            self.apply_roster_results(roster);
            kingdom.day += 1;
            kingdom.advance_threat(self.victory);
            kingdom.last_event = self.roll_kingdom_event(kingdom, roster);

            return Some(StateTransition::ToBase);
        }

        None
    }

    fn apply_roster_results(&self, roster: &mut Roster) {
        if self.party_member_states.is_empty() {
            self.apply_single_adventurer(roster);
            return;
        }

        for state in &self.party_member_states {
            if state.hp <= 0 {
                roster.record_death(&state.id);
                continue;
            }

            if let Some(adv) = roster.get_mut(&state.id) {
                adv.hp = state.hp.max(1).min(adv.max_hp);
                if state.resolve_state.is_some() {
                    adv.resolve_state = state.resolve_state.clone();
                }
                for trauma in &state.traumas {
                    if !adv
                        .traumas
                        .iter()
                        .any(|existing| existing.trauma_type == trauma.trauma_type)
                    {
                        adv.traumas.push(trauma.clone());
                    }
                }

                let stress_delta = state.stress - adv.stress;
                if stress_delta < 0 {
                    adv.reduce_stress(-stress_delta);
                }
                let total_stress_gain = stress_delta.max(0) + self.stress_gained;
                if total_stress_gain > 0 {
                    adv.apply_stress_gain(total_stress_gain);
                }

                if adv.hp <= adv.max_hp / 3 && !adv.injuries.iter().any(|i| i.id == "wounded_leg") {
                    adv.injuries.push(Injury::wounded_leg());
                }

                if self.victory {
                    adv.missions_completed += 1;
                    adv.xp += 10 + (self.mission_difficulty * 2);
                    let needed = adv.level * 20;
                    if adv.xp >= needed {
                        adv.xp -= needed;
                        adv.level += 1;
                        adv.max_hp += 3;
                        adv.hp = (adv.hp + 3).min(adv.max_hp);
                    }
                } else if !adv.injuries.iter().any(|i| i.id == "broken_arm") {
                    adv.injuries.push(Injury::broken_arm());
                }
            }
        }
    }

    fn apply_single_adventurer(&self, roster: &mut Roster) {
        let is_dead = self.final_hp.is_some_and(|final_hp| final_hp <= 0);
        if is_dead {
            roster.record_death(&self.adventurer_id);
            return;
        }

        if let Some(adv) = roster.get_mut(&self.adventurer_id) {
            if let Some(final_hp) = self.final_hp {
                adv.hp = final_hp.max(1);
            } else {
                adv.hp = (adv.hp - self.hp_lost).max(1);
            }

            if let Some(final_stress) = self.final_stress {
                let delta = final_stress - adv.stress;
                if delta >= 0 {
                    adv.apply_stress_gain(delta);
                } else {
                    adv.reduce_stress(-delta);
                }
            } else {
                adv.apply_stress_gain(self.stress_gained);
            }

            if self.victory {
                adv.missions_completed += 1;
            }
        }
    }

    fn roll_kingdom_event(
        &self,
        kingdom: &mut KingdomState,
        roster: &mut Roster,
    ) -> Option<String> {
        if !macroquad_toolkit::rng::chance(0.45) {
            return None;
        }

        match macroquad_toolkit::rng::gen_range(0, 4) {
            0 => {
                kingdom.stats.morale = (kingdom.stats.morale - 6).max(0);
                if let Some(adv) = roster.adventurers.first_mut() {
                    adv.apply_stress_gain(6);
                }
                Some("Plague: morale fell and the roster gained stress.".to_string())
            }
            1 => {
                let stolen = kingdom.stats.gold.min(25);
                kingdom.stats.gold -= stolen;
                Some(format!(
                    "Thieves: {} gold was stolen from the stores.",
                    stolen
                ))
            }
            2 => {
                if kingdom.stats.gold >= 15 {
                    kingdom.stats.gold -= 15;
                    kingdom.stats.supplies += 25;
                    Some("Traders: paid 15 gold for 25 supplies.".to_string())
                } else {
                    kingdom.stats.gold += 10;
                    Some("Traders: a small debt was forgiven for 10 gold.".to_string())
                }
            }
            _ => {
                kingdom.stats.morale = (kingdom.stats.morale + 8).min(100);
                for adv in &mut roster.adventurers {
                    adv.reduce_stress(3);
                }
                Some("Festival: morale rose and adventurers shed a little stress.".to_string())
            }
        }
    }

    pub fn draw(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        let is_dead = self.final_hp.is_some_and(|hp| hp <= 0);

        draw_background(
            textures,
            BackgroundArt::ResultsAftermath,
            Color::from_rgba(5, 6, 8, 164),
        );

        let title = if is_dead {
            "FALLEN IN BATTLE"
        } else if self.victory {
            "MISSION COMPLETE"
        } else {
            "MISSION FAILED"
        };
        let title_color = if self.victory { GREEN } else { RED };

        draw_icon(
            textures,
            if is_dead || !self.victory {
                SpriteIcon::Danger
            } else {
                SpriteIcon::Vitality
            },
            18.0,
            22.0,
            44.0,
            WHITE,
        );
        draw_ui_text(title, 74.0, 60.0, 36.0, title_color);

        let mut y = 120.0;

        if is_dead {
            draw_ui_text("The adventurer has perished.", 20.0, y, 24.0, RED);
            draw_ui_text("Their name will be remembered.", 20.0, y + 30.0, 20.0, GRAY);
            draw_ui_text(
                "[ENTER] Return to Kingdom",
                20.0,
                screen_height() - 40.0,
                20.0,
                GREEN,
            );
            return;
        }

        // Show final stats if available
        if let Some(final_hp) = self.final_hp {
            draw_icon(textures, SpriteIcon::Vitality, 18.0, y - 23.0, 24.0, WHITE);
            draw_ui_text(&format!("Final HP: {}", final_hp), 56.0, y, 20.0, GREEN);
            y += 30.0;
        } else if self.hp_lost > 0 {
            draw_icon(textures, SpriteIcon::Danger, 18.0, y - 23.0, 24.0, WHITE);
            draw_ui_text(&format!("HP Lost: -{}", self.hp_lost), 56.0, y, 20.0, RED);
            y += 30.0;
        }

        if let Some(final_stress) = self.final_stress {
            draw_icon(textures, SpriteIcon::Event, 18.0, y - 23.0, 24.0, WHITE);
            draw_ui_text(
                &format!("Final Stress: {}", final_stress),
                56.0,
                y,
                56.0,
                ORANGE,
            );
            y += 30.0;
        } else {
            draw_icon(textures, SpriteIcon::Event, 18.0, y - 23.0, 24.0, WHITE);
            draw_ui_text(
                &format!("Stress Gained: +{}", self.stress_gained),
                20.0,
                y,
                20.0,
                ORANGE,
            );
            y += 30.0;
        }

        if !self.injuries.is_empty() {
            draw_icon(textures, SpriteIcon::Danger, 18.0, y - 23.0, 24.0, WHITE);
            draw_ui_text("Injuries:", 56.0, y, 20.0, RED);
            y += 25.0;
            for injury in &self.injuries {
                draw_ui_text(&format!("  - {}", injury), 56.0, y, 18.0, PINK);
                y += 22.0;
            }
            y += 10.0;
        }

        if !self.rewards.is_empty() {
            draw_icon(textures, SpriteIcon::Gold, 18.0, y - 23.0, 24.0, WHITE);
            draw_ui_text("Rewards:", 56.0, y, 20.0, GREEN);
            y += 25.0;
            for reward in &self.rewards {
                draw_ui_text(&format!("  + {}", reward), 56.0, y, 18.0, LIME);
                y += 22.0;
            }
        }

        draw_ui_text(
            "[ENTER] Return to Kingdom",
            20.0,
            screen_height() - 40.0,
            20.0,
            GREEN,
        );
    }
}
