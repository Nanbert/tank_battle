//! 激光系统模块
//!
//! 处理激光的生成、动画和蓝量消耗

use bevy::prelude::*;

use crate::constants::*;
use crate::resources::*;
use crate::bullet::BulletOwner;
use crate::constants::{RecoilForce, LaserCharge, LaserChargeProgressBar, LaserChargeSound};

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
    let laser_half_height = 1366.0 / 2.0;

    // 计算激光位置：从坦克炮口向前延伸
    // 激光束的底部在坦克炮口，激光束向前延伸
    // 向炮口靠近30像素
    let laser_position = params.position + params.direction.extend(0.0) * (laser_half_height - 30.0);

    commands.spawn((
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
            custom_size: Some(Vec2::new(512.0, 1366.0)), // 原本长度
            ..default()
        },
        Transform {
            translation: Vec3::new(laser_position.x, laser_position.y, 0.9), // z=0.9置于上层
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        laser_animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
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
    mut query: Query<(Entity, &Transform, &RotationTimer, &PlayerTank, &mut TankFireConfig), With<PlayerTank>>,
    mut charge_query: Query<(Entity, &mut LaserCharge)>,
    mut progress_bar_query: Query<(Entity, &mut Sprite, &LaserChargeProgressBar)>,
    sound_query: Query<(Entity, &LaserChargeSound)>,
    mut player_info: ResMut<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (_entity, transform, rotation_timer, player_tank, mut fire_config) in &mut query {
        // 检查是否正在旋转
        if rotation_timer.0.elapsed() < rotation_timer.0.duration() {
            continue;
        }

        // 检查激光键
        let laser_key = match player_tank.tank_type {
            TankType::Player1 => KeyCode::KeyL,
            TankType::Player2 => KeyCode::Numpad3,
            TankType::Enemy => continue,
        };

        // 检查是否已经存在蓄力组件
        let has_charge = charge_query.iter().any(|(e, c)| e == _entity && c.tank_type == player_tank.tank_type);

        // 检查是否被打断（移动或射击）
        let is_interrupted = if player_tank.tank_type == TankType::Player1 {
            // 玩家1：检查WASD或J键
            keyboard.pressed(KeyCode::KeyW) ||
            keyboard.pressed(KeyCode::KeyS) ||
            keyboard.pressed(KeyCode::KeyA) ||
            keyboard.pressed(KeyCode::KeyD) ||
            keyboard.pressed(KeyCode::KeyJ)
        } else {
            // 玩家2：检查方向键或小键盘1键
            keyboard.pressed(KeyCode::ArrowUp) ||
            keyboard.pressed(KeyCode::ArrowDown) ||
            keyboard.pressed(KeyCode::ArrowLeft) ||
            keyboard.pressed(KeyCode::ArrowRight) ||
            keyboard.pressed(KeyCode::Numpad1)
        };

        if is_interrupted && has_charge {
            // 打断蓄力
            commands.entity(_entity).remove::<LaserCharge>();
            // 删除进度条
            for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
                if progress_bar.player_entity == _entity {
                    commands.entity(progress_entity).despawn();
                }
            }
            // 停止蓄力音效
            for (sound_entity, _) in sound_query.iter() {
                commands.entity(sound_entity).despawn();
            }
            continue;
        }

        if keyboard.pressed(laser_key) {
            // 按住按键，开始或继续蓄力
            if !has_charge {
                // 获取玩家属性
                let Some(player_stats) = player_info.players.get(&player_tank.tank_type) else {
                    continue;
                };

                // 检查蓝量是否足够（需要3点蓝量）
                if player_stats.energy_blue_bar < 3 {
                    continue;
                }

                // 创建蓄力组件（4秒蓄力）
                commands.entity(_entity).insert(LaserCharge {
                    timer: Timer::from_seconds(4.0, TimerMode::Once),
                    tank_type: player_tank.tank_type,
                });

                // 播放蓄力音效并添加标记
                let charge_sound: Handle<AudioSource> = asset_server.load(SOUND_LASER_CHARGE);
                commands.spawn((
                    AudioPlayer::new(charge_sound),
                    LaserChargeSound,
                ));

                // 创建蓄力进度条（在坦克正上方，初始满格）
                commands.spawn((
                    PlayingEntity,
                    LaserChargeProgressBar { tank_type: player_tank.tank_type, player_entity: _entity },
                    Sprite {
                        color: Color::srgb(0.0, 1.0, 0.0), // 绿色
                        custom_size: Some(Vec2::new(100.0, 8.0)), // 初始宽度100（满格）
                        ..default()
                    },
                    Transform::from_xyz(transform.translation.x, transform.translation.y + TANK_HEIGHT / 2.0 + 20.0, 2.0), // 在坦克上方
                ));
            } else {
                // 更新蓄力计时器
                for (e, mut charge) in charge_query.iter_mut() {
                    if e == _entity && charge.tank_type == player_tank.tank_type {
                        charge.timer.tick(time.delta());

                        // 更新进度条（从满格向两边递减）
                        let progress = charge.timer.elapsed_secs() / charge.timer.duration().as_secs_f32();
                        let bar_width = 100.0 * (1.0 - progress); // 从100递减到0

                        for (_, mut sprite, progress_bar) in &mut progress_bar_query {
                            if progress_bar.player_entity == _entity {
                                sprite.custom_size = Some(Vec2::new(bar_width, 8.0));
                            }
                        }
                        
                        // 蓄力完成，发射激光
                        if charge.timer.just_finished() {
                            // 获取玩家属性
                            if let Some(player_stats) = player_info.players.get_mut(&player_tank.tank_type) {
                                // 消耗整个蓝条
                                player_stats.energy_blue_bar = 0;
                                
                                // 计算激光发射方向（基于坦克当前的旋转角度）
                                let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
                                let actual_angle = euler_angle + 90.0_f32.to_radians();
                                let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

                                // 计算激光初始位置（坦克前方）
                                let laser_pos = transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

                                // 生成激光
                                spawn_laser(
                                    &mut commands,
                                    &asset_server,
                                    &mut texture_atlas_layouts,
                                    LaserSpawnParams {
                                        position: laser_pos,
                                        direction,
                                        owner_type: player_tank.tank_type,
                                    },
                                );

                                // 播放激光音效
                                let laser_sound: Handle<AudioSource> = asset_server.load(SOUND_LASER);
                                commands.spawn(AudioPlayer::new(laser_sound));

                                // 立刻应用后坐力：向后移动 0.3 个身位
                                let recoil_distance = TANK_HEIGHT * 0.3;
                                let recoil_offset = direction * -recoil_distance;
                                commands.entity(_entity).insert(RecoilForce {
                                    original_pos: transform.translation,
                                    target_offset: recoil_offset,
                                    timer: Timer::from_seconds(0.3, TimerMode::Once),
                                });

                                }
                            
                            // 移除蓄力组件
                            commands.entity(_entity).remove::<LaserCharge>();
                            
                            // 删除进度条
                            for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
                                if progress_bar.player_entity == _entity {
                                    commands.entity(progress_entity).despawn();
                                }
                            }
                            
                            // 停止蓄力音效
                            for (sound_entity, _) in sound_query.iter() {
                                commands.entity(sound_entity).despawn();
                            }
                        }
                        break;
                    }
                }
            }
        } else if has_charge {
            // 松开按键但蓄力未完成，取消蓄力
            for (e, _) in charge_query.iter() {
                if e == _entity {
                    commands.entity(_entity).remove::<LaserCharge>();
                    // 删除进度条
                    for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
                        if progress_bar.player_entity == _entity {
                            commands.entity(progress_entity).despawn();
                        }
                    }
                    // 停止蓄力音效
                    for (sound_entity, _) in sound_query.iter() {
                        commands.entity(sound_entity).despawn();
                    }
                    break;
                }
            }
        }
    }
}