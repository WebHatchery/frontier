//! Shared layout rects, palette, and formatting helpers for the base screen.

use super::{BaseTab, DETAIL_Y, MAIN_Y, SIDE_PAD};
use crate::kingdom::{Adventurer, KingdomState, Roster};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub(super) fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, Color::from_rgba(16, 13, 11, 218));
    draw_rectangle_lines(x, y, w, h, 1.0, border_color());
    draw_rectangle(x, y, w, 32.0, Color::from_rgba(43, 30, 17, 205));
    draw_ui_text(title, x + 14.0, y + 23.0, 16.0, candle_color());
}

pub(super) fn draw_wrapped_text(
    text: &str,
    x: f32,
    y: f32,
    max_width: f32,
    font_size: f32,
    color: Color,
) {
    let mut current = String::new();
    let mut line_y = y;
    for word in text.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current, word)
        };
        if measure_ui_text(&candidate, None, font_size as u16, 1.0).width > max_width
            && !current.is_empty()
        {
            draw_ui_text(&current, x, line_y, font_size, color);
            current = word.to_string();
            line_y += font_size + 5.0;
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() {
        draw_ui_text(&current, x, line_y, font_size, color);
    }
}

pub(super) fn number_key(i: usize) -> Option<KeyCode> {
    match i {
        0 => Some(KeyCode::Key1),
        1 => Some(KeyCode::Key2),
        2 => Some(KeyCode::Key3),
        3 => Some(KeyCode::Key4),
        4 => Some(KeyCode::Key5),
        5 => Some(KeyCode::Key6),
        6 => Some(KeyCode::Key7),
        7 => Some(KeyCode::Key8),
        8 => Some(KeyCode::Key9),
        _ => None,
    }
}

pub(super) fn tab_width(tab: BaseTab) -> f32 {
    match tab {
        BaseTab::DeckTraining => 126.0,
        BaseTab::Graveyard => 92.0,
        BaseTab::Buildings => 88.0,
        _ => 78.0,
    }
}

pub(super) fn action_buttons() -> [&'static str; 6] {
    ["Embark", "Treat", "Recruit", "Decks", "Save", "Load"]
}

pub(super) fn action_button_rect(index: usize) -> (f32, f32, f32, f32) {
    (
        SIDE_PAD + 18.0 + (index as f32 * 138.0),
        super::ACTION_Y + 30.0,
        126.0,
        30.0,
    )
}

pub(super) fn action_enabled(
    action: &str,
    kingdom: &KingdomState,
    roster: &Roster,
    selected_adventurer: Option<usize>,
) -> bool {
    match action {
        "Embark" => !roster.adventurers.is_empty(),
        "Treat" | "Decks" => selected_adventurer
            .and_then(|idx| roster.adventurers.get(idx))
            .is_some(),
        "Recruit" => kingdom.has_building("guild_hall"),
        "Save" | "Load" => true,
        _ => true,
    }
}

pub(super) fn adventurer_row_rect(i: usize) -> (f32, f32, f32, f32) {
    (285.0, MAIN_Y + 50.0 + (i as f32 * 34.0) - 22.0, 470.0, 30.0)
}

pub(super) fn roster_adventurer_row_rect(i: usize) -> (f32, f32, f32, f32) {
    (34.0, MAIN_Y + 52.0 + (i as f32 * 34.0) - 22.0, 690.0, 30.0)
}

pub(super) fn adventurer_row_hit_rect(active_tab: BaseTab, i: usize) -> (f32, f32, f32, f32) {
    if active_tab == BaseTab::Roster {
        roster_adventurer_row_rect(i)
    } else {
        adventurer_row_rect(i)
    }
}

pub(super) fn detail_back_button_rect() -> (f32, f32, f32, f32) {
    (screen_width() - 168.0, DETAIL_Y + 4.0, 126.0, 30.0)
}

pub(super) fn deck_close_button_rect() -> (f32, f32, f32, f32) {
    (screen_width() - 178.0, 56.0, 126.0, 30.0)
}

pub(super) fn party_mission_button_rect() -> (f32, f32, f32, f32) {
    (48.0, MAIN_Y + 420.0, 190.0, 34.0)
}

pub(super) fn party_back_button_rect() -> (f32, f32, f32, f32) {
    (254.0, MAIN_Y + 420.0, 150.0, 34.0)
}

pub(super) fn facility_card_rect(i: usize) -> (f32, f32, f32, f32) {
    let cols = 3;
    let card_w = (screen_width() - 84.0) / cols as f32;
    let card_h = 92.0;
    let col = i % cols;
    let row = i / cols;
    (
        SIDE_PAD + 14.0 + (col as f32 * (card_w + 12.0)),
        MAIN_Y + 48.0 + (row as f32 * (card_h + 12.0)),
        card_w,
        card_h,
    )
}

pub(super) fn deck_size(adv: &Adventurer) -> usize {
    let class_name = format!("{:?}", adv.class);
    crate::combat::Card::load_deck_for_class(&class_name, &adv.deck_additions).len()
}

pub(super) fn readiness_label(adv: &Adventurer) -> &'static str {
    if adv.hp <= adv.max_hp / 3 {
        "Needs Rest"
    } else if adv.stress >= 75 {
        "Fracturing"
    } else if adv.stress >= 50 {
        "Stressed"
    } else if !adv.injuries.is_empty() {
        "Wounded"
    } else {
        "Ready"
    }
}

pub(super) fn readiness_color(adv: &Adventurer) -> Color {
    match readiness_label(adv) {
        "Ready" => ready_color(),
        "Stressed" | "Wounded" => candle_color(),
        _ => danger_color(),
    }
}

pub(super) fn morale_label(value: i32) -> &'static str {
    match value {
        v if v >= 75 => "Steady",
        v if v >= 45 => "Fragile",
        v if v >= 20 => "Shaky",
        _ => "Breaking",
    }
}

pub(super) fn morale_color(value: i32) -> Color {
    if value >= 60 {
        ready_color()
    } else if value >= 30 {
        candle_color()
    } else {
        danger_color()
    }
}

pub(super) fn facility_purpose(id: &str) -> &'static str {
    match id {
        "infirmary" => "Heal injuries before they become permanent.",
        "chapel" => "Reduce stress and prevent resolve collapse.",
        "foundry" => "Improve equipment and unlock stronger cards.",
        "guild_hall" => "Recruit, dismiss, and train adventurers.",
        "watchtowers" => "Lower threat and unlock scouting missions.",
        "citadel" => "Final objective and win condition.",
        _ => "Frontier support facility.",
    }
}

pub(super) fn facility_unlocks(id: &str) -> &'static str {
    match id {
        "infirmary" => "Unlocks: Treat Wounds and safer injury recovery.",
        "chapel" => "Unlocks: Stress relief before resolve collapse.",
        "foundry" => "Unlocks: Knowledge-based card training.",
        "guild_hall" => "Unlocks: Recruitment and roster growth.",
        "watchtowers" => "Unlocks: Ruined Outpost scouting routes.",
        "citadel" => "Secures the campaign ending.",
        _ => "Facility active.",
    }
}

pub(super) fn class_guidance(adv: &Adventurer) -> &'static str {
    match format!("{:?}", adv.class).as_str() {
        "Soldier" => "Strong frontline fighter. Good for Suppress and Combat-heavy missions.",
        "Scout" => "Route finder and opportunist. Good when the mission may punish slow choices.",
        "Healer" => "Keeps the party alive and calmer. Best when wounds or stress are expected.",
        "Mystic" => {
            "High-impact control and burst damage. Best when dangerous enemies must be disrupted."
        }
        _ => "Reliable frontier hand. Match them to current wounds, stress, and route risk.",
    }
}

pub(super) fn card_type(card: &crate::combat::Card) -> &'static str {
    if card.is_attack() {
        "Attack"
    } else if card
        .effects
        .iter()
        .any(|effect| matches!(effect, crate::combat::CardEffect::Heal(_)))
    {
        "Heal"
    } else if card
        .effects
        .iter()
        .any(|effect| matches!(effect, crate::combat::CardEffect::Block(_)))
    {
        "Guard"
    } else {
        "Skill"
    }
}

pub(super) fn card_accent(card: &crate::combat::Card) -> Color {
    match card_type(card) {
        "Attack" => danger_color(),
        "Guard" => info_color(),
        "Heal" => ready_color(),
        _ => candle_color(),
    }
}

pub(super) fn title_color() -> Color {
    Color::from_rgba(236, 224, 198, 255)
}

pub(super) fn text_color() -> Color {
    Color::from_rgba(220, 212, 190, 255)
}

pub(super) fn muted_text_color() -> Color {
    Color::from_rgba(164, 153, 130, 255)
}

pub(super) fn parchment_color() -> Color {
    Color::from_rgba(196, 176, 130, 255)
}

pub(super) fn candle_color() -> Color {
    Color::from_rgba(214, 154, 62, 255)
}

pub(super) fn ready_color() -> Color {
    Color::from_rgba(112, 143, 92, 255)
}

pub(super) fn danger_color() -> Color {
    Color::from_rgba(150, 55, 48, 255)
}

pub(super) fn info_color() -> Color {
    Color::from_rgba(114, 136, 146, 255)
}

pub(super) fn mystery_color() -> Color {
    Color::from_rgba(119, 86, 132, 255)
}

pub(super) fn border_color() -> Color {
    Color::from_rgba(108, 82, 51, 210)
}
