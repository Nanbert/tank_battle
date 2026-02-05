//! 司令官生成模块
//!
//! 处理司令官实体及其相关动画的生成

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::{CommanderLife, GameTextureResources};
use crate::utils;

/// 生成司令官
pub fn spawn_commander(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    commanders: Query<Entity, With<Commander>>,
    children: Query<&Children>,
    mut commander_life: ResMut<CommanderLife>,
) {
    // 防御性编程：先销毁可能存在的旧司令官及其子节点
    for entity in commanders.iter() {
        // 先销毁所有子节点
        if let Ok(children) = children.get(entity) {
            for child in children.iter() {
                let () = commands.entity(child).try_despawn();
            }
        }
        // 再销毁父节点
        let () = commands.entity(entity).try_despawn();
    }

    // 重置 Commander 生命值
    commander_life.life_points = 3;

    let commander_texture = texture_resources.commander.clone();
    // commander.png 实际尺寸: 1400x1200, 每帧 140x120, 10列 x 10行, 共100帧
    let commander_tile_size = UVec2::new(COMMANDER_TILE_WIDTH as u32, COMMANDER_TILE_HEIGHT as u32);
    let commander_animation_indices = AnimationIndices { first: 0, last: 99 };

    let commander_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT / 2.0;
    let commander_x = 0.0;

    let commander_entity = utils::spawn_animated_sprite(
        &mut commands,
        &mut texture_atlas_layouts,
        commander_texture,
        commander_tile_size,
        10,
        10,
        commander_animation_indices,
        ANIMATION_FRAME_COMMANDER,
        Vec3::new(commander_x, commander_y, 0.0),
        Some(Vec2::new(COMMANDER_DISPLAY_WIDTH, COMMANDER_DISPLAY_HEIGHT)),
        (Commander, PlayingEntity, AnimationMode::Looping),
    );

    // 添加物理组件
    commands.entity(commander_entity).insert((
        RigidBody::Fixed,
        Collider::cuboid(COMMANDER_WIDTH / 2.0, COMMANDER_HEIGHT / 2.0),
        ActiveEvents::COLLISION_EVENTS,
    ));

    // 创建音乐动画精灵（独立实体，与 Commander 位置相同）
    let music_texture = texture_resources.music_note.clone();
    let music_tile_size = UVec2::new(
        COMMANDER_MUSIC_TILE_WIDTH as u32,
        COMMANDER_MUSIC_TILE_HEIGHT as u32,
    );
    let music_animation_indices = AnimationIndices { first: 0, last: 9 };

    utils::spawn_animated_sprite(
        &mut commands,
        &mut texture_atlas_layouts,
        music_texture,
        music_tile_size,
        10,
        1,
        music_animation_indices,
        ANIMATION_FRAME_COMMANDER_MUSIC,
        Vec3::new(commander_x, commander_y, Z_FOREST),
        Some(Vec2::new(COMMANDER_MUSIC_DISPLAY_WIDTH, COMMANDER_MUSIC_DISPLAY_HEIGHT)),
        (CommanderMusicAnimation, PlayingEntity, AnimationMode::Looping),
    );
}

/// 动画 Commander 纹理（只在存活时播放）
pub fn animate_commander(
    time: Res<Time>,
    commander_life: Res<CommanderLife>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
            &AnimationMode,
        ),
        With<Commander>,
    >,
) {
    // Commander 已死亡，不播放动画
    if commander_life.life_points == 0 {
        return;
    }

    for (mut timer, mut sprite, indices, mut current_frame, animation_mode) in &mut query {
        // 只处理循环动画模式
        if *animation_mode == AnimationMode::Looping {
            crate::utils::animate_sprite(&mut timer, &mut sprite, indices, &mut current_frame, time.delta());
        }
    }
}

/// 销毁司令官实体
pub fn despawn_commander(
    mut commands: Commands,
    commanders: Query<Entity, With<Commander>>,
    music_animations: Query<Entity, With<CommanderMusicAnimation>>,
) {
    // 销毁所有 Commander 实体
    crate::utils::cleanup_entities(&mut commands, commanders.iter());

    // 销毁所有 MusicNote 动画实体
    crate::utils::cleanup_entities(&mut commands, music_animations.iter());
}