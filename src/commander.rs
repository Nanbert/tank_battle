//! 司令官生成模块
//!
//! 处理司令官实体及其相关动画的生成

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;
use crate::resources::{CommanderLife, GameTextureResources, GameAtlasLayoutResources};
use crate::utils;

/// 生成司令官
pub fn spawn_commander(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
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
    let commander_animation_indices = crate::atlas::COMMANDER_ATLAS.animation_indices_full();

    let commander_y = MAP_BOTTOM_Y + COMMANDER_SIZE.y / 2.0;
    let commander_x = 0.0;

    let commander_entity = utils::spawn_animated_sprite(
        &mut commands,
        commander_texture,
        atlas_layouts.commander.clone(),
        commander_animation_indices,
        ANIMATION_FRAME_COMMANDER,
        Transform::from_translation(Vec3::new(commander_x, commander_y, 0.0)),
        crate::atlas::COMMANDER_ATLAS.display_size,
        (Commander, PlayingEntity, AnimationMode::Looping),
    );

    // 添加物理组件
    commands.entity(commander_entity).insert((
        RigidBody::Fixed,
        Collider::cuboid(COMMANDER_SIZE.x / 2.0, COMMANDER_SIZE.y / 2.0),
        ActiveEvents::COLLISION_EVENTS,
    ));

    // 创建音乐动画精灵（独立实体，与 Commander 位置相同）
    let music_texture = texture_resources.music_note.clone();
    let music_animation_indices = crate::atlas::MUSIC_NOTE_ATLAS.animation_indices_full();

    let _ = utils::spawn_animated_sprite(
        &mut commands,
        music_texture,
        atlas_layouts.music_note.clone(),
        music_animation_indices,
        ANIMATION_FRAME_MUSIC_NOTE,
        Transform::from_translation(Vec3::new(commander_x, commander_y, Z_FOREST)),
        crate::atlas::MUSIC_NOTE_ATLAS.display_size,
        (
            MusicNoteAnimation,
            PlayingEntity,
            AnimationMode::Looping,
        ),
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
            crate::utils::advance_next_frame(
                &mut timer,
                &mut sprite,
                &mut current_frame,
                time.delta(),
                indices.first,
                indices.last,
            );
        }
    }
}

/// 销毁司令官实体
pub fn despawn_commander(
    mut commands: Commands,
    commanders: Query<Entity, With<Commander>>,
    music_animations: Query<Entity, With<MusicNoteAnimation>>,
) {
    // 销毁所有 Commander 实体
    crate::utils::cleanup_entities(&mut commands, commanders.iter());

    // 销毁所有 MusicNote 动画实体
    crate::utils::cleanup_entities(&mut commands, music_animations.iter());
}
