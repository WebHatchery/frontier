//! Pure reward and consequence calculations for completed combats.

use crate::kingdom::PartyMemberState;
use crate::missions::Mission;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RewardBreakdown {
    pub gold: i32,
    pub supplies: i32,
    pub knowledge: i32,
    pub influence: i32,
}

pub fn calculate_rewards(victory: bool, mission: &Mission) -> RewardBreakdown {
    if victory {
        RewardBreakdown {
            gold: mission.reward_gold,
            supplies: mission.reward_supplies,
            knowledge: mission.reward_knowledge,
            influence: mission.reward_influence,
        }
    } else {
        RewardBreakdown::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberConsequence {
    pub id: String,
    pub final_hp: i32,
    pub final_stress: i32,
    pub died: bool,
    pub injury_ids: Vec<String>,
    pub xp_gain: i32,
}

pub fn calculate_party_consequences(
    victory: bool,
    mission_difficulty: i32,
    stress_gained: i32,
    members: &[PartyMemberState],
) -> Vec<MemberConsequence> {
    members
        .iter()
        .map(|member| {
            let died = member.hp <= 0;
            let final_hp = if died {
                0
            } else {
                member.hp.max(1).min(member.max_hp)
            };
            let mut injury_ids = Vec::new();
            if !died && final_hp <= member.max_hp / 3 {
                injury_ids.push("wounded_leg".to_string());
            }
            if !victory && !died {
                injury_ids.push("broken_arm".to_string());
            }
            MemberConsequence {
                id: member.id.clone(),
                final_hp,
                final_stress: (member.stress + stress_gained).clamp(0, 200),
                died,
                injury_ids,
                xp_gain: if victory {
                    10 + mission_difficulty.max(0) * 2
                } else {
                    0
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests;
