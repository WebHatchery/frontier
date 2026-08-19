//! Readable deltas produced by combat effect resolution.

use super::Unit;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolutionDelta {
    pub source: String,
    pub target: String,
    pub action: String,
    pub target_hp_delta: i32,
    pub target_block_delta: i32,
    pub target_stress_delta: i32,
    pub blocked_damage: i32,
    pub status_changes: Vec<String>,
}

impl ResolutionDelta {
    pub fn from_snapshots(
        source: &str,
        target: &str,
        action: impl Into<String>,
        before: &Unit,
        after: &Unit,
        blocked_damage: i32,
    ) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            action: action.into(),
            target_hp_delta: after.hp - before.hp,
            target_block_delta: after.block - before.block,
            target_stress_delta: after.stress - before.stress,
            blocked_damage,
            status_changes: status_changes(&before.statuses, &after.statuses),
        }
    }

    pub fn summary(&self) -> String {
        if self.blocked_damage > 0 && self.target_hp_delta >= 0 {
            return format!(
                "{} blocked {} damage from {}",
                self.target, self.blocked_damage, self.source
            );
        }
        if self.target_hp_delta < 0 {
            let damage = -self.target_hp_delta;
            if self.blocked_damage > 0 {
                return format!(
                    "{} dealt {} damage to {} ({} blocked)",
                    self.source, damage, self.target, self.blocked_damage
                );
            }
            return format!("{} dealt {} damage to {}", self.source, damage, self.target);
        }
        if self.target_hp_delta > 0 {
            return format!("{} restored {} HP", self.target, self.target_hp_delta);
        }
        if self.target_block_delta > 0 {
            return format!("{} gained {} block", self.target, self.target_block_delta);
        }
        if self.target_stress_delta != 0 {
            return format!(
                "{} {} {} stress",
                self.target,
                if self.target_stress_delta > 0 {
                    "gained"
                } else {
                    "shed"
                },
                self.target_stress_delta.abs()
            );
        }
        if let Some(status) = self.status_changes.first() {
            return format!("{} {}", self.target, status);
        }
        format!("{}: {}", self.source, self.action)
    }
}

fn status_changes(
    before: &[crate::kingdom::StatusEffect],
    after: &[crate::kingdom::StatusEffect],
) -> Vec<String> {
    let mut changes = Vec::new();
    for status in after {
        let had_status = before
            .iter()
            .any(|old| old.effect_type == status.effect_type);
        if !had_status {
            changes.push(format!("gained {:?}", status.effect_type));
        }
    }
    for status in before {
        let still_has_status = after
            .iter()
            .any(|new| new.effect_type == status.effect_type);
        if !still_has_status {
            changes.push(format!("lost {:?}", status.effect_type));
        }
    }
    changes
}
