//! Frontier Kingdom - A dark card-based expedition RPG
//!
//! Built with Macroquad for rendering, input, and audio.
//! Game logic is explicitly state-driven; Macroquad remains thin.

#![allow(clippy::enum_variant_names, clippy::too_many_arguments)]

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod combat;
mod data;
mod game;
mod kingdom;
mod missions;
mod save;
mod state;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    // Hand-built Conf means no automatic arming: without this the capture run
    // puts a full game window on the desktop for its whole duration.
    capture::headless::arm("FRONTIER_KINGDOM");

    // Built by hand (not capture::capture_window_conf) to keep sample_count: 0
    // and the always-off high_dpi this game already shipped with.
    Conf {
        window_title: "Frontier Kingdom".to_owned(),
        window_width: capture::env_i32("FRONTIER_KINGDOM_WINDOW_WIDTH", 1280),
        window_height: capture::env_i32("FRONTIER_KINGDOM_WINDOW_HEIGHT", 720),
        window_resizable: true,
        sample_count: 0,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: when FRONTIER_KINGDOM_CAPTURE_PATH is set, seed a
    // scene, simulate deterministic frames, write a PNG, and exit.
    if let Some(configs) = capture::CaptureConfig::all_from_env("FRONTIER_KINGDOM") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |_dt| {
                clear_background(Color::from_rgba(20, 20, 25, 255));
                game.update();
                game.draw();
            })
            .await;
        }
        return;
    }

    loop {
        clear_background(Color::from_rgba(20, 20, 25, 255));

        game.update();
        game.draw();

        if game.quit_requested {
            break;
        }

        next_frame().await;
    }
}
