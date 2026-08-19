//! Full-screen overlays: party formation and the deck viewer.

use super::helpers::{
    candle_color, card_accent, card_type, deck_close_button_rect, draw_wrapped_text,
    muted_text_color, panel, party_back_button_rect, party_mission_button_rect, text_color,
};
use super::panels::draw_action_button;
use super::{BaseState, MAIN_Y, SIDE_PAD};
use crate::kingdom::Roster;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

impl BaseState {
    pub(super) fn draw_party_formation(&self, roster: &Roster) {
        panel(
            SIDE_PAD,
            MAIN_Y,
            screen_width() - SIDE_PAD * 2.0,
            475.0,
            "PARTY / ADVENTURERS",
        );
        draw_ui_text(
            &format!(
                "Choose the expedition party ({}/{})",
                self.forming_party.size(),
                crate::kingdom::MAX_PARTY_SIZE
            ),
            48.0,
            MAIN_Y + 48.0,
            22.0,
            candle_color(),
        );

        for (i, adv) in roster.adventurers.iter().enumerate().take(9) {
            let y = MAIN_Y + 90.0 + (i as f32 * 40.0);
            let in_party = self.forming_party.contains(&adv.id);
            let leader = self.forming_party.leader_id() == Some(adv.id.as_str());
            let marker = if leader {
                "LEADER"
            } else if in_party {
                "ASSIGNED"
            } else {
                ""
            };
            let color = if in_party {
                candle_color()
            } else {
                text_color()
            };
            draw_ui_text(
                &format!(
                    "[{}] {:<18} {:<9} HP {}/{}  Stress {}  {}",
                    i + 1,
                    adv.name,
                    format!("{:?}", adv.class),
                    adv.hp,
                    adv.max_hp,
                    adv.stress,
                    marker
                ),
                48.0,
                y,
                18.0,
                color,
            );
        }

        draw_ui_text(
            "Tap party members to add or remove them",
            48.0,
            screen_height() - 38.0,
            18.0,
            muted_text_color(),
        );

        let (mission_x, mission_y, mission_w, mission_h) = party_mission_button_rect();
        draw_action_button(
            "Open Mission Board",
            mission_x,
            mission_y,
            mission_w,
            mission_h,
            !self.forming_party.is_empty(),
        );
        let (back_x, back_y, back_w, back_h) = party_back_button_rect();
        draw_action_button("Back to Roster", back_x, back_y, back_w, back_h, true);
    }

    pub(super) fn draw_deck_overlay(&self, roster: &Roster) {
        let Some(adv) = self
            .selected_adventurer
            .and_then(|idx| roster.adventurers.get(idx))
        else {
            return;
        };

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(4, 3, 2, 235),
        );
        panel(
            34.0,
            36.0,
            screen_width() - 68.0,
            screen_height() - 72.0,
            "DECK / TRAINING",
        );
        draw_ui_text(
            &format!("{}'s Deck", adv.name),
            62.0,
            88.0,
            34.0,
            candle_color(),
        );
        let (close_x, close_y, close_w, close_h) = deck_close_button_rect();
        draw_action_button("Close", close_x, close_y, close_w, close_h, true);

        let class_name = format!("{:?}", adv.class);
        let deck = crate::combat::Card::load_deck_for_class(&class_name, &adv.deck_additions);
        let start_x = 62.0;
        let start_y = 122.0;
        let card_w = 132.0;
        let card_h = 164.0;
        let gap = 16.0;
        let cols = ((screen_width() - 124.0) / (card_w + gap)).max(1.0) as i32;

        for (i, card) in deck.iter().enumerate() {
            let row = (i as i32) / cols;
            let col = (i as i32) % cols;
            let x = start_x + (col as f32 * (card_w + gap));
            let y = start_y + (row as f32 * (card_h + gap));
            draw_card_frame(card, x, y, card_w, card_h, false);
        }
    }
}

pub(super) fn draw_card_frame(
    card: &crate::combat::Card,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    selected: bool,
) {
    let accent = card_accent(card);
    draw_rectangle(x, y, w, h, Color::from_rgba(19, 17, 15, 245));
    draw_rectangle_lines(x, y, w, h, if selected { 3.0 } else { 2.0 }, accent);
    draw_rectangle(
        x + 6.0,
        y + 6.0,
        w - 12.0,
        24.0,
        Color::from_rgba(38, 33, 27, 240),
    );
    draw_ui_text(
        &format!("{}", card.cost),
        x + 12.0,
        y + 24.0,
        18.0,
        candle_color(),
    );
    draw_ui_text(
        card_type(card),
        x + w - 62.0,
        y + 24.0,
        14.0,
        muted_text_color(),
    );
    draw_rectangle(
        x + 8.0,
        y + 36.0,
        w - 16.0,
        h * 0.42,
        Color::from_rgba(43, 40, 38, 255),
    );
    draw_ui_text(&card.name, x + 10.0, y + h - 52.0, 15.0, text_color());
    draw_wrapped_text(
        &card.description,
        x + 10.0,
        y + h - 32.0,
        w - 20.0,
        12.0,
        muted_text_color(),
    );
}
