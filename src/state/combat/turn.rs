//! Turn flow: input handling, card play, and end-of-turn resolution.

use super::helpers::{clicked_down, combat_card_rect, end_turn_button_rect, retreat_button_rect};
use super::{CombatState, MissionContext};
use crate::combat::{Card, ResolutionDelta, Unit};
use crate::kingdom::{PartyMemberState, TraumaType};
use crate::state::{MissionState, ResultState, StateTransition};
use macroquad::prelude::*;

impl CombatState {
    pub fn update(&mut self) -> Option<StateTransition> {
        self.tick_feedback();

        let (retreat_x, retreat_y, retreat_w, retreat_h) = retreat_button_rect();
        if clicked_down(retreat_x, retreat_y, retreat_w, retreat_h)
            || is_key_pressed(KeyCode::Escape)
        {
            return Some(StateTransition::ToBase);
        }

        // Card selection with number keys OR mouse click
        for i in 0..self.hand.len().min(5) {
            let key = match i {
                0 => KeyCode::Key1,
                1 => KeyCode::Key2,
                2 => KeyCode::Key3,
                3 => KeyCode::Key4,
                4 => KeyCode::Key5,
                _ => continue,
            };

            // Keyboard selection
            if is_key_pressed(key) {
                self.select_card(i);
            }

            // Mouse click on card
            let (card_x, card_y, card_width, card_height) = combat_card_rect(i, self.hand.len());
            if clicked_down(card_x, card_y, card_width, card_height) {
                if self.selected_card == Some(i) {
                    // Clicking already selected card = play it
                    self.try_play_selected_card();
                } else {
                    self.select_card(i);
                }
            }
        }

        // Play selected card with Enter
        if is_key_pressed(KeyCode::Enter) {
            self.try_play_selected_card();
        }

        // End turn with E key or button click (button drawn in draw())
        if is_key_pressed(KeyCode::E) {
            self.end_turn();
        }
        // End Turn button bounds
        let (end_btn_x, end_btn_y, end_btn_w, end_btn_h) = end_turn_button_rect();
        if clicked_down(end_btn_x, end_btn_y, end_btn_w, end_btn_h) {
            self.end_turn();
        }

        // Check win/lose
        if self.enemy.hp <= 0 {
            // Victory - return to mission if we came from one
            if let Some(ctx) = &self.return_mission {
                // Update party member states with current HP/stress
                let updated_members: Vec<PartyMemberState> = self
                    .players
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let orig = ctx.party_members.get(i);
                        PartyMemberState {
                            id: orig.map(|m| m.id.clone()).unwrap_or_default(),
                            name: p.name.clone(),
                            hp: p.hp,
                            max_hp: p.max_hp,
                            stress: p.stress,
                            image_path: p.image_path.clone(),
                            class_name: orig
                                .map(|m| m.class_name.clone())
                                .unwrap_or_else(|| "Soldier".to_string()),
                            deck_additions: orig
                                .map(|m| m.deck_additions.clone())
                                .unwrap_or_default(),
                            traumas: p.traumas.clone(),
                            resolve_state: p.resolve_state.clone(),
                        }
                    })
                    .collect();

                let mission_state =
                    MissionState::from_mission_with_party(ctx.mission.clone(), updated_members)
                        .with_node(ctx.current_node)
                        .with_map_nodes(ctx.map_nodes.clone())
                        .with_visited(ctx.visited_nodes.clone());
                return Some(StateTransition::ToMission(mission_state));
            } else {
                // Not from mission - just show simple victory
                let leader_id = self.players.first().map(|p| p.name.as_str()).unwrap_or("");
                return Some(StateTransition::ToResults(ResultState::victory_for(
                    leader_id,
                )));
            }
        }

        // Check if all players are dead
        let all_dead = self.players.iter().all(|p| p.hp <= 0);
        if all_dead {
            // Defeat - always go to results
            if let Some(ctx) = &self.return_mission {
                let final_members = self.party_members_from_players(ctx);
                let results = ResultState::defeat_for_mission(&ctx.mission, &final_members);
                return Some(StateTransition::ToResults(results));
            } else {
                let leader_id = self.players.first().map(|p| p.name.as_str()).unwrap_or("");
                let results = ResultState::defeat_for(leader_id);
                return Some(StateTransition::ToResults(results));
            }
        }

        None
    }

    /// Try to play the currently selected card
    fn try_play_selected_card(&mut self) {
        let Some(card_idx) = self.selected_card else {
            self.set_feedback("Select a card first.".to_string());
            return;
        };
        if card_idx >= self.hand.len() || self.current_player_idx >= self.players.len() {
            self.selected_card = None;
            self.set_feedback("That card is no longer available.".to_string());
            return;
        }

        let card = self.hand[card_idx].clone();
        let effective_cost = self.effective_card_cost(&card);
        let can_afford = effective_cost <= self.energy;
        let attack_blocked = card.is_attack() && self.resolver.turn_mods.attacks_disabled;

        if !can_afford {
            self.set_feedback(format!(
                "{} needs {} energy. You have {}.",
                card.name, effective_cost, self.energy
            ));
            return;
        }
        if attack_blocked {
            self.set_feedback(format!("{} is blocked this turn.", card.name));
            return;
        }

        if self.fearful_fumble(&card) {
            self.selected_card = None;
            self.set_feedback(format!("{} fumbled.", card.name));
            return;
        }

        let player_name = self.players[self.current_player_idx].name.clone();
        let card_name = card.name.clone();
        let effects = card.effects.clone();
        self.energy -= effective_cost;
        self.resolver
            .log
            .push(format!("{} plays {}", player_name, card_name));

        let player = &mut self.players[self.current_player_idx];
        for effect in effects {
            self.resolver.resolve(&effect, player, &mut self.enemy);
        }

        self.apply_card_turn_modifiers();
        self.hand.remove(card_idx);
        self.selected_card = None;
        let feedback = self
            .resolver
            .recent_resolutions
            .last()
            .map(ResolutionDelta::summary)
            .unwrap_or_else(|| format!("{} played.", card_name));
        self.set_feedback(feedback);
    }

    fn select_card(&mut self, idx: usize) {
        if let Some(card) = self.hand.get(idx) {
            self.selected_card = Some(idx);
            self.set_feedback(format!(
                "{} selected. Click again or press Enter to play.",
                card.name
            ));
        }
    }

    fn tick_feedback(&mut self) {
        if let Some((_, time)) = &mut self.feedback {
            *time -= get_frame_time();
            if *time <= 0.0 {
                self.feedback = None;
            }
        }
    }

    fn set_feedback(&mut self, text: String) {
        self.feedback = Some((text, 2.0));
    }

    pub(super) fn effective_card_cost(&self, card: &Card) -> i32 {
        let Some(player) = self.players.get(self.current_player_idx) else {
            return card.cost;
        };

        let mut cost = card.cost;
        if player
            .traumas
            .iter()
            .any(|t| t.trauma_type == TraumaType::Broken)
        {
            cost += 1;
        }
        if card
            .effects
            .iter()
            .any(|e| matches!(e, crate::combat::CardEffect::Block(_)))
            && player
                .traumas
                .iter()
                .any(|t| t.trauma_type == TraumaType::Paranoid)
        {
            cost += 1;
        }
        cost
    }

    fn fearful_fumble(&mut self, card: &Card) -> bool {
        if !card.is_attack() {
            return false;
        }
        let Some(player) = self.players.get_mut(self.current_player_idx) else {
            return false;
        };
        if player
            .traumas
            .iter()
            .any(|t| t.trauma_type == TraumaType::Fearful)
            && macroquad_toolkit::rng::chance(0.15)
        {
            player.add_stress(3);
            self.resolver
                .log
                .push(format!("{} hesitates and loses the attack", player.name));
            true
        } else {
            false
        }
    }

    fn apply_card_turn_modifiers(&mut self) {
        if self.resolver.turn_mods.energy_to_gain > 0 {
            self.energy += self.resolver.turn_mods.energy_to_gain;
            self.resolver.turn_mods.energy_to_gain = 0;
        }

        if self.resolver.turn_mods.cards_to_draw > 0 {
            let count = self.resolver.turn_mods.cards_to_draw;
            self.draw_extra_cards(count);
            self.resolver.turn_mods.cards_to_draw = 0;
        }
    }

    fn draw_extra_cards(&mut self, count: i32) {
        let deck = self.deck_for_current_player();
        let mut remaining = count;
        for card in deck {
            if self.hand.len() >= 7 || remaining <= 0 {
                break;
            }
            if !self.hand.iter().any(|c| c.id == card.id) {
                self.hand.push(card);
                remaining -= 1;
            }
        }
    }

    fn deck_for_current_player(&self) -> Vec<Card> {
        let (class_name, deck_additions) = self
            .return_mission
            .as_ref()
            .and_then(|ctx| ctx.party_members.get(self.current_player_idx))
            .map(|m| (m.class_name.as_str(), m.deck_additions.as_slice()))
            .unwrap_or(("Soldier", &[]));
        Card::load_deck_for_class(class_name, deck_additions)
    }

    fn party_members_from_players(&self, ctx: &MissionContext) -> Vec<PartyMemberState> {
        self.players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let orig = ctx.party_members.get(i);
                PartyMemberState {
                    id: orig.map(|m| m.id.clone()).unwrap_or_default(),
                    name: p.name.clone(),
                    hp: p.hp,
                    max_hp: p.max_hp,
                    stress: p.stress,
                    image_path: p.image_path.clone(),
                    class_name: orig
                        .map(|m| m.class_name.clone())
                        .unwrap_or_else(|| "Soldier".to_string()),
                    deck_additions: orig.map(|m| m.deck_additions.clone()).unwrap_or_default(),
                    traumas: p.traumas.clone(),
                    resolve_state: p.resolve_state.clone(),
                }
            })
            .collect()
    }

    fn end_turn(&mut self) {
        self.resolver.clear_recent();
        let actor_name = self
            .players
            .get(self.current_player_idx)
            .map(|player| player.name.clone())
            .unwrap_or_else(|| "Adventurer".to_string());
        let old_intent = self.enemy.intent.description();

        // Current player status tick. Block protects the player from the
        // enemy action and is cleared only after that action resolves.
        if let Some(player) = self.players.get_mut(self.current_player_idx) {
            player.tick_statuses();
        }

        let enemy_before_action = self.enemy.clone();
        let player_before_enemy = self
            .players
            .get(self.current_player_idx)
            .cloned()
            .unwrap_or_else(|| Unit::new_player("Adventurer", 1));

        // Enemy Action
        let (dmg, stress) = self.enemy.execute_intent();
        let enemy_acted = dmg > 0 || stress > 0;
        let enemy_after_action = self.enemy.clone();
        if enemy_after_action.block != enemy_before_action.block
            || enemy_after_action.base_damage != enemy_before_action.base_damage
            || enemy_after_action.statuses.len() != enemy_before_action.statuses.len()
        {
            self.resolver
                .record_external(ResolutionDelta::from_snapshots(
                    &self.enemy.name,
                    &self.enemy.name,
                    old_intent.clone(),
                    &enemy_before_action,
                    &enemy_after_action,
                    0,
                ));
        }

        // Apply damage to current player
        let mut actual_damage = 0;
        let mut blocked_damage = 0;
        if dmg > 0 {
            if let Some(player) = self.players.get_mut(self.current_player_idx) {
                let resolution = player.take_damage_detailed(dmg);
                actual_damage = resolution.actual;
                blocked_damage = resolution.blocked;
                if self.current_player_idx < self.damage_taken.len() {
                    self.damage_taken[self.current_player_idx] += actual_damage;
                }
            }
        }

        // Apply stress with resistance (uses resolver's turn mods)
        let base_stress = 2 + stress;
        if let Some(player) = self.players.get_mut(self.current_player_idx) {
            self.resolver.apply_stress_to_player(player, base_stress);
            if self.current_player_idx < self.stress_gained.len() {
                self.stress_gained[self.current_player_idx] += base_stress;
            }
        }

        if let Some(player) = self.players.get_mut(self.current_player_idx) {
            player.block = 0;
            if actual_damage > 0 || base_stress > 0 || blocked_damage > 0 {
                self.resolver
                    .record_external(ResolutionDelta::from_snapshots(
                        &self.enemy.name,
                        &player.name,
                        old_intent.clone(),
                        &player_before_enemy,
                        player,
                        blocked_damage,
                    ));
            }
        }

        // Enemy status tick
        self.enemy.tick_statuses();
        self.enemy.block = 0;

        // Reset turn modifiers and track enemy action for next turn
        self.resolver.end_turn(enemy_acted);

        // Cycle to next living party member
        if self.players.len() > 1 {
            let start_idx = self.current_player_idx;
            loop {
                self.current_player_idx = (self.current_player_idx + 1) % self.players.len();
                // Stop if alive or back to start
                if self.players[self.current_player_idx].hp > 0
                    || self.current_player_idx == start_idx
                {
                    break;
                }
            }
        }

        // Next Turn
        self.turn += 1;
        self.energy = self.max_energy + self.resolver.turn_mods.start_turn();

        // Draw class-appropriate cards for current player
        self.hand = self.deck_for_current_player().into_iter().take(5).collect();

        // Roll new enemy intent for next turn
        self.enemy.roll_intent(self.turn);

        self.resolver.log.push(format!(
            "End turn: {} resolved. {} took {} damage and {} stress.",
            old_intent, actor_name, actual_damage, base_stress
        ));
        self.resolver.log.push(format!(
            "Turn {} begins. Enemy intent: {}.",
            self.turn,
            self.enemy.intent.description()
        ));
        self.set_feedback(format!("Turn {} begins.", self.turn));
    }
}
