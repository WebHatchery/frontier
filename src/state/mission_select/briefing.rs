//! Mission-selection briefing panels and screen chrome.

use super::cards::{draw_action_button, draw_member_row, draw_mission_card};
use super::layout::{
    action_button_rect, candle_color, danger_color, draw_wrapped_text, info_color,
    muted_text_color, panel, reward_color, text_color, title_color, BOARD_W, BOARD_X, DETAIL_W,
    DETAIL_X, HEADER_H, PANEL_H, PANEL_Y, PARTY_W, PARTY_X, SIDE_PAD,
};
use super::MissionSelectState;
use crate::kingdom::{KingdomState, PartyMemberState};
use crate::missions::{Mission, MissionType};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;
use std::collections::HashMap;

pub(super) fn draw_header(kingdom: &KingdomState) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        HEADER_H,
        Color::from_rgba(10, 7, 6, 230),
    );
    draw_line(
        0.0,
        HEADER_H,
        screen_width(),
        HEADER_H,
        2.0,
        super::layout::border_color(),
    );
    draw_ui_text("EMBARK PREPARATION", SIDE_PAD, 38.0, 32.0, title_color());
    draw_ui_text(
        &format!(
            "Day {}     Threat {}     Morale: {}",
            kingdom.day,
            kingdom.threat_level,
            morale_label(kingdom.stats.morale)
        ),
        458.0,
        36.0,
        18.0,
        muted_text_color(),
    );
    draw_ui_text(
        "Choose the route. Read the risks. Send them back into the woods.",
        SIDE_PAD,
        64.0,
        15.0,
        muted_text_color(),
    );
}

pub(super) fn draw_party_panel(state: &MissionSelectState, textures: &HashMap<String, Texture2D>) {
    panel(PARTY_X, PANEL_Y, PARTY_W, PANEL_H, "EXPEDITION PARTY");
    if state.party_members.is_empty() {
        draw_wrapped_text(
            "No party selected. Return to the kingdom dashboard and assign adventurers.",
            PARTY_X + 18.0,
            PANEL_Y + 54.0,
            PARTY_W - 36.0,
            16.0,
            muted_text_color(),
        );
        return;
    }
    for (index, member) in state.party_members.iter().enumerate() {
        draw_member_row(
            member,
            if index == 0 { "Leader" } else { "Member" },
            PARTY_X + 16.0,
            PANEL_Y + 48.0 + index as f32 * 76.0,
            PARTY_W - 32.0,
            textures,
        );
    }
    let risk = party_risk_label(&state.party_members);
    draw_ui_text(
        "Party Risk",
        PARTY_X + 18.0,
        PANEL_Y + 372.0,
        17.0,
        candle_color(),
    );
    draw_ui_text(
        risk,
        PARTY_X + 118.0,
        PANEL_Y + 372.0,
        17.0,
        risk_color(risk),
    );
    draw_wrapped_text(
        "Treat low HP and high stress before departure.",
        PARTY_X + 18.0,
        PANEL_Y + 408.0,
        PARTY_W - 36.0,
        14.0,
        muted_text_color(),
    );
}

pub(super) fn draw_mission_board(
    state: &MissionSelectState,
    kingdom: &KingdomState,
    textures: &HashMap<String, Texture2D>,
) {
    panel(BOARD_X, PANEL_Y, BOARD_W, PANEL_H, "MISSION BOARD");
    for (index, mission) in state.missions.iter().enumerate() {
        draw_mission_card(
            index,
            mission,
            index == state.selected_mission,
            state.is_mission_unlocked(mission, kingdom),
            kingdom,
            textures,
        );
    }
}

pub(super) fn draw_detail_panel(state: &MissionSelectState, kingdom: &KingdomState) {
    panel(DETAIL_X, PANEL_Y, DETAIL_W, PANEL_H, "BRIEFING");
    let Some(mission) = state.selected_mission() else {
        return;
    };
    let unlocked = state.is_mission_unlocked(mission, kingdom);
    let effective = mission.scaled_for_kingdom(kingdom);
    draw_ui_text(
        &mission.name.to_uppercase(),
        DETAIL_X + 18.0,
        PANEL_Y + 48.0,
        24.0,
        title_color(),
    );
    draw_ui_text(
        &format!(
            "{:?} - {}",
            mission.mission_type,
            region_label(&mission.region_id)
        ),
        DETAIL_X + 18.0,
        PANEL_Y + 76.0,
        17.0,
        mission_type_color(&mission.mission_type),
    );
    draw_wrapped_text(
        &mission.description,
        DETAIL_X + 18.0,
        PANEL_Y + 106.0,
        DETAIL_W - 36.0,
        16.0,
        text_color(),
    );
    if unlocked {
        draw_ui_text(
            "Expected",
            DETAIL_X + 18.0,
            PANEL_Y + 170.0,
            18.0,
            candle_color(),
        );
        draw_ui_text(
            &format!(
                "Difficulty {}    Stress {}    Length {}",
                effective.difficulty, effective.base_stress, effective.length
            ),
            DETAIL_X + 18.0,
            PANEL_Y + 198.0,
            17.0,
            text_color(),
        );
        draw_ui_text(
            "Encounters",
            DETAIL_X + 18.0,
            PANEL_Y + 238.0,
            18.0,
            candle_color(),
        );
        draw_wrapped_text(
            encounter_line(&mission.mission_type),
            DETAIL_X + 18.0,
            PANEL_Y + 264.0,
            DETAIL_W - 36.0,
            16.0,
            muted_text_color(),
        );
        draw_ui_text(
            "Rewards",
            DETAIL_X + 18.0,
            PANEL_Y + 314.0,
            18.0,
            candle_color(),
        );
        draw_ui_text(
            &format!(
                "{} Gold    {} Supplies    {} Knowledge",
                mission.reward_gold, mission.reward_supplies, mission.reward_knowledge
            ),
            DETAIL_X + 18.0,
            PANEL_Y + 342.0,
            16.0,
            reward_color(),
        );
        draw_ui_text(
            "Warning",
            DETAIL_X + 18.0,
            PANEL_Y + 388.0,
            18.0,
            candle_color(),
        );
        draw_wrapped_text(
            mission_warning(&effective, &state.party_members),
            DETAIL_X + 18.0,
            PANEL_Y + 414.0,
            DETAIL_W - 36.0,
            15.0,
            warning_color(&effective, &state.party_members),
        );
    } else {
        draw_ui_text(
            "LOCKED",
            DETAIL_X + 18.0,
            PANEL_Y + 170.0,
            25.0,
            danger_color(),
        );
        draw_wrapped_text(
            &locked_instruction(mission),
            DETAIL_X + 18.0,
            PANEL_Y + 202.0,
            DETAIL_W - 36.0,
            17.0,
            text_color(),
        );
    }
    draw_action_button(
        if unlocked { "Embark" } else { "Locked" },
        action_button_rect(0),
        unlocked,
    );
    draw_action_button("Change Party", action_button_rect(1), true);
    draw_action_button("Back", action_button_rect(2), true);
}

pub(super) fn draw_optional_hint(state: &MissionSelectState, kingdom: &KingdomState) {
    let locked = state
        .selected_mission()
        .is_some_and(|mission| !state.is_mission_unlocked(mission, kingdom));
    draw_ui_text(
        if locked {
            "Tap a mission to inspect requirements • Tap BACK to return"
        } else {
            "Tap a mission to select • Tap EMBARK to depart • Tap BACK to return"
        },
        SIDE_PAD,
        screen_height() - 20.0,
        13.0,
        muted_text_color(),
    );
}

fn party_risk_label(members: &[PartyMemberState]) -> &'static str {
    if members.iter().any(|member| member_risk(member) == "High") {
        "High"
    } else if members.iter().any(|member| member_risk(member) == "Medium") {
        "Medium"
    } else {
        "Low"
    }
}

fn member_risk(member: &PartyMemberState) -> &'static str {
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
        "Low" => super::layout::ready_color(),
        "Medium" => reward_color(),
        _ => danger_color(),
    }
}

fn mission_warning(mission: &Mission, members: &[PartyMemberState]) -> &'static str {
    if members.iter().any(|member| member.hp <= member.max_hp / 3) {
        "One hero is badly wounded."
    } else if members.iter().any(|member| member.stress >= 75) {
        "Resolve collapse likely."
    } else if mission.base_stress >= 25 {
        "Stress gain is severe."
    } else if mission.difficulty >= 3 {
        "Combat risk is high."
    } else {
        "None"
    }
}

fn warning_color(mission: &Mission, members: &[PartyMemberState]) -> Color {
    if mission_warning(mission, members) == "None" {
        super::layout::ready_color()
    } else {
        danger_color()
    }
}

fn locked_instruction(mission: &Mission) -> String {
    format!(
        "{}. Complete this requirement to unlock the route.",
        mission.unlock_requirement.description()
    )
}

fn encounter_line(mission_type: &MissionType) -> &'static str {
    match mission_type {
        MissionType::Scout => "Events, unknown paths, occasional beasts",
        MissionType::Suppress => "Beasts, ambushes, boss encounter",
        MissionType::Secure => "Combat, events, supply pressure",
        MissionType::Investigate => "Omens, stress events, unknown threats",
    }
}

fn region_label(region_id: &str) -> &'static str {
    match region_id {
        "dark_woods" => "Dark Woods",
        "ruined_outpost" => "Ruined Outpost",
        "sunken_valley" => "Sunken Valley",
        _ => "Unknown Region",
    }
}

fn mission_type_color(mission_type: &MissionType) -> Color {
    match mission_type {
        MissionType::Scout => info_color(),
        MissionType::Suppress => danger_color(),
        MissionType::Secure => super::layout::ready_color(),
        MissionType::Investigate => super::layout::mystery_color(),
    }
}

fn morale_label(value: i32) -> &'static str {
    if value >= 70 {
        "Steady"
    } else if value >= 40 {
        "Fragile"
    } else {
        "Shaky"
    }
}
