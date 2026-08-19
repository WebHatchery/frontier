//! Combat resolution - effects are validated and applied here

use super::{CardEffect, ResolutionDelta, Unit};

/// Turn-specific modifiers that reset at end of turn
#[derive(Clone, Debug, Default)]
pub struct TurnModifiers {
    /// Percentage reduction to incoming stress (e.g., 50 = 50% reduction)
    pub stress_resistance: i32,
    /// If true, cannot play attack cards
    pub attacks_disabled: bool,
    /// Tracks if enemy took an action last turn (for conditional effects)
    pub enemy_acted_last_turn: bool,
    /// Extra energy gained next turn
    pub energy_next_turn: i32,
    /// Cards to draw (accumulated during resolution)
    pub cards_to_draw: i32,
    /// Energy to gain (accumulated during resolution)
    pub energy_to_gain: i32,
}

impl TurnModifiers {
    pub fn reset(&mut self) {
        self.stress_resistance = 0;
        self.attacks_disabled = false;
        self.cards_to_draw = 0;
        self.energy_to_gain = 0;
        // Note: enemy_acted_last_turn and energy_next_turn are updated by combat logic
    }

    /// Called at start of new turn to apply energy from previous turn
    #[allow(dead_code)]
    pub fn start_turn(&mut self) -> i32 {
        let extra = self.energy_next_turn;
        self.energy_next_turn = 0;
        extra
    }
}

/// Resolves card effects into state changes
pub struct CombatResolver {
    /// Log of resolved effects for replay/debugging
    pub log: Vec<String>,
    /// Turn-specific modifiers
    pub turn_mods: TurnModifiers,
    /// Most recent effect deltas shown in the battle report.
    pub recent_resolutions: Vec<ResolutionDelta>,
}

impl CombatResolver {
    pub fn new() -> Self {
        Self {
            log: Vec::new(),
            turn_mods: TurnModifiers::default(),
            recent_resolutions: Vec::new(),
        }
    }

    /// Reset turn modifiers at end of turn
    pub fn end_turn(&mut self, enemy_acted: bool) {
        self.turn_mods.reset();
        self.turn_mods.enemy_acted_last_turn = enemy_acted;
    }

    /// Apply stress with resistance considered
    pub fn apply_stress_to_player(&self, player: &mut Unit, amount: i32) {
        let reduced = if self.turn_mods.stress_resistance > 0 {
            let reduction = (amount * self.turn_mods.stress_resistance) / 100;
            (amount - reduction).max(0)
        } else {
            amount
        };
        player.add_stress(reduced);
    }

    /// Resolve an effect from player to target (or self)
    pub fn resolve(
        &mut self,
        effect: &CardEffect,
        player: &mut Unit,
        target: &mut Unit,
    ) -> ResolutionDelta {
        let player_before = player.clone();
        let target_before = target.clone();
        match effect {
            CardEffect::Damage(amount) => {
                let mut dmg = *amount;
                // Apply Strength
                if let Some(s) = player
                    .statuses
                    .iter()
                    .find(|s| s.effect_type == crate::kingdom::StatusType::Strength)
                {
                    dmg += s.value;
                }
                // Apply Weak (-25%)
                if player.has_status(crate::kingdom::StatusType::Weak) {
                    dmg = (dmg as f32 * 0.75) as i32;
                }
                target.take_damage(dmg);
                self.log
                    .push(format!("{} takes {} damage", target.name, dmg));
            }
            CardEffect::Block(amount) => {
                player.add_block(*amount);
                self.log
                    .push(format!("{} gains {} block", player.name, amount));
            }
            CardEffect::Stress(amount) => {
                target.add_stress(*amount);
                self.log
                    .push(format!("{} gains {} stress", target.name, amount));
            }
            CardEffect::SelfStress(amount) => {
                player.add_stress(*amount);
                self.log
                    .push(format!("{} gains {} stress (self)", player.name, amount));
            }
            CardEffect::ReduceStress(amount) => {
                if player
                    .traumas
                    .iter()
                    .any(|t| t.trauma_type == crate::kingdom::TraumaType::Hopeless)
                {
                    self.log
                        .push(format!("{} cannot calm down while Hopeless", player.name));
                } else {
                    player.reduce_stress(*amount);
                    self.log
                        .push(format!("{} reduces stress by {}", player.name, amount));
                }
            }
            CardEffect::Heal(amount) => {
                player.heal(*amount);
                self.log
                    .push(format!("{} heals {} HP", player.name, amount));
            }
            CardEffect::DrawCards(count) => {
                self.turn_mods.cards_to_draw += count;
                self.log
                    .push(format!("{} will draw {} card(s)", player.name, count));
            }
            CardEffect::GainEnergy(amount) => {
                self.turn_mods.energy_to_gain += amount;
                self.log
                    .push(format!("{} will gain {} energy", player.name, amount));
            }
            CardEffect::GainEnergyNextTurn(amount) => {
                self.turn_mods.energy_next_turn += amount;
                self.log.push(format!(
                    "{} will gain {} energy next turn",
                    player.name, amount
                ));
            }
            CardEffect::ClearDebuffs => {
                player.clear_debuffs();
                self.log.push(format!("{} clears all debuffs", player.name));
            }
            CardEffect::EnemyStress(amount) => {
                // Enemies don't have stress in the same way, but this could reduce their effectiveness
                self.log
                    .push(format!("{} gains {} stress", target.name, amount));
            }
            CardEffect::DamageIfNoBlock { base, bonus } => {
                let total = if target.block == 0 {
                    base + bonus
                } else {
                    *base
                };
                target.take_damage(total);
                self.log
                    .push(format!("{} takes {} damage", target.name, total));
            }
            CardEffect::DamageIfLowHp {
                base,
                bonus,
                threshold_percent,
            } => {
                let threshold = (player.max_hp * threshold_percent) / 100;
                let total = if player.hp < threshold {
                    base + bonus
                } else {
                    *base
                };
                target.take_damage(total);
                self.log
                    .push(format!("{} takes {} damage", target.name, total));
            }
            CardEffect::DamageIfEnemyActed { base, bonus } => {
                let total = if self.turn_mods.enemy_acted_last_turn {
                    base + bonus
                } else {
                    *base
                };
                let mut dmg = total;
                // Apply Strength
                if let Some(s) = player
                    .statuses
                    .iter()
                    .find(|s| s.effect_type == crate::kingdom::StatusType::Strength)
                {
                    dmg += s.value;
                }
                // Apply Weak (-25%)
                if player.has_status(crate::kingdom::StatusType::Weak) {
                    dmg = (dmg as f32 * 0.75) as i32;
                }
                target.take_damage(dmg);
                self.log.push(format!(
                    "{} takes {} damage (enemy acted: {})",
                    target.name, dmg, self.turn_mods.enemy_acted_last_turn
                ));
            }
            CardEffect::DamageIfVulnerable { base, bonus } => {
                let is_vulnerable = target.has_status(crate::kingdom::StatusType::Vulnerable);
                let total = if is_vulnerable { base + bonus } else { *base };
                target.take_damage(total);
                self.log.push(format!(
                    "{} takes {} damage (vulnerable: {})",
                    target.name, total, is_vulnerable
                ));
            }
            CardEffect::ApplyStatus {
                effect_type,
                duration,
                value,
                target_self,
            } => {
                let status =
                    crate::kingdom::StatusEffect::new(effect_type.clone(), *duration, *value);
                let who = if *target_self {
                    &mut *player
                } else {
                    &mut *target
                };
                who.add_status(status);
                self.log.push(format!(
                    "{} gains {:?} for {} turns",
                    who.name, effect_type, duration
                ));
            }
            CardEffect::StressResistance(percent) => {
                self.turn_mods.stress_resistance = self.turn_mods.stress_resistance.max(*percent);
                self.log.push(format!(
                    "{} gains {}% stress resistance this turn",
                    player.name, percent
                ));
            }
            CardEffect::DisableAttacks => {
                self.turn_mods.attacks_disabled = true;
                self.log
                    .push(format!("{} cannot play attacks this turn", player.name));
            }
        }

        let affects_player = matches!(
            effect,
            CardEffect::Block(_)
                | CardEffect::SelfStress(_)
                | CardEffect::ReduceStress(_)
                | CardEffect::Heal(_)
                | CardEffect::DrawCards(_)
                | CardEffect::GainEnergy(_)
                | CardEffect::GainEnergyNextTurn(_)
                | CardEffect::ClearDebuffs
                | CardEffect::StressResistance(_)
                | CardEffect::DisableAttacks
                | CardEffect::ApplyStatus {
                    target_self: true,
                    ..
                }
        );
        let (before, after, target_name) = if affects_player {
            (&player_before, &*player, player.name.as_str())
        } else {
            (&target_before, &*target, target.name.as_str())
        };
        let blocked_damage = (before.block - after.block).max(0);
        let delta = ResolutionDelta::from_snapshots(
            &player_before.name,
            target_name,
            effect_label(effect),
            before,
            after,
            blocked_damage,
        );
        self.log.push(delta.summary());
        self.record_resolution(delta.clone());
        delta
    }

    pub fn record_external(&mut self, delta: ResolutionDelta) {
        self.log.push(delta.summary());
        self.record_resolution(delta);
    }

    pub fn clear_recent(&mut self) {
        self.recent_resolutions.clear();
    }

    fn record_resolution(&mut self, delta: ResolutionDelta) {
        self.recent_resolutions.push(delta);
        if self.recent_resolutions.len() > 6 {
            self.recent_resolutions.remove(0);
        }
    }
}

fn effect_label(effect: &CardEffect) -> &'static str {
    match effect {
        CardEffect::Damage(_) => "Damage",
        CardEffect::Block(_) => "Block",
        CardEffect::Stress(_) | CardEffect::SelfStress(_) => "Stress",
        CardEffect::ReduceStress(_) => "Stress relief",
        CardEffect::Heal(_) => "Heal",
        CardEffect::DrawCards(_) => "Draw cards",
        CardEffect::GainEnergy(_) | CardEffect::GainEnergyNextTurn(_) => "Energy",
        CardEffect::ClearDebuffs => "Clear debuffs",
        CardEffect::EnemyStress(_) => "Enemy stress",
        CardEffect::DamageIfNoBlock { .. } => "Damage if unguarded",
        CardEffect::DamageIfLowHp { .. } => "Damage at low HP",
        CardEffect::DamageIfEnemyActed { .. } => "Punish enemy action",
        CardEffect::DamageIfVulnerable { .. } => "Damage vulnerable target",
        CardEffect::ApplyStatus { .. } => "Status",
        CardEffect::StressResistance(_) => "Stress resistance",
        CardEffect::DisableAttacks => "Disable attacks",
    }
}

impl Default for CombatResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
