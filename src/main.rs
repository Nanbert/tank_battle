//! A simplified implementation of the classic game "Battle City 1990"
//!
//!

mod ambience;
mod app;
mod atlas;
mod bullet;
mod commander;
mod constants;
mod dash;
mod effects;
mod enemy;
mod game_state;
mod global_rng;
mod laser;
#[cfg(not(target_arch = "wasm32"))]
mod level_editor;
mod levels;
mod map;
mod physics_config;
mod player;
mod powerup;
mod powerup_strategy;
mod resources;
mod ui;
mod utils;
mod weather;

use avian2d::prelude::*;
use bevy::prelude::*;

pub fn init_game() {
    #[cfg(target_arch = "wasm32")]
    {
        // 设置 panic hook 以在 Web 端显示错误信息
        console_error_panic_hook::set_once();
    }

    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(app::configure_plugins_web());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(app::configure_plugins_desktop());
    }

    app.add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(Gravity::ZERO)
        .add_plugins(global_rng::GlobalRngPlugin { seed: None });

    app::configure_game_resources(&mut app);
    app::register_game_systems(&mut app);
    app.run();
}

#[cfg(target_arch = "wasm32")]
pub fn main() {
    init_game();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    init_game();
}
