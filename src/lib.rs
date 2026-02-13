//! Steel Command - Web Library Entry Point
//!
//! This library provides the game logic for both desktop and web platforms.

// Include all the game modules
pub mod app;
pub mod ambience;
pub mod atlas;
pub mod bullet;
pub mod commander;
pub mod constants;
pub mod dash;
pub mod effects;
pub mod enemy;
pub mod game_state;
pub mod global_rng;
pub mod laser;
pub mod levels;
pub mod map;
pub mod physics_config;
pub mod player;
pub mod powerup;
pub mod powerup_strategy;
pub mod resources;
pub mod ui;
pub mod utils;
pub mod weather;

// Re-export necessary items
pub use app::configure_game_resources;
pub use app::register_game_systems;
pub use global_rng::GlobalRngPlugin;

#[cfg(not(target_arch = "wasm32"))]
pub use app::configure_plugins_desktop;

#[cfg(target_arch = "wasm32")]
pub use app::configure_plugins_web;

// Shared initialization function
pub fn init_game() {
    use bevy::prelude::*;

    #[cfg(target_arch = "wasm32")]
    {
        // 设置 panic hook 以在 Web 端显示错误信息
        console_error_panic_hook::set_once();
    }

    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(configure_plugins_web());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(configure_plugins_desktop());
    }

    app.add_plugins(avian2d::PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(avian2d::dynamics::integrator::Gravity::ZERO)
        .add_plugins(global_rng::GlobalRngPlugin { seed: None });

    configure_game_resources(&mut app);
    register_game_systems(&mut app);
    app.run();
}

// WebAssembly exports for browser
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use crate::init_game;

    #[wasm_bindgen(start)]
    pub fn start() {
        init_game();
    }
}