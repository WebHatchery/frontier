//! Adventurer - persistent characters that remember

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ADVENTURER_ID: AtomicU64 = AtomicU64::new(0);

/// An adventurer in the kingdom's roster
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Adventurer {
    pub id: String,
    pub name: String,
    pub class: AdventurerClass,
    #[serde(default = "default_gender")]
    pub gender: Gender,

    // Stats
    pub hp: i32,
    pub max_hp: i32,
    pub stress: i32,
    #[serde(default)]
    pub resolve_state: Option<ResolveState>,
    #[serde(default)]
    pub heart_attacks: u32,
    pub level: i32,
    pub xp: i32,

    // Persistent conditions
    pub traits: Vec<Trait>,
    pub injuries: Vec<Injury>,
    pub traumas: Vec<Trauma>,
    pub statuses: Vec<StatusEffect>,

    // Cards this adventurer has unlocked/added
    pub deck_additions: Vec<String>, // Card IDs

    // State
    pub available: bool,
    pub missions_completed: u32,
    pub kills: u32,
    pub image_path: Option<String>,
}

fn default_gender() -> Gender {
    Gender::Male
}

impl Adventurer {
    pub fn new(name: &str, class: AdventurerClass, gender: Gender) -> Self {
        let base_hp = match class {
            AdventurerClass::Soldier => 45,
            AdventurerClass::Scout => 35,
            AdventurerClass::Healer => 30,
            AdventurerClass::Mystic => 32,
        };

        // Select image based on class and gender
        let gender_suffix = match gender {
            Gender::Male => "male",
            Gender::Female => "female",
        };

        // ... (lines 53-61 unchanged in replacement) ...
        let class_name = match class {
            AdventurerClass::Soldier => "soldier",
            AdventurerClass::Scout => "scout",
            AdventurerClass::Healer => "healer",
            AdventurerClass::Mystic => "mystic",
        };

        let image_path = Some(format!(
            "assets/images/characters/{}_{}.png",
            class_name, gender_suffix
        ));

        Self {
            id: uuid_simple(),
            name: name.to_string(),
            class,
            gender,
            hp: base_hp,
            max_hp: base_hp,
            stress: 0,
            resolve_state: None,
            heart_attacks: 0,
            level: 1,
            xp: 0,
            traits: vec![],
            injuries: vec![],
            traumas: vec![],
            statuses: vec![],
            deck_additions: vec![],
            available: true,
            missions_completed: 0,
            kills: 0,
            image_path,
        }
    }

    /// Check if adventurer is too stressed to deploy
    #[allow(dead_code)]
    pub fn is_stressed(&self) -> bool {
        self.stress >= 50
    }

    /// Check if adventurer is injured
    #[allow(dead_code)]
    pub fn is_injured(&self) -> bool {
        !self.injuries.is_empty()
    }

    /// Apply stress, potentially triggering trauma
    #[allow(dead_code)]
    pub fn add_stress(&mut self, amount: i32) -> Option<Trauma> {
        let before = self.stress;
        self.stress = (self.stress + amount).max(0);
        let reached_max_stress = self.stress >= 200;

        if amount <= 0 {
            return None;
        }

        if before < 100 && self.stress >= 100 && self.resolve_state.is_none() {
            self.resolve_check();
        }

        let trauma = self.trauma_for_current_stress();

        if reached_max_stress || self.stress >= 200 {
            self.heart_attack();
        }

        trauma
    }

    /// Apply stress and return human-readable consequences for the result screen.
    pub fn apply_stress_gain(&mut self, amount: i32) -> Vec<String> {
        let before = self.stress;
        let mut messages = Vec::new();
        let trauma = self.add_stress(amount);

        if let Some(trauma) = trauma {
            messages.push(format!("{} gained trauma: {}", self.name, trauma.name()));
        }

        if before < 100 && self.resolve_state.is_some() {
            match self.resolve_state {
                Some(ResolveState::Virtuous) => messages.push(format!(
                    "{} passed a Resolve Check and became Virtuous",
                    self.name
                )),
                Some(ResolveState::Afflicted) => messages.push(format!(
                    "{} failed a Resolve Check and became Afflicted",
                    self.name
                )),
                None => {}
            }
        }

        if self.heart_attacks > 0 && self.stress >= 100 {
            messages.push(format!("{} suffered a Heart Attack", self.name));
        }

        messages
    }

    fn trauma_for_current_stress(&mut self) -> Option<Trauma> {
        if self.stress >= 100 && !self.has_trauma(TraumaType::Broken) {
            let trauma = Trauma::new(TraumaType::Broken);
            self.traumas.push(trauma.clone());
            Some(trauma)
        } else if self.stress >= 75 && !self.has_trauma(TraumaType::Paranoid) {
            let trauma = Trauma::new(TraumaType::Paranoid);
            self.traumas.push(trauma.clone());
            Some(trauma)
        } else if self.stress >= 50 && !self.has_trauma(TraumaType::Fearful) {
            let trauma = Trauma::new(TraumaType::Fearful);
            self.traumas.push(trauma.clone());
            Some(trauma)
        } else {
            None
        }
    }

    fn resolve_check(&mut self) {
        if macroquad_toolkit::rng::chance(0.25) {
            self.resolve_state = Some(ResolveState::Virtuous);
            self.stress = 80;
            self.traits.push(Trait {
                id: "steadfast".to_string(),
                name: "Steadfast".to_string(),
                description: "Passed a Resolve Check; stress relief is more effective.".to_string(),
                is_positive: true,
            });
        } else {
            self.resolve_state = Some(ResolveState::Afflicted);
            if !self.has_trauma(TraumaType::Broken) {
                self.traumas.push(Trauma::new(TraumaType::Broken));
            }
            self.stress = 100;
        }
    }

    fn heart_attack(&mut self) {
        self.heart_attacks += 1;
        self.hp -= (self.max_hp / 2).max(1);
        self.stress = 100;
    }

    fn has_trauma(&self, trauma_type: TraumaType) -> bool {
        self.traumas.iter().any(|t| t.trauma_type == trauma_type)
    }

    /// Reduce stress (at base, costs resources)
    pub fn reduce_stress(&mut self, amount: i32) {
        let effective = if self.resolve_state == Some(ResolveState::Virtuous) {
            amount + 5
        } else {
            amount
        };
        self.stress = (self.stress - effective).max(0);
        if self.stress < 50 {
            self.resolve_state = None;
        }
    }

    /// Heal HP (at infirmary)
    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}

/// Simple UUID generator (timestamp-based for uniqueness)
fn uuid_simple() -> String {
    let now = macroquad::time::get_time();
    let sequence = NEXT_ADVENTURER_ID.fetch_add(1, Ordering::Relaxed);
    // The sequence suffix keeps characters created in the same frame distinct.
    format!("adv_{}_{}", (now * 1_000_000.0) as u64, sequence)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum AdventurerClass {
    Soldier, // High HP, attack cards
    Scout,   // Low HP, utility/evasion
    Healer,  // Low HP, stress/healing
    Mystic,  // Medium HP, special effects
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

/// Result of a high-stress resolve check
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ResolveState {
    Virtuous,
    Afflicted,
}

/// Positive or negative traits affecting gameplay
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trait {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_positive: bool,
}

/// Physical injuries from combat
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Injury {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: i32, // 1-3
    pub healing_days: i32,
}

impl Injury {
    #[allow(dead_code)]
    pub fn wounded_leg() -> Self {
        Self {
            id: "wounded_leg".to_string(),
            name: "Wounded Leg".to_string(),
            description: "Movement cards cost +1 energy".to_string(),
            severity: 2,
            healing_days: 3,
        }
    }

    #[allow(dead_code)]
    pub fn broken_arm() -> Self {
        Self {
            id: "broken_arm".to_string(),
            name: "Broken Arm".to_string(),
            description: "Attack cards deal -2 damage".to_string(),
            severity: 3,
            healing_days: 5,
        }
    }
}

/// Psychological trauma from stress
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trauma {
    pub trauma_type: TraumaType,
    pub severity: i32,
}

impl Trauma {
    #[allow(dead_code)]
    pub fn new(trauma_type: TraumaType) -> Self {
        Self {
            trauma_type,
            severity: 1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self.trauma_type {
            TraumaType::Fearful => "Fearful",
            TraumaType::Paranoid => "Paranoid",
            TraumaType::Broken => "Broken",
            TraumaType::Hopeless => "Hopeless",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TraumaType {
    Fearful,  // Chance to skip turn
    Paranoid, // Block cards cost +1
    Broken,   // All cards cost +1
    Hopeless, // Cannot reduce stress in combat
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum StatusType {
    Strength,   // +Value Damage Dealt
    Vulnerable, // +50% Damage Taken
    Weak,       // -25% Damage Dealt
    Stun,       // Skip Turn (duration reduces by 1 per turn)
    Regen,      // +Value HP per turn
    Block,      // Absorbs damage (Value amount), duration usually 1 turn
    Poison,     // Take Value damage per turn
    Burn,       // Take Value damage per turn
}

impl StatusType {
    /// Check if this status type is a debuff (negative effect)
    pub fn is_debuff(&self) -> bool {
        matches!(
            self,
            StatusType::Vulnerable
                | StatusType::Weak
                | StatusType::Stun
                | StatusType::Poison
                | StatusType::Burn
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StatusEffect {
    pub effect_type: StatusType,
    pub duration: i32,
    pub value: i32,
}

impl StatusEffect {
    pub fn new(effect_type: StatusType, duration: i32, value: i32) -> Self {
        Self {
            effect_type,
            duration,
            value,
        }
    }

    /// Check if this status effect is a debuff
    pub fn is_debuff(&self) -> bool {
        self.effect_type.is_debuff()
    }
}
