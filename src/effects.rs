//! 特效动画模块
//!
//! 处理爆炸、烟雾、火花、激光、森林火焰等特效动画

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use rand::Rng;
use std::time::Duration;

use crate::constants::*;
use crate::resources::{AmbienceResources, GameAudioResources, GameTextureResources, PlayerInfo, GameAtlasLayoutResources};
use crate::utils;

/// 通用动画特效生成函数
///
/// # 参数
/// * `commands` - 命令队列
/// * `texture_atlas_layouts` - 纹理图集布局资源
/// * `texture` - 纹理句柄
/// * `tile_size` - 精灵图块大小
/// * `columns` - 精灵图列数
/// * `rows` - 精灵图行数
/// * `animation_indices` - 动画帧索引范围
/// * `frame_duration` - 每帧持续时间（秒）
/// * `position` - 生成位置
/// * `display_size` - 显示尺寸（可选）
/// * `animation_mode` - 动画播放模式
/// * `components` - 额外的组件标记
fn spawn_animated_sprite_effect<M: Bundle>(
    commands: &mut Commands,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    texture: Handle<Image>,
    tile_size: UVec2,
    columns: u32,
    rows: u32,
    animation_indices: AnimationIndices,
    frame_duration: f32,
    position: Vec3,
    display_size: Option<Vec2>,
    animation_mode: AnimationMode,
    components: M,
) {
    utils::spawn_animated_sprite(
        commands,
        texture_atlas_layouts,
        texture,
        tile_size,
        columns,
        rows,
        animation_indices,
        frame_duration,
        position,
        display_size,
        (animation_mode, components),
    );
}

pub fn spawn_explosion(
    commands: &mut Commands,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    effect_resources: &GameTextureResources,
    sound_resources: &GameAudioResources,
    position: Vec3,
) {
    spawn_animated_sprite_effect(
        commands,
        &mut texture_atlas_layouts,
        effect_resources.explosion.clone(),
        UVec2::new(EXPLOSION_TILE_SIZE as u32, EXPLOSION_TILE_SIZE as u32),
        8,
        8,
        AnimationIndices { first: 0, last: 63 },
        ANIMATION_FRAME_EXPLOSION,
        position,
        Some(Vec2::new(EXPLOSION_DISPLAY_SIZE, EXPLOSION_DISPLAY_SIZE)),
        AnimationMode::OneShot,
        (Explosion, PlayingEntity),
    );

    // 使用预加载的爆炸音效
    sound_resources.play(commands, sound_resources.explosion.clone(), VOLUME_HALF);
}

pub fn spawn_forest_fire(
    commands: &mut Commands,
    terrain_atlas_layouts: &GameAtlasLayoutResources,
    effect_resources: &GameTextureResources,
    ambience_resources: &GameAudioResources,
    position: Vec3,
) {
    // 使用预加载的纹理图集布局
    let forest_fire_texture = effect_resources.forest_fire.clone();
    let forest_fire_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        ForestFire,
        AnimationMode::OneShot,
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
    utils::play_one_shot_sound(commands, ambience_resources.burn_tree.clone(), VOLUME_HALF);
}

pub fn spawn_spark(
    commands: &mut Commands,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    effect_resources: &GameTextureResources,
    position: Vec3,
) {
    spawn_animated_sprite_effect(
        commands,
        &mut texture_atlas_layouts,
        effect_resources.spark.clone(),
        UVec2::new(SPARK_TILE_SIZE as u32, SPARK_TILE_SIZE as u32),
        4,
        4,
        AnimationIndices { first: 0, last: 15 },
        ANIMATION_FRAME_SPARK,
        position,
        Some(Vec2::new(SPARK_DISPLAY_SIZE, SPARK_DISPLAY_SIZE)),
        AnimationMode::OneShot,
        (Spark, PlayingEntity),
    );
}

/// 帧循环更新辅助函数
///
/// 在指定帧范围内循环播放动画
fn advance_loop_frame(
    timer: &mut AnimationTimer,
    sprite: &mut Sprite,
    current_frame: &mut CurrentAnimationFrame,
    delta: Duration,
    loop_start: usize,
    loop_end: usize,
) {
    timer.tick(delta);
    if timer.just_finished() {
        let current = current_frame.0;
        let next_index = if current >= loop_end {
            loop_start
        } else {
            current + 1
        };
        current_frame.0 = next_index;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = next_index;
        }
    }
}

/// 通用动画系统
/// 根据 AnimationMode 组件统一处理所有动画播放
/// 替代原来的 animate_one_shot_animations 和 animate_looping_sprite 两个独立系统
pub fn animate_sprites(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
            &AnimationMode,
        ),
        Or<(With<AnimationMode>, Without<AnimationMode>)>,
    >,
    energy_ball_query: Query<(), (With<crate::constants::EnergyBall>, With<crate::constants::LaserPhase>)>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame, animation_mode) in &mut query {
        let prev_frame = current_frame.0;

        match animation_mode {
            AnimationMode::OneShot => {
                crate::utils::animate_sprite(&mut timer, &mut sprite, indices, &mut current_frame, time.delta());
                // 播放一次后销毁
                if prev_frame != current_frame.0 && current_frame.0 >= indices.last {
                    let () = commands.entity(entity).try_despawn();
                }
            }
            AnimationMode::OneShotHold => {
                // 到达最后一帧后不再更新
                if current_frame.0 < indices.last {
                    crate::utils::animate_sprite(&mut timer, &mut sprite, indices, &mut current_frame, time.delta());
                    if current_frame.0 >= indices.last {
                        if let Some(atlas) = &mut sprite.texture_atlas {
                            atlas.index = indices.last;
                        }
                    }
                }
            }
            AnimationMode::Looping => {
                // 循环播放，animate_sprite 已经处理
                crate::utils::animate_sprite(&mut timer, &mut sprite, indices, &mut current_frame, time.delta());
            }
            AnimationMode::LoopRange { start_frame: _, end_frame: _ } => {
                // 播放一次完整动画，然后在指定帧范围内循环
                // 检查是否是激光阶段的能量球
                let is_laser_phase = energy_ball_query.get(entity).is_ok();

                if is_laser_phase {
                    // 激光阶段：直接循环81-84帧
                    advance_loop_frame(&mut timer, &mut sprite, &mut current_frame, time.delta(),
                        crate::constants::ENERGY_BALL_LASER_LOOP_START,
                        crate::constants::ENERGY_BALL_LASER_LOOP_END);
                } else {
                    // 蓄力阶段：播放0-64帧，然后循环50-64
                    if current_frame.0 < crate::constants::ENERGY_BALL_CHARGE_LOOP_END {
                        // 还在播放完整动画阶段（0-64）
                        crate::utils::animate_sprite(&mut timer, &mut sprite, indices, &mut current_frame, time.delta());
                        // 播放到第64帧后，立即切换到循环模式，防止继续播放65-80帧
                        if current_frame.0 == crate::constants::ENERGY_BALL_CHARGE_LOOP_END {
                            advance_loop_frame(&mut timer, &mut sprite, &mut current_frame, time.delta(),
                                crate::constants::ENERGY_BALL_CHARGE_LOOP_START,
                                crate::constants::ENERGY_BALL_CHARGE_LOOP_END);
                        }
                    } else {
                        // 完整动画播放完毕，进入循环阶段（50-64）
                        advance_loop_frame(&mut timer, &mut sprite, &mut current_frame, time.delta(),
                            crate::constants::ENERGY_BALL_CHARGE_LOOP_START,
                            crate::constants::ENERGY_BALL_CHARGE_LOOP_END);
                    }
                }
            }
        }
    }
}

/// 更新气垫效果
pub fn update_air_cushion_effect(
    mut commands: Commands,
    player_tanks: Query<(Entity, Option<&Children>, &PlayerTank), With<PlayerTank>>,
    bubble_entities: Query<(), With<crate::constants::BubbleEffect>>,
    player_info: Res<PlayerInfo>,
    texture_resources: Res<GameTextureResources>,
) {
    for (entity, children, player_tank) in player_tanks.iter() {
        // 检查玩家是否有 air_cushion 能力
        let has_air_cushion = player_info.has_air_cushion(player_tank.tank_type);

        // 检查是否已经有气泡特效子实体
        let has_bubble = children.is_some_and(|c| c.iter().any(|child| bubble_entities.contains(child)));

        if has_air_cushion && !has_bubble {
            // 创建气泡特效
            let bubble_texture = texture_resources.bubble.clone();
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
                    Transform::from_xyz(0.0, 0.0, Z_DEFAULT),
                    crate::constants::BubbleEffect,
                ));
            });
        } else if !has_air_cushion && has_bubble {
            // 移除所有气泡特效子实体
            crate::utils::cleanup_children_by_marker(&mut commands, children, &bubble_entities);
        }
    }
}

/// 通用环境音效播放系统
/// T: 地形组件标记 (Sea, Forest, Commander)
/// P: 音效播放器标记 (SeaAmbiencePlayer, TreeAmbiencePlayer, CommanderAmbiencePlayer)
#[allow(unused_mut)]
pub fn play_ambience_generic<T, P>(
    #[allow(unused_mut)] mut commands: Commands,
    ambience_sound: Handle<AudioSource>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    terrain_entities: Query<&Transform, With<T>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<P>>,
    volume: f32,
) where
    T: Component,
    P: Component + Default,
{
    // 检查是否有玩家坦克在附近
    let mut is_near = false;

    for player_transform in player_tanks.iter() {
        for terrain_transform in terrain_entities.iter() {
            if player_transform.translation.distance(terrain_transform.translation) < DETECTION_RADIUS {
                is_near = true;
                break;
            }
        }
        if is_near {
            break;
        }
    }

    if is_near && ambience_players.is_empty() {
        let entity = utils::play_looping_sound(&mut commands, ambience_sound, volume);
        commands.entity(entity).insert(P::default());
    } else if !is_near {
        utils::cleanup_entities(&mut commands, ambience_players.iter().map(|(e, _)| e));
    }
}

/// 播放海洋的环境音效
#[allow(unused_mut)]
pub fn play_sea_ambience(
    mut commands: Commands,
    ambience_resources: Res<AmbienceResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    seas: Query<&Transform, With<Sea>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<SeaAmbiencePlayer>>,
) {
    play_ambience_generic::<Sea, SeaAmbiencePlayer>(
        commands,
        ambience_resources.sea_ambience.clone(),
        player_tanks,
        seas,
        ambience_players,
        VOLUME_HALF,
    );
}

/// 播放森林的环境音效
#[allow(unused_mut)]
pub fn play_tree_ambience(
    mut commands: Commands,
    ambience_resources: Res<AmbienceResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    forests: Query<&Transform, With<Forest>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<TreeAmbiencePlayer>>,
) {
    play_ambience_generic::<Forest, TreeAmbiencePlayer>(
        commands,
        ambience_resources.tree_ambience.clone(),
        player_tanks,
        forests,
        ambience_players,
        VOLUME_HALF,
    );
}

/// 从司令官音乐列表中随机选择一首
fn select_random_commander_music(ambience_resources: &AmbienceResources) -> Handle<AudioSource> {
    let music_files = [
        &ambience_resources.commander_music_000,
        &ambience_resources.commander_music_001,
        &ambience_resources.commander_music_002,
        &ambience_resources.commander_music_003,
    ];
    let mut rng = rand::rng();
    music_files[rng.random_range(0..music_files.len())].clone()
}

/// 播放司令官的环境音效
#[allow(unused_mut)]
pub fn play_commander_ambience(
    #[allow(unused_mut)] mut commands: Commands,
    ambience_resources: Res<AmbienceResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    commander: Query<&Transform, With<Commander>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<CommanderAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在司令官附近
    let is_near_commander = player_tanks.iter().any(|player_transform| {
        commander.iter().any(|commander_transform| {
            player_transform.translation.distance(commander_transform.translation) < DETECTION_RADIUS
        })
    });

    if is_near_commander && ambience_players.is_empty() {
        let random_music = select_random_commander_music(&ambience_resources);
        let entity = utils::play_looping_sound(&mut commands, random_music, VOLUME_COMMANDER_MUSIC);
        commands.entity(entity).insert(CommanderAmbiencePlayer);
    } else if !is_near_commander {
        utils::cleanup_entities(&mut commands, ambience_players.iter().map(|(e, _)| e));
    }
}


