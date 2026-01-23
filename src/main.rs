//! A simplified implementation of the classic game "Battle City 1990"
//!
//!

mod app;
mod bullet;
mod constants;
mod effects;
mod enemy;
mod game_state;
mod laser;
mod levels;
mod map;
mod player;
mod powerup;
mod resources;
mod terrain;
mod ui;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(app::configure_asset_plugin())
            .set(app::configure_window_plugin()),
    )
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

    app::configure_game_resources(&mut app);
    app::register_game_systems(&mut app);

    app.run();
}
