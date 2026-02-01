//! 激光系统模块
//!
//! 处理激光的生成、动画和蓝量消耗

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;

use crate::bullet::BulletOwner;
use crate::constants::*;
use crate::resources::{LaserResources, PlayerInfo, EffectResources, SoundResources, InsufficientEnergyTracker, Language};

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
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    params: LaserSpawnParams,
) -> Entity {
    // 根据玩家类型加载不同的激光纹理图（12帧，3行4列布局，每帧512x683）
    let laser_texture = match params.owner_type {
        TankType::Player1 => laser_resources.laser_blue.clone(),
        TankType::Player2 => laser_resources.laser_red.clone(),
        TankType::Enemy => unreachable!("敌方坦克没有激光大招"),
    };
    let laser_tile_size = UVec2::new(LASER_TILE_WIDTH as u32, LASER_TILE_HEIGHT as u32);
    let laser_texture_atlas = TextureAtlasLayout::from_grid(laser_tile_size, 4, 3, None, None);
    let laser_texture_atlas_layout = texture_atlas_layouts.add(laser_texture_atlas);
    let laser_animation_indices = AnimationIndices { first: 0, last: 11 };

    // 计算激光旋转角度，激光原始是竖着的，需要根据方向旋转
    // 纹理默认向上（0度），需要根据方向计算旋转角度
    let angle = params.direction.y.atan2(params.direction.x) - std::f32::consts::FRAC_PI_2;

    // 激光束高度的一半（原本长度），用于位置偏移
    let laser_half_height = LASER_HEIGHT / 2.0;

    // 计算激光位置：从坦克炮口向前延伸
    // 激光束的底部在坦克炮口，激光束向前延伸
    // 向炮口靠近30像素
    let laser_position = params.position
        + params.direction.extend(0.0) * (laser_half_height - LASER_POSITION_OFFSET);

    commands
        .spawn((
            Laser,
            PlayingEntity,
            BulletOwner {
                owner_type: params.owner_type,
            },
            Sprite {
                image: laser_texture,
                texture_atlas: Some(TextureAtlas {
                    layout: laser_texture_atlas_layout,
                    index: laser_animation_indices.first,
                }),
                custom_size: Some(Vec2::new(LASER_DISPLAY_WIDTH, LASER_HEIGHT)), // 原本长度
                ..default()
            },
            Transform {
                translation: Vec3::new(laser_position.x, laser_position.y, Z_LASER), // z=0.9置于上层
                rotation: Quat::from_rotation_z(angle),
                ..default()
            },
            laser_animation_indices,
            AnimationTimer(Timer::from_seconds(
                ANIMATION_FRAME_LASER,
                TimerMode::Repeating,
            )),
            CurrentAnimationFrame(0),
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
            &mut TankFireConfig,
        ),
        With<PlayerTank>,
    >,
    mut charge_query: Query<(Entity, &mut LaserCharge)>,
    mut progress_bar_query: Query<(Entity, &mut Sprite, &LaserChargeProgressBar)>,
    sound_query: Query<(Entity, &LaserChargeSound)>,
    mut player_info: ResMut<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    sound_resources: Res<SoundResources>,
    mut energy_tracker: ResMut<crate::resources::InsufficientEnergyTracker>,
    font_resources: Res<crate::resources::FontResources>,
    language: Res<crate::resources::Language>,
) {
    for (entity, transform, rotation_timer, player_tank, _fire_config) in &mut query {
        // 卫语句：正在旋转则跳过
        if rotation_timer.0.elapsed() < rotation_timer.0.duration() {
            continue;
        }

        // 卫语句：敌方坦克没有激光
        let laser_key = match player_tank.tank_type {
            TankType::Player1 => KeyCode::KeyL,
            TankType::Player2 => KeyCode::Numpad3,
            TankType::Enemy => continue,
        };

        // 检查蓄力状态和打断状态
        let has_charge = charge_query
            .iter()
            .any(|(e, c)| e == entity && c.tank_type == player_tank.tank_type);
        let is_interrupted = is_movement_interrupted(&keyboard, player_tank.tank_type);

        // 卫语句：被打断则取消蓄力
        if is_interrupted && has_charge {
            cancel_charge(&mut commands, entity, &mut progress_bar_query, &sound_query);
            continue;
        }

        // 卫语句：未按键且有蓄力则取消
        if !keyboard.pressed(laser_key) {
            if has_charge {
                cancel_charge(&mut commands, entity, &mut progress_bar_query, &sound_query);
            }
            continue;
        }

        // 更新能量不足冷却计时器（必须在检查之前更新）
        for timer in energy_tracker.cooldowns.values_mut() {
            timer.tick(time.delta());
        }

        // 处理蓄力逻辑
        if has_charge {
            update_charge(
                &mut commands,
                &laser_resources,
                &mut texture_atlas_layouts,
                &mut player_info,
                &mut progress_bar_query,
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

/// 检查是否被打断（移动或射击）
fn is_movement_interrupted(keyboard: &Res<ButtonInput<KeyCode>>, tank_type: TankType) -> bool {
    match tank_type {
        TankType::Player1 => {
            keyboard.pressed(KeyCode::KeyW)
                || keyboard.pressed(KeyCode::KeyS)
                || keyboard.pressed(KeyCode::KeyA)
                || keyboard.pressed(KeyCode::KeyD)
                || keyboard.pressed(KeyCode::KeyJ)
        }
        TankType::Player2 => {
            keyboard.pressed(KeyCode::ArrowUp)
                || keyboard.pressed(KeyCode::ArrowDown)
                || keyboard.pressed(KeyCode::ArrowLeft)
                || keyboard.pressed(KeyCode::ArrowRight)
                || keyboard.pressed(KeyCode::Numpad1)
        }
        TankType::Enemy => false,
    }
}

/// 取消蓄力
fn cancel_charge(
    commands: &mut Commands,
    entity: Entity,
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &LaserChargeProgressBar)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
) {
    commands.entity(entity).remove::<LaserCharge>();
    
    for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
        if progress_bar.player_entity == entity {
            let () = commands.entity(progress_entity).try_despawn();
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
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    sound_resources: &SoundResources,
    energy_tracker: &mut InsufficientEnergyTracker,
    font_cn: &Handle<Font>,
    font_en: &Handle<Font>,
    language: &Language,
) {
    let player_stats = match tank_type {
        TankType::Player1 => &player_info.player1,
        TankType::Player2 => player_info.player2.as_ref().expect("Player2 should exist"),
        TankType::Enemy => unreachable!(),
    };

    // 检查蓝量是否足够（需要3点蓝量）
    if player_stats.energy_points < 3 {
        // 检查冷却是否结束
        let can_show_warning = energy_tracker
            .cooldowns
            .get(&entity)
            .is_none_or(bevy::prelude::Timer::is_finished);

        if can_show_warning {
            // 设置能量不足提示冷却时间
            energy_tracker
                .cooldowns
                .insert(
                    entity,
                    Timer::from_seconds(INSUFFICIENT_ENERGY_DISPLAY_DURATION, TimerMode::Once),
                );

            // 触发能量不足提示
            crate::overlay_ui::spawn_insufficient_energy_warning(
                commands.reborrow(),
                font_cn.clone(),
                font_en.clone(),
                tank_type,
                *language,
            );
        }
        return;
    }

    // 创建蓄力组件
    commands.entity(entity).insert(LaserCharge {
        timer: Timer::from_seconds(LASER_CHARGE_TIME, TimerMode::Once),
        tank_type,
    });

    // 播放蓄力音效
    commands.spawn((AudioPlayer::new(sound_resources.laser_charge.clone()), LaserChargeSound));

    // 根据玩家类型选择能量球颜色
    let energy_ball_texture = match tank_type {
        TankType::Player1 => effect_resources.energy_blue_ball.clone(),
        TankType::Player2 => effect_resources.energy_red_ball.clone(),
        TankType::Enemy => unreachable!(),
    };

    let energy_ball_tile_size = UVec2::new(ENERGY_BALL_TILE_WIDTH as u32, ENERGY_BALL_TILE_HEIGHT as u32);
    let energy_ball_texture_atlas = TextureAtlasLayout::from_grid(energy_ball_tile_size, 17, 5, None, None);
    let energy_ball_texture_atlas_layout = texture_atlas_layouts.add(energy_ball_texture_atlas);
    let energy_ball_animation_indices = AnimationIndices { first: 0, last: ENERGY_BALL_END_FRAME };

    // 计算能量球位置：在炮管前方，贴紧炮管（和激光使用相同的方向计算）
    let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let actual_angle = euler_angle + ANGLE_OFFSET_DEGREES.to_radians();
    let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

    // 计算垂直向量（顺时针方向）：将方向向量旋转90度
    // (x, y) 旋转90度顺时针得到 (y, -x)
    let perp_direction = Vec2::new(direction.y, -direction.x);

    let energy_ball_pos = transform.translation
        + direction.extend(0.0) * (PLAYER_TANK_DISPLAY_HEIGHT / 2.0 + BULLET_SIZE + 5.0)
        + perp_direction.extend(0.0) * 7.0;

    commands.spawn((
        PlayingEntity,
        EnergyBall,
        LaserChargeProgressBar {
            player_entity: entity,
        },
        Sprite {
            image: energy_ball_texture,
            texture_atlas: Some(TextureAtlas {
                layout: energy_ball_texture_atlas_layout,
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
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &LaserChargeProgressBar)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    charge_query: &mut Query<(Entity, &mut LaserCharge)>,
    time: &Res<Time>,
    sound_resources: &SoundResources,
) {
    for (e, mut charge) in charge_query.iter_mut() {
        if e == entity && charge.tank_type == tank_type {
            charge.timer.tick(time.delta());

            // 蓄力完成，发射激光
            if charge.timer.just_finished() {
                fire_laser(
                    commands,
                    laser_resources,
                    texture_atlas_layouts,
                    player_info,
                    progress_bar_query,
                    sound_query,
                    entity,
                    transform,
                    tank_type,
                    sound_resources,
                );
            }
            break;
        }
    }
}

/// 发射激光
fn fire_laser(
    commands: &mut Commands,
    laser_resources: &LaserResources,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &mut ResMut<PlayerInfo>,
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &LaserChargeProgressBar)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    sound_resources: &SoundResources,
) {
    let player_stats = match tank_type {
        TankType::Player1 => &mut player_info.player1,
        TankType::Player2 => player_info.player2.as_mut().expect("Player2 should exist"),
        TankType::Enemy => unreachable!(),
    };

    // 消耗整个蓝条
    player_stats.energy_points = 0;

    // 计算激光发射方向
    let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let actual_angle = euler_angle + 90.0_f32.to_radians();
    let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

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
    commands.spawn(AudioPlayer::new(sound_resources.laser.clone()));

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

    for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
        if progress_bar.player_entity == entity {
            let () = commands.entity(progress_entity).try_despawn();
        }
    }

    for (sound_entity, _) in sound_query.iter() {
        let () = commands.entity(sound_entity).try_despawn();
    }
}

/// 激光碰撞检测系统（只收集实体，不立即销毁）
pub fn laser_collision_system(
    mut commands: Commands,
    mut frame_count: Local<u32>,
    lasers: Query<
        (
            Entity,
            &Transform,
            &CurrentAnimationFrame,
            &AnimationIndices,
        ),
        With<Laser>,
    >,
    enemies: Query<(Entity, &Transform), With<EnemyTank>>,
    bullets: Query<(Entity, &Transform), With<BulletOwner>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    forests: Query<(Entity, &Transform), With<Forest>>,
    barriers: Query<(Entity, &Transform), With<Barrier>>,
    seas: Query<(Entity, &Transform), With<Sea>>,
) {
    // 每5帧执行一次碰撞检测
    *frame_count += 1;
    if !(*frame_count).is_multiple_of(LASER_COLLISION_FRAME_INTERVAL) {
        return;
    }

    for (_laser_entity, laser_transform, _, _) in &lasers {
        let laser_bounds = calculate_laser_bounds(laser_transform);

        check_and_mark_collisions(&mut commands, &enemies, laser_bounds, ENEMY_TANK_DISPLAY_WIDTH, ENEMY_TANK_DISPLAY_HEIGHT);
        check_and_mark_collisions(&mut commands, &bullets, laser_bounds, BULLET_SIZE, BULLET_SIZE);
        check_and_mark_collisions(&mut commands, &bricks, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &steels, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &forests, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &barriers, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &seas, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
    }
}

/// 激光边界框
#[derive(Clone, Copy)]
struct LaserBounds {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

/// 计算激光旋转后的边界框
fn calculate_laser_bounds(transform: &Transform) -> LaserBounds {
    let laser_half_width = LASER_COLLIDER_HALF_WIDTH;
    let laser_half_height = LASER_COLLIDER_HALF_HEIGHT;
    let rotation = transform.rotation;

    let corners = [
        Vec2::new(-laser_half_width, -laser_half_height),
        Vec2::new(laser_half_width, -laser_half_height),
        Vec2::new(laser_half_width, laser_half_height),
        Vec2::new(-laser_half_width, laser_half_height),
    ];

    let rotated_corners: Vec<Vec2> = corners
        .iter()
        .map(|corner| {
            let rotated = rotation.mul_vec3(corner.extend(0.0));
            Vec2::new(rotated.x, rotated.y) + Vec2::new(transform.translation.x, transform.translation.y)
        })
        .collect();

    LaserBounds {
        left: rotated_corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min),
        right: rotated_corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max),
        bottom: rotated_corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min),
        top: rotated_corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max),
    }
}

/// 检查并标记碰撞实体
fn check_and_mark_collisions<T: Component>(
    commands: &mut Commands,
    entities: &Query<(Entity, &Transform), With<T>>,
    laser_bounds: LaserBounds,
    entity_width: f32,
    entity_height: f32,
) {
    let half_width = entity_width / 2.0;
    let half_height = entity_height / 2.0;

    for (entity, transform) in entities.iter() {
        let entity_left = transform.translation.x - half_width;
        let entity_right = transform.translation.x + half_width;
        let entity_bottom = transform.translation.y - half_height;
        let entity_top = transform.translation.y + half_height;

        if laser_bounds.left < entity_right
            && laser_bounds.right > entity_left
            && laser_bounds.bottom < entity_top
            && laser_bounds.top > entity_bottom
        {
            let _ = commands.entity(entity).try_insert(DespawnMarker);
        }
    }
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
        ),
        With<Laser>,
    >,
    despawn_entities: Query<(Entity, &Transform), With<DespawnMarker>>,
    effect_resources: Res<EffectResources>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁激光实体和所有标记的实体
                for (despawn_entity, transform) in despawn_entities.iter() {
                    // 使用预加载的烟雾纹理
                    let smoke_texture = effect_resources.smoke.clone();
                    let smoke_tile_size = UVec2::new(SMOKE_TILE_SIZE as u32, SMOKE_TILE_SIZE as u32);
                        let smoke_texture_atlas = TextureAtlasLayout::from_grid(smoke_tile_size, 5, 3, None, None);
                    let smoke_texture_atlas_layout = texture_atlas_layouts.add(smoke_texture_atlas);
                    let smoke_animation_indices = AnimationIndices { first: 0, last: 14 };

                    commands.spawn((
                        PlayingEntity,
                        Smoke,
                        Sprite {
                            image: smoke_texture,
                            texture_atlas: Some(TextureAtlas {
                                layout: smoke_texture_atlas_layout,
                                index: smoke_animation_indices.first,
                            }),
                            custom_size: Some(Vec2::new(SMOKE_DISPLAY_SIZE, SMOKE_DISPLAY_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
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
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
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
        ),
        With<EnergyBall>,
    >,
) {
    for (_entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
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
