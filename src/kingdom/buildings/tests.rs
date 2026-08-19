use super::*;
use crate::kingdom::KingdomState;

#[test]
fn base_upgrade_requires_both_resources() {
    let building = Building::foundry();
    let mut stats = KingdomState::default().stats;
    stats.gold = building.cost_gold;
    stats.supplies = building.cost_supplies - 1;

    assert_eq!(evaluate_construction(&building, &stats), None);
}

#[test]
fn base_upgrade_evaluator_returns_exact_costs() {
    let building = Building::watchtowers();
    let stats = KingdomState::default().stats;

    assert_eq!(
        evaluate_construction(&building, &stats),
        Some(ConstructionEvaluation {
            cost_gold: building.cost_gold,
            cost_supplies: building.cost_supplies,
        })
    );
}
