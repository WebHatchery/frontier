//! Mission and party card rendering.

use super::layout::{
    border_color, candle_color, danger_color, info_color, mission_card_rect, muted_text_color,
    ready_color, reward_color, text_color,
};
use crate::kingdom::PartyMemberState;
use crate::missions::{Mission, MissionType};
use crate::ui::{draw_icon, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use std::collections::HashMap;

pub(super) fn draw_member_row(
    member: &PartyMemberState,
    label: &str,
    x: f32,
    y: f32,
    w: f32,
    textures: &HashMap<String, Texture2D>,
) {
    draw_rectangle(x, y - 24.0, w, 66.0, Color::from_rgba(19, 17, 16, 205));
    draw_rectangle_lines(x, y - 24.0, w, 66.0, 1.0, border_color());
    if let Some(path) = &member.image_path {
        if let Some(tex) = textures.get(path) {
            draw_texture_ex(
                tex,
                x + 8.0,
                y - 17.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(48.0, 48.0)),
                    ..Default::default()
                },
            );
        }
    }
    draw_ui_text(&member.name, x + 68.0, y, 18.0, text_color());
    draw_ui_text(label, x + w - 68.0, y, 13.0, muted_text_color());
    draw_ui_text(
        &format!(
            "HP {}/{}    Stress {}",
            member.hp, member.max_hp, member.stress
        ),
        x + 68.0,
        y + 24.0,
        14.0,
        muted_text_color(),
    );
    let risk = member_risk_label(member);
    draw_ui_text(risk, x + w - 72.0, y + 24.0, 14.0, risk_color(risk));
}

pub(super) fn draw_mission_card(
    index: usize,
    mission: &Mission,
    selected: bool,
    unlocked: bool,
    kingdom: &crate::kingdom::KingdomState,
    textures: &HashMap<String, Texture2D>,
) {
    let (x, y, w, h) = mission_card_rect(index);
    draw_rectangle(
        x,
        y,
        w,
        h,
        if selected {
            Color::from_rgba(82, 57, 30, 232)
        } else {
            Color::from_rgba(20, 19, 18, 218)
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
    draw_icon(
        textures,
        mission_icon(&mission.mission_type),
        x + 8.0,
        y + 15.0,
        28.0,
        WHITE,
    );
    draw_ui_text(
        &format!("[{}] {}", index + 1, mission.name),
        x + 44.0,
        y + 23.0,
        17.0,
        if unlocked {
            text_color()
        } else {
            muted_text_color()
        },
    );
    draw_ui_text(
        &format!("{:?}", mission.mission_type),
        x + 44.0,
        y + 46.0,
        14.0,
        mission_type_color(&mission.mission_type),
    );
    if unlocked {
        let effective = mission.scaled_for_kingdom(kingdom);
        draw_ui_text(
            &format!(
                "Risk {} / Stress {}",
                effective.difficulty, effective.base_stress
            ),
            x + 132.0,
            y + 46.0,
            14.0,
            muted_text_color(),
        );
        draw_ui_text(
            &format!("{}g", mission.reward_gold),
            x + w - 58.0,
            y + 46.0,
            14.0,
            reward_color(),
        );
    } else {
        draw_ui_text("LOCKED", x + w - 76.0, y + 23.0, 15.0, danger_color());
        draw_ui_text(
            &mission.unlock_requirement.description(),
            x + 132.0,
            y + 46.0,
            14.0,
            danger_color(),
        );
    }
}

pub(super) fn draw_action_button(label: &str, rect: (f32, f32, f32, f32), enabled: bool) {
    let hovered = enabled && crate::ui::is_mouse_over(rect.0, rect.1, rect.2, rect.3);
    let fill = if !enabled {
        Color::from_rgba(31, 27, 25, 218)
    } else if hovered {
        Color::from_rgba(111, 75, 32, 245)
    } else {
        Color::from_rgba(70, 49, 27, 238)
    };
    draw_rectangle(rect.0, rect.1, rect.2, rect.3, fill);
    draw_rectangle_lines(
        rect.0,
        rect.1,
        rect.2,
        rect.3,
        1.0,
        if enabled {
            candle_color()
        } else {
            border_color()
        },
    );
    let width = measure_ui_text(label, None, 16, 1.0).width;
    draw_ui_text(
        label,
        rect.0 + (rect.2 - width) / 2.0,
        rect.1 + 24.0,
        16.0,
        if enabled {
            text_color()
        } else {
            muted_text_color()
        },
    );
}

fn mission_icon(mission_type: &MissionType) -> SpriteIcon {
    match mission_type {
        MissionType::Scout => SpriteIcon::Knowledge,
        MissionType::Suppress => SpriteIcon::Attack,
        MissionType::Secure => SpriteIcon::Guard,
        MissionType::Investigate => SpriteIcon::Event,
    }
}

fn member_risk_label(member: &PartyMemberState) -> &'static str {
    if member.hp <= member.max_hp / 3 || member.stress >= 75 {
        "High"
    } else if member.hp <= member.max_hp / 2 || member.stress >= 45 {
        "Medium"
    } else {
        "Low"
    }
}

fn risk_color(risk: &str) -> Color {
    match risk {
        "Low" => ready_color(),
        "Medium" => reward_color(),
        _ => danger_color(),
    }
}

fn mission_type_color(mission_type: &MissionType) -> Color {
    match mission_type {
        MissionType::Scout => info_color(),
        MissionType::Suppress => Color::from_rgba(171, 75, 58, 255),
        MissionType::Secure => ready_color(),
        MissionType::Investigate => Color::from_rgba(138, 104, 167, 255),
    }
}
