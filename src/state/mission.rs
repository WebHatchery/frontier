//! Mission state - expedition flow with events and encounters

use super::combat::{CombatState, MissionContext};
use super::{ResultState, StateTransition};
use crate::kingdom::PartyMemberState;
use crate::missions::{MapNode, Mission, NodeType};
use crate::ui::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

/// Active mission/expedition state with branching paths
pub struct MissionState {
    pub mission: Mission,
    /// Current node ID in the map graph
    pub current_node_id: usize,
    /// All party members on this mission (first is leader)
    pub party_members: Vec<PartyMemberState>,
    /// Generated branching map for this mission
    pub map_nodes: Vec<MapNode>,
    /// Nodes that have been visited (by ID)
    pub visited_nodes: Vec<usize>,
    /// If at a fork, which path options are available
    pub available_paths: Vec<usize>,
    /// Selected path index when at a fork
    pub selected_path: usize,
}

impl Default for MissionState {
    fn default() -> Self {
        let mission = Mission::first_mission();
        let map_nodes = mission.generate_branching_map();
        Self {
            mission,
            current_node_id: 0,
            party_members: vec![],
            map_nodes,
            visited_nodes: vec![0],
            available_paths: vec![],
            selected_path: 0,
        }
    }
}

impl MissionState {
    /// Get the party leader
    pub fn leader(&self) -> Option<&PartyMemberState> {
        self.party_members.first()
    }

    /// Get mutable party leader
    #[allow(dead_code)]
    pub fn leader_mut(&mut self) -> Option<&mut PartyMemberState> {
        self.party_members.first_mut()
    }

    /// Create from a Mission object with a full party
    pub fn from_mission_with_party(mission: Mission, party_members: Vec<PartyMemberState>) -> Self {
        let map_nodes = mission.generate_branching_map();
        Self {
            mission,
            current_node_id: 0,
            party_members,
            map_nodes,
            visited_nodes: vec![0],
            available_paths: vec![],
            selected_path: 0,
        }
    }

    /// Set the current node (used when returning from combat)
    pub fn with_node(mut self, node: usize) -> Self {
        self.current_node_id = node;
        self
    }

    /// Set map nodes (used when returning from combat to preserve the generated layout)
    pub fn with_map_nodes(mut self, map_nodes: Vec<MapNode>) -> Self {
        self.map_nodes = map_nodes;
        self
    }

    /// Set visited nodes (used when returning from combat)
    pub fn with_visited(mut self, visited: Vec<usize>) -> Self {
        self.visited_nodes = visited;
        self
    }

    /// Get the current map node
    fn current_node(&self) -> Option<&MapNode> {
        self.map_nodes.iter().find(|n| n.id == self.current_node_id)
    }

    /// Check if mission is complete (visited the final node)
    fn is_complete(&self) -> bool {
        if let Some(node) = self.current_node() {
            node.connections.is_empty()
        } else {
            false
        }
    }

    /// Get node screen position for mouse hit testing (x, y, size)
    fn get_node_screen_pos(&self, node_id: usize) -> Option<(f32, f32, f32)> {
        let node = self.map_nodes.iter().find(|n| n.id == node_id)?;

        let map_y = 180.0;
        let node_size = 48.0;
        let layer_gap = 130.0;
        let node_gap = 78.0;
        let start_x = 350.0;

        let node_x = start_x + (node.layer as f32 * layer_gap);

        let nodes_in_layer: Vec<_> = self
            .map_nodes
            .iter()
            .filter(|n| n.layer == node.layer)
            .collect();
        let layer_height = (nodes_in_layer.len() as f32 - 1.0) * node_gap;
        let layer_start_y = map_y + (260.0 - layer_height) / 2.0;
        let node_y = layer_start_y + (node.position as f32 * node_gap);

        Some((node_x, node_y, node_size))
    }

    /// Process the current node's encounter
    fn process_current_node(&mut self) -> Option<StateTransition> {
        let node = self.current_node()?.clone(); // Clone to avoid borrow issues

        match &node.node_type {
            NodeType::Combat | NodeType::Boss => {
                // Create combat with the full party
                let context = MissionContext {
                    mission: self.mission.clone(),
                    current_node: self.current_node_id,
                    party_members: self.party_members.clone(),
                    map_nodes: self.map_nodes.clone(),
                    visited_nodes: self.visited_nodes.clone(),
                };
                let combat = CombatState::for_mission(context);
                return Some(StateTransition::ToCombat(combat));
            }
            NodeType::Event => {
                if let Some(event) = crate::missions::events::random_event(
                    self.current_node_id,
                    &self.mission.region_id,
                ) {
                    if let Some(leader) = self.leader() {
                        let event_state =
                            super::EventState::new(event, leader.id.clone(), leader.name.clone())
                                .with_mission_context(
                                    self.mission.clone(),
                                    self.current_node_id,
                                    self.party_members.clone(),
                                    self.map_nodes.clone(),
                                    self.visited_nodes.clone(),
                                );
                        return Some(StateTransition::ToEvent(event_state));
                    }
                }
            }
            NodeType::Rest => {
                // Rest nodes heal the party slightly and reduce stress
                for member in &mut self.party_members {
                    let heal = (member.max_hp as f32 * 0.1) as i32;
                    member.hp = (member.hp + heal).min(member.max_hp);
                    member.stress = (member.stress - 5).max(0);
                }
            }
        }
        None
    }

    fn confirm_selected_path(&mut self) -> Option<StateTransition> {
        let next_node_id = *self.available_paths.get(self.selected_path)?;
        self.current_node_id = next_node_id;
        self.visited_nodes.push(next_node_id);
        self.available_paths.clear();
        self.selected_path = 0;
        self.finish_current_node()
    }

    fn finish_current_node(&mut self) -> Option<StateTransition> {
        if let Some(transition) = self.process_current_node() {
            return Some(transition);
        }

        if self.is_complete() {
            return Some(StateTransition::ToResults(
                ResultState::victory_for_mission(&self.mission, &self.party_members),
            ));
        }

        None
    }

    pub(super) fn advance_route(&mut self) -> Option<StateTransition> {
        let connections = self.current_node()?.connections.clone();
        if connections.is_empty() {
            return Some(StateTransition::ToResults(
                ResultState::victory_for_mission(&self.mission, &self.party_members),
            ));
        }

        if connections.len() == 1 {
            self.current_node_id = connections[0];
            self.visited_nodes.push(connections[0]);
            return self.finish_current_node();
        }

        self.available_paths = connections;
        self.selected_path = 0;
        None
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        let (advance_x, advance_y, advance_w, advance_h) = Self::advance_button_rect();
        let (retreat_x, retreat_y, retreat_w, retreat_h) = Self::retreat_button_rect();
        let retreat_clicked = crate::ui::was_clicked(retreat_x, retreat_y, retreat_w, retreat_h);

        if retreat_clicked || is_key_pressed(KeyCode::Escape) {
            return Some(StateTransition::ToBase);
        }

        // Check if we have path options to choose from
        if !self.available_paths.is_empty() {
            // Path selection with arrow keys
            if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A))
                && self.selected_path > 0
            {
                self.selected_path -= 1;
            }
            if (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D))
                && self.selected_path < self.available_paths.len().saturating_sub(1)
            {
                self.selected_path += 1;
            }

            // Number keys for quick selection
            for i in 0..self.available_paths.len().min(9) {
                let key = match i {
                    0 => KeyCode::Key1,
                    1 => KeyCode::Key2,
                    2 => KeyCode::Key3,
                    _ => continue,
                };
                if is_key_pressed(key) {
                    self.selected_path = i;
                }
            }

            // Mouse click on path nodes
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                for (idx, &node_id) in self.available_paths.iter().enumerate() {
                    if let Some((nx, ny, size)) = self.get_node_screen_pos(node_id) {
                        if mx >= nx && mx <= nx + size && my >= ny && my <= ny + size {
                            if self.selected_path == idx {
                                return self.confirm_selected_path();
                            } else {
                                // Select this path
                                self.selected_path = idx;
                                break;
                            }
                        }
                    }
                }
            }

            // Confirm path with the visible button, Space, or Enter.
            if crate::ui::was_clicked(advance_x, advance_y, advance_w, advance_h)
                || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
            {
                return self.confirm_selected_path();
            }
        } else {
            // Advance with the visible button, Space, or Enter.
            if crate::ui::was_clicked(advance_x, advance_y, advance_w, advance_h)
                || is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
            {
                return self.advance_route();
            }
        }

        None
    }

    pub fn draw(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        draw_background(
            textures,
            BackgroundArt::ExpeditionRoute,
            Color::from_rgba(0, 0, 0, 154),
        );

        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            82.0,
            Color::from_rgba(8, 7, 6, 228),
        );
        draw_line(0.0, 82.0, screen_width(), 82.0, 2.0, border_color());
        draw_ui_text(
            &format!("MISSION: {}", self.mission.name),
            24.0,
            38.0,
            28.0,
            title_color(),
        );
        draw_ui_text(
            &format!("{:?} Mission", self.mission.mission_type),
            24.0,
            66.0,
            18.0,
            muted_text_color(),
        );

        let (advance_x, advance_y, advance_w, advance_h) = Self::advance_button_rect();
        let (retreat_x, retreat_y, retreat_w, retreat_h) = Self::retreat_button_rect();
        let advance_label = if self.available_paths.is_empty() {
            if self
                .current_node()
                .is_some_and(|node| node.connections.is_empty())
            {
                "Return"
            } else {
                "Advance"
            }
        } else {
            "Confirm Path"
        };
        Self::draw_action_button(
            advance_label,
            advance_x,
            advance_y,
            advance_w,
            advance_h,
            true,
        );
        Self::draw_action_button("Retreat", retreat_x, retreat_y, retreat_w, retreat_h, true);

        draw_party_panel(&self.party_members, textures);
        draw_legend_panel();
        draw_route_panel();
        // Draw branching map
        self.draw_branching_map(textures);
        draw_current_node_panel(self.current_node(), self.available_paths.is_empty());

        // Instructions
        if self.available_paths.is_empty() {
            draw_ui_text(
                "Tap ADVANCE to continue • Tap RETREAT to leave",
                24.0,
                screen_height() - 24.0,
                16.0,
                ready_color(),
            );
        } else {
            draw_ui_text(
                "Tap a route node to select, tap again to confirm • Tap RETREAT to leave",
                24.0,
                screen_height() - 24.0,
                16.0,
                candle_color(),
            );
        }
    }

    fn advance_button_rect() -> (f32, f32, f32, f32) {
        (screen_width() - 320.0, 22.0, 136.0, 36.0)
    }

    fn retreat_button_rect() -> (f32, f32, f32, f32) {
        (screen_width() - 172.0, 22.0, 136.0, 36.0)
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
        draw_rectangle_lines(x, y, w, h, 1.0, if enabled { GOLD } else { GRAY });
        let text_width = measure_ui_text(label, None, 16, 1.0).width;
        draw_ui_text(
            label,
            x + (w - text_width) / 2.0,
            y + 25.0,
            16.0,
            if enabled { WHITE } else { GRAY },
        );
    }

    /// Draw the branching map visualization
    fn draw_branching_map(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        let map_y = 180.0;
        let node_size = 48.0;
        let layer_gap = 130.0;
        let node_gap = 78.0;
        let start_x = 350.0;

        // Group nodes by layer
        let max_layer = self.map_nodes.iter().map(|n| n.layer).max().unwrap_or(0);

        // First pass: draw connections
        for node in &self.map_nodes {
            let node_x = start_x + (node.layer as f32 * layer_gap);

            // Get nodes in this layer to calculate Y position
            let nodes_in_layer: Vec<_> = self
                .map_nodes
                .iter()
                .filter(|n| n.layer == node.layer)
                .collect();
            let layer_height = (nodes_in_layer.len() as f32 - 1.0) * node_gap;
            let layer_start_y = map_y + (260.0 - layer_height) / 2.0;
            let node_y = layer_start_y + (node.position as f32 * node_gap);

            // Draw connections to next nodes
            for &target_id in &node.connections {
                if let Some(target) = self.map_nodes.iter().find(|n| n.id == target_id) {
                    let target_x = start_x + (target.layer as f32 * layer_gap);
                    let target_nodes_in_layer: Vec<_> = self
                        .map_nodes
                        .iter()
                        .filter(|n| n.layer == target.layer)
                        .collect();
                    let target_layer_height = (target_nodes_in_layer.len() as f32 - 1.0) * node_gap;
                    let target_layer_start_y = map_y + (260.0 - target_layer_height) / 2.0;
                    let target_y = target_layer_start_y + (target.position as f32 * node_gap);

                    // Line color based on whether this is a selectable path
                    let line_color = if self.available_paths.contains(&target_id) {
                        if self.available_paths.get(self.selected_path) == Some(&target_id) {
                            candle_color()
                        } else {
                            Color::from_rgba(104, 137, 90, 255)
                        }
                    } else if self.visited_nodes.contains(&target_id)
                        || self.visited_nodes.contains(&node.id)
                    {
                        ready_color()
                    } else {
                        Color::from_rgba(66, 60, 54, 255)
                    };

                    draw_line(
                        node_x + node_size / 2.0,
                        node_y + node_size / 2.0,
                        target_x + node_size / 2.0,
                        target_y + node_size / 2.0,
                        2.0,
                        line_color,
                    );
                }
            }
        }

        // Second pass: draw nodes
        for node in &self.map_nodes {
            let node_x = start_x + (node.layer as f32 * layer_gap);

            let nodes_in_layer: Vec<_> = self
                .map_nodes
                .iter()
                .filter(|n| n.layer == node.layer)
                .collect();
            let layer_height = (nodes_in_layer.len() as f32 - 1.0) * node_gap;
            let layer_start_y = map_y + (260.0 - layer_height) / 2.0;
            let node_y = layer_start_y + (node.position as f32 * node_gap);

            // Node color
            let (bg_color, border_color) = if node.id == self.current_node_id {
                (Color::from_rgba(155, 106, 36, 255), title_color())
            } else if self.available_paths.contains(&node.id) {
                let is_selected = self.available_paths.get(self.selected_path) == Some(&node.id);
                if is_selected {
                    (Color::from_rgba(91, 126, 75, 255), candle_color())
                } else {
                    (Color::from_rgba(50, 74, 52, 255), ready_color())
                }
            } else if self.visited_nodes.contains(&node.id) {
                (Color::from_rgba(88, 114, 71, 255), title_color())
            } else {
                (Color::from_rgba(35, 33, 31, 255), muted_text_color())
            };

            draw_rectangle(node_x, node_y, node_size, node_size, bg_color);
            draw_rectangle_lines(node_x, node_y, node_size, node_size, 2.0, border_color);

            // Node icon
            let (icon, icon_color) = match &node.node_type {
                NodeType::Combat => (SpriteIcon::Attack, danger_color()),
                NodeType::Boss => (SpriteIcon::Danger, mystery_color()),
                NodeType::Event => (SpriteIcon::Event, info_color()),
                NodeType::Rest => (SpriteIcon::Rest, ready_color()),
            };
            let icon_tint =
                if self.visited_nodes.contains(&node.id) || node.id == self.current_node_id {
                    Color::from_rgba(240, 226, 198, 255)
                } else {
                    icon_color
                };
            draw_icon(textures, icon, node_x + 8.0, node_y + 8.0, 32.0, icon_tint);

            // Show selection number if path choice
            if let Some(idx) = self.available_paths.iter().position(|&id| id == node.id) {
                draw_ui_text(
                    &format!("[{}]", idx + 1),
                    node_x + 15.0,
                    node_y - 5.0,
                    16.0,
                    candle_color(),
                );
            }
        }

        // Progress indicator
        let current_layer = self.current_node().map(|n| n.layer).unwrap_or(0);
        let progress = format!("Layer {}/{}", current_layer + 1, max_layer + 1);
        draw_ui_text(&progress, 350.0, 147.0, 18.0, candle_color());
    }
}

#[cfg(test)]
mod tests;

fn draw_party_panel(
    party_members: &[PartyMemberState],
    textures: &std::collections::HashMap<String, Texture2D>,
) {
    panel(24.0, 104.0, 284.0, 244.0, "EXPEDITION PARTY");
    if party_members.is_empty() {
        draw_ui_text("No party assigned.", 42.0, 156.0, 17.0, muted_text_color());
        return;
    }

    for (i, member) in party_members.iter().enumerate().take(4) {
        let y = 152.0 + (i as f32 * 48.0);
        if let Some(path) = &member.image_path {
            if let Some(tex) = textures.get(path) {
                draw_texture_ex(
                    tex,
                    42.0,
                    y - 30.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(38.0, 38.0)),
                        ..Default::default()
                    },
                );
            }
        }
        draw_ui_text(&member.name, 92.0, y - 9.0, 16.0, text_color());
        draw_ui_text(
            &format!(
                "HP {}/{}    Stress {}",
                member.hp, member.max_hp, member.stress
            ),
            92.0,
            y + 12.0,
            13.0,
            muted_text_color(),
        );
    }
}

fn draw_legend_panel() {
    panel(24.0, 366.0, 284.0, 132.0, "NODE LEGEND");
    let rows = [
        ("?", "Event / unknown", info_color()),
        ("X", "Combat", danger_color()),
        ("+", "Rest", ready_color()),
        ("!", "Boss / critical threat", mystery_color()),
    ];
    for (i, (icon, label, color)) in rows.iter().enumerate() {
        let y = 410.0 + (i as f32 * 23.0);
        draw_ui_text(icon, 46.0, y, 18.0, *color);
        draw_ui_text(label, 76.0, y, 15.0, muted_text_color());
    }
}

fn draw_route_panel() {
    panel(328.0, 104.0, 896.0, 456.0, "EXPEDITION ROUTE");
    draw_ui_text(
        "Read the route before committing. Branches become decisions when the trail forks.",
        350.0,
        536.0,
        15.0,
        muted_text_color(),
    );
}

fn draw_current_node_panel(node: Option<&MapNode>, can_advance: bool) {
    panel(24.0, 516.0, 284.0, 104.0, "CURRENT REPORT");
    let Some(node) = node else {
        draw_ui_text(
            "Route data unavailable.",
            42.0,
            568.0,
            16.0,
            muted_text_color(),
        );
        return;
    };
    let label = match node.node_type {
        NodeType::Combat => "Combat contact ahead",
        NodeType::Event => "Uncertain trail marker",
        NodeType::Rest => "Rest point",
        NodeType::Boss => "Command warning: boss",
    };
    draw_ui_text(label, 42.0, 566.0, 17.0, text_color());
    draw_ui_text(
        if can_advance {
            "Advance to reveal the report."
        } else {
            "Choose the next route."
        },
        42.0,
        594.0,
        14.0,
        muted_text_color(),
    );
}

fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    draw_rectangle(x, y, w, h, Color::from_rgba(13, 11, 10, 210));
    draw_rectangle(x, y, w, 32.0, Color::from_rgba(42, 30, 18, 222));
    draw_rectangle_lines(x, y, w, h, 1.0, border_color());
    draw_ui_text(title, x + 14.0, y + 22.0, 15.0, candle_color());
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

fn ready_color() -> Color {
    Color::from_rgba(130, 177, 101, 255)
}

fn danger_color() -> Color {
    Color::from_rgba(168, 58, 48, 255)
}

fn info_color() -> Color {
    Color::from_rgba(118, 151, 164, 255)
}

fn mystery_color() -> Color {
    Color::from_rgba(138, 104, 167, 255)
}

fn border_color() -> Color {
    Color::from_rgba(105, 76, 43, 210)
}
