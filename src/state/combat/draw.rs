//! Combat rendering: header, party panel, enemy stage, and card hand.

use super::helpers::{
    border_color, candle_color, card_accent, card_preview, card_type, clicked_down,
    combat_card_rect, danger_color, draw_wrapped_text, end_turn_button_rect, hovered_card_index,
    info_color, intent_warning, muted_text_color, mystery_color, panel, ready_color,
    retreat_button_rect, text_color, title_color,
};
use super::CombatState;
use crate::combat::{Card, Unit};
use crate::kingdom::ResolveState;
use crate::ui::{draw_background, draw_icon, BackgroundArt, SpriteIcon};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

impl CombatState {
    pub fn draw(&self, textures: &std::collections::HashMap<String, Texture2D>) {
        draw_background(
            textures,
            BackgroundArt::CombatArena,
            Color::from_rgba(0, 0, 0, 178),
        );

        draw_header(self.turn);
        draw_party_panel(
            &self.players,
            self.current_player_idx,
            self.energy,
            self.max_energy,
            textures,
        );
        draw_enemy_stage(&self.enemy, textures);

        let preview_idx = hovered_card_index(&self.hand).or(self.selected_card);
        draw_report_panel(self, preview_idx);
        draw_feedback_panel(self.feedback.as_ref());

        let mut hovered_card_idx: Option<usize> = None;
        for (i, card) in self.hand.iter().enumerate() {
            let (x, y, w, h) = combat_card_rect(i, self.hand.len());
            let is_hovered = crate::ui::is_mouse_over(x, y, w, h);
            if is_hovered {
                hovered_card_idx = Some(i);
            }
            let effective_cost = self.effective_card_cost(card);
            let can_afford = effective_cost <= self.energy;
            let attack_blocked = card.is_attack() && self.resolver.turn_mods.attacks_disabled;
            draw_combat_card(
                i,
                card,
                self.hand.len(),
                self.selected_card == Some(i),
                is_hovered,
                can_afford && !attack_blocked,
                attack_blocked,
                effective_cost,
                textures,
            );
        }

        let (retreat_x, retreat_y, retreat_w, retreat_h) = retreat_button_rect();
        draw_action_button("Retreat", retreat_x, retreat_y, retreat_w, retreat_h);

        let (end_btn_x, end_btn_y, end_btn_w, end_btn_h) = end_turn_button_rect();
        draw_action_button("End Turn", end_btn_x, end_btn_y, end_btn_w, end_btn_h);
        draw_ui_text(
            "Tap a card to select, tap again to play • Tap END TURN",
            24.0,
            screen_height() - 26.0,
            13.0,
            muted_text_color(),
        );

        if let Some(idx) = hovered_card_idx {
            if let Some(card) = self.hand.get(idx) {
                crate::ui::card_tooltip(&card.name, &card.description);
            }
        }
    }
}

fn draw_header(turn: usize) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        72.0,
        Color::from_rgba(8, 7, 6, 232),
    );
    draw_line(0.0, 72.0, screen_width(), 72.0, 2.0, border_color());
    draw_ui_text("COMBAT", 24.0, 42.0, 34.0, title_color());
    draw_ui_text(&format!("Turn {}", turn), 188.0, 42.0, 20.0, candle_color());
}

fn draw_party_panel(
    players: &[Unit],
    current_player_idx: usize,
    energy: i32,
    max_energy: i32,
    textures: &std::collections::HashMap<String, Texture2D>,
) {
    panel(24.0, 92.0, 260.0, 328.0, "PLAYER AREA");
    let Some(active) = players.get(current_player_idx) else {
        return;
    };

    draw_ui_text(&active.name, 44.0, 148.0, 24.0, title_color());
    draw_ui_text(
        &format!(
            "HP {}/{}    Block {}",
            active.hp, active.max_hp, active.block
        ),
        44.0,
        178.0,
        16.0,
        text_color(),
    );
    draw_ui_text(
        &format!(
            "Stress {}    Energy {}/{}",
            active.stress, energy, max_energy
        ),
        44.0,
        204.0,
        16.0,
        muted_text_color(),
    );
    if let Some(resolve) = &active.resolve_state {
        let (label, color) = match resolve {
            ResolveState::Virtuous => ("Virtuous", ready_color()),
            ResolveState::Afflicted => ("Afflicted", danger_color()),
        };
        draw_ui_text(label, 44.0, 230.0, 16.0, color);
    }

    draw_ui_text("Party", 44.0, 268.0, 16.0, candle_color());
    for (i, player) in players.iter().enumerate().take(4) {
        let y = 300.0 + (i as f32 * 30.0);
        let marker = if i == current_player_idx { ">" } else { " " };
        draw_ui_text(marker, 44.0, y, 15.0, candle_color());
        if let Some(path) = &player.image_path {
            if let Some(tex) = textures.get(path) {
                draw_texture_ex(
                    tex,
                    64.0,
                    y - 20.0,
                    if player.hp <= 0 {
                        Color::from_rgba(90, 90, 90, 255)
                    } else {
                        WHITE
                    },
                    DrawTextureParams {
                        dest_size: Some(vec2(22.0, 22.0)),
                        ..Default::default()
                    },
                );
            }
        }
        draw_ui_text(
            &format!("{}  {}/{}", player.name, player.hp, player.max_hp),
            94.0,
            y,
            14.0,
            if player.hp <= 0 {
                danger_color()
            } else {
                text_color()
            },
        );
    }
}

fn draw_enemy_stage(enemy: &Unit, textures: &std::collections::HashMap<String, Texture2D>) {
    panel(308.0, 92.0, 644.0, 204.0, "ENEMY AREA");
    let center_x = 630.0;

    if let Some(path) = &enemy.image_path {
        if let Some(tex) = textures.get(path) {
            draw_texture_ex(
                tex,
                center_x - 68.0,
                128.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(136.0, 136.0)),
                    ..Default::default()
                },
            );
        }
    } else {
        draw_circle(center_x, 190.0, 54.0, Color::from_rgba(62, 44, 38, 255));
    }

    let name_w = measure_ui_text(&enemy.name, None, 26, 1.0).width;
    draw_ui_text(
        &enemy.name,
        center_x - name_w / 2.0,
        128.0,
        26.0,
        title_color(),
    );
    let hp = format!("HP {}/{}    Block {}", enemy.hp, enemy.max_hp, enemy.block);
    let hp_w = measure_ui_text(&hp, None, 17, 1.0).width;
    draw_ui_text(&hp, center_x - hp_w / 2.0, 280.0, 17.0, text_color());

    let intent = format!("Intent: {}", enemy.intent.description());
    let intent_color = match &enemy.intent {
        crate::combat::EnemyIntent::Attack(_) => danger_color(),
        crate::combat::EnemyIntent::Block(_) => info_color(),
        crate::combat::EnemyIntent::Buff => candle_color(),
        crate::combat::EnemyIntent::Debuff => mystery_color(),
        crate::combat::EnemyIntent::Unknown => muted_text_color(),
    };
    let intent_icon = match &enemy.intent {
        crate::combat::EnemyIntent::Attack(_) => SpriteIcon::Attack,
        crate::combat::EnemyIntent::Block(_) => SpriteIcon::Guard,
        crate::combat::EnemyIntent::Buff => SpriteIcon::Vitality,
        crate::combat::EnemyIntent::Debuff => SpriteIcon::Event,
        crate::combat::EnemyIntent::Unknown => SpriteIcon::Danger,
    };
    draw_rectangle(720.0, 144.0, 196.0, 86.0, Color::from_rgba(22, 18, 16, 220));
    draw_rectangle_lines(720.0, 144.0, 196.0, 86.0, 1.0, intent_color);
    draw_icon(textures, intent_icon, 734.0, 153.0, 30.0, WHITE);
    draw_ui_text("NEXT", 772.0, 172.0, 16.0, muted_text_color());
    draw_wrapped_text(&intent, 740.0, 202.0, 156.0, 20.0, intent_color);

    if !enemy.statuses.is_empty() {
        let mut x = 332.0;
        for status in enemy.statuses.iter().take(4) {
            draw_ui_text(
                &format!("{:?} {}", status.effect_type, status.duration),
                x,
                274.0,
                14.0,
                ready_color(),
            );
            x += 112.0;
        }
    }
}

fn draw_report_panel(state: &CombatState, preview_idx: Option<usize>) {
    panel(308.0, 314.0, 644.0, 106.0, "BATTLE REPORT");
    if let Some(idx) = preview_idx {
        if let Some(card) = state.hand.get(idx) {
            draw_ui_text(&card.name, 330.0, 362.0, 20.0, candle_color());
            draw_wrapped_text(
                &card_preview(state, card),
                330.0,
                390.0,
                590.0,
                15.0,
                text_color(),
            );
            return;
        }
    }

    let mut y = 358.0;
    let mut drew_any = false;
    for line in state.resolver.log.iter().rev().take(3).rev() {
        draw_ui_text(line, 330.0, y, 15.0, muted_text_color());
        y += 22.0;
        drew_any = true;
    }
    if !drew_any {
        let Some(player) = state.players.get(state.current_player_idx) else {
            return;
        };
        draw_ui_text(
            &intent_warning(&player.name, &state.enemy.intent),
            330.0,
            372.0,
            16.0,
            muted_text_color(),
        );
    }
}

fn draw_feedback_panel(feedback: Option<&(String, f32)>) {
    let Some((message, time_left)) = feedback else {
        return;
    };

    let alpha = ((*time_left / 2.0).clamp(0.0, 1.0) * 210.0) as u8;
    let width = measure_ui_text(message, None, 18, 1.0).width + 42.0;
    let x = (screen_width() - width) / 2.0;
    let y = 432.0;
    draw_rectangle(x, y, width, 42.0, Color::from_rgba(28, 21, 14, alpha));
    draw_rectangle_lines(x, y, width, 42.0, 1.0, candle_color());
    draw_ui_text(message, x + 20.0, y + 27.0, 18.0, text_color());
}

fn draw_combat_card(
    i: usize,
    card: &Card,
    hand_len: usize,
    selected: bool,
    hovered: bool,
    can_play: bool,
    attack_blocked: bool,
    effective_cost: i32,
    textures: &std::collections::HashMap<String, Texture2D>,
) {
    let (x, y, w, h) = combat_card_rect(i, hand_len);
    let accent = card_accent(card);
    let border = if selected {
        candle_color()
    } else if hovered && can_play {
        ready_color()
    } else if !can_play {
        danger_color()
    } else {
        accent
    };

    draw_rectangle(x, y, w, h, Color::from_rgba(18, 16, 14, 245));
    draw_rectangle_lines(x, y, w, h, if selected { 3.0 } else { 2.0 }, border);
    draw_rectangle(
        x + 6.0,
        y + 6.0,
        w - 12.0,
        26.0,
        Color::from_rgba(38, 32, 26, 246),
    );
    draw_ui_text(
        &effective_cost.to_string(),
        x + 14.0,
        y + 25.0,
        20.0,
        if can_play {
            candle_color()
        } else {
            danger_color()
        },
    );
    draw_ui_text(
        card_type(card),
        x + w - 64.0,
        y + 24.0,
        13.0,
        muted_text_color(),
    );
    let card_icon = if card.is_attack() {
        SpriteIcon::Attack
    } else if card
        .effects
        .iter()
        .any(|effect| matches!(effect, crate::combat::CardEffect::Block(_)))
    {
        SpriteIcon::Guard
    } else if card
        .effects
        .iter()
        .any(|effect| matches!(effect, crate::combat::CardEffect::Heal(_)))
    {
        SpriteIcon::Vitality
    } else {
        SpriteIcon::Event
    };
    draw_icon(textures, card_icon, x + w - 34.0, y + 7.0, 26.0, WHITE);

    let art_x = x + 8.0;
    let art_y = y + 38.0;
    let art_w = w - 16.0;
    let art_h = 78.0;
    draw_rectangle(
        art_x,
        art_y,
        art_w,
        art_h,
        Color::from_rgba(42, 38, 34, 255),
    );
    if let Some(path) = &card.image_path {
        if let Some(tex) = textures.get(path) {
            draw_texture_ex(
                tex,
                art_x,
                art_y,
                if attack_blocked {
                    Color::from_rgba(130, 105, 140, 255)
                } else {
                    WHITE
                },
                DrawTextureParams {
                    dest_size: Some(vec2(art_w, art_h)),
                    ..Default::default()
                },
            );
        }
    }
    draw_rectangle(art_x, art_y, art_w, art_h, Color::from_rgba(0, 0, 0, 55));

    draw_ui_text(&card.name, x + 10.0, y + 136.0, 15.0, text_color());
    let status = if attack_blocked {
        "Blocked this turn"
    } else if can_play {
        "Ready"
    } else {
        "Need energy"
    };
    draw_ui_text(
        status,
        x + 10.0,
        y + 158.0,
        12.0,
        if can_play {
            ready_color()
        } else {
            danger_color()
        },
    );
    draw_wrapped_text(
        &card.description,
        x + 10.0,
        y + 178.0,
        w - 20.0,
        11.0,
        muted_text_color(),
    );
}

fn draw_action_button(label: &str, x: f32, y: f32, w: f32, h: f32) {
    let hovered = crate::ui::is_mouse_over(x, y, w, h);
    let pressed = clicked_down(x, y, w, h);
    let fill = if pressed {
        Color::from_rgba(130, 92, 39, 255)
    } else if hovered {
        Color::from_rgba(95, 67, 31, 245)
    } else {
        Color::from_rgba(70, 49, 27, 238)
    };
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 1.0, candle_color());
    let tw = measure_ui_text(label, None, 16, 1.0).width;
    draw_ui_text(label, x + (w - tw) / 2.0, y + 24.0, 16.0, text_color());
}
