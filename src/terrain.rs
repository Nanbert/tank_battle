//! 地形和实体生成模块
//!
//! 处理墙壁、地图地形、指挥官、玩家坦克、道具等实体生成

use bevy::prelude::*;
use bevy::audio::Volume;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::*;

/// 生成墙壁
pub fn spawn_walls(commands: &mut Commands) {
    // 左边界墙
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(20.0, MAP_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(-MAP_WIDTH / 2.0 - 10.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, MAP_HEIGHT / 2.0),
    ));

    // 右边界墙
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(20.0, MAP_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(MAP_WIDTH / 2.0 + 10.0, 0.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(10.0, MAP_HEIGHT / 2.0),
    ));

    // 上边界墙
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(MAP_WIDTH, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, MAP_HEIGHT / 2.0 + 10.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(MAP_WIDTH / 2.0, 10.0),
    ));

    // 下边界墙
    commands.spawn((
        Wall,
        Sprite {
            color: Color::srgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(MAP_WIDTH, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, -MAP_HEIGHT / 2.0 - 10.0, 0.0),
        RigidBody::Fixed,
        Collider::cuboid(MAP_WIDTH / 2.0, 10.0),
    ));
}

pub fn is_stat_at_max_value(text: &str, player_stats: &PlayerStats) -> bool {
    match text {
        s if s.starts_with("Speed") => player_stats.speed >= 100,
        s if s.starts_with("Shells") => player_stats.shells >= 5,
        s if s.starts_with("Protection") => player_stats.protection >= 100,
        s if s.starts_with("Fire Speed") => player_stats.fire_speed >= 100,
        _ => false,
    }
}
