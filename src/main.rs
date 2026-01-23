//! A simplified implementation of the classic game "Battle City 1990"
//!
//!
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::cast_precision_loss)]

mod constants;
mod resources;
mod map;
mod levels;
mod bullet;
mod laser;
mod enemy;
mod player;
mod ui;
mod effects;
mod terrain;
mod game_state;
mod app;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(app::configure_asset_plugin()))
        .add_plugins(app::configure_window_plugin())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

    app::configure_game_resources(&mut app);
    app::register_game_systems(&mut app);

    app.run();
}
