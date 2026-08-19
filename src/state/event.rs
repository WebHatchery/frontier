//! Event state - narrative encounters with choices

use super::{MissionState, StateTransition};
use crate::kingdom::PartyMemberState;
use crate::missions::events::{Event, EventOutcome};
use crate::missions::{MapNode, Mission};
use crate::ui::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;
use std::collections::HashMap;

/// State for handling mission events
pub struct EventState {
    pub event: Event,
    pub selected_choice: usize,
    #[allow(dead_code)]
    pub adventurer_id: String,
    #[allow(dead_code)]
    pub adventurer_name: String,
    pub return_to_mission: bool,
    /// Applied outcomes to pass back
    pub stress_change: i32,
    pub hp_change: i32,
    pub supplies_change: i32,
    pub knowledge_change: i32,
    pub trigger_combat: Option<String>,
    pub skip_node: bool,
    /// Mission context to return to
    pub mission_context: Option<MissionReturnContext>,
}

/// Context for returning to mission after event
#[derive(Clone)]
pub struct MissionReturnContext {
    pub mission: Mission,
    pub current_node: usize,
    pub party_members: Vec<PartyMemberState>,
    pub map_nodes: Vec<MapNode>,
    pub visited_nodes: Vec<usize>,
}

impl EventState {
    pub fn new(event: Event, adventurer_id: String, adventurer_name: String) -> Self {
        Self {
            event,
            selected_choice: 0,
            adventurer_id,
            adventurer_name,
            return_to_mission: false,
            stress_change: 0,
            hp_change: 0,
            supplies_change: 0,
            knowledge_change: 0,
            trigger_combat: None,
            skip_node: false,
            mission_context: None,
        }
    }

    /// Create event with mission context for returning
    pub fn with_mission_context(
        mut self,
        mission: Mission,
        current_node: usize,
        party_members: Vec<PartyMemberState>,
        map_nodes: Vec<MapNode>,
        visited_nodes: Vec<usize>,
    ) -> Self {
        self.mission_context = Some(MissionReturnContext {
            mission,
            current_node,
            party_members,
            map_nodes,
            visited_nodes,
        });
        self
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        // Choice layout constants (must match draw)
        let panel_x = 100.0;
        let panel_y = 80.0;
        let panel_w = screen_width() - 200.0;
        let choices_y = panel_y + 200.0;
        let choice_height = 45.0;

        // Choice selection
        let choice_count = self.event.choices.len();

        // Keyboard navigation
        if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) && self.selected_choice > 0 {
            self.selected_choice -= 1;
        }
        if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S))
            && self.selected_choice < choice_count.saturating_sub(1)
        {
            self.selected_choice += 1;
        }

        // Number keys and mouse clicks
        for i in 0..choice_count.min(9) {
            let key = match i {
                0 => KeyCode::Key1,
                1 => KeyCode::Key2,
                2 => KeyCode::Key3,
                3 => KeyCode::Key4,
                4 => KeyCode::Key5,
                _ => continue,
            };

            // Keyboard
            if is_key_pressed(key) {
                self.selected_choice = i;
            }

            // Mouse click on choice
            let choice_y = choices_y + 35.0 + (i as f32 * 50.0) - 15.0;
            if crate::ui::was_clicked(panel_x + 20.0, choice_y, panel_w - 40.0, choice_height) {
                if self.selected_choice == i {
                    // Click on selected = confirm
                    return self.confirm_choice();
                } else {
                    self.selected_choice = i;
                }
            }
        }

        // Confirm choice with Enter
        if is_key_pressed(KeyCode::Enter) {
            return self.confirm_choice();
        }

        None
    }

    /// Confirm the currently selected choice and return transition
    fn confirm_choice(&mut self) -> Option<StateTransition> {
        if let Some(choice) = self.event.choices.get(self.selected_choice) {
            // Process outcomes
            for outcome in &choice.outcomes {
                match outcome {
                    EventOutcome::Stress(amt) => self.stress_change += amt,
                    EventOutcome::Heal(amt) => self.hp_change += amt,
                    EventOutcome::Supplies(amt) => self.supplies_change += amt,
                    EventOutcome::Knowledge(amt) => self.knowledge_change += amt,
                    EventOutcome::Combat(enemy_id) => {
                        self.trigger_combat = Some(enemy_id.clone());
                    }
                    EventOutcome::SkipNode => self.skip_node = true,
                    EventOutcome::RevealTrait => {
                        self.knowledge_change += 5;
                    }
                    EventOutcome::Nothing => {}
                }
            }
            self.return_to_mission = true;

            // Return to mission if we have context, otherwise go to base
            if let Some(ctx) = &self.mission_context {
                // Apply HP/stress changes to party members
                let mut updated_members = ctx.party_members.clone();
                for member in &mut updated_members {
                    member.hp = (member.hp + self.hp_change).clamp(0, member.max_hp);
                    member.stress = (member.stress + self.stress_change).clamp(0, 100);
                }

                let mission_state =
                    MissionState::from_mission_with_party(ctx.mission.clone(), updated_members)
                        .with_node(ctx.current_node)
                        .with_map_nodes(ctx.map_nodes.clone())
                        .with_visited(ctx.visited_nodes.clone());
                return Some(StateTransition::ToMission(mission_state));
            } else {
                return Some(StateTransition::ToBase);
            }
        }
        None
    }

    pub fn draw(&self, textures: &HashMap<String, Texture2D>) {
        draw_background(
            textures,
            BackgroundArt::EventShrine,
            Color::from_rgba(0, 0, 0, 146),
        );

        // Event panel
        let panel_x = 100.0;
        let panel_y = 80.0;
        let panel_w = screen_width() - 200.0;
        let panel_h = screen_height() - 160.0;

        draw_rectangle(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            Color::from_rgba(30, 23, 19, 246),
        );
        draw_rectangle_lines(
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            2.0,
            Color::from_rgba(153, 111, 62, 255),
        );

        // Title
        draw_icon(
            textures,
            SpriteIcon::Event,
            panel_x + 18.0,
            panel_y + 14.0,
            32.0,
            WHITE,
        );
        draw_ui_text(
            &self.event.title,
            panel_x + 58.0,
            panel_y + 40.0,
            32.0,
            Color::from_rgba(228, 177, 84, 255),
        );

        // Description
        let desc_y = panel_y + 80.0;
        // Simple word wrap
        let max_width = panel_w - 40.0;
        let words: Vec<&str> = self.event.description.split_whitespace().collect();
        let mut line = String::new();
        let mut y = desc_y;

        for word in words {
            let test_line = if line.is_empty() {
                word.to_string()
            } else {
                format!("{} {}", line, word)
            };

            // Rough estimate: 8 pixels per character at size 18
            if test_line.len() as f32 * 8.0 > max_width {
                draw_ui_text(
                    &line,
                    panel_x + 20.0,
                    y,
                    18.0,
                    Color::from_rgba(208, 195, 173, 255),
                );
                y += 25.0;
                line = word.to_string();
            } else {
                line = test_line;
            }
        }
        if !line.is_empty() {
            draw_ui_text(
                &line,
                panel_x + 20.0,
                y,
                18.0,
                Color::from_rgba(208, 195, 173, 255),
            );
        }

        // Choices
        let choices_y = panel_y + 200.0;
        draw_ui_text(
            "CHOOSE:",
            panel_x + 20.0,
            choices_y,
            20.0,
            Color::from_rgba(228, 177, 84, 255),
        );

        for (i, choice) in self.event.choices.iter().enumerate() {
            let y = choices_y + 35.0 + (i as f32 * 50.0);
            let choice_y = y - 15.0;
            let choice_h = 45.0;
            let is_selected = i == self.selected_choice;
            let is_hovered =
                crate::ui::is_mouse_over(panel_x + 20.0, choice_y, panel_w - 40.0, choice_h);

            // Choice background with hover
            let bg_color = if is_selected {
                Color::from_rgba(76, 59, 32, 245)
            } else if is_hovered {
                Color::from_rgba(63, 52, 41, 240)
            } else {
                Color::from_rgba(27, 23, 21, 235)
            };
            draw_rectangle(panel_x + 20.0, choice_y, panel_w - 40.0, choice_h, bg_color);

            if is_selected {
                draw_rectangle_lines(
                    panel_x + 20.0,
                    choice_y,
                    panel_w - 40.0,
                    choice_h,
                    2.0,
                    Color::from_rgba(125, 158, 101, 255),
                );
            } else if is_hovered {
                draw_rectangle_lines(
                    panel_x + 20.0,
                    choice_y,
                    panel_w - 40.0,
                    choice_h,
                    1.0,
                    Color::from_rgba(153, 111, 62, 255),
                );
            }

            // Choice text
            let text_color = if is_selected {
                Color::from_rgba(236, 224, 198, 255)
            } else {
                Color::from_rgba(164, 153, 130, 255)
            };
            draw_ui_text(
                &format!("[{}] {}", i + 1, choice.text),
                panel_x + 30.0,
                y + 10.0,
                18.0,
                text_color,
            );
        }

        // Instructions
        draw_ui_text(
            "Click choice to select, click again to confirm • Or use [↑/↓] and [ENTER]",
            panel_x + 20.0,
            panel_y + panel_h - 30.0,
            16.0,
            Color::from_rgba(125, 158, 101, 255),
        );
    }
}
