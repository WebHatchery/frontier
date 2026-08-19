use super::*;

#[test]
fn locked_missions_cannot_launch() {
    let mut state =
        MissionSelectState::new("leader".to_string(), "Leader".to_string(), 45, 45, 0, None);
    state.selected_mission = state
        .missions
        .iter()
        .position(|mission| mission.id == "scout_outpost")
        .expect("mission fixture should contain scout_outpost");

    assert!(state
        .start_selected_mission(&KingdomState::default())
        .is_none());
}

#[test]
fn unlocked_missions_launch_with_the_selected_party() {
    let mut state =
        MissionSelectState::new("leader".to_string(), "Leader".to_string(), 45, 45, 0, None);
    state.selected_mission = state
        .missions
        .iter()
        .position(|mission| mission.id == "scout_dark_woods")
        .expect("starter mission fixture should exist");

    assert!(matches!(
        state.start_selected_mission(&KingdomState::default()),
        Some(StateTransition::ToMission(_))
    ));
}
