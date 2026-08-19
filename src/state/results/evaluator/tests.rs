use super::*;

fn member(id: &str, hp: i32, max_hp: i32, stress: i32) -> PartyMemberState {
    PartyMemberState {
        id: id.to_string(),
        name: id.to_string(),
        hp,
        max_hp,
        stress,
        image_path: None,
        class_name: "Soldier".to_string(),
        deck_additions: vec![],
        traumas: vec![],
        resolve_state: None,
    }
}

#[test]
fn victory_rewards_are_copied_from_the_mission() {
    let rewards = calculate_rewards(true, &Mission::suppress_beasts());

    assert_eq!(rewards.gold, 45);
    assert_eq!(rewards.supplies, 20);
    assert_eq!(rewards.knowledge, 5);
    assert_eq!(rewards.influence, 10);
}

#[test]
fn failure_has_no_rewards_and_marks_survivor_injury() {
    let rewards = calculate_rewards(false, &Mission::first_mission());
    let consequences =
        calculate_party_consequences(false, 1, 15, &[member("survivor", 20, 45, 80)]);

    assert_eq!(rewards, RewardBreakdown::default());
    assert_eq!(consequences[0].final_stress, 95);
    assert!(!consequences[0].died);
    assert_eq!(consequences[0].injury_ids, vec!["broken_arm"]);
}

#[test]
fn damage_stress_and_death_are_preserved_for_a_party() {
    let consequences = calculate_party_consequences(
        true,
        3,
        8,
        &[member("fallen", 0, 45, 20), member("wounded", 10, 45, 90)],
    );

    assert!(consequences[0].died);
    assert_eq!(consequences[0].final_hp, 0);
    assert_eq!(consequences[1].final_hp, 10);
    assert_eq!(consequences[1].final_stress, 98);
    assert_eq!(consequences[1].injury_ids, vec!["wounded_leg"]);
    assert_eq!(consequences[1].xp_gain, 16);
}
