//! Combat system modules
//!
//! Cards emit effects; systems resolve them. Cards never directly mutate state.

mod card;
mod effects;
mod resolution;
mod resolver;
mod unit;

pub use card::{Card, CardClass};
pub use effects::CardEffect;
pub use resolution::ResolutionDelta;
pub use resolver::CombatResolver;
pub use unit::{EnemyAiPattern, EnemyIntent, Unit};
