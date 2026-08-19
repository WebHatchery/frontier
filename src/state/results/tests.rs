use super::*;

#[test]
fn failure_result_preserves_party_and_has_no_rewards() {
    let party = vec![PartyMemberState {
        id: "leader".to_string(),
        name: "Leader".to_string(),
        hp: 0,
        max_hp: 45,
        stress: 100,
        image_path: None,
        class_name: "Soldier".to_string(),
        deck_additions: vec![],
        traumas: vec![],
        resolve_state: None,
    }];
    let result = ResultState::defeat_for_mission(&Mission::first_mission(), &party);

    assert!(!result.victory);
    assert_eq!(result.party_member_states.len(), 1);
    assert_eq!(result.reward_gold, 0);
    assert_eq!(result.reward_supplies, 0);
}

#[test]
fn victory_result_progresses_the_mission_chain_with_rewards() {
    let result = ResultState::victory_for_mission(&Mission::suppress_beasts(), &[]);

    assert!(result.victory);
    assert_eq!(result.mission_id.as_deref(), Some("suppress_beasts"));
    assert_eq!(result.reward_gold, 45);
    assert_eq!(result.reward_influence, 10);
}
