//! 特效动画模块
//!
//! 处理爆炸、烟雾、火花、激光、森林火焰等特效动画

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::{
    GameAudioResources, GameTextureResources, PlayerInfo,
};
use crate::utils;

/// 通用动画特效生成函数
///
/// # 参数
/// * `commands` - 命令队列
/// * `texture_atlas_layouts` - 纹理图集布局资源
pub fn spawn_explosion(
    commands: &mut Commands,
    effect_resources: &GameTextureResources,
    atlas_layouts: &crate::resources::GameAtlasLayoutResources,
    sound_resources: &GameAudioResources,
    position: Vec3,
) {
    let _ = utils::spawn_animated_sprite(
        commands,
        effect_resources.explosion.clone(),
        atlas_layouts.explosion.clone(),
        crate::atlas::EXPLOSION_ATLAS.animation_indices_full(),
        ANIMATION_FRAME_EXPLOSION,
        Transform::from_translation(position),
        crate::atlas::EXPLOSION_ATLAS.display_size,
        (AnimationMode::OneShot, Explosion, PlayingEntity),
    );

    // 使用预加载的爆炸音效
    utils::play_one_shot_sound(commands, sound_resources.explosion.clone(), VOLUME_HALF);
}

pub fn spawn_forest_fire(
    commands: &mut Commands,
    effect_resources: &GameTextureResources,
    atlas_layouts: &crate::resources::GameAtlasLayoutResources,
    sound_resources: &GameAudioResources,
    position: Vec3,
) {
    let _ = utils::spawn_animated_sprite(
        commands,
        effect_resources.forest_fire.clone(),
        atlas_layouts.forest_fire.clone(),
        crate::atlas::FOREST_FIRE_ATLAS.animation_indices_full(),
        ANIMATION_FRAME_FOREST_FIRE,
        Transform::from_translation(position),
        crate::atlas::FOREST_FIRE_ATLAS.display_size,
        (AnimationMode::OneShot, ForestFire, PlayingEntity),
    );

    // 播放树林燃烧音效
    utils::play_one_shot_sound(commands, sound_resources.burn_tree.clone(), VOLUME_HALF);
}

pub fn spawn_spark(
    commands: &mut Commands,
    effect_resources: &GameTextureResources,
    atlas_layouts: &crate::resources::GameAtlasLayoutResources,
    audio_handle: Handle<AudioSource>,
    volume: f32,
    position: Vec3,
) {
    let _ = utils::spawn_animated_sprite(
        commands,
        effect_resources.spark.clone(),
        atlas_layouts.spark.clone(),
        crate::atlas::SPARK_ATLAS.animation_indices_full(),
        ANIMATION_FRAME_SPARK,
        Transform::from_translation(position),
        crate::atlas::SPARK_ATLAS.display_size,
        (AnimationMode::OneShot, Spark, PlayingEntity),
    );

    // 播放指定的音效
    utils::play_one_shot_sound(commands, audio_handle, volume);
}

/// 通用动画系统
/// 根据 AnimationMode 组件统一处理所有动画播放
/// 替代原来的 animate_one_shot_animations 和 animate_looping_sprite 两个独立系统
pub fn animate_effects(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut spawn_events: MessageWriter<crate::enemy::SpawnEnemyEvent>,
    mut laser_end_events: MessageWriter<crate::constants::LaserEndEvent>,
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
    born_position_query: Query<&crate::enemy::BornPosition>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame, animation_mode) in &mut query {
        let prev_frame = current_frame.0;

        match animation_mode {
            AnimationMode::OneShot => {
                crate::utils::advance_next_frame(
                    &mut timer,
                    &mut sprite,
                    &mut current_frame,
                    time.delta(),
                    indices.first,
                    indices.last,
                );
                // 播放一次后销毁
                if prev_frame != current_frame.0 && current_frame.0 >= indices.last {
                    let () = commands.entity(entity).try_despawn();
                }
            }
            AnimationMode::Looping => {
                // 循环播放
                crate::utils::advance_next_frame(
                    &mut timer,
                    &mut sprite,
                    &mut current_frame,
                    time.delta(),
                    indices.first,
                    indices.last,
                );
            }
            AnimationMode::LoopRange {
                start_frame,
                end_frame,
            } => {
                // 在指定帧范围内循环播放（用于能量球）
                utils::advance_next_frame(
                    &mut timer,
                    &mut sprite,
                    &mut current_frame,
                    time.delta(),
                    *start_frame,
                    *end_frame,
                );
            }
            AnimationMode::Conditional { tank_type } => {
                // 只有条件满足时才播放动画（用于履带、玩家坦克纹理等）
                let key_bindings = tank_type.get_key_bindings();
                let is_moving = key_bindings.is_moving(&keyboard_input);
                if is_moving {
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
            AnimationMode::AtFrameWithEvent {
                trigger_frame,
                event_type,
            } => {
                crate::utils::advance_next_frame(
                    &mut timer,
                    &mut sprite,
                    &mut current_frame,
                    time.delta(),
                    indices.first,
                    indices.last,
                );
                // 在指定帧触发事件
                if prev_frame != current_frame.0 && current_frame.0 == *trigger_frame {
                    match event_type {
                        AnimationEventType::SpawnEnemy => {
                            if let Ok(born_position) = born_position_query.get(entity) {
                                spawn_events.write(crate::enemy::SpawnEnemyEvent {
                                    position: born_position.0,
                                });
                            }
                        }
                        AnimationEventType::LaserAnimationEnd {
                            direction,
                            start_point,
                            owner_type,
                            energy_ball_entity,
                        } => {
                            laser_end_events.write(crate::constants::LaserEndEvent {
                                direction: *direction,
                                start_point: *start_point,
                                owner_type: *owner_type,
                                energy_ball_entity: *energy_ball_entity,
                            });
                        }
                    }
                }
                // 动画播放完毕后销毁
                if prev_frame != current_frame.0 && current_frame.0 >= indices.last {
                    let () = commands.entity(entity).try_despawn();
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
        let has_bubble =
            children.is_some_and(|c| c.iter().any(|child| bubble_entities.contains(child)));

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
pub fn play_ambience_generic<T, P>(
    mut commands: Commands,
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
            if player_transform
                .translation
                .distance(terrain_transform.translation)
                < DETECTION_RADIUS
            {
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
pub fn play_sea_ambience(
    commands: Commands,
    audio_resources: Res<GameAudioResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    seas: Query<&Transform, With<Sea>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<SeaAmbiencePlayer>>,
) {
    play_ambience_generic::<Sea, SeaAmbiencePlayer>(
        commands,
        audio_resources.sea_ambience.clone(),
        player_tanks,
        seas,
        ambience_players,
        VOLUME_HALF,
    );
}

/// 播放森林的环境音效
pub fn play_tree_ambience(
    commands: Commands,
    audio_resources: Res<GameAudioResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    forests: Query<&Transform, With<Forest>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<TreeAmbiencePlayer>>,
) {
    play_ambience_generic::<Forest, TreeAmbiencePlayer>(
        commands,
        audio_resources.tree_ambience.clone(),
        player_tanks,
        forests,
        ambience_players,
        VOLUME_HALF,
    );
}

/// 从音符音乐列表中随机选择一首
fn select_random_music_note(audio_resources: &GameAudioResources) -> Handle<AudioSource> {
    let music_files = [
        &audio_resources.music_note_000,
        &audio_resources.music_note_001,
        &audio_resources.music_note_002,
        &audio_resources.music_note_003,
    ];
    let mut rng = rand::rng();
    music_files[rng.random_range(0..music_files.len())].clone()
}

/// 播放司令官的环境音效
pub fn play_commander_ambience(
    mut commands: Commands,
    audio_resources: Res<GameAudioResources>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    commander: Query<&Transform, With<Commander>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<CommanderAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在司令官附近
    let is_near_commander = player_tanks.iter().any(|player_transform| {
        commander.iter().any(|commander_transform| {
            player_transform
                .translation
                .distance(commander_transform.translation)
                < DETECTION_RADIUS
        })
    });

    if is_near_commander && ambience_players.is_empty() {
        let random_music = select_random_music_note(&audio_resources);
        let entity = utils::play_looping_sound(&mut commands, random_music, VOLUME_MUSIC_NOTE);
        commands.entity(entity).insert(CommanderAmbiencePlayer);
    } else if !is_near_commander {
        utils::cleanup_entities(&mut commands, ambience_players.iter().map(|(e, _)| e));
    }
}
