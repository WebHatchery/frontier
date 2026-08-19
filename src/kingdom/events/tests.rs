use super::*;

#[test]
fn each_kingdom_event_has_a_stable_stat_delta() {
    let plague = evaluate_kingdom_event(KingdomEventKind::Plague, 100);
    assert_eq!(plague.morale_delta, -6);
    assert_eq!(plague.stress_delta, 6);

    let thieves = evaluate_kingdom_event(KingdomEventKind::Thieves, 10);
    assert_eq!(thieves.gold_delta, -10);

    let traders = evaluate_kingdom_event(KingdomEventKind::Traders, 20);
    assert_eq!(traders.gold_delta, -15);
    assert_eq!(traders.supplies_delta, 25);

    let poor_traders = evaluate_kingdom_event(KingdomEventKind::Traders, 5);
    assert_eq!(poor_traders.gold_delta, 10);

    let festival = evaluate_kingdom_event(KingdomEventKind::Festival, 100);
    assert_eq!(festival.morale_delta, 8);
    assert_eq!(festival.stress_delta, -3);
}
