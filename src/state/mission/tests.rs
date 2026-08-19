use super::*;

fn member() -> PartyMemberState {
    PartyMemberState {
        id: "leader".to_string(),
        name: "Leader".to_string(),
        hp: 45,
        max_hp: 45,
        stress: 0,
        image_path: None,
        class_name: "Soldier".to_string(),
        deck_additions: vec![],
        traumas: vec![],
        resolve_state: None,
    }
}

fn route(node_type: NodeType) -> MissionState {
    MissionState {
        mission: Mission::first_mission(),
        current_node_id: 0,
        party_members: vec![member()],
        map_nodes: vec![
            MapNode {
                id: 0,
                node_type: NodeType::Event,
                connections: vec![1],
                layer: 0,
                position: 0,
            },
            MapNode {
                id: 1,
                node_type,
                connections: vec![],
                layer: 1,
                position: 0,
            },
        ],
        visited_nodes: vec![0],
        available_paths: vec![],
        selected_path: 0,
    }
}

#[test]
fn reaching_a_safe_final_node_produces_a_success_result() {
    let mut state = route(NodeType::Rest);

    let transition = state.advance_route();
    match transition {
        Some(StateTransition::ToResults(result)) => {
            assert!(result.victory);
            assert_eq!(result.mission_id.as_deref(), Some("scout_dark_woods"));
        }
        _ => panic!("route should resolve into a successful result"),
    }
}

#[test]
fn reaching_a_combat_node_launches_combat_before_completion() {
    let mut state = route(NodeType::Combat);

    assert!(matches!(
        state.advance_route(),
        Some(StateTransition::ToCombat(_))
    ));
}
