//! 玩家坦克系统模块
//!
//! 处理玩家坦克的生成、移动、回城、冲刺和碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::powerup;
use crate::resources::{
    BarrierDamageTracker, BlueBarRegenTimer,
    GameMode, PlayerInfo, PlayerStatChanged, PlayerStats, RecallTimer, RecallTimers,
    StatType, PlayerTankResources,
};
use crate::utils;

/// 玩家1初始X坐标（左侧）
const PLAYER1_START_X: f32 = -PLAYER_COLLIDER_HALF - COMMANDER_WIDTH / 2.0 - PLAYER_SPAWN_OFFSET;

/// 玩家2初始X坐标（右侧）
const PLAYER2_START_X: f32 = PLAYER_COLLIDER_HALF + COMMANDER_WIDTH / 2.0 + PLAYER_SPAWN_OFFSET;

/// 玩家初始Y坐标（底部）
const PLAYER_START_Y: f32 = MAP_BOTTOM_Y + PLAYER_COLLIDER_HALF;

/// 生成玩家坦克
pub fn spawn_player_tank(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
    tank_type: TankType,
) -> Entity {
    let (x_pos, display_size, collider_half) = match tank_type {
        TankType::Player1 => (
            PLAYER1_START_X,
            Vec2::new(PLAYER_TANK_DISPLAY_WIDTH, PLAYER_TANK_DISPLAY_HEIGHT),
            PLAYER_COLLIDER_HALF,
        ),
        TankType::Player2 => (
            PLAYER2_START_X,
            Vec2::new(PLAYER_TANK_DISPLAY_WIDTH, PLAYER_TANK_DISPLAY_HEIGHT),
            PLAYER_COLLIDER_HALF,
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
            custom_size: Some(display_size),
            ..default()
        })
        .insert(Transform::from_xyz(
            x_pos,
            PLAYER_START_Y,
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
        // 根据玩家类型选择按键绑定
        let key_bindings = player_tank.tank_type.get_key_bindings();

        let direction = key_bindings.get_direction(&keyboard_input);

        // 检查是否需要转向
        let needs_rotation = if direction.length() > 0.0 {
            let angle = direction.y.atan2(direction.x);
            let target_angle = angle - ANGLE_OFFSET_DEGREES.to_radians();

            let current_euler = target_rotation.angle;
            let angle_diff = utils::calculate_angle_difference(target_angle, current_euler);

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
        // 获取玩家的 speed 百分比
        let speed_percent = player_info.get_speed_percent(player_tank.tank_type);
        // 实际速度 = 基础速度 × (1 + speed百分比/100)
        // 转向时保持 50% 速度，减少卡顿感
        let base_speed = PLAYER_TANK_SPEED * (1.0 + speed_percent);
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
            let angle_diff = utils::calculate_angle_difference(target_angle, current_euler);

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
            MAP_LEFT_X + PLAYER_COLLIDER_HALF,
            MAP_RIGHT_X - PLAYER_COLLIDER_HALF,
        );
        transform.translation.y = transform.translation.y.clamp(
            MAP_BOTTOM_Y + PLAYER_COLLIDER_HALF,
            MAP_TOP_Y - PLAYER_COLLIDER_HALF,
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

        // 根据玩家类型选择按键绑定
        let key_bindings = player_tank.tank_type.get_key_bindings();

        let is_recall_key_pressed = key_bindings.is_recalling(&keyboard_input);

        if is_recall_key_pressed && !is_recalling {
            // 计算初始位置
            let initial_position = if player_tank.tank_type == TankType::Player1 {
                Vec3::new(PLAYER1_START_X, PLAYER_START_Y, 0.0)
            } else {
                Vec3::new(PLAYER2_START_X, PLAYER_START_Y, 0.0)
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
                    color: COLOR_GREEN,
                    custom_size: Some(Vec2::new(PROGRESS_BAR_INITIAL_WIDTH, PROGRESS_BAR_HEIGHT)), // 初始宽度100（满格）
                    ..default()
                },
                Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y + PLAYER_TANK_DISPLAY_HEIGHT / 2.0 + PROGRESS_BAR_Y_OFFSET,
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
        let is_interrupted = utils::is_movement_interrupted(&keyboard_input, player_tank.tank_type);

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
    let key_bindings = match tank_type {
        TankType::Player1 => PlayerKeyBindings::player1(),
        TankType::Player2 => PlayerKeyBindings::player2(),
        TankType::Enemy => return false,
    };
    key_bindings.is_recalling(keyboard_input)
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
                player_pos.y + PLAYER_COLLIDER_HALF + PROGRESS_BAR_Y_OFFSET;
        }
    }
}

/// 重置玩家坦克位置到出生点
pub fn reset_player_positions(
    mut player_tanks: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut KinematicCharacterController,
            &PlayerTank,
        ),
        With<PlayerTank>,
    >,
) {
    for (mut transform, mut velocity, mut character_controller, player_tank) in &mut player_tanks {
        // 重置物理引擎速度和位移累积
        velocity.linvel = Vec2::ZERO;
        velocity.angvel = 0.0;
        character_controller.translation = None;

        match player_tank.tank_type {
            TankType::Player1 => {
                // 玩家1出生位置：左侧
                transform.translation.x = PLAYER1_START_X;
                transform.translation.y = PLAYER_START_Y;
            }
            TankType::Player2 => {
                // 玩家2出生位置：右侧
                transform.translation.x = PLAYER2_START_X;
                transform.translation.y = PLAYER_START_Y;
            }
            TankType::Enemy => {}
        }
        transform.rotation = Quat::IDENTITY;
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
        // 一次性获取玩家统计数据，避免重复查询
        let Some(player_stats) = player_info.get_stats_mut(player_tank.tank_type) else {
            // 单人模式下 Player2 不存在，跳过
            continue;
        };

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
                    if player_stats.track_chain {
                        // 拥有 track_chain，免疫伤害，直接跳过
                        continue;
                    }

                    // 设置屏障伤害冷却时间
                    barrier_damage_tracker
                        .cooldowns
                        .insert(
                            player_entity,
                            Timer::from_seconds(BARRIER_DAMAGE_COOLDOWN, TimerMode::Once),
                        );

                    // 永久减少 speed 20 和 protection 20（固定值）
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
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    game_mode: GameMode,
    player_info: &mut ResMut<PlayerInfo>,
    player_tank_resources: &PlayerTankResources,
) {
    // 使用预加载的玩家坦克纹理
    let player1_texture = player_tank_resources.player1.clone();
    let player2_texture = player_tank_resources.player2.clone();
    let player_tile_size = UVec2::new(PLAYER_TILE_WIDTH as u32, PLAYER_TILE_HEIGHT as u32);
    let player_texture_atlas_layout = utils::create_texture_atlas(player_tile_size, 2, 1);
    let player_texture_atlas = texture_atlas_layouts.add(player_texture_atlas_layout);
    let player_animation_indices = AnimationIndices { first: 0, last: 1 };

    // 根据游戏模式生成玩家
    match game_mode {
        GameMode::OnePlayer => {
            // 单人模式：只生成玩家1
            let _player1_tank_entity = spawn_player_tank(
                commands,
                player1_texture,
                player_texture_atlas,
                player_animation_indices,
                TankType::Player1,
            );

            // 初始化玩家1信息
            player_info.player1 = PlayerStats::new_default();
            player_info.player2 = None;
        }

        GameMode::TwoPlayers => {
            // 双人模式：生成玩家1和玩家2
            let _player1_tank_entity = spawn_player_tank(
                commands,
                player1_texture,
                player_texture_atlas.clone(),
                player_animation_indices,
                TankType::Player1,
            );

            let _player2_tank_entity = spawn_player_tank(
                commands,
                player2_texture,
                player_texture_atlas,
                player_animation_indices,
                TankType::Player2,
            );

            // 初始化玩家1和玩家2信息
            player_info.player1 = PlayerStats::new_default();
            player_info.player2 = Some(PlayerStats::new_default());
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
    player_info.player1 = PlayerStats::new_default();
    player_info.player2 = None;
}

/// 恢复玩家能量点数
pub fn recover_energy(
    time: Res<Time>,
    mut regen_timer: ResMut<BlueBarRegenTimer>,
    mut player_info: ResMut<PlayerInfo>,
) {
    // 检查是否有玩家能量不满
    if player_info.needs_energy_regen() {
        regen_timer.timer.tick(time.delta());

        // 当计时器触发时，恢复1点能量
        if regen_timer.timer.just_finished() {
            player_info.recover_all_energy();
        }
    } else {
        // 所有玩家能量都满时，重置计时器
        regen_timer.timer.reset();
    }
}

/// 初始化玩家坦克
pub fn init_players(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    game_mode: Res<GameMode>,
    mut player_info: ResMut<PlayerInfo>,
    existing_players: Query<Entity, With<PlayerTank>>,
    player_tank_resources: Res<PlayerTankResources>,
) {
    // 防御性编程：先清理可能存在的旧玩家坦克和信息
    for entity in existing_players.iter() {
        let () = commands.entity(entity).try_despawn();
    }
    // 玩家信息会在 spawn_players 中重新初始化

    // 生成新玩家
    spawn_players(
        &mut commands,
        &mut texture_atlas_layouts,
        match *game_mode {
            GameMode::OnePlayer => GameMode::OnePlayer,
            GameMode::TwoPlayers => GameMode::TwoPlayers,
        },
        &mut player_info,
        &player_tank_resources,
    );
}

/// 更新玩家坦克炮管
/// 根据玩家 shells 数量动态添加或更新炮管纹理
/// shells = 1: 单管炮管
/// shells = 2: 双管炮管
pub fn update_barrel_system(
    mut commands: Commands,
    player_tanks: Query<(Entity, Option<&Children>, &PlayerTank, &Transform), With<PlayerTank>>,
    barrels: Query<(Entity, &Sprite), With<Barrel>>,
    player_info: Res<PlayerInfo>,
    player_tank_resources: Res<PlayerTankResources>,
) {
    for (entity, children, player_tank, _transform) in player_tanks.iter() {
        // 获取玩家的 shells 数量
        let shells = player_info.get_shells(player_tank.tank_type);

        // 根据 shells 数量选择炮管纹理和尺寸
        let (barrel_texture, barrel_display_size) = if shells == 1 {
            (
                player_tank_resources.single_barrel.clone(),
                Vec2::new(SINGLE_BARREL_DISPLAY_WIDTH, SINGLE_BARREL_DISPLAY_HEIGHT),
            )
        } else {
            (
                player_tank_resources.double_barrel.clone(),
                Vec2::new(DOUBLE_BARREL_DISPLAY_WIDTH, DOUBLE_BARREL_DISPLAY_HEIGHT),
            )
        };

        // 检查是否已经有炮管子实体
        let mut barrel_entity: Option<Entity> = None;
        if let Some(children) = children {
            for child in children {
                if barrels.get(*child).is_ok() {
                    barrel_entity = Some(*child);
                    break;
                }
            }
        }

        if let Some(barrel_ent) = barrel_entity {
            // 如果已有炮管，只更新其纹理
            // 炮管位置固定，作为子实体会自动跟随父实体旋转
            if let Ok((_, sprite)) = barrels.get(barrel_ent) {
                // 检查纹理是否已更改
                if sprite.image != barrel_texture {
                    commands.entity(barrel_ent).insert(Sprite {
                        image: barrel_texture,
                        custom_size: Some(barrel_display_size),
                        ..default()
                    });
                }
            }
        } else {
            // 如果没有炮管，添加炮管作为子实体
            // 炮管位于坦克中心前方，会自动跟随坦克旋转
            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    Barrel,
                    Sprite {
                        image: barrel_texture,
                        custom_size: Some(barrel_display_size),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.1), // 炮管位于坦克中心
                ));
            });
        }
    }
}

/// 处理炮管后坐力效果
pub fn handle_barrel_recoil_force(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BarrelRecoilForce), With<Barrel>>,
) {
    for (entity, mut transform, mut recoil) in &mut query {
        recoil.timer.tick(time.delta());

        // 计算后坐力进度（0.0 到 1.0）
        let progress = recoil.timer.elapsed_secs() / recoil.timer.duration().as_secs_f32();

        if progress < 0.5 {
            // 前半段：炮管向后移动
            let recoil_progress = progress * 2.0; // 0.0 到 1.0
            let offset = -BARREL_RECOIL_DISTANCE * recoil_progress;
            transform.translation.y = offset;
        } else {
            // 后半段：炮管向前恢复
            let recovery_progress = (progress - 0.5) * 2.0; // 0.0 到 1.0
            let offset = -BARREL_RECOIL_DISTANCE * (1.0 - recovery_progress);
            transform.translation.y = offset;
        }

        // 后坐力时间结束，移除组件并恢复位置
        if recoil.timer.just_finished() {
            transform.translation.y = 0.0;
            commands.entity(entity).remove::<BarrelRecoilForce>();
        }
    }
}
