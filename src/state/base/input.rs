//! Input handling and state mutation for the base screen.

use super::helpers::{
    action_buttons, adventurer_row_hit_rect, deck_close_button_rect, detail_back_button_rect,
    facility_card_rect, number_key, party_back_button_rect, party_mission_button_rect, tab_width,
};
use super::{BaseState, BaseTab, FocusArea, ACTION_Y, MAIN_Y, SIDE_PAD};
use crate::kingdom::{KingdomState, Party, Roster};
use crate::state::{MissionSelectState, StateTransition};
use macroquad::prelude::*;

impl BaseState {
    pub fn update(
        &mut self,
        kingdom: &mut KingdomState,
        roster: &mut Roster,
    ) -> Option<StateTransition> {
        if self.viewing_deck {
            if is_key_pressed(KeyCode::Escape)
                || crate::ui::was_clicked(
                    deck_close_button_rect().0,
                    deck_close_button_rect().1,
                    deck_close_button_rect().2,
                    deck_close_button_rect().3,
                )
            {
                self.viewing_deck = false;
            }
            return None;
        }

        if self.focus == FocusArea::PartyFormation {
            return self.update_party_formation(roster);
        }

        if is_key_pressed(KeyCode::Tab) {
            self.active_tab = self.active_tab.next();
            self.focus = if self.active_tab == BaseTab::Buildings {
                FocusArea::Buildings
            } else {
                FocusArea::Roster
            };
        }

        self.update_tabs();
        if let Some(transition) = self.update_action_buttons(kingdom, roster) {
            return Some(transition);
        }
        if self.update_detail_buttons() {
            return None;
        }
        self.update_selection(kingdom, roster);
        self.update_shortcuts(kingdom, roster)
    }

    fn update_party_formation(&mut self, roster: &Roster) -> Option<StateTransition> {
        for i in 0..roster.adventurers.len().min(9) {
            let key = number_key(i);
            let y = MAIN_Y + 42.0 + (i as f32 * 40.0);
            let clicked = crate::ui::was_clicked(44.0, y - 24.0, 720.0, 34.0);

            if key.is_some_and(is_key_pressed) || clicked {
                if let Some(adv) = roster.adventurers.get(i) {
                    if self.forming_party.contains(&adv.id) {
                        if self.forming_party.leader_id() != Some(adv.id.as_str()) {
                            self.forming_party.remove_member(&adv.id);
                        }
                    } else if !self.forming_party.is_full() {
                        self.forming_party.add_member(&adv.id);
                    }
                }
            }
        }

        let (mission_x, mission_y, mission_w, mission_h) = party_mission_button_rect();
        if crate::ui::was_clicked(mission_x, mission_y, mission_w, mission_h)
            && !self.forming_party.is_empty()
        {
            return Some(StateTransition::ToMissionSelect(
                MissionSelectState::for_party(self.forming_party.clone(), roster),
            ));
        }

        let (back_x, back_y, back_w, back_h) = party_back_button_rect();
        if crate::ui::was_clicked(back_x, back_y, back_w, back_h) {
            self.forming_party = Party::default();
            self.focus = FocusArea::Roster;
            self.active_tab = BaseTab::Roster;
            return None;
        }

        if is_key_pressed(KeyCode::Enter) && !self.forming_party.is_empty() {
            return Some(StateTransition::ToMissionSelect(
                MissionSelectState::for_party(self.forming_party.clone(), roster),
            ));
        }

        if is_key_pressed(KeyCode::Escape) {
            self.forming_party = Party::default();
            self.focus = FocusArea::Roster;
            self.active_tab = BaseTab::Roster;
        }

        None
    }

    fn update_tabs(&mut self) {
        let mut x = SIDE_PAD;
        for tab in BaseTab::ALL {
            let w = tab_width(tab);
            if crate::ui::was_clicked(x, 62.0, w, 28.0) {
                self.active_tab = tab;
                self.focus = if tab == BaseTab::Buildings {
                    FocusArea::Buildings
                } else {
                    FocusArea::Roster
                };
            }
            x += w + 8.0;
        }
    }

    fn update_action_buttons(
        &mut self,
        kingdom: &mut KingdomState,
        roster: &mut Roster,
    ) -> Option<StateTransition> {
        for (i, action) in action_buttons().iter().enumerate() {
            let x = SIDE_PAD + 18.0 + (i as f32 * 138.0);
            if crate::ui::was_clicked(x, ACTION_Y + 30.0, 126.0, 30.0) {
                match *action {
                    "Embark" => self.start_party_from_selected(roster),
                    "Roster" => {
                        self.active_tab = BaseTab::Roster;
                        self.focus = FocusArea::Roster;
                    }
                    "Facilities" => {
                        self.active_tab = BaseTab::Buildings;
                        self.focus = FocusArea::Buildings;
                    }
                    "Treat" => self.treat_selected_adventurer(kingdom, roster),
                    "Recruit" => {
                        if kingdom.has_building("guild_hall") {
                            return Some(StateTransition::ToRecruit);
                        }
                    }
                    "Decks" if self.selected_adventurer.is_some() => {
                        self.viewing_deck = true;
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn update_detail_buttons(&mut self) -> bool {
        if self.selected_adventurer.is_none() && self.selected_building.is_none() {
            return false;
        }

        let (x, y, w, h) = detail_back_button_rect();
        if crate::ui::was_clicked(x, y, w, h) {
            self.selected_adventurer = None;
            self.selected_building = None;
            self.viewing_deck = false;
            return true;
        }

        false
    }

    fn update_selection(&mut self, kingdom: &mut KingdomState, roster: &mut Roster) {
        match self.active_tab {
            BaseTab::Buildings => {
                let count = kingdom.buildings.len().min(9);
                for i in 0..count {
                    let key = number_key(i);
                    let (x, y, w, h) = facility_card_rect(i);
                    if key.is_some_and(is_key_pressed) || crate::ui::was_clicked(x, y, w, h) {
                        if self.selected_building == Some(i)
                            && crate::ui::was_clicked(x, y, w, h)
                            && self.can_build(kingdom, i)
                        {
                            self.try_construct_building(kingdom, i);
                        } else {
                            self.selected_building = Some(i);
                            self.selected_adventurer = None;
                        }
                    }
                }
            }
            BaseTab::Graveyard | BaseTab::Journal => {}
            _ => {
                let count = roster.adventurers.len().min(9);
                for i in 0..count {
                    let key = number_key(i);
                    let (x, y, w, h) = adventurer_row_hit_rect(self.active_tab, i);
                    let clicked = matches!(self.active_tab, BaseTab::Kingdom | BaseTab::Roster)
                        && crate::ui::was_clicked(x, y, w, h);
                    if key.is_some_and(is_key_pressed) || clicked {
                        if self.selected_adventurer == Some(i) && clicked {
                            self.start_party_from_selected(roster);
                        } else {
                            self.selected_adventurer = Some(i);
                            self.selected_building = None;
                        }
                    }
                }
            }
        }
    }

    fn update_shortcuts(
        &mut self,
        kingdom: &mut KingdomState,
        roster: &mut Roster,
    ) -> Option<StateTransition> {
        if is_key_pressed(KeyCode::M) {
            self.start_party_from_selected(roster);
        }

        if is_key_pressed(KeyCode::D) && self.selected_adventurer.is_some() {
            self.viewing_deck = true;
        }

        if is_key_pressed(KeyCode::H) || is_key_pressed(KeyCode::T) {
            self.treat_selected_adventurer(kingdom, roster);
        }

        if is_key_pressed(KeyCode::U) {
            if let Some(adv_idx) = self.selected_adventurer {
                self.try_unlock_card(kingdom, roster, adv_idx);
            }
        }

        if is_key_pressed(KeyCode::R) && kingdom.has_building("guild_hall") {
            return Some(StateTransition::ToRecruit);
        }

        if is_key_pressed(KeyCode::Enter) && self.active_tab == BaseTab::Buildings {
            if let Some(idx) = self.selected_building {
                self.try_construct_building(kingdom, idx);
            }
        }

        None
    }

    fn start_party_from_selected(&mut self, roster: &Roster) {
        let idx = self.selected_adventurer.unwrap_or(0);
        if let Some(adventurer) = roster.adventurers.get(idx) {
            self.forming_party = Party::with_leader(&adventurer.id);
            self.focus = FocusArea::PartyFormation;
            self.active_tab = BaseTab::Roster;
        }
    }

    fn treat_selected_adventurer(&mut self, kingdom: &mut KingdomState, roster: &mut Roster) {
        let Some(idx) = self.selected_adventurer else {
            return;
        };
        let Some(adv) = roster.adventurers.get_mut(idx) else {
            return;
        };

        if kingdom.has_building("infirmary") && adv.hp < adv.max_hp && kingdom.stats.supplies >= 10
        {
            adv.heal(10);
            kingdom.stats.supplies -= 10;
            return;
        }

        if kingdom.has_building("chapel") && adv.stress > 0 && kingdom.stats.supplies >= 10 {
            adv.reduce_stress(20);
            kingdom.stats.supplies -= 10;
        }
    }

    pub(super) fn can_build(&self, kingdom: &KingdomState, idx: usize) -> bool {
        kingdom.buildings.get(idx).is_some_and(|building| {
            !building.built
                && kingdom.stats.gold >= building.cost_gold
                && kingdom.stats.supplies >= building.cost_supplies
        })
    }

    /// Try to construct a building at the given index.
    fn try_construct_building(&mut self, kingdom: &mut KingdomState, idx: usize) {
        if !self.can_build(kingdom, idx) {
            return;
        }

        if let Some(building) = kingdom.buildings.get_mut(idx) {
            kingdom.stats.gold -= building.cost_gold;
            kingdom.stats.supplies -= building.cost_supplies;
            building.built = true;
            building.level = 1;
            if building.id == "citadel" {
                kingdom.game_won = true;
            }
        }
    }

    fn try_unlock_card(&mut self, kingdom: &mut KingdomState, roster: &mut Roster, adv_idx: usize) {
        if !kingdom.has_building("foundry") {
            return;
        }

        let Some(adv) = roster.adventurers.get(adv_idx) else {
            return;
        };

        let class_name = format!("{:?}", adv.class);
        let known_cards = adv.deck_additions.clone();
        let Ok(all_cards) = crate::data::cards::CardData::load_all() else {
            return;
        };

        let mut candidates: Vec<_> = all_cards
            .iter()
            .filter(|card| {
                card.class_matches(&class_name)
                    && card.is_unlockable()
                    && !known_cards.iter().any(|id| id == &card.id)
            })
            .collect();
        candidates.sort_by_key(|card| card.required_knowledge);

        if let Some(card) = candidates
            .into_iter()
            .find(|card| kingdom.stats.knowledge >= card.required_knowledge)
        {
            kingdom.stats.knowledge -= card.required_knowledge;
            if let Some(adv) = roster.adventurers.get_mut(adv_idx) {
                adv.deck_additions.push(card.id.clone());
            }
        }
    }
}
