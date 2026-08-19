//! Top-level rendering for the base screen: background, header, and tab views.

use super::helpers::{
    candle_color, danger_color, deck_size, draw_wrapped_text, muted_text_color, panel,
    readiness_label, ready_color, text_color,
};
use super::panels::{
    draw_adventurer_row, draw_adventurer_summary, draw_command_table_background,
    draw_facility_card, draw_goals_panel, draw_header, draw_readiness_summary,
    draw_resources_panel, draw_shortcuts, draw_tabs,
};
use super::{BaseState, BaseTab, FocusArea, MAIN_H, MAIN_Y, SIDE_PAD};
use crate::kingdom::{KingdomState, Roster};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

impl BaseState {
    pub fn draw(
        &self,
        kingdom: &KingdomState,
        roster: &Roster,
        textures: &std::collections::HashMap<String, Texture2D>,
    ) {
        draw_command_table_background(textures);
        draw_header(kingdom, self.active_tab);
        draw_tabs(self.active_tab);

        if self.focus == FocusArea::PartyFormation {
            self.draw_party_formation(roster);
        } else {
            match self.active_tab {
                BaseTab::Kingdom => self.draw_kingdom_dashboard(kingdom, roster, textures),
                BaseTab::Roster => self.draw_roster_tab(kingdom, roster),
                BaseTab::Missions => self.draw_missions_tab(kingdom, roster),
                BaseTab::Buildings => self.draw_buildings_tab(kingdom),
                BaseTab::DeckTraining => self.draw_deck_training_tab(kingdom, roster),
                BaseTab::Graveyard => self.draw_graveyard_tab(roster),
                BaseTab::Journal => self.draw_journal_tab(kingdom),
            }

            self.draw_action_bar(kingdom, roster);
            self.draw_detail_panel(kingdom, roster);
        }

        if self.viewing_deck {
            self.draw_deck_overlay(roster);
        }

        draw_shortcuts();
    }

    fn draw_kingdom_dashboard(
        &self,
        kingdom: &KingdomState,
        roster: &Roster,
        textures: &std::collections::HashMap<String, Texture2D>,
    ) {
        let w = screen_width();
        let left_w = 220.0;
        let center_w = (w - 96.0) * 0.48;
        let right_x = SIDE_PAD + left_w + center_w + 24.0;
        let right_w = (w - right_x - SIDE_PAD).max(260.0);

        draw_resources_panel(kingdom, textures, SIDE_PAD, MAIN_Y, left_w, MAIN_H);
        draw_adventurer_summary(
            roster,
            self.selected_adventurer,
            SIDE_PAD + left_w + 12.0,
            MAIN_Y,
            center_w,
            MAIN_H,
            self.forming_party.leader_id(),
        );
        draw_goals_panel(kingdom, right_x, MAIN_Y, right_w, MAIN_H);
    }

    fn draw_roster_tab(&self, kingdom: &KingdomState, roster: &Roster) {
        panel(SIDE_PAD, MAIN_Y, 735.0, MAIN_H, "ADVENTURERS");
        for (i, adv) in roster.adventurers.iter().enumerate().take(5) {
            draw_adventurer_row(
                i,
                adv,
                self.selected_adventurer == Some(i),
                44.0,
                MAIN_Y + 52.0,
            );
        }

        panel(790.0, MAIN_Y, screen_width() - 814.0, MAIN_H, "READINESS");
        draw_readiness_summary(kingdom, roster, 812.0, MAIN_Y + 48.0);
    }

    fn draw_missions_tab(&self, kingdom: &KingdomState, roster: &Roster) {
        panel(SIDE_PAD, MAIN_Y, 520.0, MAIN_H, "MISSION BOARD");
        let mut y = MAIN_Y + 50.0;
        for (quest, done) in kingdom.quest_log() {
            let color = if done { ready_color() } else { text_color() };
            draw_ui_text(
                &format!("{} {}", if done { "[x]" } else { "[ ]" }, quest),
                48.0,
                y,
                18.0,
                color,
            );
            y += 32.0;
        }

        panel(565.0, MAIN_Y, screen_width() - 589.0, MAIN_H, "EMBARK PREP");
        let selected = self
            .selected_adventurer
            .and_then(|idx| roster.adventurers.get(idx))
            .or_else(|| roster.adventurers.first());
        if let Some(adv) = selected {
            draw_ui_text(
                "Selected Leader",
                592.0,
                MAIN_Y + 50.0,
                18.0,
                muted_text_color(),
            );
            draw_ui_text(&adv.name, 592.0, MAIN_Y + 82.0, 26.0, candle_color());
            draw_ui_text(
                &format!(
                    "{:?} - HP {}/{} - Stress {} - {}",
                    adv.class,
                    adv.hp,
                    adv.max_hp,
                    adv.stress,
                    readiness_label(adv)
                ),
                592.0,
                MAIN_Y + 114.0,
                18.0,
                text_color(),
            );
            draw_wrapped_text(
                "Embark opens the mission board with this hero as party leader. Add more exhausted hands only if the route demands it.",
                592.0,
                MAIN_Y + 150.0,
                screen_width() - 630.0,
                16.0,
                muted_text_color(),
            );
        }
    }

    fn draw_buildings_tab(&self, kingdom: &KingdomState) {
        panel(
            SIDE_PAD,
            MAIN_Y,
            screen_width() - SIDE_PAD * 2.0,
            MAIN_H,
            "FACILITIES",
        );
        for (i, building) in kingdom.buildings.iter().enumerate() {
            draw_facility_card(
                i,
                building,
                self.selected_building == Some(i),
                self.can_build(kingdom, i),
            );
        }
    }

    fn draw_deck_training_tab(&self, kingdom: &KingdomState, roster: &Roster) {
        panel(SIDE_PAD, MAIN_Y, 430.0, MAIN_H, "TRAINING");
        let selected = self
            .selected_adventurer
            .and_then(|idx| roster.adventurers.get(idx));
        if let Some(adv) = selected {
            draw_ui_text(&adv.name, 48.0, MAIN_Y + 54.0, 24.0, candle_color());
            draw_ui_text(
                &format!("Deck: {} cards", deck_size(adv)),
                48.0,
                MAIN_Y + 86.0,
                18.0,
                text_color(),
            );
            let foundry_status = if kingdom.has_building("foundry") {
                "Foundry built. Press U to learn the next affordable card."
            } else {
                "Build the Foundry before advanced card training."
            };
            draw_wrapped_text(
                foundry_status,
                48.0,
                MAIN_Y + 120.0,
                360.0,
                16.0,
                muted_text_color(),
            );
        }

        panel(
            480.0,
            MAIN_Y,
            screen_width() - 504.0,
            MAIN_H,
            "TRAINING NOTES",
        );
        draw_wrapped_text(
            "Training is intentionally tied to Knowledge. The command table should make upgrades feel like hard choices, not automatic shopping.",
            506.0,
            MAIN_Y + 55.0,
            screen_width() - 555.0,
            18.0,
            text_color(),
        );
    }

    fn draw_graveyard_tab(&self, roster: &Roster) {
        panel(
            SIDE_PAD,
            MAIN_Y,
            screen_width() - SIDE_PAD * 2.0,
            MAIN_H,
            "GRAVEYARD / TRAUMA LOG",
        );
        if roster.graveyard.is_empty() {
            draw_ui_text(
                "No names carved into the boards yet.",
                48.0,
                MAIN_Y + 62.0,
                20.0,
                muted_text_color(),
            );
        } else {
            for (i, adv) in roster.graveyard.iter().enumerate().take(5) {
                draw_ui_text(
                    &adv.name,
                    48.0,
                    MAIN_Y + 58.0 + (i as f32 * 32.0),
                    20.0,
                    danger_color(),
                );
            }
        }
    }

    fn draw_journal_tab(&self, kingdom: &KingdomState) {
        panel(
            SIDE_PAD,
            MAIN_Y,
            screen_width() - SIDE_PAD * 2.0,
            MAIN_H,
            "JOURNAL",
        );
        let event = kingdom
            .last_event
            .as_deref()
            .unwrap_or("No fresh omens from the frontier.");
        draw_wrapped_text(
            event,
            48.0,
            MAIN_Y + 58.0,
            screen_width() - 96.0,
            18.0,
            text_color(),
        );
    }
}
