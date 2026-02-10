//! 玩家坦克冲刺系统模块
//!
//! 处理玩家坦克的冲刺功能，包括冲刺输入、移动和碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::effects;
use crate::utils;

use crate::constants::*;
use crate::resources::{
    DashTimer, GameAtlasLayoutResources, GameAudioResources, GameTextureResources, GameTrackers,
    PlayerInfo, PlayerStatChanged, StatType,
};

/// 处理冲刺输入
pub fn handle_dash_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    mut game_trackers: ResMut<GameTrackers>,
    mut player_info: ResMut<PlayerInfo>,
    font_resources: Res<crate::resources::GameTextureResources>,
    audio_resources: Res<GameAudioResources>,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    language: Res<crate::resources::Language>,
    time: Res<Time>,
) {
    // 更新所有能量不足冷却计时器
    game_trackers
        .insufficient_energy_tracker
        .tick_all(time.delta());

    for (entity, transform, player_tank) in &query {
        // 检查是否正在冲刺
        let is_dashing = game_trackers.dash_timers.timers.contains_key(&entity);

        // 根据玩家类型选择按键绑定
        let key_bindings = player_tank.tank_type.get_key_bindings();

        let is_dash_key_pressed = keyboard_input.just_pressed(key_bindings.dash);

        if is_dash_key_pressed && !is_dashing {
            // 检查蓝条是否足够（需要至少1点蓝条）
            let tank_type = player_tank.tank_type;
            let energy_cost = 1; // 1点蓝条（1/3蓝条）
            let mut has_enough_energy = false;

            player_info.with_stats_mut(tank_type, |player_stats| {
                if player_stats.energy_points >= energy_cost {
                    player_stats.energy_points -= energy_cost;
                    has_enough_energy = true;
                }
            });

            if has_enough_energy {
                // 计算坦克当前朝向
                let direction =
                    crate::utils::calculate_direction_from_rotation(&transform.rotation);

                // 播放冲刺音效
                utils::play_one_shot_sound(
                    &mut commands,
                    audio_resources.dash.clone(),
                    VOLUME_HALF * 2.0,
                );

                // 添加冲刺尘土特效（与坦克纹理有90度相位差，放大2倍）
                let dust_atlas = atlas_layouts.dash_dust_effect.clone();
                let animation_indices = crate::atlas::DASH_DUST_ATLAS.animation_indices_full();

                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        DashDustEffect,
                        AnimationMode::Looping,
                        Sprite::from_atlas_image(
                            texture_resources.dash_dust_effect.clone(),
                            TextureAtlas {
                                layout: dust_atlas,
                                index: animation_indices.first,
                            },
                        ),
                        Transform {
                            translation: Vec3::new(0.0, -50.0, -0.1), // 位于坦克后面50像素
                            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), // 旋转90度
                            scale: Vec3::splat(2.0), // 放大2倍
                        },
                        animation_indices,
                        AnimationTimer(Timer::from_seconds(
                            DASH_DUST_ANIMATION_FRAME,
                            TimerMode::Repeating,
                        )),
                        CurrentAnimationFrame(0),
                    ));
                });

                // 开始冲刺
                let dash_timer = DashTimer::new(direction, DASH_DURATION);
                game_trackers.dash_timers.timers.insert(entity, dash_timer);

                // 添加冲刺标记
                commands.entity(entity).insert(IsDashing);
            } else {
                // 能量不足，显示提示
                game_trackers.insufficient_energy_tracker.try_show_warning(
                    &mut commands,
                    player_tank.tank_type,
                    &font_resources,
                    *language,
                );
            }
        }
    }
}

/// 更新冲刺移动
pub fn update_dash_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<
        (
            Entity,
            &mut KinematicCharacterController,
            &mut Transform,
            Option<&IsDashing>,
            Option<&Children>,
        ),
        With<PlayerTank>,
    >,
    dash_dust_query: Query<(), With<DashDustEffect>>,
    mut game_trackers: ResMut<GameTrackers>,
) {
    for (entity, mut character_controller, mut transform, is_dashing, children) in &mut player_query {
        if matches!(is_dashing, Some(IsDashing))
            && let Some(dash_timer) = game_trackers.dash_timers.timers.get_mut(&entity)
        {
            // 更新计时器
            dash_timer.timer.tick(time.delta());

            // 计算冲刺速度：距离 / 时间
            let dash_speed = DASH_DISTANCE / DASH_DURATION;

            // 设置移动
            let movement = dash_timer.direction * dash_speed * time.delta_secs();
            character_controller.translation = Some(movement);

            // 限制坦克在地图边界内
            utils::clamp_entity_position(
                &mut transform,
                TANK_DISPLAY_SIZE.x / 2.0,
                TANK_DISPLAY_SIZE.y / 2.0,
            );

            // 检查是否完成
            if dash_timer.timer.just_finished() {
                // 移除冲刺标记、计时器和尘土特效
                commands.entity(entity).remove::<IsDashing>();
                game_trackers.dash_timers.timers.remove(&entity);

                // 移除尘土特效子实体
                if let Some(children_ref) = children {
                    for child in children_ref.iter() {
                        if dash_dust_query.get(child).is_ok() {
                            commands.entity(child).despawn();
                        }
                    }
                }

                // 清理扣血追踪
                game_trackers
                    .dash_damage_tracker
                    .has_taken_damage
                    .remove(&entity);
            }
        }
    }
}

/// 处理冲刺碰撞
pub fn handle_dash_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<crate::bullet::EffectEvent>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    player_tanks: Query<(Entity, &PlayerTank, &Transform, Option<&IsDashing>)>,
    enemy_tanks: Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut game_trackers: ResMut<GameTrackers>,
    texture_resources: Res<GameTextureResources>,
    audio_resources: Res<GameAudioResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else {
            continue;
        };

        // 提取碰撞信息（一次性查询所有需要的实体和位置）
        let Some(collision_info) =
            extract_dash_collision_info(*e1, *e2, &player_tanks, &enemy_tanks, &bricks, &steels)
        else {
            continue;
        };

        // 卫语句：不在冲刺状态则跳过
        if !collision_info.is_dashing {
            continue;
        }

        match collision_info.target {
            DashTarget::Brick { entity, position } => {
                // 发送火花特效事件
                effect_events.write(crate::bullet::EffectEvent::Spark {
                    position,
                    audio_handle: audio_resources.brick_hit.clone(),
                    volume: 0.5,
                });

                // 销毁 brick
                let () = commands.entity(entity).try_despawn();

                // 应用伤害并检查死亡
                let (_, is_dead) = apply_dash_damage(
                    &mut player_info,
                    &collision_info.player_tank,
                    &mut game_trackers,
                    collision_info.player_entity,
                );

                if is_dead {
                    let transform = Transform::from_translation(collision_info.tank_position);
                    kill_player_tank(
                        &mut commands,
                        &atlas_layouts,
                        &transform,
                        collision_info.player_entity,
                        &texture_resources,
                        &audio_resources,
                    );
                }
            }

            DashTarget::Steel { entity, position } => {
                // 检查 protection 是否为 100%
                let can_break_steel = player_info
                    .get_stat_value(collision_info.player_tank.tank_type, |p| p.protection)
                    >= 100;

                if can_break_steel {
                    // protection = 100%，可以撞碎铁块，不扣血
                    effect_events.write(crate::bullet::EffectEvent::Spark {
                        position,
                        audio_handle: audio_resources.metal_crash.clone(),
                        volume: 1.0,
                    });
                    let () = commands.entity(entity).try_despawn();
                } else {
                    // protection < 100%，玩家死亡
                    effect_events.write(crate::bullet::EffectEvent::Explosion {
                        position: collision_info.tank_position,
                    });

                    let transform = Transform::from_translation(collision_info.tank_position);
                    // 直接杀死玩家（将生命值设为0并销毁）
                    player_info.with_stats_mut(
                        collision_info.player_tank.tank_type,
                        |player_stats| {
                            player_stats.life_points = 0;
                        },
                    );
                    kill_player_tank(
                        &mut commands,
                        &atlas_layouts,
                        &transform,
                        collision_info.player_entity,
                        &texture_resources,
                        &audio_resources,
                    );
                }
            }

            DashTarget::Enemy { entity, position } => {
                // 发送爆炸特效事件
                effect_events.write(crate::bullet::EffectEvent::Explosion { position });

                // 销毁敌方坦克
                let () = commands.entity(entity).try_despawn();

                // 增加分数
                let tank_type = collision_info.player_tank.tank_type;
                player_info.with_stats_mut(tank_type, |player_stats| {
                    player_stats.score += 100;
                });

                // 发送分数变更事件
                stat_changed_events.write(PlayerStatChanged {
                    player_type: tank_type,
                    stat_type: StatType::Score,
                });

                // 应用伤害并检查死亡
                let (_, is_dead) = apply_dash_damage(
                    &mut player_info,
                    &collision_info.player_tank,
                    &mut game_trackers,
                    collision_info.player_entity,
                );

                if is_dead {
                    let transform = Transform::from_translation(collision_info.tank_position);
                    kill_player_tank(
                        &mut commands,
                        &atlas_layouts,
                        &transform,
                        collision_info.player_entity,
                        &texture_resources,
                        &audio_resources,
                    );
                }
            }
        }
    }
}

/// 冲刺碰撞信息结构
struct DashCollisionInfo {
    player_entity: Entity,
    player_tank: PlayerTank,
    tank_position: Vec3,
    is_dashing: bool,
    target: DashTarget,
}

/// 碰撞目标类型
enum DashTarget {
    Brick { entity: Entity, position: Vec3 },
    Steel { entity: Entity, position: Vec3 },
    Enemy { entity: Entity, position: Vec3 },
}

/// 提取冲刺碰撞信息
fn extract_dash_collision_info(
    e1: Entity,
    e2: Entity,
    player_tanks: &Query<(Entity, &PlayerTank, &Transform, Option<&IsDashing>)>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
) -> Option<DashCollisionInfo> {
    if let Some((player_entity, other_entity)) =
        crate::utils::extract_collision_pair(e1, e2, player_tanks)
        && let Ok((_, player_tank, tank_transform, is_dashing)) = player_tanks.get(player_entity)
        && let Some(target) = get_collision_target(other_entity, enemy_tanks, bricks, steels)
    {
        return Some(DashCollisionInfo {
            player_entity,
            player_tank: *player_tank,
            tank_position: tank_transform.translation,
            is_dashing: is_dashing.is_some(),
            target,
        });
    }
    None
}

/// 获取碰撞目标（同时获取位置信息，避免后续重复查询）
fn get_collision_target(
    target_entity: Entity,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
) -> Option<DashTarget> {
    // 检查是否是 brick
    if let Ok((_, transform)) = bricks.get(target_entity) {
        return Some(DashTarget::Brick {
            entity: target_entity,
            position: transform.translation,
        });
    }

    // 检查是否是 steel
    if let Ok((_, transform)) = steels.get(target_entity) {
        return Some(DashTarget::Steel {
            entity: target_entity,
            position: transform.translation,
        });
    }

    // 检查是否是敌方坦克
    if let Ok((_, transform)) = enemy_tanks.get(target_entity) {
        return Some(DashTarget::Enemy {
            entity: target_entity,
            position: transform.translation,
        });
    }

    None
}

/// 应用冲刺伤害（统一的扣血逻辑）
/// 返回 (扣血量, 是否死亡)
fn apply_dash_damage(
    player_info: &mut ResMut<PlayerInfo>,
    player_tank: &PlayerTank,
    game_trackers: &mut GameTrackers,
    player_entity: Entity,
) -> (usize, bool) {
    // 检查本次 dash 是否已经扣过血
    if game_trackers
        .dash_damage_tracker
        .has_taken_damage
        .contains(&player_entity)
    {
        return (0, false);
    }

    let tank_type = player_tank.tank_type;
    let mut health_cost = 0;
    let mut is_dead = false;

    player_info.with_stats_mut(tank_type, |player_stats| {
        health_cost = if player_stats.protection < 40 {
            2 // 2/3血条
        } else {
            usize::from(player_stats.protection < 80) // 1/3血条 或 不扣血
        };

        player_stats.life_points = player_stats.life_points.saturating_sub(health_cost);
        is_dead = player_stats.life_points == 0;
    });

    // 标记本次 dash 已经扣过血
    if health_cost > 0 {
        game_trackers
            .dash_damage_tracker
            .has_taken_damage
            .insert(player_entity);
    }

    (health_cost, is_dead)
}

/// 销毁玩家坦克（不减少生命值，只销毁实体）
fn kill_player_tank(
    commands: &mut Commands,
    atlas_layouts: &GameAtlasLayoutResources,
    tank_transform: &Transform,
    player_entity: Entity,
    texture_resources: &GameTextureResources,
    audio_resources: &GameAudioResources,
) {
    // 生成爆炸效果
    effects::spawn_explosion(
        commands,
        texture_resources,
        atlas_layouts,
        audio_resources,
        tank_transform.translation,
    );

    // 销毁玩家坦克
    let () = commands.entity(player_entity).try_despawn();
}
