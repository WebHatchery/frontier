//! Combat units - players and enemies

use crate::kingdom::{ResolveState, StatusEffect, StatusType, Trauma, TraumaType};
use serde::{Deserialize, Serialize};

/// What an enemy intends to do next turn
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EnemyIntent {
    Attack(i32), // Damage amount
    Block(i32),  // Block amount
    Buff,        // Strengthening self
    Debuff,      // Weakening player
    Unknown,     // Intent not yet revealed
}

impl EnemyIntent {
    /// Get display text for the intent
    pub fn description(&self) -> String {
        match self {
            EnemyIntent::Attack(dmg) => format!("Attack {}", dmg),
            EnemyIntent::Block(amt) => format!("Block {}", amt),
            EnemyIntent::Buff => "Buff".to_string(),
            EnemyIntent::Debuff => "Debuff".to_string(),
            EnemyIntent::Unknown => "???".to_string(),
        }
    }
}

/// Enemy AI pattern loaded from data.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum EnemyAiPattern {
    #[default]
    Bruiser,
    Guardian,
    Harrier,
    Hexer,
    Regenerator,
    Ravager,
}

/// A combat unit (player adventurer or enemy)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub block: i32,
    pub stress: i32,
    pub is_player: bool,
    pub image_path: Option<String>,
    pub base_damage: i32,
    pub intent: EnemyIntent,
    pub statuses: Vec<StatusEffect>,
    #[serde(default)]
    pub ai_pattern: EnemyAiPattern,
    #[serde(default)]
    pub traumas: Vec<Trauma>,
    #[serde(default)]
    pub resolve_state: Option<ResolveState>,
    #[serde(default)]
    pub heart_attacks: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DamageResolution {
    pub requested: i32,
    pub blocked: i32,
    pub actual: i32,
}

impl Unit {
    pub fn new_player(name: &str, max_hp: i32) -> Self {
        Self {
            name: name.to_string(),
            hp: max_hp,
            max_hp,
            block: 0,
            stress: 0,
            is_player: true,
            image_path: None,
            base_damage: 0,
            intent: EnemyIntent::Unknown,
            statuses: vec![],
            ai_pattern: EnemyAiPattern::Bruiser,
            traumas: vec![],
            resolve_state: None,
            heart_attacks: 0,
        }
    }

    pub fn new_enemy(name: &str, max_hp: i32, image_path: Option<String>) -> Self {
        Self {
            name: name.to_string(),
            hp: max_hp,
            max_hp,
            block: 0,
            stress: 0,
            is_player: false,
            image_path,
            base_damage: 6,
            intent: EnemyIntent::Attack(6),
            statuses: vec![],
            ai_pattern: EnemyAiPattern::Bruiser,
            traumas: vec![],
            resolve_state: None,
            heart_attacks: 0,
        }
    }

    /// Create enemy with specific base damage
    #[allow(dead_code)]
    pub fn new_enemy_with_damage(
        name: &str,
        max_hp: i32,
        base_damage: i32,
        image_path: Option<String>,
    ) -> Self {
        Self::new_enemy_with_pattern(
            name,
            max_hp,
            base_damage,
            image_path,
            EnemyAiPattern::Bruiser,
        )
    }

    /// Create enemy with specific base damage and AI pattern
    pub fn new_enemy_with_pattern(
        name: &str,
        max_hp: i32,
        base_damage: i32,
        image_path: Option<String>,
        ai_pattern: EnemyAiPattern,
    ) -> Self {
        Self {
            name: name.to_string(),
            hp: max_hp,
            max_hp,
            block: 0,
            stress: 0,
            is_player: false,
            image_path,
            base_damage,
            intent: EnemyIntent::Attack(base_damage),
            statuses: vec![],
            ai_pattern,
            traumas: vec![],
            resolve_state: None,
            heart_attacks: 0,
        }
    }

    /// Roll a new intent based on enemy AI pattern
    pub fn roll_intent(&mut self, turn: usize) {
        if self.is_player {
            return;
        }

        // Check for Stun status
        if self.has_status(StatusType::Stun) {
            self.intent = EnemyIntent::Unknown; // Or explicit Stun intent
            return;
        }

        let pattern = turn % 4;
        self.intent = match self.ai_pattern {
            EnemyAiPattern::Bruiser => match pattern {
                0 => EnemyIntent::Attack(self.base_damage),
                1 => EnemyIntent::Attack(self.base_damage + 2),
                2 => EnemyIntent::Block(5),
                _ => EnemyIntent::Attack(self.base_damage),
            },
            EnemyAiPattern::Guardian => match pattern {
                0 => EnemyIntent::Block(8),
                1 => EnemyIntent::Attack(self.base_damage),
                2 => EnemyIntent::Buff,
                _ => EnemyIntent::Attack(self.base_damage + 1),
            },
            EnemyAiPattern::Harrier => match pattern {
                0 => EnemyIntent::Attack((self.base_damage - 1).max(1)),
                1 => EnemyIntent::Debuff,
                2 => EnemyIntent::Attack(self.base_damage + 1),
                _ => EnemyIntent::Attack((self.base_damage - 1).max(1)),
            },
            EnemyAiPattern::Hexer => match pattern {
                0 => EnemyIntent::Debuff,
                1 => EnemyIntent::Attack(self.base_damage),
                2 => EnemyIntent::Debuff,
                _ => EnemyIntent::Buff,
            },
            EnemyAiPattern::Regenerator => match pattern {
                0 => EnemyIntent::Block(4),
                1 => EnemyIntent::Buff,
                2 => EnemyIntent::Attack(self.base_damage),
                _ => EnemyIntent::Attack(self.base_damage + 1),
            },
            EnemyAiPattern::Ravager => match pattern {
                0 => EnemyIntent::Attack(self.base_damage + 4),
                1 => EnemyIntent::Attack(self.base_damage + 1),
                2 => EnemyIntent::Block(3),
                _ => EnemyIntent::Attack(self.base_damage + 3),
            },
        };
    }

    /// Execute the current intent, returning damage dealt (if attack)
    pub fn execute_intent(&mut self) -> (i32, i32) {
        if self.has_status(StatusType::Stun) {
            return (0, 0);
        }

        match self.intent {
            EnemyIntent::Attack(dmg) => {
                let mut actual = dmg;
                if let Some(strength) = self
                    .statuses
                    .iter()
                    .find(|s| s.effect_type == StatusType::Strength)
                {
                    actual += strength.value;
                }
                if self.has_status(StatusType::Weak) {
                    actual = (actual as f32 * 0.75) as i32;
                }
                (actual.max(0), 0)
            }
            EnemyIntent::Block(amt) => {
                self.block += amt;
                (0, 0)
            }
            EnemyIntent::Buff => {
                match self.ai_pattern {
                    EnemyAiPattern::Guardian => {
                        self.add_status(StatusEffect::new(StatusType::Strength, 2, 2));
                    }
                    EnemyAiPattern::Regenerator => {
                        self.add_status(StatusEffect::new(StatusType::Regen, 3, 3));
                    }
                    _ => {
                        self.base_damage += 2;
                    }
                }
                (0, 0)
            }
            EnemyIntent::Debuff => (0, 2), // Returns stress to add
            EnemyIntent::Unknown => (0, 0),
        }
    }

    pub fn take_damage(&mut self, amount: i32) -> i32 {
        self.take_damage_detailed(amount).actual
    }

    pub fn take_damage_detailed(&mut self, amount: i32) -> DamageResolution {
        let mut final_damage = amount;

        // Vulnerable: +50% damage
        if self.has_status(StatusType::Vulnerable) {
            final_damage = (final_damage as f32 * 1.5) as i32;
        }

        let blocked = final_damage.min(self.block);
        self.block -= blocked;
        let remaining = final_damage - blocked;
        self.hp -= remaining;

        DamageResolution {
            requested: amount,
            blocked,
            actual: remaining,
        }
    }

    pub fn add_block(&mut self, amount: i32) {
        self.block += amount;
    }

    pub fn add_stress(&mut self, amount: i32) {
        let before = self.stress;
        self.stress = (self.stress + amount).max(0);
        let reached_max_stress = self.stress >= 200;

        if !self.is_player || amount <= 0 {
            return;
        }

        if before < 100 && self.stress >= 100 && self.resolve_state.is_none() {
            if macroquad_toolkit::rng::chance(0.25) {
                self.resolve_state = Some(ResolveState::Virtuous);
                self.stress = 80;
                self.add_status(StatusEffect::new(StatusType::Strength, 3, 2));
                self.add_status(StatusEffect::new(StatusType::Regen, 3, 2));
            } else {
                self.resolve_state = Some(ResolveState::Afflicted);
                self.stress = 100;
                self.add_status(StatusEffect::new(StatusType::Weak, 3, 0));
                if !self
                    .traumas
                    .iter()
                    .any(|t| t.trauma_type == TraumaType::Broken)
                {
                    self.traumas.push(Trauma::new(TraumaType::Broken));
                }
            }
        }

        if reached_max_stress || self.stress >= 200 {
            self.heart_attacks += 1;
            self.hp -= (self.max_hp / 2).max(1);
            self.stress = 100;
        }
    }

    pub fn reduce_stress(&mut self, amount: i32) {
        self.stress = (self.stress - amount).max(0);
    }

    /// Heal HP up to max
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    /// Clear all debuff status effects
    pub fn clear_debuffs(&mut self) {
        self.statuses.retain(|s| !s.is_debuff());
    }

    pub fn add_status(&mut self, effect: StatusEffect) {
        // Check if existing status of same type, extend duration or value?
        // Simple simplified rule: Add to list. (Duplicates stack intensity?)
        // Or unique by type?
        // Let's replace if exists for simplicity, or add if unique.
        if let Some(existing) = self
            .statuses
            .iter_mut()
            .find(|s| s.effect_type == effect.effect_type)
        {
            existing.duration = existing.duration.max(effect.duration);
            if effect.value > existing.value {
                existing.value = effect.value;
            }
        } else {
            self.statuses.push(effect);
        }
    }

    pub fn has_status(&self, status_type: StatusType) -> bool {
        self.statuses.iter().any(|s| s.effect_type == status_type)
    }

    pub fn tick_statuses(&mut self) {
        // Apply end-of-turn effects like Regen/Bleed here?
        // Or simplify: just reduce duration.
        // Let's implement Regen here.

        let mut hp_change = 0;

        self.statuses.retain_mut(|s| {
            if s.effect_type == StatusType::Regen {
                hp_change += s.value;
            } else if s.effect_type == StatusType::Poison || s.effect_type == StatusType::Burn {
                hp_change -= s.value;
            }

            s.duration -= 1;
            s.duration > 0
        });

        if hp_change != 0 {
            self.hp = (self.hp + hp_change).min(self.max_hp);
        }
    }
}
