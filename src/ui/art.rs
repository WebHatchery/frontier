//! Sprite-backed scene art shared by every top-level game screen.

use macroquad::prelude::*;
use std::collections::HashMap;

pub const ICON_ATLAS_PATH: &str = "assets/images/ui/frontier_icons.png";

#[derive(Clone, Copy)]
pub enum BackgroundArt {
    Base,
    Recruit,
    MissionBoard,
    ExpeditionRoute,
    EventShrine,
    CombatArena,
    ResultsAftermath,
}

impl BackgroundArt {
    fn path(self) -> &'static str {
        match self {
            Self::Base => "assets/images/backgrounds/base_command.png",
            Self::Recruit => "assets/images/backgrounds/recruit_hall.png",
            Self::MissionBoard => "assets/images/backgrounds/mission_board.png",
            Self::ExpeditionRoute => "assets/images/backgrounds/expedition_route.png",
            Self::EventShrine => "assets/images/backgrounds/event_shrine.png",
            Self::CombatArena => "assets/images/backgrounds/combat_arena.png",
            Self::ResultsAftermath => "assets/images/backgrounds/results_aftermath.png",
        }
    }
}

#[derive(Clone, Copy)]
pub enum SpriteIcon {
    Gold,
    Supplies,
    Security,
    Morale,
    Knowledge,
    Influence,
    Attack,
    Guard,
    Vitality,
    Event,
    Rest,
    Danger,
}

impl SpriteIcon {
    fn atlas_index(self) -> usize {
        match self {
            Self::Gold => 0,
            Self::Supplies => 1,
            Self::Security => 2,
            Self::Morale => 3,
            Self::Knowledge => 4,
            Self::Influence => 5,
            Self::Attack => 6,
            Self::Guard => 7,
            Self::Vitality => 8,
            Self::Event => 9,
            Self::Rest => 10,
            Self::Danger => 11,
        }
    }
}

pub fn draw_background(
    textures: &HashMap<String, Texture2D>,
    background: BackgroundArt,
    overlay: Color,
) {
    clear_background(Color::from_rgba(11, 10, 10, 255));
    if let Some(texture) = textures.get(background.path()) {
        draw_texture_ex(
            texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
    }
    draw_rectangle(0.0, 0.0, screen_width(), screen_height(), overlay);
}

pub fn draw_icon(
    textures: &HashMap<String, Texture2D>,
    icon: SpriteIcon,
    x: f32,
    y: f32,
    size: f32,
    tint: Color,
) -> bool {
    let Some(texture) = textures.get(ICON_ATLAS_PATH) else {
        return false;
    };

    let columns = 4.0;
    let rows = 3.0;
    let cell_w = texture.width() / columns;
    let cell_h = texture.height() / rows;
    let index = icon.atlas_index() as f32;
    let source = Rect::new(
        (index % columns) * cell_w,
        (index / columns).floor() * cell_h,
        cell_w,
        cell_h,
    );
    draw_texture_ex(
        texture,
        x,
        y,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(size, size)),
            source: Some(source),
            ..Default::default()
        },
    );
    true
}
