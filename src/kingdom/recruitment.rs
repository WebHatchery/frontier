//! Pure recruitment pricing and eligibility rules.

use super::AdventurerClass;

pub const MAX_ROSTER_SIZE: usize = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecruitmentBlock {
    InsufficientGold,
    RosterFull,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecruitmentEvaluation {
    pub cost: i32,
    pub can_hire: bool,
    pub blocked_by: Option<RecruitmentBlock>,
}

pub fn recruit_cost(class: &AdventurerClass) -> i32 {
    match class {
        AdventurerClass::Soldier => 50,
        AdventurerClass::Scout => 40,
        AdventurerClass::Healer => 60,
        AdventurerClass::Mystic => 70,
    }
}

pub fn evaluate_recruitment(gold: i32, roster_count: usize, cost: i32) -> RecruitmentEvaluation {
    let blocked_by = if roster_count >= MAX_ROSTER_SIZE {
        Some(RecruitmentBlock::RosterFull)
    } else if gold < cost {
        Some(RecruitmentBlock::InsufficientGold)
    } else {
        None
    };
    RecruitmentEvaluation {
        cost,
        can_hire: blocked_by.is_none(),
        blocked_by,
    }
}

#[cfg(test)]
mod tests;
