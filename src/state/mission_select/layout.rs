//! Shared mission-selection geometry, palette, and text helpers.

use crate::ui::{draw_background as draw_scene_background, BackgroundArt};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use std::collections::HashMap;

pub(super) const HEADER_H: f32 = 78.0;
pub(super) const SIDE_PAD: f32 = 24.0;
pub(super) const PANEL_Y: f32 = 92.0;
pub(super) const PANEL_H: f32 = 458.0;
pub(super) const PARTY_X: f32 = 24.0;
pub(super) const PARTY_W: f32 = 306.0;
pub(super) const BOARD_X: f32 = 350.0;
pub(super) const BOARD_W: f32 = 430.0;
pub(super) const DETAIL_X: f32 = 800.0;
pub(super) const DETAIL_W: f32 = 440.0;
pub(super) const CARD_H: f32 = 58.0;
pub(super) const CARD_GAP: f32 = 6.0;

pub(super) fn draw_background(textures: &HashMap<String, Texture2D>) {
    draw_scene_background(
        textures,
        BackgroundArt::MissionBoard,
        Color::from_rgba(5, 4, 4, 190),
    );
}

pub(super) fn mission_card_rect(i: usize) -> (f32, f32, f32, f32) {
    (
        BOARD_X + 14.0,
        PANEL_Y + 42.0 + i as f32 * (CARD_H + CARD_GAP),
        BOARD_W - 28.0,
        CARD_H,
    )
}

pub(super) fn action_button_rect(index: usize) -> (f32, f32, f32, f32) {
    match index {
        0 => (DETAIL_X, 574.0, 142.0, 38.0),
        1 => (958.0, 574.0, 126.0, 38.0),
        _ => (1100.0, 574.0, 126.0, 38.0),
    }
}

pub(super) fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, Color::from_rgba(13, 11, 10, 210));
    draw_rectangle(x, y, w, 32.0, Color::from_rgba(42, 30, 18, 222));
    draw_rectangle_lines(x, y, w, h, 1.0, border_color());
    draw_ui_text(title, x + 14.0, y + 22.0, 15.0, candle_color());
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

pub(super) fn reward_color() -> Color {
    Color::from_rgba(224, 180, 72, 255)
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
