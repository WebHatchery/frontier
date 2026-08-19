//! Persistent panel chrome: background, header, tabs, and summary readouts.

use super::helpers::{
    action_buttons, action_enabled, border_color, candle_color, danger_color, draw_wrapped_text,
    facility_card_rect, facility_purpose, info_color, morale_color, morale_label, muted_text_color,
    mystery_color, panel, parchment_color, readiness_color, readiness_label, ready_color,
    tab_width, text_color, title_color,
};
use super::{BaseState, BaseTab, ACTION_H, ACTION_Y, HEADER_H, SIDE_PAD};
use crate::kingdom::{Adventurer, Building, KingdomState, Roster};
use crate::ui::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl BaseState {
    pub(super) fn draw_action_bar(&self, kingdom: &KingdomState, roster: &Roster) {
        panel(
            SIDE_PAD,
            ACTION_Y,
            screen_width() - SIDE_PAD * 2.0,
            ACTION_H,
            "KINGDOM ACTIONS",
        );
        for (i, action) in action_buttons().iter().enumerate() {
            let x = SIDE_PAD + 18.0 + (i as f32 * 138.0);
            let enabled = action_enabled(action, kingdom, roster, self.selected_adventurer);
            draw_action_button(action, x, ACTION_Y + 30.0, 126.0, 30.0, enabled);
        }
    }
}

pub(super) fn draw_command_table_background(
    textures: &std::collections::HashMap<String, Texture2D>,
) {
    draw_background(
        textures,
        BackgroundArt::Base,
        Color::from_rgba(8, 6, 5, 168),
    );
}

pub(super) fn draw_header(kingdom: &KingdomState, active_tab: BaseTab) {
    let morale = morale_label(kingdom.stats.morale);
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        HEADER_H,
        Color::from_rgba(10, 7, 6, 218),
    );
    draw_line(0.0, HEADER_H, screen_width(), HEADER_H, 2.0, border_color());
    draw_ui_text("FRONTIER KINGDOM", SIDE_PAD, 38.0, 34.0, title_color());
    draw_ui_text(
        &format!(
            "Day {}    Threat {}    Morale: {}    {}",
            kingdom.day,
            kingdom.threat_level,
            morale,
            active_tab.label()
        ),
        430.0,
        36.0,
        20.0,
        muted_text_color(),
    );
}

pub(super) fn draw_tabs(active_tab: BaseTab) {
    let mut x = SIDE_PAD;
    for tab in BaseTab::ALL {
        let w = tab_width(tab);
        let selected = tab == active_tab;
        draw_rectangle(
            x,
            62.0,
            w,
            28.0,
            if selected {
                Color::from_rgba(97, 66, 27, 235)
            } else {
                Color::from_rgba(26, 23, 21, 210)
            },
        );
        draw_rectangle_lines(
            x,
            62.0,
            w,
            28.0,
            if selected { 2.0 } else { 1.0 },
            if selected {
                candle_color()
            } else {
                border_color()
            },
        );
        draw_ui_text(
            tab.label(),
            x + 10.0,
            82.0,
            16.0,
            if selected {
                title_color()
            } else {
                muted_text_color()
            },
        );
        x += w + 8.0;
    }
}

pub(super) fn draw_resources_panel(
    kingdom: &KingdomState,
    textures: &std::collections::HashMap<String, Texture2D>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
) {
    panel(x, y, w, h, "RESOURCES");
    let stats = &kingdom.stats;
    let rows = [
        ("Gold", stats.gold, candle_color(), SpriteIcon::Gold),
        (
            "Supplies",
            stats.supplies,
            parchment_color(),
            SpriteIcon::Supplies,
        ),
        (
            "Security",
            stats.security,
            info_color(),
            SpriteIcon::Security,
        ),
        (
            "Morale",
            stats.morale,
            morale_color(stats.morale),
            SpriteIcon::Morale,
        ),
        (
            "Knowledge",
            stats.knowledge,
            mystery_color(),
            SpriteIcon::Knowledge,
        ),
        (
            "Influence",
            stats.influence,
            muted_text_color(),
            SpriteIcon::Influence,
        ),
    ];
    let mut row_y = y + 52.0;
    for (label, value, color, icon) in rows {
        draw_icon(textures, icon, x + 14.0, row_y - 19.0, 24.0, WHITE);
        draw_ui_text(label, x + 46.0, row_y, 17.0, muted_text_color());
        draw_ui_text(&value.to_string(), x + w - 70.0, row_y, 20.0, color);
        row_y += 30.0;
    }
}

pub(super) fn draw_adventurer_summary(
    roster: &Roster,
    selected: Option<usize>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    leader_id: Option<&str>,
) {
    panel(x, y, w, h, "PARTY / ADVENTURERS");
    for (i, adv) in roster.adventurers.iter().enumerate().take(5) {
        draw_adventurer_row(i, adv, selected == Some(i), x + 20.0, y + 50.0);
        if leader_id == Some(adv.id.as_str()) {
            draw_ui_text(
                "Leader",
                x + w - 90.0,
                y + 74.0 + (i as f32 * 34.0),
                15.0,
                candle_color(),
            );
        }
    }
}

pub(super) fn draw_goals_panel(kingdom: &KingdomState, x: f32, y: f32, w: f32, h: f32) {
    panel(x, y, w, h, "CURRENT GOALS");
    let mut row_y = y + 48.0;
    for (quest, done) in kingdom.quest_log().into_iter().take(5) {
        let color = if done { ready_color() } else { text_color() };
        let marker = if done { "[x]" } else { "[ ]" };
        draw_ui_text(
            &format!("{} {}", marker, quest),
            x + 18.0,
            row_y,
            16.0,
            color,
        );
        row_y += 28.0;
    }
    if let Some(event) = &kingdom.last_event {
        draw_ui_text("Alert", x + 18.0, y + h - 58.0, 16.0, danger_color());
        draw_wrapped_text(
            event,
            x + 18.0,
            y + h - 35.0,
            w - 36.0,
            14.0,
            muted_text_color(),
        );
    }
}

pub(super) fn draw_readiness_summary(kingdom: &KingdomState, roster: &Roster, x: f32, y: f32) {
    let ready = roster
        .adventurers
        .iter()
        .filter(|adv| readiness_label(adv) == "Ready")
        .count();
    draw_ui_text(
        &format!("Ready heroes: {}", ready),
        x,
        y,
        20.0,
        ready_color(),
    );
    draw_ui_text(
        &format!("Supplies available: {}", kingdom.stats.supplies),
        x,
        y + 34.0,
        18.0,
        parchment_color(),
    );
    draw_wrapped_text(
        "Send ready heroes, rest the shaken, and keep enough supplies for treatment after the mission.",
        x,
        y + 72.0,
        screen_width() - x - 56.0,
        16.0,
        muted_text_color(),
    );
}

pub(super) fn draw_adventurer_row(
    i: usize,
    adv: &Adventurer,
    selected: bool,
    x: f32,
    start_y: f32,
) {
    let y = start_y + (i as f32 * 34.0);
    let bg_x = x - 10.0;
    let bg_y = y - 22.0;
    let bg_w = if x < 100.0 { 690.0 } else { 470.0 };
    let row_h = 30.0;
    draw_rectangle(
        bg_x,
        bg_y,
        bg_w,
        row_h,
        if selected {
            Color::from_rgba(83, 58, 29, 225)
        } else {
            Color::from_rgba(22, 20, 18, 175)
        },
    );
    if selected {
        draw_rectangle_lines(bg_x, bg_y, bg_w, row_h, 2.0, candle_color());
    }
    draw_ui_text(
        &format!("[{}] {}", i + 1, adv.name),
        x,
        y,
        18.0,
        text_color(),
    );
    draw_ui_text(
        &format!("{:?}", adv.class),
        x + 220.0,
        y,
        15.0,
        muted_text_color(),
    );
    draw_ui_text(
        readiness_label(adv),
        x + 330.0,
        y,
        16.0,
        readiness_color(adv),
    );
}

pub(super) fn draw_facility_card(i: usize, building: &Building, selected: bool, can_build: bool) {
    let (x, y, w, h) = facility_card_rect(i);
    draw_rectangle(
        x,
        y,
        w,
        h,
        if selected {
            Color::from_rgba(77, 52, 26, 230)
        } else {
            Color::from_rgba(23, 21, 19, 218)
        },
    );
    draw_rectangle_lines(
        x,
        y,
        w,
        h,
        if selected { 2.0 } else { 1.0 },
        if selected {
            candle_color()
        } else {
            border_color()
        },
    );
    draw_ui_text(
        &building.name.to_uppercase(),
        x + 14.0,
        y + 28.0,
        18.0,
        title_color(),
    );
    draw_ui_text(
        facility_purpose(&building.id),
        x + 14.0,
        y + 52.0,
        13.0,
        muted_text_color(),
    );
    let status = if building.built {
        "Status: Built".to_string()
    } else {
        format!(
            "Cost: {}g / {}s",
            building.cost_gold, building.cost_supplies
        )
    };
    draw_ui_text(
        &status,
        x + 14.0,
        y + h - 20.0,
        15.0,
        if building.built {
            ready_color()
        } else if can_build {
            candle_color()
        } else {
            danger_color()
        },
    );
    draw_ui_text(
        if building.built { "Active" } else { "[Build]" },
        x + w - 82.0,
        y + h - 20.0,
        15.0,
        if building.built {
            muted_text_color()
        } else {
            candle_color()
        },
    );
}

pub(super) fn draw_action_button(label: &str, x: f32, y: f32, w: f32, h: f32, enabled: bool) {
    let hovered = crate::ui::is_mouse_over(x, y, w, h);
    let fill = if !enabled {
        Color::from_rgba(31, 28, 25, 200)
    } else if hovered {
        Color::from_rgba(114, 78, 32, 245)
    } else {
        Color::from_rgba(74, 52, 28, 235)
    };
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(
        x,
        y,
        w,
        h,
        1.0,
        if enabled {
            candle_color()
        } else {
            border_color()
        },
    );
    let tw = measure_ui_text(label, None, 16, 1.0).width;
    draw_ui_text(
        label,
        x + (w - tw) / 2.0,
        y + 21.0,
        16.0,
        if enabled {
            text_color()
        } else {
            muted_text_color()
        },
    );
}

pub(super) fn draw_shortcuts() {
    draw_ui_text(
        "Shortcuts: 1-9 Select - Tab Tabs - M Party - D Deck - H/T Treat - U Train - F5 Save - F9 Load",
        SIDE_PAD,
        screen_height() - 18.0,
        14.0,
        muted_text_color(),
    );
}
