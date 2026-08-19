//! Mission selection state - choose which mission to embark on

use super::{MissionState, StateTransition};
use crate::kingdom::{KingdomState, Party, PartyMemberState, Roster};
use crate::missions::{load_missions, Mission, MissionType};
use crate::ui::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const HEADER_H: f32 = 86.0;
const SIDE_PAD: f32 = 24.0;
const PANEL_Y: f32 = 104.0;
const PANEL_H: f32 = 455.0;
const PARTY_X: f32 = 24.0;
const PARTY_W: f32 = 306.0;
const BOARD_X: f32 = 350.0;
const BOARD_W: f32 = 430.0;
const DETAIL_X: f32 = 800.0;
const DETAIL_W: f32 = 440.0;
const CARD_H: f32 = 62.0;
const CARD_GAP: f32 = 8.0;

/// State for selecting a mission before departure
pub struct MissionSelectState {
    pub missions: Vec<Mission>,
    pub selected_mission: usize,
    /// Party members going on this mission (leader is first)
    pub party_members: Vec<PartyMemberState>,
}

impl MissionSelectState {
    /// Create mission select with a pre-selected adventurer (backwards compatible, single-person party)
    #[allow(dead_code)]
    pub fn new(
        adventurer_id: String,
        adventurer_name: String,
        hp: i32,
        max_hp: i32,
        stress: i32,
        image: Option<String>,
    ) -> Self {
        let member = PartyMemberState {
            id: adventurer_id,
            name: adventurer_name,
            hp,
            max_hp,
            stress,
            image_path: image,
            class_name: "Soldier".to_string(),
            deck_additions: vec![],
            traumas: vec![],
            resolve_state: None,
        };
        Self {
            missions: load_missions(),
            selected_mission: 0,
            party_members: vec![member],
        }
    }

    /// Create mission select from a party and roster
    pub fn for_party(party: Party, roster: &Roster) -> Self {
        let party_members: Vec<PartyMemberState> = party
            .member_ids
            .iter()
            .filter_map(|id| roster.get(id))
            .map(PartyMemberState::from_adventurer)
            .collect();

        Self {
            missions: load_missions(),
            selected_mission: 0,
            party_members,
        }
    }

    /// Get the party leader's info
    pub fn leader(&self) -> Option<&PartyMemberState> {
        self.party_members.first()
    }

    /// Check if a mission is unlocked
    pub fn is_mission_unlocked(&self, mission: &Mission, kingdom: &KingdomState) -> bool {
        mission.unlock_requirement.is_met(kingdom)
    }

    pub fn update(&mut self, _roster: &Roster, kingdom: &KingdomState) -> Option<StateTransition> {
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) && self.selected_mission > 0
        {
            self.selected_mission -= 1;
        }
        if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S))
            && self.selected_mission < self.missions.len().saturating_sub(1)
        {
            self.selected_mission += 1;
        }

        for i in 0..self.missions.len().min(9) {
            if is_key_pressed(number_key(i)) {
                self.selected_mission = i;
            }
        }

        for i in 0..self.missions.len() {
            let (x, y, w, h) = mission_card_rect(i);
            if crate::ui::was_clicked(x, y, w, h) {
                if self.selected_mission == i {
                    if let Some(transition) = self.start_selected_mission(kingdom) {
                        return Some(transition);
                    }
                } else {
                    self.selected_mission = i;
                }
            }
        }

        if is_key_pressed(KeyCode::Enter) {
            if let Some(transition) = self.start_selected_mission(kingdom) {
                return Some(transition);
            }
        }

        if crate::ui::was_clicked(DETAIL_X, 579.0, 142.0, 38.0) {
            if let Some(transition) = self.start_selected_mission(kingdom) {
                return Some(transition);
            }
        }
        if crate::ui::was_clicked(958.0, 579.0, 126.0, 38.0)
            || crate::ui::was_clicked(1100.0, 579.0, 126.0, 38.0)
            || is_key_pressed(KeyCode::Escape)
        {
            return Some(StateTransition::ToBase);
        }

        None
    }

    fn start_selected_mission(&self, kingdom: &KingdomState) -> Option<StateTransition> {
        let mission = self.missions.get(self.selected_mission)?;
        if !self.is_mission_unlocked(mission, kingdom) || self.leader().is_none() {
            return None;
        }

        let scaled_mission = mission.scaled_for_kingdom(kingdom);
        let mission_state =
            MissionState::from_mission_with_party(scaled_mission, self.party_members.clone());
        Some(StateTransition::ToMission(mission_state))
    }

    pub fn draw(
        &self,
        kingdom: &KingdomState,
        textures: &std::collections::HashMap<String, Texture2D>,
    ) {
        self.draw_background(textures);
        self.draw_header(kingdom);
        self.draw_party_panel(textures);
        self.draw_mission_board(kingdom, textures);
        self.draw_detail_panel(kingdom);
        self.draw_shortcuts(kingdom);
    }

    fn selected_mission(&self) -> Option<&Mission> {
        self.missions.get(self.selected_mission)
    }

    fn draw_background(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        draw_background(
            textures,
            BackgroundArt::MissionBoard,
            Color::from_rgba(5, 4, 4, 190),
        );
    }

    fn draw_header(&self, kingdom: &KingdomState) {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            HEADER_H,
            Color::from_rgba(10, 7, 6, 230),
        );
        draw_line(0.0, HEADER_H, screen_width(), HEADER_H, 2.0, border_color());
        draw_ui_text("EMBARK PREPARATION", SIDE_PAD, 42.0, 34.0, title_color());
        draw_ui_text(
            &format!(
                "Day {}     Threat {}     Morale: {}",
                kingdom.day,
                kingdom.threat_level,
                morale_label(kingdom.stats.morale)
            ),
            458.0,
            40.0,
            18.0,
            muted_text_color(),
        );
        draw_ui_text(
            "Choose the route. Read the risks. Send them back into the woods.",
            SIDE_PAD,
            70.0,
            16.0,
            muted_text_color(),
        );
    }

    fn draw_party_panel(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        panel(PARTY_X, PANEL_Y, PARTY_W, PANEL_H, "EXPEDITION PARTY");

        if self.party_members.is_empty() {
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

        for (i, member) in self.party_members.iter().enumerate() {
            let y = PANEL_Y + 48.0 + (i as f32 * 82.0);
            let leader = if i == 0 { "Leader" } else { "Member" };
            draw_member_row(member, leader, PARTY_X + 16.0, y, PARTY_W - 32.0, textures);
        }

        let party_risk = party_risk_label(&self.party_members);
        draw_ui_text(
            "Party Risk",
            PARTY_X + 18.0,
            PANEL_Y + 386.0,
            17.0,
            candle_color(),
        );
        draw_ui_text(
            party_risk,
            PARTY_X + 118.0,
            PANEL_Y + 386.0,
            17.0,
            risk_color(party_risk),
        );
        draw_wrapped_text(
            "High stress and low HP are the signs to treat wounds before departure.",
            PARTY_X + 18.0,
            PANEL_Y + 420.0,
            PARTY_W - 36.0,
            14.0,
            muted_text_color(),
        );
    }

    fn draw_mission_board(
        &self,
        kingdom: &KingdomState,
        textures: &std::collections::HashMap<String, Texture2D>,
    ) {
        panel(BOARD_X, PANEL_Y, BOARD_W, PANEL_H, "MISSION BOARD");
        for (i, mission) in self.missions.iter().enumerate() {
            let unlocked = self.is_mission_unlocked(mission, kingdom);
            draw_mission_card(
                i,
                mission,
                i == self.selected_mission,
                unlocked,
                kingdom,
                textures,
            );
        }
    }

    fn draw_detail_panel(&self, kingdom: &KingdomState) {
        panel(DETAIL_X, PANEL_Y, DETAIL_W, PANEL_H, "BRIEFING");
        let Some(mission) = self.selected_mission() else {
            return;
        };

        let unlocked = self.is_mission_unlocked(mission, kingdom);
        let effective = mission.scaled_for_kingdom(kingdom);

        draw_ui_text(
            &mission.name.to_uppercase(),
            DETAIL_X + 18.0,
            PANEL_Y + 48.0,
            26.0,
            title_color(),
        );
        draw_ui_text(
            &format!(
                "{:?} - {}",
                mission.mission_type,
                region_label(&mission.region_id)
            ),
            DETAIL_X + 18.0,
            PANEL_Y + 78.0,
            17.0,
            mission_type_color(&mission.mission_type),
        );
        draw_wrapped_text(
            &mission.description,
            DETAIL_X + 18.0,
            PANEL_Y + 110.0,
            DETAIL_W - 36.0,
            16.0,
            text_color(),
        );

        if !unlocked {
            draw_ui_text(
                "LOCKED",
                DETAIL_X + 18.0,
                PANEL_Y + 174.0,
                25.0,
                danger_color(),
            );
            draw_wrapped_text(
                &locked_instruction(mission),
                DETAIL_X + 18.0,
                PANEL_Y + 205.0,
                DETAIL_W - 36.0,
                17.0,
                text_color(),
            );
        } else {
            draw_ui_text(
                "Expected",
                DETAIL_X + 18.0,
                PANEL_Y + 174.0,
                18.0,
                candle_color(),
            );
            draw_ui_text(
                &format!(
                    "Difficulty {}    Stress Gain {}    Length {}",
                    effective.difficulty, effective.base_stress, effective.length
                ),
                DETAIL_X + 18.0,
                PANEL_Y + 202.0,
                17.0,
                text_color(),
            );
            draw_ui_text(
                "Possible Encounters",
                DETAIL_X + 18.0,
                PANEL_Y + 242.0,
                18.0,
                candle_color(),
            );
            draw_ui_text(
                encounter_line(&mission.mission_type),
                DETAIL_X + 18.0,
                PANEL_Y + 270.0,
                17.0,
                muted_text_color(),
            );
            draw_ui_text(
                "Rewards",
                DETAIL_X + 18.0,
                PANEL_Y + 316.0,
                18.0,
                candle_color(),
            );
            draw_ui_text(
                &format!(
                    "{} Gold    {} Supplies    {} Knowledge",
                    mission.reward_gold, mission.reward_supplies, mission.reward_knowledge
                ),
                DETAIL_X + 18.0,
                PANEL_Y + 344.0,
                17.0,
                reward_color(),
            );
            draw_ui_text(
                "Warnings",
                DETAIL_X + 18.0,
                PANEL_Y + 390.0,
                18.0,
                candle_color(),
            );
            draw_ui_text(
                mission_warning(&effective, &self.party_members),
                DETAIL_X + 112.0,
                PANEL_Y + 390.0,
                17.0,
                warning_color(&effective, &self.party_members),
            );
        }

        draw_action_button(
            if unlocked { "Embark" } else { "Locked" },
            DETAIL_X,
            579.0,
            142.0,
            38.0,
            unlocked,
        );
        draw_action_button("Change Party", 958.0, 579.0, 126.0, 38.0, true);
        draw_action_button("Back", 1100.0, 579.0, 126.0, 38.0, true);
    }

    fn draw_shortcuts(&self, kingdom: &KingdomState) {
        let locked = self
            .selected_mission()
            .map(|mission| !self.is_mission_unlocked(mission, kingdom))
            .unwrap_or(false);
        let line = if locked {
            "Shortcuts: Up/Down Select - Locked missions explain requirements - Esc Back"
        } else {
            "Shortcuts: Up/Down Select - Enter Embark - Esc Back"
        };
        draw_ui_text(
            line,
            SIDE_PAD,
            screen_height() - 24.0,
            13.0,
            muted_text_color(),
        );
    }
}

fn draw_member_row(
    member: &PartyMemberState,
    label: &str,
    x: f32,
    y: f32,
    w: f32,
    textures: &std::collections::HashMap<String, Texture2D>,
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

fn draw_mission_card(
    i: usize,
    mission: &Mission,
    selected: bool,
    unlocked: bool,
    kingdom: &KingdomState,
    textures: &std::collections::HashMap<String, Texture2D>,
) {
    let (x, y, w, h) = mission_card_rect(i);
    let fill = if selected {
        Color::from_rgba(82, 57, 30, 232)
    } else {
        Color::from_rgba(20, 19, 18, 218)
    };
    draw_rectangle(x, y, w, h, fill);
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

    let icon = match mission.mission_type {
        MissionType::Scout => SpriteIcon::Knowledge,
        MissionType::Suppress => SpriteIcon::Attack,
        MissionType::Secure => SpriteIcon::Guard,
        MissionType::Investigate => SpriteIcon::Event,
    };
    draw_icon(textures, icon, x + 8.0, y + 18.0, 28.0, WHITE);
    draw_ui_text(
        &format!("[{}] {}", i + 1, mission.name),
        x + 44.0,
        y + 24.0,
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
        y + 48.0,
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
            y + 48.0,
            14.0,
            muted_text_color(),
        );
        draw_ui_text(
            &format!("{}g", mission.reward_gold),
            x + w - 58.0,
            y + 48.0,
            14.0,
            reward_color(),
        );
    } else {
        draw_ui_text("LOCKED", x + w - 76.0, y + 24.0, 15.0, danger_color());
        draw_ui_text(
            &mission.unlock_requirement.description(),
            x + 132.0,
            y + 48.0,
            14.0,
            danger_color(),
        );
    }
}

fn draw_action_button(label: &str, x: f32, y: f32, w: f32, h: f32, enabled: bool) {
    let hovered = crate::ui::is_mouse_over(x, y, w, h);
    let fill = if !enabled {
        Color::from_rgba(31, 27, 25, 218)
    } else if hovered {
        Color::from_rgba(111, 75, 32, 245)
    } else {
        Color::from_rgba(70, 49, 27, 238)
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
        y + 24.0,
        16.0,
        if enabled {
            text_color()
        } else {
            muted_text_color()
        },
    );
}

fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, Color::from_rgba(13, 11, 10, 210));
    draw_rectangle(x, y, w, 32.0, Color::from_rgba(42, 30, 18, 222));
    draw_rectangle_lines(x, y, w, h, 1.0, border_color());
    draw_ui_text(title, x + 14.0, y + 22.0, 15.0, candle_color());
}

fn mission_card_rect(i: usize) -> (f32, f32, f32, f32) {
    (
        BOARD_X + 14.0,
        PANEL_Y + 46.0 + (i as f32 * (CARD_H + CARD_GAP)),
        BOARD_W - 28.0,
        CARD_H,
    )
}

fn number_key(i: usize) -> KeyCode {
    match i {
        0 => KeyCode::Key1,
        1 => KeyCode::Key2,
        2 => KeyCode::Key3,
        3 => KeyCode::Key4,
        4 => KeyCode::Key5,
        5 => KeyCode::Key6,
        6 => KeyCode::Key7,
        7 => KeyCode::Key8,
        _ => KeyCode::Key9,
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

fn party_risk_label(members: &[PartyMemberState]) -> &'static str {
    if members
        .iter()
        .any(|member| member_risk_label(member) == "High")
    {
        "High"
    } else if members
        .iter()
        .any(|member| member_risk_label(member) == "Medium")
    {
        "Medium"
    } else {
        "Low"
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

fn locked_instruction(mission: &Mission) -> String {
    let req = mission.unlock_requirement.description();
    if req.contains("watchtowers") {
        "Requires: Watchtowers. Build Watchtowers to scout beyond the ruined road.".to_string()
    } else {
        format!(
            "{}. Complete this requirement to make the route readable and available.",
            req
        )
    }
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

fn morale_label(morale: i32) -> &'static str {
    if morale >= 70 {
        "Steady"
    } else if morale >= 40 {
        "Fragile"
    } else {
        "Shaky"
    }
}

fn draw_wrapped_text(text: &str, x: f32, y: f32, max_width: f32, font_size: f32, color: Color) {
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
            line_y += font_size + 6.0;
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        draw_ui_text(&line, x, line_y, font_size, color);
    }
}

fn text_color() -> Color {
    Color::from_rgba(230, 221, 205, 255)
}

fn muted_text_color() -> Color {
    Color::from_rgba(158, 145, 126, 255)
}

fn title_color() -> Color {
    Color::from_rgba(239, 224, 190, 255)
}

fn candle_color() -> Color {
    Color::from_rgba(207, 151, 54, 255)
}

fn reward_color() -> Color {
    Color::from_rgba(224, 180, 72, 255)
}

fn danger_color() -> Color {
    Color::from_rgba(168, 58, 48, 255)
}

fn border_color() -> Color {
    Color::from_rgba(105, 76, 43, 210)
}

fn risk_color(risk: &str) -> Color {
    match risk {
        "Low" => Color::from_rgba(130, 177, 101, 255),
        "Medium" => Color::from_rgba(220, 164, 73, 255),
        _ => danger_color(),
    }
}

fn warning_color(mission: &Mission, members: &[PartyMemberState]) -> Color {
    let warning = mission_warning(mission, members);
    if warning == "None" {
        Color::from_rgba(130, 177, 101, 255)
    } else {
        danger_color()
    }
}

fn mission_type_color(mission_type: &MissionType) -> Color {
    match mission_type {
        MissionType::Scout => Color::from_rgba(118, 151, 164, 255),
        MissionType::Suppress => Color::from_rgba(171, 75, 58, 255),
        MissionType::Secure => Color::from_rgba(122, 158, 104, 255),
        MissionType::Investigate => Color::from_rgba(138, 104, 167, 255),
    }
}
