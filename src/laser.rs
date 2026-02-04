//! 激光系统模块
//!
//! 处理激光的生成、动画和蓝量消耗

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;

use crate::bullet::Bullet;
use crate::constants::*;
use crate::resources::{LaserResources, PlayerInfo, EffectResources, SoundResources, InsufficientEnergyTracker, Language};
use crate::utils;

/// 激光生成参数
pub struct LaserSpawnParams {
    pub position: Vec3,
    pub direction: Vec2,
    pub owner_type: TankType,
}

/// 生成激光实体（像手电筒一样，瞬间出现，不移动）
pub fn spawn_laser(
    commands: &mut Commands,
    laser_resources: &LaserResources,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    params: LaserSpawnParams,
) -> Entity {
    // 根据玩家类型加载不同的激光纹理图（12帧，3行4列布局，每帧512x683）
    let laser_texture = match params.owner_type {
        TankType::Player1 => laser_resources.laser_blue.clone(),
        TankType::Player2 => laser_resources.laser_red.clone(),
        TankType::Enemy => unreachable!("敌方坦克没有激光大招"),
    };
    let laser_tile_size = UVec2::new(LASER_TILE_WIDTH as u32, LASER_TILE_HEIGHT as u32);
    let laser_texture_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, laser_tile_size, 4, 3);
    let laser_animation_indices = AnimationIndices { first: 0, last: 11 };

    // 计算激光旋转角度，激光原始是竖着的，需要根据方向旋转
    // 纹理默认向上（0度），需要根据方向计算旋转角度
    let angle = params.direction.y.atan2(params.direction.x) - std::f32::consts::FRAC_PI_2;

    // 激光束高度的一半（原本长度），用于位置偏移
    let laser_half_height = LASER_HEIGHT / 2.0;

    // 计算激光位置：从坦克炮口向前延伸
    // 激光束的底部在坦克炮口，激光束向前延伸
    let laser_position = params.position
        + params.direction.extend(0.0) * (laser_half_height + LASER_POSITION_OFFSET);

    let bullet_type = if matches!(params.owner_type, TankType::Enemy) {
        Bullet::Enemy
    } else {
        Bullet::Player(params.owner_type)
    };

    commands
        .spawn((
            Laser,
            PlayingEntity,
            bullet_type,
            Sprite {
                image: laser_texture,
                texture_atlas: Some(TextureAtlas {
                    layout: laser_texture_atlas,
                    index: laser_animation_indices.first,
                }),
                custom_size: Some(Vec2::new(LASER_DISPLAY_WIDTH, LASER_HEIGHT)),
                ..default()
            },
            Transform {
                translation: Vec3::new(laser_position.x, laser_position.y, Z_LASER),
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            laser_animation_indices,
            AnimationTimer(Timer::from_seconds(
                ANIMATION_FRAME_LASER,
                TimerMode::Repeating,
            )),
            CurrentAnimationFrame(0),
            // 保存激光方向和位置，用于碰撞检测
            LaserDirection(params.direction),
            LaserStartPoint(params.position),
        ))
        .id()
}

/// 玩家激光射击系统（蓄力发射）
pub fn player_laser_system(
    mut commands: Commands,
    laser_resources: Res<LaserResources>,
    effect_resources: Res<EffectResources>,
    time: Res<Time>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &RotationTimer,
            &PlayerTank,
        ),
        With<PlayerTank>,
    >,
    mut charge_query: Query<(Entity, &mut LaserCharge)>,
    mut energy_ball_query: Query<(Entity, &mut Sprite, &EnergyBall)>,
    sound_query: Query<(Entity, &LaserChargeSound)>,
    mut player_info: ResMut<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    sound_resources: Res<SoundResources>,
    mut energy_tracker: ResMut<crate::resources::InsufficientEnergyTracker>,
    font_resources: Res<crate::resources::FontResources>,
    language: Res<crate::resources::Language>,
) {
    for (entity, transform, rotation_timer, player_tank) in &mut query {
        // 卫语句：正在旋转则跳过
        if rotation_timer.0.elapsed() < rotation_timer.0.duration() {
            continue;
        }

        // 卫语句：敌方坦克没有激光
        let key_bindings = player_tank.tank_type.get_key_bindings();

        // 使用 laser 键
        let laser_key = key_bindings.laser;

        // 检查蓄力状态和打断状态
        let has_charge = charge_query
            .iter()
            .any(|(e, c)| e == entity && c.tank_type == player_tank.tank_type);
        let is_interrupted = utils::is_movement_interrupted(&keyboard, player_tank.tank_type);

        // 卫语句：被打断则取消蓄力
        if is_interrupted && has_charge {
            cancel_charge(&mut commands, entity, &mut energy_ball_query, &sound_query);
            continue;
        }

        // 卫语句：未按键且有蓄力则取消
        if !keyboard.pressed(laser_key) {
            if has_charge {
                cancel_charge(&mut commands, entity, &mut energy_ball_query, &sound_query);
            }
            continue;
        }

        // 更新能量不足冷却计时器（必须在检查之前更新）
        energy_tracker.tick_all(time.delta());

        // 处理蓄力逻辑
        if has_charge {
            update_charge(
                &mut commands,
                &laser_resources,
                &mut texture_atlas_layouts,
                &mut player_info,
                &mut energy_ball_query,
                &sound_query,
                entity,
                transform,
                player_tank.tank_type,
                &mut charge_query,
                &time,
                &sound_resources,
            );
        } else {
            start_charge(
                &mut commands,
                &player_info,
                &effect_resources,
                &mut texture_atlas_layouts,
                entity,
                transform,
                player_tank.tank_type,
                &sound_resources,
                &mut energy_tracker,
                &font_resources.cn,
                &font_resources.en,
                &language,
            );
        }
    }
}

/// 取消蓄力
fn cancel_charge(
    commands: &mut Commands,
    entity: Entity,
    energy_ball_query: &mut Query<(Entity, &mut Sprite, &EnergyBall)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
) {
    commands.entity(entity).remove::<LaserCharge>();

    // 清理关联的能量球
    for (energy_ball_entity, _, energy_ball) in energy_ball_query.iter() {
        if energy_ball.player_entity == entity {
            let () = commands.entity(energy_ball_entity).try_despawn();
        }
    }

    for (sound_entity, _) in sound_query.iter() {
        let () = commands.entity(sound_entity).try_despawn();
    }
}

/// 开始蓄力
fn start_charge(
    commands: &mut Commands,
    player_info: &ResMut<PlayerInfo>,
    effect_resources: &EffectResources,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    sound_resources: &SoundResources,
    energy_tracker: &mut InsufficientEnergyTracker,
    font_cn: &Handle<Font>,
    font_en: &Handle<Font>,
    language: &Language,
) {
    let player_stats = player_info.get_stats(tank_type).expect("Player should exist");

    // 检查蓝量是否足够（需要3点蓝量）
    if player_stats.energy_points < 3 {
        // 能量不足，显示提示
        energy_tracker.try_show_warning(
            commands,
            tank_type,
            font_cn.clone(),
            font_en.clone(),
            *language,
        );
        return;
    }

    // 创建蓄力组件
    commands.entity(entity).insert(LaserCharge {
        timer: Timer::from_seconds(LASER_CHARGE_TIME, TimerMode::Once),
        tank_type,
    });

    // 播放蓄力音效并添加标记组件
    commands.spawn((
        AudioPlayer::new(sound_resources.laser_charge.clone()),
        PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(1.0)),
        LaserChargeSound,
    ));

    // 根据玩家类型选择能量球颜色
    let energy_ball_texture = match tank_type {
        TankType::Player1 => effect_resources.energy_blue_ball.clone(),
        TankType::Player2 => effect_resources.energy_red_ball.clone(),
        TankType::Enemy => unreachable!(),
    };

    let energy_ball_tile_size = UVec2::new(ENERGY_BALL_TILE_WIDTH as u32, ENERGY_BALL_TILE_HEIGHT as u32);
    let energy_ball_texture_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, energy_ball_tile_size, 17, 5);
    let energy_ball_animation_indices = AnimationIndices { first: 0, last: ENERGY_BALL_END_FRAME };

    // 计算能量球位置：在炮管前方，贴紧炮管（和激光使用相同的方向计算）
    let direction = crate::utils::calculate_direction_from_rotation(&transform.rotation);

    // 计算垂直向量（顺时针方向）：将方向向量旋转90度
    // (x, y) 旋转90度顺时针得到 (y, -x)
    let perp_direction = Vec2::new(direction.y, -direction.x);

    // 计算实际角度用于旋转能量球
    let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let actual_angle = euler_angle + ANGLE_OFFSET_DEGREES.to_radians();

    let energy_ball_pos = transform.translation
        + direction.extend(0.0) * (PLAYER_TANK_DISPLAY_HEIGHT / 2.0 + BULLET_SIZE + 5.0)
        + perp_direction.extend(0.0) * 7.0;

    commands.spawn((
        PlayingEntity,
        EnergyBall {
            player_entity: entity,
        },
        Sprite {
            image: energy_ball_texture,
            texture_atlas: Some(TextureAtlas {
                layout: energy_ball_texture_atlas,
                index: energy_ball_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(ENERGY_BALL_DISPLAY_WIDTH, ENERGY_BALL_DISPLAY_HEIGHT)),
            ..default()
        },
        Transform {
            translation: energy_ball_pos,
            rotation: Quat::from_rotation_z(actual_angle - std::f32::consts::FRAC_PI_2),
            scale: Vec3::ONE,
        },
        energy_ball_animation_indices,
        AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_ENERGY_BALL,
            TimerMode::Repeating,
        )),
        CurrentAnimationFrame(0),
    ));
}

/// 更新蓄力
fn update_charge(
    commands: &mut Commands,
    laser_resources: &LaserResources,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &mut ResMut<PlayerInfo>,
    energy_ball_query: &mut Query<(Entity, &mut Sprite, &EnergyBall)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    charge_query: &mut Query<(Entity, &mut LaserCharge)>,
    time: &Res<Time>,
    sound_resources: &SoundResources,
) {
    if let Ok((_, mut charge)) = charge_query.get_mut(entity) {
        charge.timer.tick(time.delta());

        // 蓄力完成，发射激光
        if charge.timer.just_finished() {
            fire_laser(
                commands,
                laser_resources,
                texture_atlas_layouts,
                player_info,
                energy_ball_query,
                sound_query,
                entity,
                transform,
                tank_type,
                sound_resources,
            );
        }
    }
}

/// 发射激光
fn fire_laser(
    mut commands: &mut Commands,
    laser_resources: &LaserResources,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &mut ResMut<PlayerInfo>,
    energy_ball_query: &mut Query<(Entity, &mut Sprite, &EnergyBall)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    sound_resources: &SoundResources,
) {
    // 消耗整个蓝条
    player_info.with_stats_mut(tank_type, |stats| {
        stats.energy_points = 0;
    });

    // 计算激光发射方向
    let direction = crate::utils::calculate_direction_from_rotation(&transform.rotation);

    // 计算激光初始位置
    let laser_pos = transform.translation
        + direction.extend(0.0) * (PLAYER_TANK_DISPLAY_HEIGHT / 2.0 + BULLET_SIZE);

    // 生成激光
    spawn_laser(
        commands,
        laser_resources,
        texture_atlas_layouts,
        LaserSpawnParams {
            position: laser_pos,
            direction,
            owner_type: tank_type,
        },
    );

    // 播放激光音效
    sound_resources.play(&mut commands, sound_resources.laser.clone(), 1.0);

    // 应用后坐力
    let recoil_distance = PLAYER_TANK_DISPLAY_HEIGHT * RECOIL_DISTANCE_FACTOR;
    let recoil_offset = direction * -recoil_distance;
    commands.entity(entity).insert(RecoilForce {
        original_pos: transform.translation,
        target_offset: recoil_offset,
        timer: Timer::from_seconds(RECOIL_DURATION, TimerMode::Once),
    });

    // 清理蓄力相关组件
    commands.entity(entity).remove::<LaserCharge>();

    // 清理关联的能量球
    for (energy_ball_entity, _, energy_ball) in energy_ball_query.iter() {
        if energy_ball.player_entity == entity {
            let () = commands.entity(energy_ball_entity).try_despawn();
        }
    }

    for (sound_entity, _) in sound_query.iter() {
        let () = commands.entity(sound_entity).try_despawn();
    }
}

/// 射线检测激光路径上的所有实体
/// 在激光动画结束时调用，检测激光路径上的碰撞实体
/// 返回被命中的实体列表及其位置
/// 优化版本：使用单一批量查询替代多次独立查询，提高性能
fn check_laser_collision(
    laser_start: Vec3,
    laser_direction: Vec2,
    collidables: &Query<(Entity, &Transform), Or<(With<EnemyTank>, With<Bullet>, With<Brick>, With<Steel>, With<Forest>, With<Barrier>, With<Sea>)>>,
    player_tanks: &Query<(), With<PlayerTank>>,
    commanders: &Query<(), With<Commander>>,
) -> Vec<(Entity, Vec3)> {
    // 激光起点（坦克炮口）
    let laser_origin = laser_start + laser_direction.extend(0.0) * LASER_POSITION_OFFSET;
    // 激光终点
    let laser_end = laser_origin + laser_direction.extend(0.0) * LASER_HEIGHT;

    // 激光宽度的一半（用于碰撞检测，使用实际可见宽度）
    let laser_half_width = LASER_COLLISION_WIDTH / 2.0;

    // 计算激光的方向向量和法向量
    let laser_dir = (laser_end - laser_origin).truncate().normalize();
    let laser_normal = Vec2::new(-laser_dir.y, laser_dir.x); // 逆时针旋转90度

    // 检查与所有可碰撞实体的碰撞，收集所有命中的实体
    let mut hit_entities = Vec::new();

    for (entity, transform) in collidables.iter() {
        // 跳过玩家坦克和 Commander（保护对象）
        if player_tanks.contains(entity) || commanders.contains(entity) {
            continue;
        }

        let entity_center = transform.translation.truncate();

        // 计算实体中心到激光起点的向量
        let to_entity = entity_center - laser_origin.truncate();

        // 计算沿激光方向的投影距离
        let projection = to_entity.dot(laser_dir);

        // 检查是否在激光长度范围内
        if projection < 0.0 || projection > LASER_HEIGHT {
            continue;
        }

        // 计算垂直距离（到激光线的距离）
        let perpendicular_distance = to_entity.dot(laser_normal).abs();

        // 检查是否在激光宽度范围内（使用最大的实体尺寸确保不遗漏）
        // 使用 BRICK_TEXTURE_WIDTH 作为通用尺寸，因为它足够大
        if perpendicular_distance > laser_half_width + BRICK_TEXTURE_WIDTH / 2.0 {
            continue;
        }

        // 命中！
        hit_entities.push((entity, transform.translation));
    }

    hit_entities
}

/// 处理后坐力效果
pub fn handle_recoil_force(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut RecoilForce)>,
) {
    for (entity, mut transform, mut recoil) in &mut query {
        recoil.timer.tick(time.delta());

        // 使用平滑插值应用后坐力位移
        let progress = recoil.timer.elapsed_secs() / recoil.timer.duration().as_secs_f32();
        let current_offset = recoil.target_offset * (1.0 - progress);

        // 从原始位置插值到当前位置
        transform.translation.x = recoil.original_pos.x + current_offset.x;
        transform.translation.y = recoil.original_pos.y + current_offset.y;

        // 后坐力时间结束，移除组件
        if recoil.timer.just_finished() {
            commands.entity(entity).remove::<RecoilForce>();
        }
    }
}

/// 处理激光动画
/// 优化版本：使用单一批量查询替代多个独立查询，提高碰撞检测性能
pub fn animate_laser(
    time: Res<Time>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
            &LaserDirection,
            &LaserStartPoint,
        ),
        With<Laser>,
    >,
    collidables: Query<(Entity, &Transform), Or<(With<EnemyTank>, With<Bullet>, With<Brick>, With<Steel>, With<Forest>, With<Barrier>, With<Sea>)>>,
    player_tanks: Query<(), With<PlayerTank>>,
    commanders: Query<(), With<Commander>>,
    effect_resources: Res<EffectResources>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame, laser_direction, laser_start) in &mut query {
        let prev_frame = current_frame.0;
        crate::utils::animate_sprite(&mut timer, &mut sprite, indices, &mut current_frame, time.delta());

        if prev_frame != current_frame.0 && timer.just_finished() {
            if current_frame.0 >= indices.last {
                // 动画播放完毕，执行一次碰撞检测（优化：使用批量查询）
                let hit_entities = check_laser_collision(
                    laser_start.0,
                    laser_direction.0,
                    &collidables,
                    &player_tanks,
                    &commanders,
                );
                
                // 销毁激光实体和所有标记的实体，生成烟雾特效
                for (despawn_entity, transform) in hit_entities {
                    // 使用预加载的烟雾纹理
                    let smoke_texture = effect_resources.smoke.clone();
                    let smoke_tile_size = UVec2::new(SMOKE_TILE_SIZE as u32, SMOKE_TILE_SIZE as u32);
                    let smoke_texture_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, smoke_tile_size, 5, 3);
                    let smoke_animation_indices = AnimationIndices { first: 0, last: 14 };

                    commands.spawn((
                        PlayingEntity,
                        crate::constants::Smoke,
                        crate::constants::OneShotAnimation,
                        Sprite {
                            image: smoke_texture,
                            texture_atlas: Some(TextureAtlas {
                                layout: smoke_texture_atlas,
                                index: smoke_animation_indices.first,
                            }),
                            custom_size: Some(Vec2::new(SMOKE_DISPLAY_SIZE, SMOKE_DISPLAY_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(
                            transform.x,
                            transform.y,
                            Z_DEFAULT,
                        ),
                        smoke_animation_indices,
                        AnimationTimer(Timer::from_seconds(
                            ANIMATION_FRAME_SMOKE,
                            TimerMode::Repeating,
                        )),
                        CurrentAnimationFrame(0),
                    ));

                    let () = commands.entity(despawn_entity).try_despawn();
                }
                let () = commands.entity(entity).try_despawn();
            }
        }
    }
}

/// 处理能量球动画
pub fn animate_energy_ball(
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
            &EnergyBall,
        ),
    >,
) {
    for (_entity, mut timer, mut sprite, indices, mut current_frame, _energy_ball) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，保持在最后一帧（蓄力完成时会由 fire_laser 清理）
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = indices.last;
                }
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}
