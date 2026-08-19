//! Layout rects, card formatting, palette, and text helpers for combat.

use super::CombatState;
use crate::combat::Card;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub(super) fn clicked_down(x: f32, y: f32, w: f32, h: f32) -> bool {
    crate::ui::is_mouse_over(x, y, w, h) && is_mouse_button_pressed(MouseButton::Left)
}

pub(super) fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, Color::from_rgba(13, 11, 10, 210));
    draw_rectangle(x, y, w, 32.0, Color::from_rgba(42, 30, 18, 222));
    draw_rectangle_lines(x, y, w, h, 1.0, border_color());
    draw_ui_text(title, x + 14.0, y + 22.0, 15.0, candle_color());
}

pub(super) fn combat_card_rect(i: usize, hand_len: usize) -> (f32, f32, f32, f32) {
    let card_w = 142.0;
    let card_h = 202.0;
    let gap = 14.0;
    let count = hand_len.clamp(1, 5) as f32;
    let total_w = count * card_w + (count - 1.0) * gap;
    let x = (screen_width() - total_w) / 2.0 + (i as f32 * (card_w + gap));
    (x, screen_height() - 244.0, card_w, card_h)
}

pub(super) fn end_turn_button_rect() -> (f32, f32, f32, f32) {
    (screen_width() - 340.0, screen_height() - 58.0, 144.0, 38.0)
}

pub(super) fn retreat_button_rect() -> (f32, f32, f32, f32) {
    (screen_width() - 172.0, 18.0, 136.0, 36.0)
}

pub(super) fn hovered_card_index(hand: &[Card]) -> Option<usize> {
    for i in 0..hand.len().min(5) {
        let (x, y, w, h) = combat_card_rect(i, hand.len());
        if crate::ui::is_mouse_over(x, y, w, h) {
            return Some(i);
        }
    }
    None
}

pub(super) fn card_preview(state: &CombatState, card: &Card) -> String {
    let player = state.players.get(state.current_player_idx);
    let mut parts = Vec::new();
    for effect in &card.effects {
        match effect {
            crate::combat::CardEffect::Damage(amount) => parts.push(format!(
                "Deal {} damage to {}. Enemy HP after: {}/{}.",
                amount,
                state.enemy.name,
                (state.enemy.hp - amount).max(0),
                state.enemy.max_hp
            )),
            crate::combat::CardEffect::Block(amount) => parts.push(format!(
                "Gain {} Block. Block after: {}.",
                amount,
                player.map(|p| p.block + amount).unwrap_or(*amount)
            )),
            crate::combat::CardEffect::Heal(amount) => parts.push(format!(
                "Heal {} HP. HP after: {}/{}.",
                amount,
                player
                    .map(|p| (p.hp + amount).min(p.max_hp))
                    .unwrap_or(*amount),
                player.map(|p| p.max_hp).unwrap_or(*amount)
            )),
            crate::combat::CardEffect::ReduceStress(amount) => {
                parts.push(format!("Reduce stress by {}.", amount));
            }
            crate::combat::CardEffect::DrawCards(amount) => {
                parts.push(format!("Draw {} card(s).", amount));
            }
            crate::combat::CardEffect::GainEnergy(amount) => {
                parts.push(format!("Gain {} energy this turn.", amount));
            }
            crate::combat::CardEffect::EnemyStress(amount) => {
                parts.push(format!("Apply {} stress to the enemy.", amount));
            }
            crate::combat::CardEffect::ApplyStatus {
                effect_type,
                duration,
                ..
            } => parts.push(format!("Apply {:?} for {} turn(s).", effect_type, duration)),
            _ => parts.push(card.description.clone()),
        }
    }
    if parts.is_empty() {
        card.description.clone()
    } else {
        parts.join(" ")
    }
}

pub(super) fn intent_warning(player_name: &str, intent: &crate::combat::EnemyIntent) -> String {
    match intent {
        crate::combat::EnemyIntent::Attack(amount) => {
            format!(
                "{} will take {} damage unless blocked.",
                player_name, amount
            )
        }
        crate::combat::EnemyIntent::Block(amount) => {
            format!("{} will gain {} Block if left alone.", "Enemy", amount)
        }
        crate::combat::EnemyIntent::Buff => "Enemy is preparing a buff.".to_string(),
        crate::combat::EnemyIntent::Debuff => {
            format!("{} is about to be weakened or stressed.", player_name)
        }
        crate::combat::EnemyIntent::Unknown => "Enemy intent is hidden.".to_string(),
    }
}

pub(super) fn card_type(card: &Card) -> &'static str {
    if card.is_attack() {
        "Attack"
    } else if card
        .effects
        .iter()
        .any(|effect| matches!(effect, crate::combat::CardEffect::Block(_)))
    {
        "Guard"
    } else if card
        .effects
        .iter()
        .any(|effect| matches!(effect, crate::combat::CardEffect::Heal(_)))
    {
        "Heal"
    } else if card.effects.iter().any(|effect| {
        matches!(
            effect,
            crate::combat::CardEffect::EnemyStress(_)
                | crate::combat::CardEffect::ApplyStatus { .. }
        )
    }) {
        "Mystic"
    } else {
        "Skill"
    }
}

pub(super) fn card_accent(card: &Card) -> Color {
    match card_type(card) {
        "Attack" => Color::from_rgba(143, 61, 49, 255),
        "Guard" => Color::from_rgba(105, 128, 139, 255),
        "Heal" => Color::from_rgba(128, 160, 96, 255),
        "Mystic" => Color::from_rgba(132, 96, 158, 255),
        _ => Color::from_rgba(171, 126, 62, 255),
    }
}

pub(super) fn draw_wrapped_text(
    text: &str,
    x: f32,
    y: f32,
    max_width: f32,
    font_size: f32,
    color: Color,
) {
    let mut line = String::new();
    let mut line_y = y;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };
        if measure_ui_text(&candidate, None, font_size as u16, 1.0).width > max_width
            && !line.is_empty()
        {
            draw_ui_text(&line, x, line_y, font_size, color);
            line = word.to_string();
            line_y += font_size + 5.0;
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_ui_text(&line, x, line_y, font_size, color);
    }
}

pub(super) fn text_color() -> Color {
    Color::from_rgba(230, 221, 205, 255)
}

pub(super) fn muted_text_color() -> Color {
    Color::from_rgba(158, 145, 126, 255)
}

pub(super) fn title_color() -> Color {
    Color::from_rgba(239, 224, 190, 255)
}

pub(super) fn candle_color() -> Color {
    Color::from_rgba(207, 151, 54, 255)
}

pub(super) fn ready_color() -> Color {
    Color::from_rgba(130, 177, 101, 255)
}

pub(super) fn danger_color() -> Color {
    Color::from_rgba(168, 58, 48, 255)
}

pub(super) fn info_color() -> Color {
    Color::from_rgba(118, 151, 164, 255)
}

pub(super) fn mystery_color() -> Color {
    Color::from_rgba(138, 104, 167, 255)
}

pub(super) fn border_color() -> Color {
    Color::from_rgba(105, 76, 43, 210)
}
