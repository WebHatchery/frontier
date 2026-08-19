//! Game state modules
//!
//! Each state represents a distinct game mode with its own update/draw logic.

mod base;
mod combat;
mod event;
mod mission;
mod mission_select;
mod recruit;
mod results;

pub use base::BaseState;
pub use combat::CombatState;
pub use event::EventState;
pub use mission::MissionState;
pub use mission_select::MissionSelectState;
pub use recruit::RecruitState;
pub use results::ResultState;

use crate::kingdom::Party;

/// Explicit state transitions - no magic callbacks
pub enum StateTransition {
    ToBase,
    ToPartyFormation(Party),
    ToMissionSelect(MissionSelectState),
    ToMission(MissionState),
    ToCombat(CombatState),
    ToResults(ResultState),
    ToEvent(EventState),
    ToRecruit,
}
