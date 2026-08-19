//! Touch and keyboard intents for mission selection.

use super::layout::{action_button_rect, mission_card_rect};
use super::MissionSelectState;
use crate::kingdom::{KingdomState, Party};
use crate::state::StateTransition;
use macroquad::prelude::*;

pub(super) fn update(
    state: &mut MissionSelectState,
    kingdom: &KingdomState,
) -> Option<StateTransition> {
    move_selection(state);
    select_with_number_keys(state);

    for i in 0..state.missions.len() {
        let rect = mission_card_rect(i);
        if crate::ui::was_clicked(rect.0, rect.1, rect.2, rect.3) {
            if state.selected_mission == i {
                return state.start_selected_mission(kingdom);
            }
            state.selected_mission = i;
        }
    }

    let embark = action_button_rect(0);
    if crate::ui::was_clicked(embark.0, embark.1, embark.2, embark.3) {
        return state.start_selected_mission(kingdom);
    }
    let change_party = action_button_rect(1);
    if crate::ui::was_clicked(
        change_party.0,
        change_party.1,
        change_party.2,
        change_party.3,
    ) {
        return Some(StateTransition::ToPartyFormation(Party {
            member_ids: state
                .party_members
                .iter()
                .map(|member| member.id.clone())
                .collect(),
        }));
    }
    let back = action_button_rect(2);
    if crate::ui::was_clicked(back.0, back.1, back.2, back.3) || is_key_pressed(KeyCode::Escape) {
        return Some(StateTransition::ToBase);
    }
    if is_key_pressed(KeyCode::Enter) {
        return state.start_selected_mission(kingdom);
    }
    None
}

fn move_selection(state: &mut MissionSelectState) {
    if (is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W)) && state.selected_mission > 0 {
        state.selected_mission -= 1;
    }
    if (is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S))
        && state.selected_mission < state.missions.len().saturating_sub(1)
    {
        state.selected_mission += 1;
    }
}

fn select_with_number_keys(state: &mut MissionSelectState) {
    for i in 0..state.missions.len().min(9) {
        let key = match i {
            0 => KeyCode::Key1,
            1 => KeyCode::Key2,
            2 => KeyCode::Key3,
            3 => KeyCode::Key4,
            4 => KeyCode::Key5,
            _ => KeyCode::Key6,
        };
        if is_key_pressed(key) {
            state.selected_mission = i;
        }
    }
}
