//! A simplified implementation of the classic game "Battle City 1990"
//!
//!

mod app;
mod ambience;
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

use bevy::prelude::*;
use avian2d::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(app::configure_asset_plugin())
            .set(app::configure_window_plugin())
            // 添加音频配置，设置全局音量
            .set(bevy::audio::AudioPlugin {
                global_volume: bevy::audio::GlobalVolume {
                    volume: bevy::audio::Volume::Linear(0.8),
                },
                default_spatial_scale: bevy::audio::SpatialScale::default(),
            }),
    )
    .add_plugins(PhysicsPlugins::default().with_length_unit(100.0))
    // 全局零重力配置
    .insert_resource(Gravity::ZERO)
    // 添加全局随机数生成器（使用系统时间作为种子）
    .add_plugins(global_rng::GlobalRngPlugin { seed: None });

    app::configure_game_resources(&mut app);
    app::register_game_systems(&mut app);

    app.run();
}
