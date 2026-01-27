//! 特效动画模块
//!
//! 处理爆炸、烟雾、火花、激光、森林火焰等特效动画

#![allow(clippy::wildcard_imports)]

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::constants::*;

pub fn spawn_explosion(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载爆炸精灵图（8x8，共64帧，每帧512x512）
    let explosion_texture: Handle<Image> = asset_server.load(TEXTURE_EXPLOSION);
    let explosion_tile_size = UVec2::new(EXPLOSION_TILE_SIZE as u32, EXPLOSION_TILE_SIZE as u32);
    let explosion_texture_atlas =
        TextureAtlasLayout::from_grid(explosion_tile_size, 8, 8, None, None);
    let explosion_texture_atlas_layout = texture_atlas_layouts.add(explosion_texture_atlas);
    let explosion_animation_indices = AnimationIndices { first: 0, last: 63 };

    commands.spawn((
        Explosion,
        PlayingEntity,
        Sprite {
            image: explosion_texture,
            texture_atlas: Some(TextureAtlas {
                layout: explosion_texture_atlas_layout,
                index: explosion_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(300.0, 300.0)),
            ..default()
        },
        Transform::from_translation(position),
        explosion_animation_indices,
        AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_EXPLOSION,
            TimerMode::Repeating,
        )),
        CurrentAnimationFrame(0),
    ));

    // 播放爆炸音效
    let explosion_sound: Handle<AudioSource> = asset_server.load(SOUND_EXPLOSION);
    commands.spawn((
        AudioPlayer::new(explosion_sound),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(VOLUME_HALF)),
    ));
}

pub fn spawn_forest_fire(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载树林燃烧精灵图（10帧，每帧131x131，1.5秒播完）
    let forest_fire_texture: Handle<Image> = asset_server.load("maps/tree_fire_sheet.png");
    let forest_fire_tile_size = UVec2::new(131, 131);
    let forest_fire_texture_atlas =
        TextureAtlasLayout::from_grid(forest_fire_tile_size, 10, 1, None, None);
    let forest_fire_texture_atlas_layout = texture_atlas_layouts.add(forest_fire_texture_atlas);
    let forest_fire_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        ForestFire,
        PlayingEntity,
        Sprite::from_atlas_image(
            forest_fire_texture,
            TextureAtlas {
                layout: forest_fire_texture_atlas_layout,
                index: forest_fire_animation_indices.first,
            },
        ),
        Transform::from_translation(position),
        forest_fire_animation_indices,
        AnimationTimer(Timer::from_seconds(
            FOREST_FIRE_DURATION / 10.0,
            TimerMode::Repeating,
        )), // 1.5秒播完10帧
        CurrentAnimationFrame(0),
    ));

    // 播放树林燃烧音效
    let burn_tree_sound: Handle<AudioSource> = asset_server.load(SOUND_BURN_TREE);
    commands.spawn(AudioPlayer::new(burn_tree_sound));
}

pub fn spawn_spark(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载打击效果图片（4x4，共16帧，每帧1024x1024）
    let spark_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL_HIT);
    let spark_tile_size = UVec2::new(SPARK_TILE_SIZE as u32, SPARK_TILE_SIZE as u32);
    let spark_texture_atlas = TextureAtlasLayout::from_grid(spark_tile_size, 4, 4, None, None);
    let spark_texture_atlas_layout = texture_atlas_layouts.add(spark_texture_atlas);
    let spark_animation_indices = AnimationIndices { first: 0, last: 15 };

    commands.spawn((
        Spark,
        PlayingEntity,
        Sprite {
            image: spark_texture,
            texture_atlas: Some(TextureAtlas {
                layout: spark_texture_atlas_layout,
                index: spark_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(200.0, 200.0)),
            ..default()
        },
        Transform::from_translation(position),
        spark_animation_indices,
        AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_SPARK,
            TimerMode::Repeating,
        )),
        CurrentAnimationFrame(0),
    ));
}

pub fn animate_explosion(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Explosion>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁爆炸实体
                let () = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

/// 处理烟雾动画
pub fn animate_smoke(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Smoke>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁烟雾实体
                let () = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

/// 森林燃烧动画
pub fn animate_forest_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<ForestFire>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁森林燃烧实体
                let () = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

/// 通用循环动画系统
/// 用于播放需要循环播放的动画（如森林、海洋等）
pub fn animate_looping_sprite<T: Component>(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<T>,
    >,
) {
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

pub fn animate_spark(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Spark>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁实体
                let () = commands.entity(entity).try_despawn();
            } else {
                // 继续播放动画
                let next_index = current + 1;
                current_frame.0 = next_index;
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = next_index;
                }
            }
        }
    }
}


