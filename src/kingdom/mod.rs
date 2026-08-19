//! Kingdom management modules

mod adventurer;
mod buildings;
mod events;
mod party;
mod recruitment;
mod roster;
mod stats;
mod unlock;

pub use adventurer::{
    Adventurer, AdventurerClass, Gender, Injury, ResolveState, StatusEffect, StatusType, Trauma,
    TraumaType,
};
pub use buildings::{evaluate_construction, Building};
pub use events::{evaluate_kingdom_event, KingdomEventKind};
pub use party::{Party, PartyMemberState, MAX_PARTY_SIZE};
pub use recruitment::{evaluate_recruitment, recruit_cost, MAX_ROSTER_SIZE};
pub use roster::Roster;
pub use stats::KingdomState;
pub use unlock::UnlockRequirement;
