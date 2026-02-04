//! 玩家坦克冲刺系统模块
//!
//! 处理玩家坦克的冲刺功能，包括冲刺输入、移动和碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::effects;

use crate::constants::*;
use crate::resources::{
    DashDamageTracker, DashTimer, DashTimers, PlayerInfo, PlayerStatChanged, StatType, EffectResources, SoundResources,
    InsufficientEnergyTracker,
};

/// 处理冲刺输入
pub fn handle_dash_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    mut dash_timers: ResMut<DashTimers>,
    mut player_info: ResMut<PlayerInfo>,
    mut energy_tracker: ResMut<InsufficientEnergyTracker>,
    font_resources: Res<crate::resources::FontResources>,
    language: Res<crate::resources::Language>,
    time: Res<Time>,
) {
    // 更新所有能量不足冷却计时器
    energy_tracker.tick_all(time.delta());

    for (entity, transform, player_tank) in &query {
        // 检查是否正在冲刺
        let is_dashing = dash_timers.timers.contains_key(&entity);

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
                let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
                let actual_angle = euler_angle + ANGLE_OFFSET_DEGREES.to_radians();
                let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

                // 开始冲刺
                let dash_timer = DashTimer::new(direction, DASH_DURATION);
                dash_timers.timers.insert(entity, dash_timer);

                // 添加冲刺标记
                commands.entity(entity).insert(IsDashing);
            } else {
                // 能量不足，显示提示
                energy_tracker.try_show_warning(
                    &mut commands,
                    player_tank.tank_type,
                    font_resources.cn.clone(),
                    font_resources.en.clone(),
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
        ),
        With<PlayerTank>,
    >,
    mut dash_timers: ResMut<DashTimers>,
    mut dash_damage_tracker: ResMut<DashDamageTracker>,
) {
    for (entity, mut character_controller, mut transform, is_dashing) in &mut player_query {
        if matches!(is_dashing, Some(IsDashing))
            && let Some(dash_timer) = dash_timers.timers.get_mut(&entity)
        {
            // 更新计时器
            dash_timer.timer.tick(time.delta());

            // 计算冲刺速度：距离 / 时间
            let dash_speed = DASH_DISTANCE / DASH_DURATION;

            // 设置移动
            let movement = dash_timer.direction * dash_speed * time.delta_secs();
            character_controller.translation = Some(movement);

            // 限制坦克在地图边界内
            transform.translation.x = transform.translation.x.clamp(
                MAP_LEFT_X + PLAYER_TANK_DISPLAY_WIDTH / 2.0,
                MAP_RIGHT_X - PLAYER_TANK_DISPLAY_WIDTH / 2.0,
            );
            transform.translation.y = transform.translation.y.clamp(
                MAP_BOTTOM_Y + PLAYER_TANK_DISPLAY_HEIGHT / 2.0,
                MAP_TOP_Y - PLAYER_TANK_DISPLAY_HEIGHT / 2.0,
            );

            // 检查是否完成
            if dash_timer.timer.just_finished() {
                // 移除冲刺标记和计时器
                commands.entity(entity).remove::<IsDashing>();
                dash_timers.timers.remove(&entity);

                // 清理扣血追踪
                dash_damage_tracker.has_taken_damage.remove(&entity);
            }
        }
    }
}

/// 处理冲刺碰撞
pub fn handle_dash_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<crate::bullet::EffectEvent>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_tanks: Query<(Entity, &PlayerTank, &Transform, Option<&IsDashing>)>,
    enemy_tanks: Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut dash_damage_tracker: ResMut<DashDamageTracker>,
    effect_resources: Res<EffectResources>,
    sound_resources: Res<SoundResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 提取碰撞信息
        let Some((player_entity, brick_entity, steel_entity, enemy_entity, player_tank, tank_position)) =
            extract_dash_collision_info(*e1, *e2, &player_tanks, &enemy_tanks, &bricks, &steels, &mut player_info, &mut commands, &mut effect_events, &mut texture_atlas_layouts, &effect_resources, &sound_resources)
        else { continue; };

        // 处理 brick 碰撞
        if let Some(b_entity) = brick_entity {
            handle_brick_collision(
                &mut commands,
                &mut effect_events,
                &mut texture_atlas_layouts,
                &bricks,
                &mut player_info,
                &player_tank,
                tank_position,
                player_entity,
                b_entity,
                &mut dash_damage_tracker,
                &effect_resources,
                &sound_resources,
            );
            continue;
        }

        // 处理 steel 碰撞（protection = 100% 时）
        if let Some(s_entity) = steel_entity {
            handle_steel_break(
                &mut commands,
                &mut effect_events,
                &steels,
                s_entity,
            );
            continue;
        }

        // 处理敌方坦克碰撞
        if let Some(e_entity) = enemy_entity {
            handle_dash_enemy_tank_collision(
                &mut commands,
                &mut effect_events,
                &mut texture_atlas_layouts,
                &enemy_tanks,
                &mut player_info,
                &mut stat_changed_events,
                &player_tank,
                tank_position,
                player_entity,
                e_entity,
                &mut dash_damage_tracker,
                &effect_resources,
                &sound_resources,
            );
        }
    }
}

/// 提取冲刺碰撞信息
fn extract_dash_collision_info(
    e1: Entity,
    e2: Entity,
    player_tanks: &Query<(Entity, &PlayerTank, &Transform, Option<&IsDashing>)>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
    player_info: &mut ResMut<PlayerInfo>,
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    effect_resources: &EffectResources,
    sound_resources: &SoundResources,
) -> Option<(Entity, Option<Entity>, Option<Entity>, Option<Entity>, PlayerTank, Vec3)> {
    // 尝试从 e1 获取玩家坦克
    if let Ok((player_entity, player_tank, tank_transform, is_dashing)) = player_tanks.get(e1) {
        if let Some((brick_entity, steel_entity, enemy_entity)) = handle_player_entity_collision(
            player_entity,
            player_tank,
            tank_transform.translation,
            is_dashing,
            e2,
            enemy_tanks,
            bricks,
            steels,
            player_info,
            commands,
            effect_events,
            texture_atlas_layouts,
            effect_resources,
            sound_resources,
        ) {
            return Some((player_entity, brick_entity, steel_entity, enemy_entity, *player_tank, tank_transform.translation));
        }
    }

    // 尝试从 e2 获取玩家坦克
    if let Ok((player_entity, player_tank, tank_transform, is_dashing)) = player_tanks.get(e2) {
        if let Some((brick_entity, steel_entity, enemy_entity)) = handle_player_entity_collision(
            player_entity,
            player_tank,
            tank_transform.translation,
            is_dashing,
            e1,
            enemy_tanks,
            bricks,
            steels,
            player_info,
            commands,
            effect_events,
            texture_atlas_layouts,
            effect_resources,
            sound_resources,
        ) {
            return Some((player_entity, brick_entity, steel_entity, enemy_entity, *player_tank, tank_transform.translation));
        }
    }

    // 检查是否是玩家坦克与敌方坦克的碰撞（玩家不在 e1 或 e2 中）
    if let Some((player_entity, enemy_entity)) = check_enemy_collision_none(e1, e2, player_tanks, enemy_tanks) {
        if let Ok((_, player_tank, tank_transform, _)) = player_tanks.get(player_entity) {
            return Some((player_entity, None, None, Some(enemy_entity), *player_tank, tank_transform.translation));
        }
    }
    None
}

/// 处理玩家实体碰撞
fn handle_player_entity_collision(
    player_entity: Entity,
    player_tank: &PlayerTank,
    tank_position: Vec3,
    is_dashing: Option<&IsDashing>,
    other_entity: Entity,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
    player_info: &ResMut<PlayerInfo>,
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    effect_resources: &EffectResources,
    sound_resources: &SoundResources,
) -> Option<(Option<Entity>, Option<Entity>, Option<Entity>)> {
    // 卫语句：不在冲刺状态则跳过
    let Some(_) = is_dashing else { return None };

    // 处理 steel 碰撞
    if steels.get(other_entity).is_ok() {
        let can_break_steel = player_info.get_stat_value(player_tank.tank_type, |p| p.protection) >= 100;

        if can_break_steel {
            return Some((None, Some(other_entity), None));
        }

        // protection < 100%，玩家死亡
        handle_steel_collision(
            commands,
            effect_events,
            texture_atlas_layouts,
            player_info,
            player_tank,
            tank_position,
            player_entity,
            effect_resources,
            sound_resources,
        );
        return None;
    }

    // 处理 brick 碰撞
    if bricks.get(other_entity).is_ok() {
        return Some((Some(other_entity), None, None));
    }

    // 处理敌方坦克碰撞
    if enemy_tanks.get(other_entity).is_ok() {
        return Some((None, None, Some(other_entity)));
    }

    None
}

/// 检查敌方坦克碰撞（无玩家实体）
fn check_enemy_collision_none(
    e1: Entity,
    e2: Entity,
    player_tanks: &Query<(Entity, &PlayerTank, &Transform, Option<&IsDashing>)>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
) -> Option<(Entity, Entity)> {
    if let Ok((player_entity, _, _, is_dashing)) = player_tanks.get(e1) {
        if is_dashing.is_some() && enemy_tanks.get(e2).is_ok() {
            return Some((player_entity, e2));
        }
    } else if let Ok((player_entity, _, _, is_dashing)) = player_tanks.get(e2)
        && is_dashing.is_some()
        && enemy_tanks.get(e1).is_ok()
    {
        return Some((player_entity, e1));
    }
    None
}

/// 销毁玩家坦克
fn kill_player_tank(
    commands: &mut Commands,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    tank_transform: &Transform,
    player_entity: Entity,
    effect_resources: &EffectResources,
    sound_resources: &SoundResources,
) {
    // 生成爆炸效果
    effects::spawn_explosion(
        commands,
        texture_atlas_layouts,
        effect_resources,
        sound_resources,
        tank_transform.translation,
    );

    // 销毁玩家坦克
    let () = commands.entity(player_entity).try_despawn();
}

/// 处理砖块碰撞
fn handle_brick_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    player_info: &mut ResMut<PlayerInfo>,
    player_tank: &PlayerTank,
    tank_position: Vec3,
    player_entity: Entity,
    brick_entity: Entity,
    dash_damage_tracker: &mut DashDamageTracker,
    effect_resources: &EffectResources,
    sound_resources: &SoundResources,
) {
    // 获取 brick 位置用于生成效果
    if let Ok((_, brick_transform)) = bricks.get(brick_entity) {
        // 播放砖块被击中的音效
        sound_resources.play(commands, sound_resources.brick_hit.clone(), 0.5);

        // 发送火花特效事件
        effect_events.write(crate::bullet::EffectEvent::Spark {
            position: brick_transform.translation,
        });

        // 销毁 brick
        let () = commands.entity(brick_entity).try_despawn();
    }

    // 检查本次 dash 是否已经扣过血
    if dash_damage_tracker
        .has_taken_damage
        .contains(&player_entity)
    {
        return; // 已经扣过血，不再重复扣血
    }

    // 根据 protection 百分比决定扣血量
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
        dash_damage_tracker.has_taken_damage.insert(player_entity);
    }

    // 检查玩家是否死亡
    if is_dead {
        let transform = Transform::from_translation(tank_position);
        kill_player_tank(
            commands,
            texture_atlas_layouts,
            &transform,
            player_entity,
            effect_resources,
            sound_resources,
        );
    }
}

/// 处理钢铁碰撞
#[allow(clippy::needless_pass_by_ref_mut)]
fn handle_steel_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &ResMut<PlayerInfo>,
    player_tank: &PlayerTank,
    tank_position: Vec3,
    player_entity: Entity,
    effect_resources: &EffectResources,
    sound_resources: &SoundResources,
) {
    // 检查 protection 是否为 100%
    let can_break_steel = player_info.get_stat_value(player_tank.tank_type, |p| p.protection) >= 100;

    if can_break_steel {
        // protection = 100%，可以撞碎铁块，不扣血
        // 发送火花特效事件
        effect_events.write(crate::bullet::EffectEvent::Spark {
            position: tank_position,
        });
        // 铁块被撞碎的效果（可以在这里添加更多效果）
    } else {
        // protection < 100%，玩家死亡
        // 发送爆炸特效事件
        effect_events.write(crate::bullet::EffectEvent::Explosion {
            position: tank_position,
        });

        let transform = Transform::from_translation(tank_position);
        kill_player_tank(
            commands,
            texture_atlas_layouts,
            &transform,
            player_entity,
            effect_resources,
            sound_resources,
        );
    }
}

/// 处理钢铁破碎
fn handle_steel_break(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
    steel_entity: Entity,
) {
    // 获取 steel 位置用于生成效果
    if let Ok((_, steel_transform)) = steels.get(steel_entity) {
        // 发送火花特效事件
        effect_events.write(crate::bullet::EffectEvent::Spark {
            position: steel_transform.translation,
        });

        // 销毁 steel
        let () = commands.entity(steel_entity).try_despawn();
    }
}

/// 处理冲刺与敌方坦克碰撞
fn handle_dash_enemy_tank_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    player_info: &mut ResMut<PlayerInfo>,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    player_tank: &PlayerTank,
    tank_position: Vec3,
    player_entity: Entity,
    enemy_entity: Entity,
    dash_damage_tracker: &mut DashDamageTracker,
    effect_resources: &EffectResources,
    sound_resources: &SoundResources,
) {
    // 获取敌方坦克位置用于生成爆炸效果
    if let Ok((_, enemy_transform)) = enemy_tanks.get(enemy_entity) {
        // 发送爆炸特效事件
        effect_events.write(crate::bullet::EffectEvent::Explosion {
            position: enemy_transform.translation,
        });
    }

    // 销毁敌方坦克
    let () = commands.entity(enemy_entity).try_despawn();

    // 检查本次 dash 是否已经扣过血
    if dash_damage_tracker
        .has_taken_damage
        .contains(&player_entity)
    {
        return; // 已经扣过血，不再重复扣血
    }

    // 增加分数和根据 protection 百分比决定扣血量
    let tank_type = player_tank.tank_type;
    let mut health_cost = 0;
    let mut is_dead = false;

    player_info.with_stats_mut(tank_type, |player_stats| {
        player_stats.score += 100;
        health_cost = if player_stats.protection < 40 {
            DASH_DAMAGE_COST_HIGH // 2/3血条
        } else {
            usize::from(player_stats.protection < 80) // 1/3血条 或 不扣血
        };
        player_stats.life_points = player_stats.life_points.saturating_sub(health_cost);
        is_dead = player_stats.life_points == 0;
    });

    // 发送分数变更事件
    stat_changed_events.write(PlayerStatChanged {
        player_type: tank_type,
        stat_type: StatType::Score,
    });

    // 标记本次 dash 已经扣过血
    if health_cost > 0 {
        dash_damage_tracker.has_taken_damage.insert(player_entity);
    }

    // 检查玩家是否死亡
    if is_dead {
        let transform = Transform::from_translation(tank_position);
        kill_player_tank(
            commands,
            texture_atlas_layouts,
            &transform,
            player_entity,
            effect_resources,
            sound_resources,
        );
    }
}