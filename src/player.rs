//! 玩家坦克系统模块
//!
//! 处理玩家坦克的生成、移动、回城、冲刺和碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::effects;

use crate::constants::*;
use crate::powerup;
use crate::resources::{
    BarrierDamageTracker, BlueBarRegenTimer, DashDamageTracker, DashTimer, DashTimers,
    GameMode, PlayerInfo, PlayerStatChanged, PlayerStats, RecallTimer, RecallTimers,
    StatType,
};

/// 生成玩家坦克
pub fn spawn_player_tank(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
    tank_type: TankType,
) -> Entity {
    let (x_pos, custom_size, collider_half) = match tank_type {
        TankType::Player1 => (
            -TANK_WIDTH / 2.0 - COMMANDER_WIDTH / 2.0 - PLAYER_SPAWN_OFFSET,
            Vec2::new(PLAYER_TANK_DISPLAY_WIDTH, PLAYER_TANK_DISPLAY_HEIGHT),
            PLAYER_COLLIDER_HALF,
        ),
        TankType::Player2 => (
            TANK_WIDTH / 2.0 + COMMANDER_WIDTH / 2.0 + 50.0,
            Vec2::new(80.0, 90.0),
            TANK_WIDTH / 2.0,
        ),
        TankType::Enemy => unreachable!("敌方坦克不应该使用此函数"),
    };

    commands
        .spawn_empty()
        .insert(PlayerTank { tank_type })
        .insert(PlayingEntity)
        .insert(TankFireConfig::default())
        .insert(RotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))
        .insert(TargetRotation {
            angle: 0.0_f32.to_radians(),
        })
        .insert(Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            }),
            custom_size: Some(custom_size),
            ..default()
        })
        .insert(Transform::from_xyz(
            x_pos,
            MAP_BOTTOM_Y + TANK_HEIGHT / 2.0,
            0.0,
        ))
        .insert(Velocity {
            linvel: Vec2::default(),
            angvel: 0.0,
        })
        .insert(animation_indices)
        .insert(AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_ENEMY_MOVE,
            TimerMode::Repeating,
        )))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(collider_half, collider_half))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(
            ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_STATIC
                | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
        )
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(KinematicCharacterController {
            offset: CharacterLength::Absolute(CHARACTER_CONTROLLER_OFFSET),
            filter_groups: None,
            autostep: Some(bevy_rapier2d::prelude::CharacterAutostep {
                max_height: CharacterLength::Absolute(CHARACTER_CONTROLLER_MAX_HEIGHT),
                min_width: CharacterLength::Absolute(CHARACTER_CONTROLLER_MIN_WIDTH),
                include_dynamic_bodies: false,
            }),
            ..default()
        })
        .id()
}

/// 生成玩家1坦克
pub fn move_player_tank(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_info: Res<PlayerInfo>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut KinematicCharacterController,
            &mut RotationTimer,
            &mut TargetRotation,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &PlayerTank,
            Option<&IsDashing>,
        ),
        With<PlayerTank>,
    >,
) {
    for (
        _entity,
        mut transform,
        mut character_controller,
        mut rotation_timer,
        mut target_rotation,
        mut animation_timer,
        mut sprite,
        animation_indices,
        player_tank,
        is_dashing,
    ) in &mut query
    {
        // 如果正在冲刺，跳过移动处理
        if is_dashing.is_some() {
            continue;
        }
        // 根据玩家索引选择不同的控制键
        let direction = if player_tank.tank_type == TankType::Player1 {
            // 玩家1使用 WASD
            let w_pressed = keyboard_input.pressed(KeyCode::KeyW);
            let s_pressed = keyboard_input.pressed(KeyCode::KeyS);
            let a_pressed = keyboard_input.pressed(KeyCode::KeyA);
            let d_pressed = keyboard_input.pressed(KeyCode::KeyD);
            match (w_pressed, s_pressed, a_pressed, d_pressed) {
                (true, false, false, false) => Vec2::new(0.0, 1.0), // 上
                (false, true, false, false) => Vec2::new(0.0, -1.0), // 下
                (false, false, true, false) => Vec2::new(-1.0, 0.0), // 左
                (false, false, false, true) => Vec2::new(1.0, 0.0), // 右
                _ => Vec2::ZERO, // 其他情况（包括多个键同时按下）停止移动
            }
        } else {
            // 玩家2使用方向键
            let up_pressed = keyboard_input.pressed(KeyCode::ArrowUp);
            let down_pressed = keyboard_input.pressed(KeyCode::ArrowDown);
            let left_pressed = keyboard_input.pressed(KeyCode::ArrowLeft);
            let right_pressed = keyboard_input.pressed(KeyCode::ArrowRight);
            match (up_pressed, down_pressed, left_pressed, right_pressed) {
                (true, false, false, false) => Vec2::new(0.0, 1.0), // 上
                (false, true, false, false) => Vec2::new(0.0, -1.0), // 下
                (false, false, true, false) => Vec2::new(-1.0, 0.0), // 左
                (false, false, false, true) => Vec2::new(1.0, 0.0), // 右
                _ => Vec2::ZERO, // 其他情况（包括多个键同时按下）停止移动
            }
        };

        // 检查是否需要转向
        let needs_rotation = if direction.length() > 0.0 {
            let angle = direction.y.atan2(direction.x);
            let target_angle = angle - ANGLE_OFFSET_DEGREES.to_radians();

            let current_euler = target_rotation.angle;
            let angle_diff = std::f32::consts::PI.mul_add(3.0, target_angle - current_euler)
                % (std::f32::consts::PI * 2.0)
                - std::f32::consts::PI;

            if angle_diff.abs() > ANGLE_DIFF_THRESHOLD {
                target_rotation.angle = target_angle;
                // 只在角度变化较大时才重置计时器，避免频繁重置
                if angle_diff.abs() > ANGLE_DIFF_RESET_THRESHOLD {
                    rotation_timer.reset();
                }
                true
            } else {
                false
            }
        } else {
            character_controller.translation = None;
            false
        };

        // 使用 KinematicCharacterController 的 translation 字段控制移动
        // 获取玩家的 speed 百分比加成
        let speed_bonus = match player_tank.tank_type {
            TankType::Player1 => player_info.player1.speed as f32 / 100.0,
            TankType::Player2 => player_info
                .player2
                .as_ref()
                .map_or(0.0, |p| p.speed as f32 / 100.0),
            TankType::Enemy => 0.0,
        };
        // 实际速度 = 基础速度 × (1 + speed百分比/100)
        // 转向时保持 50% 速度，减少卡顿感
        let base_speed = PLAYER_TANK_SPEED * (1.0 + speed_bonus);
        let speed = if needs_rotation {
            base_speed * ROTATION_SPEED_FACTOR
        } else {
            base_speed
        };

        let is_moving = direction.length() > 0.0;
        if is_moving {
            character_controller.translation = Some(direction * speed * time.delta_secs());
        } else {
            character_controller.translation = None;
        }

        // 处理纹理动画
        if let Some(atlas) = &mut sprite.texture_atlas {
            if is_moving {
                animation_timer.tick(time.delta());
                if animation_timer.just_finished() {
                    atlas.index = if atlas.index == animation_indices.last {
                        animation_indices.first
                    } else {
                        atlas.index + 1
                    }
                }
            } else {
                atlas.index = animation_indices.last;
                animation_timer.reset();
            }
        }

        // 只在需要旋转时才更新旋转计时器和计算旋转
        if needs_rotation || !rotation_timer.is_finished() {
            rotation_timer.tick(time.delta());

            // 平滑旋转
            let current_euler = transform.rotation.to_euler(EulerRot::XYZ).2;
            let target_angle = target_rotation.angle;
            let angle_diff = std::f32::consts::PI.mul_add(3.0, target_angle - current_euler)
                % (std::f32::consts::PI * 2.0)
                - std::f32::consts::PI;

            if angle_diff.abs() > 0.01 && !rotation_timer.is_finished() {
                // 计算旋转进度（0.0 到 1.0）
                let progress =
                    rotation_timer.elapsed_secs() / rotation_timer.duration().as_secs_f32();
                // 使用缓动函数使旋转更平滑
                let eased_progress = progress * progress * 2.0f32.mul_add(-progress, 3.0);
                // 插值计算当前角度
                let new_angle = current_euler + angle_diff * eased_progress;
                transform.rotation = Quat::from_rotation_z(new_angle);
            } else if angle_diff.abs() > 0.01 {
                // 旋转完成，直接设置为目标角度
                transform.rotation = Quat::from_rotation_z(target_angle);
            }
        }

        // 限制坦克在地图边界内
        transform.translation.x = transform.translation.x.clamp(
            MAP_LEFT_X + TANK_WIDTH / 2.0,
            MAP_RIGHT_X - TANK_WIDTH / 2.0,
        );
        transform.translation.y = transform.translation.y.clamp(
            MAP_BOTTOM_Y + TANK_HEIGHT / 2.0,
            MAP_TOP_Y - TANK_HEIGHT / 2.0,
        );
    }
}

/// 处理回城输入
pub fn handle_recall_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    mut recall_timers: ResMut<RecallTimers>,
) {
    for (entity, transform, player_tank) in &query {
        // 检查是否正在回城
        let is_recalling = recall_timers.timers.contains_key(&entity);

        // 根据玩家索引选择不同的回城键
        let is_recall_key_pressed = if player_tank.tank_type == TankType::Player1 {
            // 玩家1使用 I 键回城
            keyboard_input.pressed(KeyCode::KeyI)
        } else {
            // 玩家2使用小键盘4键回城
            keyboard_input.pressed(KeyCode::Numpad4)
        };

        if is_recall_key_pressed && !is_recalling {
            // 计算初始位置
            let initial_position = if player_tank.tank_type == TankType::Player1 {
                Vec3::new(
                    -TANK_WIDTH / 2.0 - COMMANDER_WIDTH / 2.0 - PLAYER_SPAWN_OFFSET,
                    MAP_BOTTOM_Y + TANK_HEIGHT / 2.0,
                    0.0,
                )
            } else {
                Vec3::new(
                    TANK_WIDTH / 2.0 + COMMANDER_WIDTH / 2.0 + PLAYER_SPAWN_OFFSET,
                    MAP_BOTTOM_Y + TANK_HEIGHT / 2.0,
                    0.0,
                )
            };

            // 开始回城
            let recall_timer = RecallTimer::new(initial_position, RECALL_TIME);
            recall_timers.timers.insert(entity, recall_timer);

            // 添加回城标记
            commands.entity(entity).insert(IsRecalling);

            // 创建回城进度条（在坦克正上方，初始满格）
            commands.spawn((
                PlayingEntity,
                RecallProgressBar {
                    player_entity: entity,
                },
                Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0), // 绿色
                    custom_size: Some(Vec2::new(PROGRESS_BAR_INITIAL_WIDTH, PROGRESS_BAR_HEIGHT)), // 初始宽度100（满格）
                    ..default()
                },
                Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y + TANK_HEIGHT / 2.0 + PROGRESS_BAR_Y_OFFSET,
                    Z_PROGRESS_BAR,
                ), // 在坦克上方
            ));
        }
    }
}

/// 更新回城计时器
pub fn update_recall_timers(
    time: Res<Time>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<
        (
            Entity,
            &mut Transform,
            &PlayerTank,
            Option<&IsRecalling>,
            Option<&Children>,
        ),
        With<PlayerTank>,
    >,
    mut recall_timers: ResMut<RecallTimers>,
    mut progress_bar_query: Query<(Entity, &mut Sprite, &RecallProgressBar)>,
) {
    for (entity, mut transform, player_tank, is_recalling, children) in &mut player_query {
        // 卫语句：不在回城状态则跳过
        let Some(IsRecalling) = is_recalling else { continue; };
        
        let Some(recall_timer) = recall_timers.timers.get_mut(&entity) else { continue; };

        let is_recall_key_pressed = is_recall_key_pressed(&keyboard_input, player_tank.tank_type);
        let is_interrupted = is_movement_interrupted(&keyboard_input, player_tank.tank_type);

        // 卫语句：取消回城
        if !is_recall_key_pressed || is_interrupted {
            cancel_recall(&mut commands, &mut progress_bar_query, entity, &mut recall_timers);
            continue;
        }

        // 更新计时器和进度条
        recall_timer.timer.tick(time.delta());
        update_progress_bar(&mut progress_bar_query, entity, recall_timer);

        // 卫语句：回城完成
        if recall_timer.timer.just_finished() {
            let start_position = recall_timer.start_position;
            complete_recall(&mut commands, &mut progress_bar_query, entity, &mut recall_timers,
                           &mut transform, children, start_position);
        }
    }
}

/// 检查是否按住回城键
fn is_recall_key_pressed(keyboard_input: &Res<ButtonInput<KeyCode>>, tank_type: TankType) -> bool {
    match tank_type {
        TankType::Player1 => keyboard_input.pressed(KeyCode::KeyI),
        TankType::Player2 => keyboard_input.pressed(KeyCode::Numpad4),
        TankType::Enemy => false,
    }
}

/// 检查是否被打断（移动或射击）
fn is_movement_interrupted(keyboard_input: &Res<ButtonInput<KeyCode>>, tank_type: TankType) -> bool {
    match tank_type {
        TankType::Player1 => {
            keyboard_input.pressed(KeyCode::KeyW)
                || keyboard_input.pressed(KeyCode::KeyS)
                || keyboard_input.pressed(KeyCode::KeyA)
                || keyboard_input.pressed(KeyCode::KeyD)
                || keyboard_input.pressed(KeyCode::KeyJ)
        }
        TankType::Player2 => {
            keyboard_input.pressed(KeyCode::ArrowUp)
                || keyboard_input.pressed(KeyCode::ArrowDown)
                || keyboard_input.pressed(KeyCode::ArrowLeft)
                || keyboard_input.pressed(KeyCode::ArrowRight)
                || keyboard_input.pressed(KeyCode::Numpad1)
        }
        TankType::Enemy => false,
    }
}

/// 取消回城
fn cancel_recall(
    commands: &mut Commands,
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &RecallProgressBar)>,
    entity: Entity,
    recall_timers: &mut ResMut<RecallTimers>,
) {
    commands.entity(entity).remove::<IsRecalling>();
    recall_timers.timers.remove(&entity);

    for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
        if progress_bar.player_entity == entity {
            let () = commands.entity(progress_entity).try_despawn();
        }
    }
}

/// 更新进度条
fn update_progress_bar(
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &RecallProgressBar)>,
    entity: Entity,
    recall_timer: &RecallTimer,
) {
    let progress = recall_timer.timer.elapsed_secs() / recall_timer.timer.duration().as_secs_f32();
    let bar_width = PROGRESS_BAR_INITIAL_WIDTH * (1.0 - progress);

    for (_, mut sprite, progress_bar) in progress_bar_query.iter_mut() {
        if progress_bar.player_entity == entity {
            sprite.custom_size = Some(Vec2::new(bar_width, PROGRESS_BAR_HEIGHT));
        }
    }
}

/// 完成回城
fn complete_recall(
    commands: &mut Commands,
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &RecallProgressBar)>,
    entity: Entity,
    recall_timers: &mut ResMut<RecallTimers>,
    transform: &mut Transform,
    children: Option<&Children>,
    initial_position: Vec3,
) {
    // 删除所有子实体
    if let Some(children) = children {
        for child in children.iter() {
            let () = commands.entity(child).try_despawn();
        }
    }

    transform.translation = initial_position;

    commands.entity(entity).remove::<IsRecalling>();
    recall_timers.timers.remove(&entity);

    for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
        if progress_bar.player_entity == entity {
            let () = commands.entity(progress_entity).try_despawn();
        }
    }
}

/// 更新回城进度条位置
pub fn update_recall_progress_bars(
    mut param_set: ParamSet<(
        Query<(Entity, &Transform)>,
        Query<(&RecallProgressBar, &mut Transform), Without<PlayerTank>>,
    )>,
) {
    let mut player_transforms: Vec<(Entity, Vec3)> = Vec::new();

    // 先收集所有玩家的位置信息
    for (entity, transform) in &param_set.p0() {
        player_transforms.push((entity, transform.translation));
    }

    // 然后更新进度条位置
    for (progress_bar, mut progress_transform) in &mut param_set.p1() {
        if let Some((_, player_pos)) = player_transforms
            .iter()
            .find(|(e, _)| *e == progress_bar.player_entity)
        {
            // 更新倒计时文本位置（跟随坦克）
            progress_transform.translation.x = player_pos.x;
            progress_transform.translation.y =
                player_pos.y + TANK_HEIGHT / 2.0 + PROGRESS_BAR_Y_OFFSET;
        }
    }
}

/// 处理冲刺输入
pub fn handle_dash_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    mut dash_timers: ResMut<DashTimers>,
    mut player_info: ResMut<PlayerInfo>,
) {
    for (entity, transform, player_tank) in &query {
        // 检查是否正在冲刺
        let is_dashing = dash_timers.timers.contains_key(&entity);

        // 根据玩家索引选择不同的冲刺键
        let is_dash_key_pressed = if player_tank.tank_type == TankType::Player1 {
            // 玩家1使用 K 键冲刺
            keyboard_input.just_pressed(KeyCode::KeyK)
        } else {
            // 玩家2使用小键盘2键冲刺
            keyboard_input.just_pressed(KeyCode::Numpad2)
        };

        if is_dash_key_pressed && !is_dashing {
            // 检查蓝条是否足够（需要至少1点蓝条）
            let player_stats = match player_tank.tank_type {
                TankType::Player1 => &mut player_info.player1,
                TankType::Player2 => player_info.player2.as_mut().expect("Player2 should exist"),
                TankType::Enemy => unreachable!(),
            };
            let energy_cost = 1; // 1点蓝条（1/3蓝条）
            if player_stats.energy_points >= energy_cost {
                // 立即扣除蓝条
                player_stats.energy_points -= energy_cost;

                // 计算坦克当前朝向
                let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
                let actual_angle = euler_angle + ANGLE_OFFSET_DEGREES.to_radians();
                let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

                // 开始冲刺
                let dash_timer = DashTimer::new(direction, DASH_DURATION);
                dash_timers.timers.insert(entity, dash_timer);

                // 添加冲刺标记
                commands.entity(entity).insert(IsDashing);
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
                MAP_LEFT_X + TANK_WIDTH / 2.0,
                MAP_RIGHT_X - TANK_WIDTH / 2.0,
            );
            transform.translation.y = transform.translation.y.clamp(
                MAP_BOTTOM_Y + TANK_HEIGHT / 2.0,
                MAP_TOP_Y - TANK_HEIGHT / 2.0,
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
    asset_server: Res<AssetServer>,
    player_tanks: Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: Query<(Entity, &Transform), With<PlayerTank>>,
    enemy_tanks: Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    mut player_info: ResMut<PlayerInfo>,
    player_avatars: Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut dash_damage_tracker: ResMut<DashDamageTracker>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 提取碰撞信息
        let Some((player_entity, brick_entity, steel_entity, enemy_entity)) =
            extract_dash_collision_info(*e1, *e2, &player_tanks, &enemy_tanks, &bricks, &steels, &player_info, &mut commands, &mut effect_events, &player_tanks_with_transform, &player_avatars)
        else { continue; };

        // 处理 brick 碰撞
        if let Some(b_entity) = brick_entity {
            handle_brick_collision(
                &mut commands,
                &mut effect_events,
                &asset_server,
                &mut texture_atlas_layouts,
                &player_tanks,
                &player_tanks_with_transform,
                &bricks,
                &mut player_info,
                &player_avatars,
                player_entity,
                b_entity,
                &mut dash_damage_tracker,
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
                &asset_server,
                &mut texture_atlas_layouts,
                &player_tanks,
                &player_tanks_with_transform,
                &enemy_tanks,
                &mut player_info,
                &player_avatars,
                &mut stat_changed_events,
                player_entity,
                e_entity,
                &mut dash_damage_tracker,
            );
        }
    }
}

/// 提取冲刺碰撞信息
fn extract_dash_collision_info(
    e1: Entity,
    e2: Entity,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
    player_info: &ResMut<PlayerInfo>,
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
) -> Option<(Entity, Option<Entity>, Option<Entity>, Option<Entity>)> {
    // 尝试从 e1 获取玩家坦克
    if let Ok((player_entity, player_tank, is_dashing)) = player_tanks.get(e1) {
        return handle_player_entity_collision(
            player_entity,
            player_tank,
            is_dashing,
            e2,
            enemy_tanks,
            bricks,
            steels,
            player_info,
            commands,
            effect_events,
            player_tanks,
            player_tanks_with_transform,
            player_avatars,
        );
    }

    // 尝试从 e2 获取玩家坦克
    if let Ok((player_entity, player_tank, is_dashing)) = player_tanks.get(e2) {
        return handle_player_entity_collision(
            player_entity,
            player_tank,
            is_dashing,
            e1,
            enemy_tanks,
            bricks,
            steels,
            player_info,
            commands,
            effect_events,
            player_tanks,
            player_tanks_with_transform,
            player_avatars,
        );
    }

    // 检查是否是玩家坦克与敌方坦克的碰撞（玩家不在 e1 或 e2 中）
    check_enemy_collision_none(e1, e2, player_tanks, enemy_tanks)
        .map(|(pe, ee)| (pe, None, None, Some(ee)))
}

/// 处理玩家实体碰撞
fn handle_player_entity_collision(
    player_entity: Entity,
    player_tank: &PlayerTank,
    is_dashing: Option<&IsDashing>,
    other_entity: Entity,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
    player_info: &ResMut<PlayerInfo>,
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
) -> Option<(Entity, Option<Entity>, Option<Entity>, Option<Entity>)> {
    // 卫语句：不在冲刺状态则跳过
    let Some(_) = is_dashing else { return None };

    // 处理 steel 碰撞
    if steels.get(other_entity).is_ok() {
        let can_break_steel = match player_tank.tank_type {
            TankType::Player1 => player_info.player1.protection >= 100,
            TankType::Player2 => player_info
                .player2
                .as_ref()
                .is_some_and(|p| p.protection >= 100),
            TankType::Enemy => false,
        };

        if can_break_steel {
            return Some((player_entity, None, Some(other_entity), None));
        }

        // protection < 100%，玩家死亡
        handle_steel_collision(
            commands,
            effect_events,
            player_tanks,
            player_tanks_with_transform,
            player_info,
            player_avatars,
            player_entity,
        );
        return None;
    }

    // 处理 brick 碰撞
    if bricks.get(other_entity).is_ok() {
        return Some((player_entity, Some(other_entity), None, None));
    }

    // 处理敌方坦克碰撞
    if let Some(enemy) = check_enemy_collision(player_entity, other_entity, player_tanks, enemy_tanks) {
        return Some((player_entity, None, None, Some(enemy)));
    }

    None
}

/// 检查敌方坦克碰撞
fn check_enemy_collision(
    e1: Entity,
    e2: Entity,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
) -> Option<Entity> {
    if let Ok((_, _, is_dashing)) = player_tanks.get(e1)
        && is_dashing.is_some()
        && enemy_tanks.get(e2).is_ok()
    {
        return Some(e2);
    }
    None
}

/// 检查敌方坦克碰撞（无玩家实体）
fn check_enemy_collision_none(
    e1: Entity,
    e2: Entity,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
) -> Option<(Entity, Entity)> {
    if let Ok((player_entity, _, is_dashing)) = player_tanks.get(e1) {
        if is_dashing.is_some() && enemy_tanks.get(e2).is_ok() {
            return Some((player_entity, e2));
        }
    } else if let Ok((player_entity, _, is_dashing)) = player_tanks.get(e2)
        && is_dashing.is_some()
        && enemy_tanks.get(e1).is_ok()
    {
        return Some((player_entity, e1));
    }
    None
}

/// 处理砖块碰撞
fn handle_brick_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    player_info: &mut ResMut<PlayerInfo>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    player_entity: Entity,
    brick_entity: Entity,
    dash_damage_tracker: &mut DashDamageTracker,
) {
    // 获取玩家坦克信息
    let player_tank = player_tanks
        .iter()
        .find_map(
            |(e, pt, _)| {
                if e == player_entity { Some(pt) } else { None }
            },
        )
        .unwrap();

    // 获取 brick 位置用于生成效果
    if let Ok((_, brick_transform)) = bricks.get(brick_entity) {
        // 播放砖块被击中的音效
        let brick_hit_sound: Handle<AudioSource> = asset_server.load(SOUND_BRICK_HIT);
        commands.spawn((
            AudioPlayer::new(brick_hit_sound),
            PlaybackSettings::ONCE.with_volume(Volume::Linear(0.5)),
        ));

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
    let player_stats = match player_tank.tank_type {
        TankType::Player1 => &mut player_info.player1,
        TankType::Player2 => player_info.player2.as_mut().expect("Player2 should exist"),
        TankType::Enemy => unreachable!(),
    };
    let health_cost = if player_stats.protection < 40 {
        2 // 2/3血条
    } else {
        usize::from(player_stats.protection < 80) // 1/3血条 或 不扣血
    };

    player_stats.life_points = player_stats.life_points.saturating_sub(health_cost);

    // 标记本次 dash 已经扣过血
    if health_cost > 0 {
        dash_damage_tracker.has_taken_damage.insert(player_entity);
    }

    // 检查玩家是否死亡
    if player_stats.life_points == 0 {
        // 获取玩家坦克位置用于生成爆炸效果
        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
            // 生成爆炸效果
            effects::spawn_explosion(
                commands,
                asset_server,
                texture_atlas_layouts,
                tank_transform.translation,
            );
        }

        // 销毁玩家坦克
        let () = commands.entity(player_entity).try_despawn();

        // 标记对应玩家的头像为死亡状态
        for (avatar_entity, player_index) in player_avatars.iter() {
            if player_index.player_type == player_tank.tank_type {
                commands.entity(avatar_entity).insert(PlayerDead);
            }
        }

        // 启动 Game Over 延迟计时器（1.2秒）
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(GAME_OVER_DELAY, TimerMode::Once)),
        ));
    }
}

/// 处理钢铁碰撞
#[allow(clippy::needless_pass_by_ref_mut)]
fn handle_steel_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    player_info: &ResMut<PlayerInfo>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    player_entity: Entity,
) {
    // 获取玩家坦克信息
    let player_tank = player_tanks
        .iter()
        .find_map(
            |(e, pt, _)| {
                if e == player_entity { Some(pt) } else { None }
            },
        )
        .unwrap();

    // 检查 protection 是否为 100%
    let can_break_steel = match player_tank.tank_type {
        TankType::Player1 => player_info.player1.protection >= 100,
        TankType::Player2 => player_info
            .player2
            .as_ref()
            .is_some_and(|p| p.protection >= 100),
        TankType::Enemy => false,
    };

    if can_break_steel {
        // protection = 100%，可以撞碎铁块，不扣血
        // 发送火花特效事件
        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
            effect_events.write(crate::bullet::EffectEvent::Spark {
                position: tank_transform.translation,
            });
        }
        // 铁块被撞碎的效果（可以在这里添加更多效果）
    } else {
        // protection < 100%，玩家死亡
        // 发送爆炸特效事件
        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
            effect_events.write(crate::bullet::EffectEvent::Explosion {
                position: tank_transform.translation,
            });
        }

        // 销毁玩家坦克
        let () = commands.entity(player_entity).try_despawn();

        // 标记对应玩家的头像为死亡状态
        for (avatar_entity, player_index) in player_avatars.iter() {
            if player_index.player_type == player_tank.tank_type {
                commands.entity(avatar_entity).insert(PlayerDead);
            }
        }

        // 启动 Game Over 延迟计时器（1.2秒）
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
        ));
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
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    player_info: &mut ResMut<PlayerInfo>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    player_entity: Entity,
    enemy_entity: Entity,
    dash_damage_tracker: &mut DashDamageTracker,
) {
    // 获取玩家坦克信息
    let (_, player_tank, _) = player_tanks.get(player_entity).unwrap();

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

    // 增加分数
    let player_stats = match player_tank.tank_type {
        TankType::Player1 => &mut player_info.player1,
        TankType::Player2 => player_info.player2.as_mut().expect("Player2 should exist"),
        TankType::Enemy => unreachable!(),
    };
    player_stats.score += 100;

    // 发送分数变更事件
    stat_changed_events.write(PlayerStatChanged {
        player_type: player_tank.tank_type,
        stat_type: StatType::Score,
    });

    // 根据 protection 百分比决定扣血量
    let health_cost = if player_stats.protection < 40 {
        DASH_DAMAGE_COST_HIGH // 2/3血条
    } else {
        usize::from(player_stats.protection < 80) // 1/3血条 或 不扣血
    };
    player_stats.life_points = player_stats.life_points.saturating_sub(health_cost);

    // 标记本次 dash 已经扣过血
    if health_cost > 0 {
        dash_damage_tracker.has_taken_damage.insert(player_entity);
    }

    // 检查玩家是否死亡
    if player_stats.life_points == 0 {
        // 获取玩家坦克位置用于生成爆炸效果
        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
            // 生成爆炸效果
            effects::spawn_explosion(
                commands,
                asset_server,
                texture_atlas_layouts,
                tank_transform.translation,
            );
        }

        // 销毁玩家坦克
        let () = commands.entity(player_entity).try_despawn();

        // 标记对应玩家的头像为死亡状态
        for (avatar_entity, player_index) in player_avatars.iter() {
            if player_index.player_type == player_tank.tank_type {
                commands.entity(avatar_entity).insert(PlayerDead);
            }
        }

        // 启动 Game Over 延迟计时器（1.2秒）
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(GAME_OVER_DELAY, TimerMode::Once)),
        ));
    }
}

/// 处理屏障碰撞
pub fn handle_barrier_collision(
    time: Res<Time>,
    player_tanks: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    barriers: Query<(&Transform, Entity), With<Barrier>>,
    mut player_info: ResMut<PlayerInfo>,
    mut barrier_damage_tracker: ResMut<BarrierDamageTracker>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
) {
    const COLLISION_THRESHOLD: f32 = BARRIER_WIDTH;

    // 更新所有冷却计时器
    for timer in barrier_damage_tracker.cooldowns.values_mut() {
        timer.tick(time.delta());
    }

    // 检测玩家坦克与 barrier 的距离
    for (player_entity, player_transform, player_tank) in player_tanks.iter() {
        for (barrier_transform, _barrier_entity) in barriers.iter() {
            // 计算距离
            let distance = (player_transform.translation - barrier_transform.translation).length();

            // 如果距离小于阈值，则认为碰撞

            if distance < COLLISION_THRESHOLD {
                // 检查冷却是否结束
                let can_take_damage = barrier_damage_tracker
                    .cooldowns
                    .get(&player_entity)
                    .is_none_or(bevy::prelude::Timer::is_finished);

                if can_take_damage {
                    // 检查玩家是否有 track_chain，如果有则免疫伤害
                    let player_stats = match player_tank.tank_type {
                        TankType::Player1 => &player_info.player1,
                        TankType::Player2 => player_info.player2.as_ref().expect("Player2 should exist"),
                        TankType::Enemy => unreachable!(),
                    };

                    if player_stats.track_chain {
                        // 拥有 track_chain，免疫伤害，直接跳过
                        continue;
                    }

                    // 设置 2 秒冷却
                    barrier_damage_tracker
                        .cooldowns
                        .insert(player_entity, Timer::from_seconds(2.0, TimerMode::Once));

                    // 永久减少 speed 20 和 protection 20（固定值）
                    let player_stats = match player_tank.tank_type {
                        TankType::Player1 => &mut player_info.player1,
                        TankType::Player2 => player_info.player2.as_mut().expect("Player2 should exist"),
                        TankType::Enemy => unreachable!(),
                    };
                    player_stats.speed = player_stats
                        .speed
                        .saturating_sub(powerup::POWERUP_ATTRIBUTE_INCREASE);
                    player_stats.protection = player_stats
                        .protection
                        .saturating_sub(powerup::POWERUP_ATTRIBUTE_INCREASE);

                    // 发送 speed 和 protection 变更事件
                    stat_changed_events.write(PlayerStatChanged {
                        player_type: player_tank.tank_type,
                        stat_type: StatType::Speed,
                    });
                    stat_changed_events.write(PlayerStatChanged {
                        player_type: player_tank.tank_type,
                        stat_type: StatType::Protection,
                    });
                }
            }
        }
    }
}

/// 生成玩家坦克和信息
fn spawn_players(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    game_mode: GameMode,
    player_info: &mut ResMut<PlayerInfo>,
) {
    // 加载玩家坦克纹理和创建精灵图
    let player1_texture = asset_server.load(TEXTURE_PLAYER_TANK1);
    let player2_texture = asset_server.load(TEXTURE_PLAYER_TANK2);
    let player_tile_size = UVec2::new(PLAYER_TILE_WIDTH as u32, PLAYER_TILE_HEIGHT as u32);
    let player_texture_atlas = TextureAtlasLayout::from_grid(player_tile_size, 2, 1, None, None);
    let player_texture_atlas_layout = texture_atlas_layouts.add(player_texture_atlas);
    let player_animation_indices = AnimationIndices { first: 0, last: 1 };

    // 根据游戏模式生成玩家
    match game_mode {
        GameMode::OnePlayer => {
            // 单人模式：只生成玩家1
            let _player1_tank_entity = spawn_player_tank(
                commands,
                player1_texture,
                player_texture_atlas_layout,
                player_animation_indices,
                TankType::Player1,
            );

            // 初始化玩家1信息
            player_info.player1 = PlayerStats {
                name: "Li Yun Long".to_string(),
                speed: INITIAL_ATTRIBUTE_VALUE,
                fire_speed: INITIAL_ATTRIBUTE_VALUE,
                protection: INITIAL_ATTRIBUTE_VALUE,
                shells: 1,
                penetrate: false,
                track_chain: false,
                air_cushion: false,
                fire_shell: false,
                life_points: 3,
                energy_points: 3,
                score: 0,
            };
            player_info.player2 = None;
        }

        GameMode::TwoPlayers => {
            // 双人模式：生成玩家1和玩家2
            let _player1_tank_entity = spawn_player_tank(
                commands,
                player1_texture,
                player_texture_atlas_layout.clone(),
                player_animation_indices,
                TankType::Player1,
            );

            let _player2_tank_entity = spawn_player_tank(
                commands,
                player2_texture,
                player_texture_atlas_layout,
                player_animation_indices,
                TankType::Player2,
            );

            // 初始化玩家1信息
            player_info.player1 = PlayerStats {
                name: "Li Yun Long".to_string(),
                speed: 40,
                fire_speed: 40,
                protection: 40,
                shells: 1,
                penetrate: false,
                track_chain: false,
                air_cushion: false,
                fire_shell: false,
                life_points: 3,
                energy_points: 3,
                score: 0,
            };

            // 初始化玩家2信息
            player_info.player2 = Some(PlayerStats {
                name: "Chu Yun Fei".to_string(),
                speed: 40,
                fire_speed: 40,
                protection: 40,
                shells: 1,
                penetrate: false,
                track_chain: false,
                air_cushion: false,
                fire_shell: false,
                life_points: 3,
                energy_points: 3,
                    score: 0,
                },
            );
        }
    }
}

/// 销毁所有玩家坦克
pub fn despawn_players(
    mut commands: Commands,
    player_tanks: Query<Entity, With<PlayerTank>>,
    mut player_info: ResMut<PlayerInfo>,
) {
    // 销毁所有玩家坦克
    for entity in player_tanks.iter() {
        let () = commands.entity(entity).try_despawn();
    }

    // 清空玩家信息
    player_info.player1 = PlayerStats {
        name: "Player 1".to_string(),
        speed: 1,
        fire_speed: 1,
        protection: 0,
        shells: 1,
        penetrate: false,
        track_chain: false,
        air_cushion: false,
        fire_shell: false,
        life_points: 0,
        energy_points: 3,
        score: 0,
    };
    player_info.player2 = None;
}

/// 恢复玩家能量点数
pub fn recover_energy(
    time: Res<Time>,
    mut regen_timer: ResMut<BlueBarRegenTimer>,
    mut player_info: ResMut<PlayerInfo>,
) {
    // 检查是否有玩家能量不满
    let any_player_needs_regen =
        player_info.player1.energy_points < 3
            || player_info
                .player2
                .as_ref()
                .is_some_and(|p| p.energy_points < 3);

    // 只有当有玩家能量不满时才更新计时器
    if any_player_needs_regen {
        regen_timer.timer.tick(time.delta());

        // 当计时器触发时，恢复1点能量
        if regen_timer.timer.just_finished() {
            if player_info.player1.energy_points < 3 {
                player_info.player1.energy_points =
                    (player_info.player1.energy_points + 1).min(3);
            }
            if let Some(ref mut p2) = player_info.player2 {
                if p2.energy_points < 3 {
                    p2.energy_points = (p2.energy_points + 1).min(3);
                }
            }
        }
    } else {
        // 所有玩家能量都满时，重置计时器
        regen_timer.timer.reset();
    }
}

/// 初始化玩家坦克
pub fn init_players(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    game_mode: Res<GameMode>,
    mut player_info: ResMut<PlayerInfo>,
    existing_players: Query<Entity, With<PlayerTank>>,
) {
    // 防御性编程：先清理可能存在的旧玩家坦克和信息
    for entity in existing_players.iter() {
        let () = commands.entity(entity).try_despawn();
    }
    // 玩家信息会在 spawn_players 中重新初始化

    // 生成新玩家
    spawn_players(
        &mut commands,
        &asset_server,
        &mut texture_atlas_layouts,
        match *game_mode {
            GameMode::OnePlayer => GameMode::OnePlayer,
            GameMode::TwoPlayers => GameMode::TwoPlayers,
        },
        &mut player_info,
    );
}
