//! Title menu state for starting, continuing, or leaving the campaign.

use super::StateTransition;
use crate::save::SaveData;
use crate::ui::{draw_background, BackgroundArt};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use std::collections::HashMap;

const BUTTON_W: f32 = 280.0;
const BUTTON_H: f32 = 48.0;
const BUTTON_GAP: f32 = 12.0;

pub struct MenuState {
    pub has_save: bool,
    pub notice: Option<String>,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            has_save: SaveData::exists(&SaveData::default_path()),
            notice: None,
        }
    }

    pub fn with_notice(notice: impl Into<String>) -> Self {
        let mut state = Self::new();
        state.notice = Some(notice.into());
        state
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        let new_game = button_rect(0);
        let continue_game = button_rect(1);
        let exit_game = button_rect(2);

        if crate::ui::was_clicked(new_game.0, new_game.1, new_game.2, new_game.3) {
            return Some(StateTransition::StartNewGame);
        }
        if self.has_save
            && crate::ui::was_clicked(
                continue_game.0,
                continue_game.1,
                continue_game.2,
                continue_game.3,
            )
        {
            return Some(StateTransition::ContinueGame);
        }
        if crate::ui::was_clicked(exit_game.0, exit_game.1, exit_game.2, exit_game.3) {
            return Some(StateTransition::Quit);
        }

        None
    }

    pub fn draw(&self, textures: &HashMap<String, Texture2D>) {
        draw_background(
            textures,
            BackgroundArt::Base,
            Color::from_rgba(5, 4, 4, 178),
        );
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::from_rgba(7, 5, 4, 150),
        );

        let title = "FRONTIER KINGDOM";
        let title_width = measure_ui_text(title, None, 46, 1.0).width;
        draw_ui_text(
            title,
            (screen_width() - title_width) / 2.0,
            170.0,
            46.0,
            title_color(),
        );
        let subtitle = "Every road asks for a name, a purse, and a willing hand.";
        let subtitle_width = measure_ui_text(subtitle, None, 18, 1.0).width;
        draw_ui_text(
            subtitle,
            (screen_width() - subtitle_width) / 2.0,
            205.0,
            18.0,
            muted_color(),
        );

        for (index, label) in ["Start New Game", "Continue", "Exit"].iter().enumerate() {
            let rect = button_rect(index);
            let enabled = index != 1 || self.has_save;
            draw_menu_button(label, rect, enabled);
        }

        if let Some(notice) = &self.notice {
            let width = measure_ui_text(notice, None, 17, 1.0).width;
            draw_ui_text(
                notice,
                (screen_width() - width) / 2.0,
                500.0,
                17.0,
                danger_color(),
            );
        }
        draw_ui_text(
            "Tap a button to begin",
            24.0,
            screen_height() - 28.0,
            15.0,
            muted_color(),
        );
    }
}

impl Default for MenuState {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn button_rect(index: usize) -> (f32, f32, f32, f32) {
    let x = (screen_width() - BUTTON_W) / 2.0;
    (
        x,
        260.0 + index as f32 * (BUTTON_H + BUTTON_GAP),
        BUTTON_W,
        BUTTON_H,
    )
}

fn draw_menu_button(label: &str, rect: (f32, f32, f32, f32), enabled: bool) {
    let hovered = enabled && crate::ui::is_mouse_over(rect.0, rect.1, rect.2, rect.3);
    let fill = if !enabled {
        Color::from_rgba(26, 23, 21, 220)
    } else if hovered {
        Color::from_rgba(112, 77, 31, 245)
    } else {
        Color::from_rgba(68, 48, 27, 238)
    };
    draw_rectangle(rect.0, rect.1, rect.2, rect.3, fill);
    draw_rectangle_lines(
        rect.0,
        rect.1,
        rect.2,
        rect.3,
        if hovered { 2.0 } else { 1.0 },
        if enabled {
            candle_color()
        } else {
            border_color()
        },
    );
    let width = measure_ui_text(label, None, 20, 1.0).width;
    draw_ui_text(
        label,
        rect.0 + (rect.2 - width) / 2.0,
        rect.1 + 31.0,
        20.0,
        if enabled {
            text_color()
        } else {
            border_color()
        },
    );
}

fn title_color() -> Color {
    Color::from_rgba(239, 224, 190, 255)
}

fn text_color() -> Color {
    Color::from_rgba(230, 221, 205, 255)
}

fn muted_color() -> Color {
    Color::from_rgba(164, 153, 130, 255)
}

fn danger_color() -> Color {
    Color::from_rgba(168, 58, 48, 255)
}

fn candle_color() -> Color {
    Color::from_rgba(214, 154, 62, 255)
}

fn border_color() -> Color {
    Color::from_rgba(108, 82, 51, 210)
}
