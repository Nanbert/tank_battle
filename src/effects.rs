//! 特效动画模块
//!
//! 处理爆炸、烟雾、火花、激光、森林火焰等特效动画

#![allow(clippy::wildcard_imports)]

use bevy::audio::Volume;
use bevy::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::{AmbienceResources, EffectResources, PlayerInfo, TerrainAtlasLayouts, SoundResources};

pub fn spawn_explosion(
    commands: &mut Commands,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    effect_resources: &EffectResources,
    sound_resources: &SoundResources,
    position: Vec3,
) {
    // 使用预加载的爆炸纹理
    let explosion_texture = effect_resources.explosion.clone();
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
            custom_size: Some(Vec2::new(EXPLOSION_DISPLAY_SIZE, EXPLOSION_DISPLAY_SIZE)),
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

    // 使用预加载的爆炸音效
    commands.spawn((
        AudioPlayer::new(sound_resources.explosion.clone()),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(VOLUME_HALF)),
    ));
}

pub fn spawn_forest_fire(
    commands: &mut Commands,
    terrain_atlas_layouts: &TerrainAtlasLayouts,
    effect_resources: &EffectResources,
    ambience_resources: &AmbienceResources,
    position: Vec3,
) {
    // 使用预加载的纹理图集布局
    let forest_fire_texture = effect_resources.forest_fire.clone();
    let forest_fire_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        ForestFire,
        PlayingEntity,
        Sprite::from_atlas_image(
            forest_fire_texture,
            TextureAtlas {
                layout: terrain_atlas_layouts.forest_fire.clone(),
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
    commands.spawn((
        AudioPlayer::new(ambience_resources.burn_tree.clone()),
    ));
}

pub fn spawn_spark(
    commands: &mut Commands,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    effect_resources: &EffectResources,
    position: Vec3,
) {
    // 使用预加载的打击效果纹理
    let spark_texture = effect_resources.spark.clone();
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
            custom_size: Some(Vec2::new(SPARK_DISPLAY_SIZE, SPARK_DISPLAY_SIZE)),
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

/// 更新气垫效果
pub fn update_air_cushion_effect(
    mut commands: Commands,
    player_tanks: Query<(Entity, Option<&Children>, &PlayerTank), With<PlayerTank>>,
    bubble_effects: Query<&crate::constants::BubbleEffect>,
    player_info: Res<PlayerInfo>,
    effect_resources: Res<EffectResources>,
) {
    for (entity, children, player_tank) in player_tanks.iter() {
        // 检查玩家是否有 air_cushion 能力
        let has_air_cushion = match player_tank.tank_type {
            TankType::Player1 => player_info.player1.air_cushion,
            TankType::Player2 => player_info
                .player2
                .as_ref()
                .is_some_and(|stats| stats.air_cushion),
            TankType::Enemy => false,
        };

        if has_air_cushion {
            // 检查是否已经有气泡特效子实体
            let has_bubble_sprite = children.is_some_and(|children| {
                children.iter().any(|child| bubble_effects.contains(child))
            });

            if !has_bubble_sprite {
                // 使用预加载的气泡纹理
                let bubble_texture = effect_resources.bubble.clone();

                // 创建气泡特效实体
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Sprite {
                            image: bubble_texture,
                            custom_size: Some(Vec2::new(
                                crate::powerup::POWERUP_BUBBLE_SIZE,
                                crate::powerup::POWERUP_BUBBLE_SIZE,
                            )),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, Z_DEFAULT), // 在坦克中心
                        crate::constants::BubbleEffect,
                    ));
                });
            }
        } else {
            // 移除所有气泡特效子实体
            if let Some(children) = children {
                for child in children.iter() {
                    if bubble_effects.contains(child) {
                        let () = commands.entity(child).try_despawn();
                    }
                }
            }
        }
    }
}

/// 播放海洋的环境音效
pub fn play_sea_ambience(
    mut commands: Commands,
    ambience_resources: Res<AmbienceResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    seas: Query<&Transform, With<Sea>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<SeaAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在海附近
    let mut is_near_sea = false;

    for player_transform in player_tanks.iter() {
        for sea_transform in seas.iter() {
            let distance = player_transform
                .translation
                .distance(sea_transform.translation);
            if distance < DETECTION_RADIUS {
                is_near_sea = true;
                break;
            }
        }
        if is_near_sea {
            break;
        }
    }

    if is_near_sea {
        // 如果在海附近但没有播放音效，则播放
        if ambience_players.is_empty() {
            let sea_ambience_sound = ambience_resources.sea_ambience.clone();
            commands.spawn((
                AudioPlayer::new(sea_ambience_sound),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(VOLUME_HALF)),
                SeaAmbiencePlayer,
            ));
        }
    } else {
        // 如果不在海附近但有播放音效，则停止
        for (entity, _) in ambience_players.iter() {
            let () = commands.entity(entity).try_despawn();
        }
    }
}

/// 播放司令官的环境音效
pub fn play_commander_ambience(
    mut commands: Commands,
    ambience_resources: Res<AmbienceResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    commander: Query<&Transform, With<Commander>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<CommanderAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在司令官附近
    let mut is_near_commander = false;

    for player_transform in player_tanks.iter() {
        for commander_transform in commander.iter() {
            let distance = player_transform
                .translation
                .distance(commander_transform.translation);
            if distance < DETECTION_RADIUS {
                is_near_commander = true;
                break;
            }
        }
        if is_near_commander {
            break;
        }
    }

    if is_near_commander {
        // 如果在司令官附近但没有播放音效，则播放
        if ambience_players.is_empty() {
            // 从 commander_music_000 到 commander_music_003 中随机选择
            let music_files = [
                &ambience_resources.commander_music_000,
                &ambience_resources.commander_music_001,
                &ambience_resources.commander_music_002,
                &ambience_resources.commander_music_003,
            ];
            let mut rng = rand::rng();
            let random_music = music_files[rng.random_range(0..music_files.len())];

            let commander_music_sound = random_music.clone();
            commands.spawn((
                AudioPlayer::new(commander_music_sound),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(VOLUME_COMMANDER_MUSIC)),
                CommanderAmbiencePlayer,
            ));
        }
    } else {
        // 如果不在司令官附近但有播放音效，则停止
        for (entity, _) in ambience_players.iter() {
            let () = commands.entity(entity).try_despawn();
        }
    }
}

/// 播放森林的环境音效
pub fn play_tree_ambience(
    mut commands: Commands,
    ambience_resources: Res<AmbienceResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    forests: Query<&Transform, With<Forest>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<TreeAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在森林附近
    let mut is_near_forest = false;

    for player_transform in player_tanks.iter() {
        for forest_transform in forests.iter() {
            let distance = player_transform
                .translation
                .distance(forest_transform.translation);
            if distance < DETECTION_RADIUS {
                is_near_forest = true;
                break;
            }
        }
        if is_near_forest {
            break;
        }
    }

    if is_near_forest {
        // 如果在森林附近但没有播放音效，则播放
        if ambience_players.is_empty() {
            let tree_ambience_sound = ambience_resources.tree_ambience.clone();
            commands.spawn((
                AudioPlayer::new(tree_ambience_sound),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(VOLUME_HALF)),
                TreeAmbiencePlayer,
            ));
        }
    } else {
        // 如果不在森林附近但有播放音效，则停止
        for (entity, _) in ambience_players.iter() {
            let () = commands.entity(entity).try_despawn();
        }
    }
}


