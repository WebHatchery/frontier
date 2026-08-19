use super::*;
use crate::combat::{CardEffect, Unit};
use crate::kingdom::StatusType;

#[test]
fn damage_delta_reports_actual_and_blocked_damage() {
    let mut resolver = CombatResolver::new();
    let mut player = Unit::new_player("Marcus", 45);
    let mut enemy = Unit::new_enemy("Boar", 30, None);
    enemy.block = 4;

    let delta = resolver.resolve(&CardEffect::Damage(7), &mut player, &mut enemy);

    assert_eq!(delta.target_hp_delta, -3);
    assert_eq!(delta.blocked_damage, 4);
    assert!(delta.summary().contains("4 blocked"));
}

#[test]
fn block_and_status_deltas_name_the_affected_player_and_enemy() {
    let mut resolver = CombatResolver::new();
    let mut player = Unit::new_player("Elena", 35);
    let mut enemy = Unit::new_enemy("Wolf", 20, None);

    let block = resolver.resolve(&CardEffect::Block(5), &mut player, &mut enemy);
    assert_eq!(block.target, "Elena");
    assert_eq!(block.target_block_delta, 5);

    let status = resolver.resolve(
        &CardEffect::ApplyStatus {
            effect_type: StatusType::Vulnerable,
            duration: 2,
            value: 0,
            target_self: false,
        },
        &mut player,
        &mut enemy,
    );
    assert_eq!(status.target, "Wolf");
    assert_eq!(status.status_changes, vec!["gained Vulnerable"]);
}
