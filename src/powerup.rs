//! 道具系统
//!
//! 处理道具生成、动画和玩家拾取道具的碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::{CommanderLife, PlayerInfo, PlayerStatChanged, PowerUpResources, SoundResources, StatType};
use crate::utils;

/// 道具碰撞检测距离
pub const POWERUP_COLLISION_DISTANCE: f32 = 100.0;

/// 气泡特效尺寸
pub const POWERUP_BUBBLE_SIZE: f32 = 100.0;

/// 道具属性增加量
pub const POWERUP_ATTRIBUTE_INCREASE: usize = 20;

/// 道具类型枚举
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum PowerUp {
    SpeedUp,
    Protection,
    FireSpeed,
    FireShell,
    TrackChain,
    Penetrate,
    Repair,
    Hamburger,
    AirCushion,
    Shell,
}

/// 道具动画系统
pub fn animate_powerup(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<PowerUp>,
    >,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        crate::utils::animate_sprite(&mut timer, &mut sprite, indices, &mut current_frame, time.delta());
    }
}

/// 道具碰撞检测和拾取系统
pub fn handle_powerup_collision(
    mut commands: Commands,
    sound_resources: Res<SoundResources>,
    powerups: Query<(Entity, &Transform, &PowerUp)>,
    player_tanks: Query<(&Transform, &PlayerTank, Entity), With<PlayerTank>>,
    mut controllers: Query<&mut KinematicCharacterController>,
    mut player_info: ResMut<PlayerInfo>,
    mut commander_life: ResMut<CommanderLife>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
) {
    for (tank_transform, player_tank, tank_entity) in player_tanks.iter() {
        let mut picked_powerup: Option<PowerUp> = None;
        let mut powerup_entity_to_despawn: Option<Entity> = None;

        // 检查道具碰撞
        for (powerup_entity, powerup_transform, powerup_type) in powerups.iter() {
            let distance = (powerup_transform.translation - tank_transform.translation).length();
            if distance < POWERUP_COLLISION_DISTANCE {
                picked_powerup = Some(*powerup_type);
                powerup_entity_to_despawn = Some(powerup_entity);
            }
        }

        if let Some(powerup_type) = picked_powerup {
            let powerup_entity = powerup_entity_to_despawn.unwrap();

            // 播放道具音效
            let powerup_sound = sound_resources.hit.clone();
            commands.spawn((
                AudioPlayer::new(powerup_sound),
            ));
            let () = commands.entity(powerup_entity).try_despawn();

            // 根据道具类型应用效果并发送事件
            let tank_type = player_tank.tank_type;
            let mut stat_type = None;
            let mut update_filter_groups = false;

            player_info.with_stats_mut(tank_type, |player_stats| {
                stat_type = match powerup_type {
                    PowerUp::SpeedUp => {
                        if player_stats.speed < MAX_ATTRIBUTE_VALUE {
                            player_stats.speed = (player_stats.speed + POWERUP_ATTRIBUTE_INCREASE)
                                .min(MAX_ATTRIBUTE_VALUE);
                        }
                        Some(StatType::Speed)
                    }
                    PowerUp::Protection => {
                        if player_stats.protection < MAX_ATTRIBUTE_VALUE {
                            player_stats.protection = (player_stats.protection
                                + POWERUP_ATTRIBUTE_INCREASE)
                                .min(MAX_ATTRIBUTE_VALUE);
                        }
                        Some(StatType::Protection)
                    }
                    PowerUp::FireSpeed => {
                        if player_stats.fire_speed < MAX_ATTRIBUTE_VALUE {
                            player_stats.fire_speed = (player_stats.fire_speed
                                + POWERUP_ATTRIBUTE_INCREASE)
                                .min(MAX_ATTRIBUTE_VALUE);
                        }
                        Some(StatType::FireSpeed)
                    }
                    PowerUp::FireShell => {
                        player_stats.fire_shell = true;
                        Some(StatType::FireShell)
                    }
                    PowerUp::TrackChain => {
                        player_stats.track_chain = true;
                        Some(StatType::TrackChain)
                    }
                    PowerUp::Penetrate => {
                        player_stats.penetrate = true;
                        Some(StatType::Penetrate)
                    }
                    PowerUp::Repair => {
                        if player_stats.life_points < COMMANDER_LIFE_MAX {
                            player_stats.life_points += 1;
                        }
                        None // 修理道具不需要闪烁文字
                    }
                    PowerUp::Hamburger => {
                        None // 汉堡道具不影响玩家属性，不发送事件
                    }
                    PowerUp::AirCushion => {
                        player_stats.air_cushion = true;
                        update_filter_groups = true;
                        Some(StatType::AirCushion)
                    }
                    PowerUp::Shell => {
                        // 增加 1 颗子弹，最多 2 颗
                        if player_stats.shells < 2 {
                            player_stats.shells += 1;
                        }
                        Some(StatType::Shell)
                    }
                };
            });

            // 处理汉堡道具效果（修改 Commander 生命）
            if let PowerUp::Hamburger = powerup_type {
                if commander_life.life_points < COMMANDER_LIFE_MAX {
                    commander_life.life_points += 1;
                }
            }

            // 更新 filter_groups，排除海（GROUP_2）
            if update_filter_groups {
                if let Ok(mut controller) = controllers.get_mut(tank_entity) {
                    controller.filter_groups = Some(CollisionGroups::new(
                        Group::all(),
                        Group::all() & !SEA_GROUP,
                    ));
                }
            }

            // 发送属性变更事件（如果有）
            if let Some(st) = stat_type {
                stat_changed_events.write(PlayerStatChanged {
                    player_type: tank_type,
                    stat_type: st,
                });
            }
        }
    }
}

/// 第一关测试道具生成
///
/// 第一关固定生成 fire_shell 道具用于测试火焰特效效果。
/// 道具会生成在地图范围内，避开坦克出生点和司令官区域。
pub fn spawn_test_powerup_stage1(
    mut commands: Commands,
    powerup_resources: Res<PowerUpResources>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let powerup_type = PowerUp::FireShell; // 第一关测试用：改为 fire_shell

    // 定义禁止区域
    // 上方：坦克高度区域（MAP_TOP_Y - ENEMY_TANK_DISPLAY_HEIGHT 到 MAP_TOP_Y）
    // 下方：commander高度区域（MAP_BOTTOM_Y 到 MAP_BOTTOM_Y + COMMANDER_HEIGHT）
    let top_forbidden_y = MAP_TOP_Y - ENEMY_TANK_DISPLAY_HEIGHT;
    let bottom_forbidden_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT;

    // 在随机位置生成道具（在地图范围内），避开禁止区域
    let mut rng = rand::rng();
    let x = rng.random_range(MAP_LEFT_X + 100.0..MAP_RIGHT_X - 100.0);
    let y = rng.random_range(bottom_forbidden_y + 100.0..top_forbidden_y - 100.0);
    let position = Vec3::new(x, y, 0.0);

    spawn_powerup_batch(
        &mut commands,
        &powerup_resources,
        &mut texture_atlas_layouts,
        powerup_type,
        &[position],
    );
}

/// 随机生成道具
pub fn spawn_power_ups_random(
    mut commands: Commands,
    powerup_resources: Res<PowerUpResources>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let powerup_types = [
        PowerUp::SpeedUp,
        PowerUp::Protection,
        PowerUp::FireSpeed,
        PowerUp::FireShell,
        PowerUp::TrackChain,
        PowerUp::Penetrate,
        PowerUp::Repair,
        PowerUp::Hamburger,
        PowerUp::AirCushion,
        PowerUp::Shell,
    ];

    let mut rng = rand::rng();
    let powerup_type = powerup_types[rng.random_range(0..powerup_types.len())];

    // 定义禁止区域
    // 上方：坦克高度区域（MAP_TOP_Y - ENEMY_TANK_DISPLAY_HEIGHT 到 MAP_TOP_Y）
    // 下方：commander高度区域（MAP_BOTTOM_Y 到 MAP_BOTTOM_Y + COMMANDER_HEIGHT）
    let top_forbidden_y = MAP_TOP_Y - ENEMY_TANK_DISPLAY_HEIGHT;
    let bottom_forbidden_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT;

    // 在随机位置生成道具（在地图范围内），避开禁止区域
    let x = rng.random_range(MAP_LEFT_X + 100.0..MAP_RIGHT_X - 100.0);
    let y = rng.random_range(bottom_forbidden_y + 100.0..top_forbidden_y - 100.0);
    let position = Vec3::new(x, y, 0.0);

    spawn_powerup_batch(
        &mut commands,
        &powerup_resources,
        &mut texture_atlas_layouts,
        powerup_type,
        &[position],
    );
}

/// 批量生成道具
///
/// 在指定位置生成多个相同类型的道具。
///
/// 参数：
/// - commands: 命令队列
/// - powerup_resources: 道具资源
/// - texture_atlas_layouts: 纹理图集布局资源
/// - powerup_type: 道具类型
/// - texture_path: 道具纹理路径
/// - positions: 生成位置数组
fn spawn_powerup_batch(
    commands: &mut Commands,
    powerup_resources: &PowerUpResources,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    powerup_type: PowerUp,
    positions: &[Vec3],
) {
    let texture = match powerup_type {
        PowerUp::SpeedUp => powerup_resources.speed_up.clone(),
        PowerUp::Protection => powerup_resources.protection.clone(),
        PowerUp::FireSpeed => powerup_resources.fire_speed.clone(),
        PowerUp::FireShell => powerup_resources.fire_shell.clone(),
        PowerUp::TrackChain => powerup_resources.track_chain.clone(),
        PowerUp::Penetrate => powerup_resources.penetrate.clone(),
        PowerUp::Repair => powerup_resources.repair.clone(),
        PowerUp::Hamburger => powerup_resources.hamburger.clone(),
        PowerUp::AirCushion => powerup_resources.air_cushion.clone(),
        PowerUp::Shell => powerup_resources.shell.clone(),
    };
    let tile_size = UVec2::new(87, 69);
    let texture_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, tile_size, 3, 1);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    for pos in positions {
        commands.spawn((
            powerup_type,
            PlayingEntity,
            Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: texture_atlas.clone(),
                    index: animation_indices.first,
                },
            ),
            Transform::from_xyz(pos.x, pos.y, 0.8), // z=0.8 使道具高于除了树之外的所有图层
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            CurrentAnimationFrame(0),
        ));
    }
}

/// 销毁所有道具
pub fn despawn_powerups(mut commands: Commands, powerups: Query<Entity, With<PowerUp>>) {
    for entity in powerups.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}

/// 更新履带特效
/// 根据玩家是否拥有 track_chain 能力，动态添加或移除履带子实体
pub fn update_track_chain_effect(
    mut commands: Commands,
    player_tanks: Query<(Entity, Option<&Children>, &PlayerTank), With<PlayerTank>>,
    track_chain_entities: Query<(), With<crate::constants::TrackChainEffect>>,
    player_info: Res<PlayerInfo>,
    powerup_resources: Res<PowerUpResources>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, children, player_tank) in player_tanks.iter() {
        // 检查玩家是否有 track_chain 能力
        let has_track_chain = player_info.get_stat_value(player_tank.tank_type, |stats| stats.track_chain as usize) > 0;

        // 检查是否已经有履带特效子实体
        let has_track_chain_sprite = children.is_some_and(|c| c.iter().any(|child| track_chain_entities.contains(child)));

        if has_track_chain && !has_track_chain_sprite {
            // 创建履带特效
            let track_train_texture = powerup_resources.track_train.clone();
            let track_train_tile_size = UVec2::new(
                crate::constants::TRACK_CHAIN_TILE_WIDTH as u32,
                crate::constants::TRACK_CHAIN_TILE_HEIGHT as u32,
            );
            let track_train_texture_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, track_train_tile_size, 2, 1);
            let track_train_animation_indices = AnimationIndices { first: 0, last: 1 };

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Sprite {
                        image: track_train_texture,
                        texture_atlas: Some(TextureAtlas {
                            layout: track_train_texture_atlas,
                            index: track_train_animation_indices.first,
                        }),
                        custom_size: Some(Vec2::new(
                            crate::constants::TRACK_CHAIN_DISPLAY_WIDTH,
                            crate::constants::TRACK_CHAIN_DISPLAY_HEIGHT,
                        )),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, crate::constants::Z_DEFAULT + 0.1),
                    track_train_animation_indices,
                    AnimationTimer(Timer::from_seconds(
                        crate::constants::TRACK_CHAIN_ANIMATION_FRAME,
                        TimerMode::Repeating,
                    )),
                    CurrentAnimationFrame(0),
                    crate::constants::TrackChainEffect,
                ));
            });
        } else if !has_track_chain && has_track_chain_sprite {
            // 移除所有履带特效子实体
            if let Some(children) = children {
                for child in children.iter() {
                    if track_chain_entities.contains(child) {
                        let () = commands.entity(child).try_despawn();
                    }
                }
            }
        }
    }
}

/// 履带动画系统
/// 仅在坦克移动时播放履带动画，静止时停止
pub fn animate_track_chain(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_tanks: Query<(&PlayerTank, &Children), With<PlayerTank>>,
    mut track_chain_query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<TrackChainEffect>,
    >,
) {
    // 先收集所有履带子实体及其父坦克的移动状态
    let mut track_chain_moving = std::collections::HashMap::new();

    for (player_tank, children) in player_tanks.iter() {
        // 根据玩家类型检测键盘输入，判断是否在移动
        let is_moving = if player_tank.tank_type == TankType::Player1 {
            // 玩家1使用 WASD
            keyboard_input.pressed(KeyCode::KeyW)
                || keyboard_input.pressed(KeyCode::KeyS)
                || keyboard_input.pressed(KeyCode::KeyA)
                || keyboard_input.pressed(KeyCode::KeyD)
        } else {
            // 玩家2使用方向键
            keyboard_input.pressed(KeyCode::ArrowUp)
                || keyboard_input.pressed(KeyCode::ArrowDown)
                || keyboard_input.pressed(KeyCode::ArrowLeft)
                || keyboard_input.pressed(KeyCode::ArrowRight)
        };

        for child in children {
            track_chain_moving.insert(child, is_moving);
        }
    }

    // 更新履带动画
    for (entity, mut timer, mut sprite, indices, mut current_frame) in track_chain_query.iter_mut() {
        // 检查父坦克是否在移动
        let is_moving = track_chain_moving.get(&entity).copied().unwrap_or(false);

        if is_moving {
            // 移动时播放动画
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
        } else {
            // 静止时停止动画，重置计时器
            timer.reset();
        }
    }
}
