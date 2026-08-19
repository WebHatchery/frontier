use super::*;
use crate::kingdom::KingdomState;

#[test]
fn mission_chain_unlocks_only_after_the_required_step() {
    let requirement = UnlockRequirement::MissionComplete {
        mission_id: "scout_outpost".to_string(),
    };
    let mut kingdom = KingdomState::default();

    assert!(!requirement.is_met(&kingdom));
    kingdom.record_mission_complete("scout_outpost");
    assert!(requirement.is_met(&kingdom));
}

#[test]
fn building_and_knowledge_unlocks_are_independent_requirements() {
    let mut kingdom = KingdomState::default();
    let building = UnlockRequirement::Building {
        building: "watchtowers".to_string(),
    };
    let knowledge = UnlockRequirement::Knowledge { amount: 50 };

    assert!(!building.is_met(&kingdom));
    assert!(!knowledge.is_met(&kingdom));
    kingdom
        .buildings
        .iter_mut()
        .find(|b| b.id == "watchtowers")
        .unwrap()
        .built = true;
    kingdom.stats.knowledge = 50;
    assert!(building.is_met(&kingdom));
    assert!(knowledge.is_met(&kingdom));
}
