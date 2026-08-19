use super::*;

#[test]
fn low_resources_block_recruitment_without_changing_the_price() {
    let evaluation = evaluate_recruitment(39, 3, recruit_cost(&AdventurerClass::Scout));

    assert_eq!(evaluation.cost, 40);
    assert!(!evaluation.can_hire);
    assert_eq!(
        evaluation.blocked_by,
        Some(RecruitmentBlock::InsufficientGold)
    );
}

#[test]
fn full_roster_blocks_even_when_the_kingdom_can_pay() {
    let evaluation = evaluate_recruitment(500, MAX_ROSTER_SIZE, 50);

    assert!(!evaluation.can_hire);
    assert_eq!(evaluation.blocked_by, Some(RecruitmentBlock::RosterFull));
}

#[test]
fn affordable_recruitment_is_allowed_below_capacity() {
    let evaluation = evaluate_recruitment(75, MAX_ROSTER_SIZE - 1, 60);

    assert!(evaluation.can_hire);
    assert_eq!(evaluation.blocked_by, None);
}
