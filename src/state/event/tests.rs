use super::*;
use crate::missions::events::Event;
use crate::missions::{MapNode, NodeType};

#[test]
fn combat_triggering_event_choice_launches_the_named_enemy() {
    let mission = Mission::first_mission();
    let member = PartyMemberState {
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
    };
    let map_nodes = vec![MapNode {
        id: 0,
        node_type: NodeType::Event,
        connections: vec![1],
        layer: 0,
        position: 0,
    }];
    let mut event = EventState::new(
        Event::twisted_path(),
        "leader".to_string(),
        "Leader".to_string(),
    )
    .with_mission_context(mission.clone(), 0, vec![member], map_nodes, vec![0]);
    event.selected_choice = 1;

    match event.confirm_choice() {
        Some(StateTransition::ToCombat(combat)) => assert_eq!(combat.enemy.name, "Shadow Wolf"),
        _ => panic!("the clear path should trigger the named combat encounter"),
    }
}
