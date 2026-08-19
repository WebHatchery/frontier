//! UI modules - immediate mode, stateless rendering with mouse support

mod art;

use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

// Import toolkit utilities
pub use art::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad_toolkit::input::{is_hovered_rect, was_clicked_rect, was_pressed_rect};
pub use macroquad_toolkit::input::{is_mouse_over, was_clicked};

/// Draw a button and return true if clicked
///
/// Note: Frontier uses a different parameter order (text first) than other games
#[allow(dead_code)]
pub fn button(text: &str, x: f32, y: f32, w: f32, h: f32) -> bool {
    // Frontier uses a specific style
    let style = macroquad_toolkit::ui::ButtonStyle {
        normal: Color::from_rgba(40, 50, 50, 255),
        hovered: Color::from_rgba(60, 80, 60, 255),
        pressed: Color::from_rgba(80, 100, 80, 255),
        border: GRAY,
        text_color: LIGHTGRAY,
        disabled: Color::new(0.1, 0.1, 0.1, 1.0),
    };

    // Frontier uses on_release behavior
    macroquad_toolkit::ui::button_on_release(x, y, w, h, text, &style)
}

/// Draw a button with custom colors
#[allow(dead_code)]
pub fn button_colored(text: &str, x: f32, y: f32, w: f32, h: f32, base_color: Color) -> bool {
    // Use the toolkit helper but respecting frontier's argument order
    macroquad_toolkit::ui::colored_button(x, y, w, h, text, base_color)
}

/// Show a compact tooltip when the mouse is over a keyword label.
#[allow(dead_code)]
pub fn keyword_tooltip(keyword: &str, x: f32, y: f32, w: f32, h: f32) {
    if is_mouse_over(x, y, w, h) {
        if let Some(definition) = keyword_definition(keyword) {
            draw_tooltip(keyword, definition);
        }
    }
}

/// Show a card details tooltip with definitions for any known keywords in the text.
pub fn card_tooltip(card_name: &str, description: &str) {
    let mut body = description.to_string();
    let mut definitions = Vec::new();

    for keyword in [
        "Stress",
        "Block",
        "Vulnerable",
        "Weak",
        "Stun",
        "Poison",
        "Burn",
        "Regen",
        "Strength",
        "Energy",
    ] {
        if description.contains(keyword) {
            if let Some(definition) = keyword_definition(keyword) {
                definitions.push(format!("{}: {}", keyword, definition));
            }
        }
    }

    if !definitions.is_empty() {
        body.push('\n');
        body.push_str(&definitions.join("\n"));
    }

    draw_tooltip(card_name, &body);
}

fn keyword_definition(keyword: &str) -> Option<&'static str> {
    match keyword {
        "Stress" => Some("Persistent pressure. At 100, a Resolve Check can cause Virtue or Affliction; at 200, Heart Attack damage is applied."),
        "Block" => Some("Temporary protection that absorbs incoming damage before HP is lost."),
        "Vulnerable" => Some("The affected unit takes 50% more damage."),
        "Weak" => Some("The affected unit deals 25% less damage."),
        "Stun" => Some("The affected unit skips its next action while the effect lasts."),
        "Poison" => Some("Damage over time applied at the end of the affected unit's turn."),
        "Burn" => Some("Damage over time applied at the end of the affected unit's turn."),
        "Regen" => Some("Healing over time applied at the end of the affected unit's turn."),
        "Strength" => Some("Bonus outgoing damage while the status lasts."),
        "Energy" => Some("Resource spent to play cards. It refreshes at the start of each turn."),
        "Resolve" => Some("A high-stress check that creates either a Virtuous boost or an Afflicted penalty."),
        _ => None,
    }
}

fn draw_tooltip(title: &str, body: &str) {
    let (mx, my) = mouse_position();
    let width = 320.0;
    let x = (mx + 18.0).min(screen_width() - width - 12.0).max(12.0);
    let mut y = (my + 18.0).min(screen_height() - 190.0).max(12.0);

    let mut lines = Vec::new();
    for paragraph in body.split('\n') {
        wrap_text(paragraph, 42, &mut lines);
    }

    let height = 46.0 + (lines.len() as f32 * 18.0);
    if y + height > screen_height() - 12.0 {
        y = (screen_height() - height - 12.0).max(12.0);
    }

    draw_rectangle(x, y, width, height, Color::from_rgba(18, 18, 24, 245));
    draw_rectangle_lines(x, y, width, height, 1.0, YELLOW);
    draw_ui_text(title, x + 12.0, y + 24.0, 18.0, WHITE);

    let mut line_y = y + 48.0;
    for line in lines {
        draw_ui_text(&line, x + 12.0, line_y, 14.0, LIGHTGRAY);
        line_y += 18.0;
    }
}

fn wrap_text(text: &str, max_chars: usize, output: &mut Vec<String>) {
    if text.trim().is_empty() {
        output.push(String::new());
        return;
    }

    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + word.len() + 1 > max_chars {
            output.push(line);
            line = String::new();
        }

        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }

    if !line.is_empty() {
        output.push(line);
    }
}

/// A clickable card/panel - returns true if clicked, also handles hover highlighting
#[allow(dead_code)]
pub struct ClickableRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[allow(dead_code)]
impl ClickableRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.h)
    }

    pub fn is_hovered(&self) -> bool {
        is_hovered_rect(self.rect())
    }

    pub fn was_clicked(&self) -> bool {
        was_clicked_rect(self.rect())
    }

    pub fn was_pressed(&self) -> bool {
        was_pressed_rect(self.rect())
    }
}
