//! 司令官生成模块
//!
//! 处理司令官实体及其相关动画的生成

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::{CommanderLife, CommanderMusicResources, CommanderResources};
use crate::utils;

/// 生成司令官
pub fn spawn_commander(
    mut commands: Commands,
    commander_resources: Res<CommanderResources>,
    commander_music_resources: Res<CommanderMusicResources>,
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

    let commander_texture = commander_resources.texture.clone();
    // commander.png 实际尺寸: 1400x1200, 每帧 140x120, 10列 x 10行, 共100帧
    let commander_tile_size = UVec2::new(COMMANDER_TILE_WIDTH as u32, COMMANDER_TILE_HEIGHT as u32);
    let commander_texture_atlas_layout = utils::create_texture_atlas(commander_tile_size, 10, 10);
    let commander_texture_atlas = texture_atlas_layouts.add(commander_texture_atlas_layout);
    let commander_animation_indices = AnimationIndices { first: 0, last: 99 };

    let commander_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT / 2.0;
    let commander_x = 0.0;

    let _commander_entity = commands.spawn((
        Commander,
        PlayingEntity,
        Sprite {
            image: commander_texture,
            texture_atlas: Some(TextureAtlas {
                layout: commander_texture_atlas,
                index: commander_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(COMMANDER_DISPLAY_WIDTH, COMMANDER_DISPLAY_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(commander_x, commander_y, 0.0),
        commander_animation_indices,
        AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_COMMANDER,
            TimerMode::Repeating,
        )),
        CurrentAnimationFrame(0),
        RigidBody::Fixed,
        Collider::cuboid(COMMANDER_WIDTH / 2.0, COMMANDER_HEIGHT / 2.0),
        ActiveEvents::COLLISION_EVENTS,
    )).id();

    // 创建音乐动画精灵（独立实体，与 Commander 位置相同）
    let music_texture = commander_music_resources.music_note.clone();
    let music_tile_size = UVec2::new(
        COMMANDER_MUSIC_TILE_WIDTH as u32,
        COMMANDER_MUSIC_TILE_HEIGHT as u32,
    );
    let music_texture_atlas_layout = utils::create_texture_atlas(music_tile_size, 10, 1);
    let music_texture_atlas = texture_atlas_layouts.add(music_texture_atlas_layout);
    let music_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        CommanderMusicAnimation,
        PlayingEntity,
        Sprite {
            image: music_texture,
            texture_atlas: Some(TextureAtlas {
                layout: music_texture_atlas,
                index: music_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(COMMANDER_MUSIC_DISPLAY_WIDTH, COMMANDER_MUSIC_DISPLAY_HEIGHT)),
            ..default()
        },
        Transform::from_translation(Vec3::new(commander_x, commander_y, Z_FOREST)), // z=1.0 使动画在 Commander 上方
        music_animation_indices,
        AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_COMMANDER_MUSIC,
            TimerMode::Repeating,
        )), // 每0.1秒切换一帧
        CurrentAnimationFrame(0),
    ));
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
        ),
        With<Commander>,
    >,
) {
    // Commander 已死亡，不播放动画
    if commander_life.life_points == 0 {
        return;
    }

    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            let current = current_frame.0;
            let next_index = if current == indices.last {
                indices.first
            } else {
                current + 1
            };
            current_frame.0 = next_index;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = next_index;
            }
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
    for entity in commanders.iter() {
        let () = commands.entity(entity).try_despawn();
    }

    // 销毁所有 MusicNote 动画实体
    for entity in music_animations.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}