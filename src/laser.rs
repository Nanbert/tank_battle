//! 激光系统模块
//!
//! 处理激光的生成、动画和蓝量消耗

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;

use crate::bullet::BulletOwner;
use crate::constants::*;
use crate::resources::PlayerInfo;

/// 激光生成参数
pub struct LaserSpawnParams {
    pub position: Vec3,
    pub direction: Vec2,
    pub owner_type: TankType,
}

/// 生成激光实体（像手电筒一样，瞬间出现，不移动）
pub fn spawn_laser(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    params: LaserSpawnParams,
) -> Entity {
    // 根据玩家类型加载不同的激光纹理图（12帧，3行4列布局，每帧512x683）
    let laser_texture: Handle<Image> = match params.owner_type {
        TankType::Player1 => asset_server.load(TEXTURE_LASER_BLUE),
        TankType::Player2 => asset_server.load(TEXTURE_LASER_RED),
        TankType::Enemy => unreachable!("敌方坦克没有激光大招"),
    };
    let laser_tile_size = UVec2::new(512, 683);
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
                custom_size: Some(Vec2::new(512.0, LASER_HEIGHT)), // 原本长度
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
    asset_server: Res<AssetServer>,
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

        // 处理蓄力逻辑
        if has_charge {
            update_charge(
                &mut commands,
                &asset_server,
                &mut texture_atlas_layouts,
                &mut player_info,
                &mut progress_bar_query,
                &sound_query,
                entity,
                transform,
                player_tank.tank_type,
                &mut charge_query,
                &time,
            );
        } else {
            start_charge(
                &mut commands,
                &asset_server,
                &player_info,
                entity,
                transform,
                player_tank.tank_type,
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
    asset_server: &Res<AssetServer>,
    player_info: &ResMut<PlayerInfo>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
) {
    let Some(player_stats) = player_info.players.get(&tank_type) else {
        return;
    };

    // 检查蓝量是否足够（需要3点蓝量）
    if player_stats.energy_blue_bar < 3 {
        return;
    }

    // 创建蓄力组件
    commands.entity(entity).insert(LaserCharge {
        timer: Timer::from_seconds(LASER_CHARGE_TIME, TimerMode::Once),
        tank_type,
    });

    // 播放蓄力音效
    let charge_sound: Handle<AudioSource> = asset_server.load(SOUND_LASER_CHARGE);
    commands.spawn((AudioPlayer::new(charge_sound), LaserChargeSound));

    // 创建蓄力进度条
    commands.spawn((
        PlayingEntity,
        LaserChargeProgressBar {
            player_entity: entity,
        },
        Sprite {
            color: Color::srgb(0.0, 1.0, 0.0),
            custom_size: Some(Vec2::new(
                LASER_CHARGE_PROGRESS_BAR_WIDTH,
                PROGRESS_BAR_HEIGHT,
            )),
            ..default()
        },
        Transform::from_xyz(
            transform.translation.x,
            transform.translation.y + TANK_HEIGHT / 2.0 + PROGRESS_BAR_Y_OFFSET,
            Z_PROGRESS_BAR,
        ),
    ));
}

/// 更新蓄力
fn update_charge(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &mut ResMut<PlayerInfo>,
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &LaserChargeProgressBar)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    charge_query: &mut Query<(Entity, &mut LaserCharge)>,
    time: &Res<Time>,
) {
    for (e, mut charge) in charge_query.iter_mut() {
        if e == entity && charge.tank_type == tank_type {
            charge.timer.tick(time.delta());

            // 更新进度条
            let progress = charge.timer.elapsed_secs() / charge.timer.duration().as_secs_f32();
            let bar_width = LASER_CHARGE_PROGRESS_BAR_WIDTH * (1.0 - progress);

            for (_, mut sprite, progress_bar) in progress_bar_query.iter_mut() {
                if progress_bar.player_entity == entity {
                    sprite.custom_size = Some(Vec2::new(bar_width, PROGRESS_BAR_HEIGHT));
                }
            }

            // 蓄力完成，发射激光
            if charge.timer.just_finished() {
                fire_laser(
                    commands,
                    asset_server,
                    texture_atlas_layouts,
                    player_info,
                    progress_bar_query,
                    sound_query,
                    entity,
                    transform,
                    tank_type,
                );
            }
            break;
        }
    }
}

/// 发射激光
fn fire_laser(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &mut ResMut<PlayerInfo>,
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &LaserChargeProgressBar)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
) {
    let Some(player_stats) = player_info.players.get_mut(&tank_type) else {
        return;
    };

    // 消耗整个蓝条
    player_stats.energy_blue_bar = 0;

    // 计算激光发射方向
    let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let actual_angle = euler_angle + 90.0_f32.to_radians();
    let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

    // 计算激光初始位置
    let laser_pos = transform.translation
        + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

    // 生成激光
    spawn_laser(
        commands,
        asset_server,
        texture_atlas_layouts,
        LaserSpawnParams {
            position: laser_pos,
            direction,
            owner_type: tank_type,
        },
    );

    // 播放激光音效
    let laser_sound: Handle<AudioSource> = asset_server.load(SOUND_LASER);
    commands.spawn(AudioPlayer::new(laser_sound));

    // 应用后坐力
    let recoil_distance = TANK_HEIGHT * RECOIL_DISTANCE_FACTOR;
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
