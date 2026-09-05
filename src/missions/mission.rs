//! Mission definitions and types
//!
//! Expeditions are contracts with uncertainty.

use serde::{Deserialize, Serialize};
// use crate::data::load_asset;
use crate::kingdom::UnlockRequirement;

/// Mission types from the GDD
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MissionType {
    /// Low danger, knowledge-focused
    Scout,
    /// Combat-heavy, stabilizes regions
    Suppress,
    /// Enables trade or settlement
    Secure,
    /// Narrative events, high stress
    Investigate,
}

/// Type of encounter at a mission node
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    /// Combat encounter
    Combat,
    /// Narrative event with choices
    Event,
    /// Safe rest point
    Rest,
    /// Boss encounter (end of mission)
    Boss,
}

/// A node in a mission map (supports branching paths)
#[derive(Clone, Debug)]
pub struct MapNode {
    /// Unique ID within this mission
    pub id: usize,
    /// What happens at this node
    pub node_type: NodeType,
    /// Which nodes can be reached from here (indices into the map)
    pub connections: Vec<usize>,
    /// Row/layer in the map (for drawing)
    pub layer: usize,
    /// Position within the layer (0 = left, higher = right)
    pub position: usize,
}

/// A mission available to undertake
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub mission_type: MissionType,
    pub region_id: String,

    /// Number of nodes in the expedition
    pub length: usize,

    /// Base difficulty (affects encounter strength)
    pub difficulty: i32,

    /// Expected rewards
    #[serde(default = "default_reward_gold")]
    pub reward_gold: i32,
    pub reward_supplies: i32,
    pub reward_knowledge: i32,
    pub reward_influence: i32,

    /// Stress this mission is likely to cause
    pub base_stress: i32,

    /// Requirement to unlock this mission
    #[serde(default)]
    pub unlock_requirement: UnlockRequirement,
}

fn default_reward_gold() -> i32 {
    20
}

impl Mission {
    /// First available mission - Scout the Dark Woods
    pub fn first_mission() -> Self {
        Self {
            id: "scout_dark_woods".to_string(),
            name: "Scout the Dark Woods".to_string(),
            description: "Map the forest edge. Avoid direct confrontation if possible.".to_string(),
            mission_type: MissionType::Scout,
            region_id: "dark_woods".to_string(),
            length: 5,
            difficulty: 1,
            reward_gold: 25,
            reward_supplies: 10,
            reward_knowledge: 15,
            reward_influence: 0,
            base_stress: 8,
            unlock_requirement: UnlockRequirement::None,
        }
    }

    /// Combat-focused mission
    pub fn suppress_beasts() -> Self {
        Self {
            id: "suppress_beasts".to_string(),
            name: "Suppress Forest Beasts".to_string(),
            description: "Clear the creatures blocking the main road.".to_string(),
            mission_type: MissionType::Suppress,
            region_id: "dark_woods".to_string(),
            length: 6,
            difficulty: 2,
            reward_gold: 45,
            reward_supplies: 20,
            reward_knowledge: 5,
            reward_influence: 10,
            base_stress: 15,
            unlock_requirement: UnlockRequirement::None,
        }
    }

    /// Generate node types for this mission based on mission type
    /// Returns a Vec of NodeTypes, one for each node in the mission
    /// Note: Kept for potential fallback; replaced by generate_branching_map
    #[allow(dead_code)]
    pub fn generate_node_types(&self) -> Vec<NodeType> {
        // Combat probability based on mission type
        let combat_chance = match self.mission_type {
            MissionType::Scout => 0.25,       // 25% combat
            MissionType::Suppress => 0.60,    // 60% combat
            MissionType::Secure => 0.40,      // 40% combat
            MissionType::Investigate => 0.20, // 20% combat
        };

        let mut nodes = Vec::with_capacity(self.length);

        for i in 0..self.length {
            // First node is always an event (arrival)
            if i == 0 {
                nodes.push(NodeType::Event);
                continue;
            }

            // Last node: Boss for Suppress, Event for others
            if i == self.length - 1 {
                let final_node = match self.mission_type {
                    MissionType::Suppress => NodeType::Boss,
                    _ => NodeType::Event,
                };
                nodes.push(final_node);
                continue;
            }

            // Middle nodes: random based on combat chance
            // Add rest points occasionally (every 3rd-4th node if not combat)
            let roll: f32 = macroquad_toolkit::rng::rand();
            if roll < combat_chance {
                nodes.push(NodeType::Combat);
            } else if i > 0 && i % 3 == 0 {
                // Every 3rd node that isn't combat could be a rest
                let rest_roll: f32 = macroquad_toolkit::rng::rand();
                if rest_roll < 0.3 {
                    nodes.push(NodeType::Rest);
                } else {
                    nodes.push(NodeType::Event);
                }
            } else {
                nodes.push(NodeType::Event);
            }
        }

        nodes
    }

    /// Get effective difficulty for combat (Suppress = harder, Scout = easier)
    pub fn combat_difficulty(&self) -> i32 {
        match self.mission_type {
            MissionType::Scout => self.difficulty.saturating_sub(1).max(1),
            MissionType::Suppress => self.difficulty + 1,
            _ => self.difficulty,
        }
    }

    /// Clone this mission with global threat pressure applied.
    pub fn scaled_for_kingdom(&self, kingdom: &crate::kingdom::KingdomState) -> Self {
        let mut mission = self.clone();
        mission.difficulty += kingdom.threat_difficulty_bonus();
        mission.base_stress += kingdom.scaled_stress_bonus();
        mission
    }

    /// Generate a branching map for this mission
    /// Returns a Vec of MapNodes forming a layered graph
    pub fn generate_branching_map(&self) -> Vec<MapNode> {
        let num_layers = self.length;
        let mut nodes: Vec<MapNode> = Vec::new();
        let mut node_id = 0;

        // Combat probability based on mission type
        let combat_chance = match self.mission_type {
            MissionType::Scout => 0.25,
            MissionType::Suppress => 0.60,
            MissionType::Secure => 0.40,
            MissionType::Investigate => 0.20,
        };

        // Track node indices at each layer for connecting
        let mut layer_nodes: Vec<Vec<usize>> = Vec::new();

        for layer in 0..num_layers {
            // Determine how many nodes in this layer
            // First and last layers have 1 node, middle layers have 1-3
            let nodes_in_layer = if layer == 0 || layer == num_layers - 1 {
                1
            } else {
                // More branches for longer missions
                let max_branches = if self.length >= 6 { 3 } else { 2 };
                macroquad_toolkit::rng::gen_range(1, max_branches + 1)
            };

            let mut layer_node_indices = Vec::new();

            for pos in 0..nodes_in_layer {
                // Determine node type
                let node_type = if layer == 0 {
                    NodeType::Event // Start is always event
                } else if layer == num_layers - 1 {
                    match self.mission_type {
                        MissionType::Suppress => NodeType::Boss,
                        _ => NodeType::Event,
                    }
                } else {
                    // Random based on mission type
                    let roll: f32 = macroquad_toolkit::rng::rand();
                    if roll < combat_chance {
                        NodeType::Combat
                    } else if layer % 3 == 0 && macroquad_toolkit::rng::chance(0.3) {
                        NodeType::Rest
                    } else {
                        NodeType::Event
                    }
                };

                let node = MapNode {
                    id: node_id,
                    node_type,
                    connections: Vec::new(), // Will be filled in next pass
                    layer,
                    position: pos,
                };

                layer_node_indices.push(node_id);
                nodes.push(node);
                node_id += 1;
            }

            layer_nodes.push(layer_node_indices);
        }

        // Connect layers - each node connects to 1-2 nodes in next layer
        for layer in 0..num_layers.saturating_sub(1) {
            let current_layer = &layer_nodes[layer];
            let next_layer = &layer_nodes[layer + 1];

            for &node_idx in current_layer {
                // Connect to at least one node in next layer
                let num_connections = if next_layer.len() == 1 {
                    1
                } else {
                    macroquad_toolkit::rng::gen_range(1, 2.min(next_layer.len()) + 1)
                };

                // Pick which nodes to connect to
                let mut available: Vec<usize> = next_layer.clone();
                for _ in 0..num_connections {
                    if available.is_empty() {
                        break;
                    }
                    let pick = macroquad_toolkit::rng::gen_range(0, available.len());
                    nodes[node_idx].connections.push(available[pick]);
                    available.remove(pick);
                }
            }

            // Ensure all nodes in next layer are reachable
            for &next_node in next_layer {
                let has_incoming = current_layer
                    .iter()
                    .any(|&n| nodes[n].connections.contains(&next_node));
                if !has_incoming && !current_layer.is_empty() {
                    // Add connection from random node in current layer
                    let from =
                        current_layer[macroquad_toolkit::rng::gen_range(0, current_layer.len())];
                    nodes[from].connections.push(next_node);
                }
            }
        }

        nodes
    }
}

/// Load missions from the JSON asset file
pub fn load_missions() -> Vec<Mission> {
    match macroquad_toolkit::data_loader::load_json_file_with_fallback_sync::<Vec<Mission>>(
        "assets/missions.json",
        macroquad_toolkit::include_json_str!("../../assets/missions.json"),
        macroquad_toolkit::data_loader::JsonFallbackPolicy::ReadError,
    ) {
        Ok(missions) => missions,
        Err(e) => {
            eprintln!("Warning: Could not load missions.json: {}", e);
            // Fallback to hardcoded missions
            available_missions()
        }
    }
}

/// Available missions for the player to choose from (hardcoded fallback)
pub fn available_missions() -> Vec<Mission> {
    vec![Mission::first_mission(), Mission::suppress_beasts()]
}
