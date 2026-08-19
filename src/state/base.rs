//! Kingdom base state - command-table management UI.

mod details;
mod draw;
mod helpers;
mod input;
mod overlays;
mod panels;

use crate::kingdom::Party;

const HEADER_H: f32 = 92.0;
const MAIN_Y: f32 = 110.0;
const MAIN_H: f32 = 245.0;
const ACTION_Y: f32 = 372.0;
const ACTION_H: f32 = 76.0;
const DETAIL_Y: f32 = 466.0;
const SIDE_PAD: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseTab {
    Kingdom,
    Roster,
    Missions,
    Buildings,
    DeckTraining,
    Graveyard,
    Journal,
}

impl BaseTab {
    const ALL: [BaseTab; 7] = [
        BaseTab::Kingdom,
        BaseTab::Roster,
        BaseTab::Missions,
        BaseTab::Buildings,
        BaseTab::DeckTraining,
        BaseTab::Graveyard,
        BaseTab::Journal,
    ];

    fn label(self) -> &'static str {
        match self {
            BaseTab::Kingdom => "Kingdom",
            BaseTab::Roster => "Roster",
            BaseTab::Missions => "Missions",
            BaseTab::Buildings => "Buildings",
            BaseTab::DeckTraining => "Deck / Training",
            BaseTab::Graveyard => "Graveyard",
            BaseTab::Journal => "Journal",
        }
    }

    fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|tab| *tab == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }
}

/// Focus area for compatibility with party formation flow.
#[derive(PartialEq, Clone, Default)]
pub enum FocusArea {
    #[default]
    Roster,
    Buildings,
    PartyFormation,
}

/// State for managing the kingdom base.
pub struct BaseState {
    pub selected_building: Option<usize>,
    pub selected_adventurer: Option<usize>,
    pub focus: FocusArea,
    pub active_tab: BaseTab,
    pub viewing_deck: bool,
    /// Current party being formed.
    pub forming_party: Party,
}

impl Default for BaseState {
    fn default() -> Self {
        Self {
            selected_building: Some(0),
            selected_adventurer: Some(0),
            focus: FocusArea::Roster,
            active_tab: BaseTab::Kingdom,
            viewing_deck: false,
            forming_party: Party::default(),
        }
    }
}

impl BaseState {
    pub fn for_party_formation(forming_party: Party) -> Self {
        Self {
            selected_building: None,
            selected_adventurer: forming_party.leader_id().map(|_| 0),
            focus: FocusArea::PartyFormation,
            active_tab: BaseTab::Roster,
            viewing_deck: false,
            forming_party,
        }
    }
}
