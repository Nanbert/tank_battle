//! 激光系统模块
//!
//! 处理激光的生成、动画和蓝量消耗
//!
//! # 激光动画逻辑
//!
//! ## 能量球动画
//!
//! ### 蓄力阶段（3秒）
//! - 帧序列：0 → 1 → 2 → ... → 64
//! - 播放模式：
//!   - 先播放第 0-64 帧（一次性）
//!   - 播放完成后，循环播放第 20-64 帧
//!
//! ### 激光发射阶段
//! - 帧序列：81 → 82 → 83 → 84 → 81 → 82 → ...
//! - 播放模式：循环播放第 81-84 帧
//!
//! ## 激光动画
//!
//! ### 激光发射阶段
//! - 帧序列：0 → 1 → 2 → ... → 11（共 12 帧）
//! - 播放模式：一次性播放（不循环）
//! - 播放完毕后：销毁整个激光系统（激光实体 + 能量球实体）

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;

use crate::bullet::Bullet;
use crate::constants::*;
use crate::resources::{
    GameAtlasLayoutResources, GameAudioResources, GameTextureResources, Language, PlayerInfo,
};
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;
use crate::utils;

/// 激光命中结果
///
/// 表示激光射线检测命中的实体及其位置
#[derive(Debug, Clone, Copy)]
pub struct HitResult {
    /// 被命中的实体
    pub entity: Entity,
    /// 命中位置（世界坐标）
    pub position: Vec3,
}

/// 激光生成参数
pub struct LaserSpawnParams {
    pub position: Vec3,
    pub direction: Vec2,
    pub owner_type: TankType,
    pub energy_ball_entity: Option<Entity>,
}

/// 激光资源打包结构体
pub struct LaserResources {
    pub laser_texture: Handle<Image>,
    pub laser_texture_atlas: Handle<TextureAtlasLayout>,
    pub laser_atlas_info: &'static crate::atlas::TextureAtlasInfo,
    pub energy_ball_texture: Handle<Image>,
    pub energy_ball_texture_atlas: Handle<TextureAtlasLayout>,
    pub energy_ball_atlas_info: &'static crate::atlas::TextureAtlasInfo,
}

impl LaserResources {
    /// 根据玩家类型获取对应的激光资源
    pub fn from_player_type(
        tank_type: TankType,
        texture_resources: &GameTextureResources,
        atlas_layouts: &GameAtlasLayoutResources,
    ) -> Self {
        match tank_type {
            TankType::Player1 => Self {
                laser_texture: texture_resources.laser_blue.clone(),
                laser_texture_atlas: atlas_layouts.laser_blue.clone(),
                laser_atlas_info: &crate::atlas::LASER_BLUE_ATLAS,
                energy_ball_texture: texture_resources.energy_blue_ball.clone(),
                energy_ball_texture_atlas: atlas_layouts.energy_blue_ball.clone(),
                energy_ball_atlas_info: &crate::atlas::ENERGY_BALL_BLUE_ATLAS,
            },
            TankType::Player2 => Self {
                laser_texture: texture_resources.laser_red.clone(),
                laser_texture_atlas: atlas_layouts.laser_red.clone(),
                laser_atlas_info: &crate::atlas::LASER_RED_ATLAS,
                energy_ball_texture: texture_resources.energy_red_ball.clone(),
                energy_ball_texture_atlas: atlas_layouts.energy_red_ball.clone(),
                energy_ball_atlas_info: &crate::atlas::ENERGY_BALL_RED_ATLAS,
            },
            TankType::Enemy => unreachable!("敌方坦克没有激光大招"),
        }
    }
}

/// 生成激光实体（像手电筒一样，瞬间出现，不移动）
pub fn spawn_laser(
    commands: &mut Commands,
    resources: &LaserResources,
    params: LaserSpawnParams,
) -> Entity {
    // 计算激光旋转角度，激光原始是竖着的，需要根据方向旋转
    // 纹理默认向上（0度），需要根据方向计算旋转角度
    let angle = params.direction.y.atan2(params.direction.x) - std::f32::consts::FRAC_PI_2;

    // 激光束高度的一半（原本长度），用于位置偏移
    let laser_half_height = resources.laser_atlas_info.display_size.y / 2.0;

    // 计算激光位置：从坦克炮口向前延伸
    // 激光束的底部在坦克炮口，激光束向前延伸
    let laser_position = params.position
        + params.direction.extend(0.0) * (laser_half_height + LASER_POSITION_OFFSET);

    // 获取动画帧范围
    let animation_indices = resources.laser_atlas_info.animation_indices_full();

    crate::utils::spawn_animated_sprite(
        commands,
        resources.laser_texture.clone(),
        resources.laser_texture_atlas.clone(),
        animation_indices,
        ANIMATION_FRAME_LASER,
        Transform {
            translation: Vec3::new(laser_position.x, laser_position.y, Z_LASER),
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
        resources.laser_atlas_info.display_size,
        (
            Laser,
            PlayingEntity,
            AnimationMode::AtFrameWithEvent {
                trigger_frame: animation_indices.last,
                event_type: crate::constants::AnimationEventType::LaserAnimationEnd {
                    direction: params.direction,
                    start_point: params.position,
                    owner_type: params.owner_type,
                    energy_ball_entity: params.energy_ball_entity,
                },
            },
        ),
    )
}

/// 玩家激光射击系统（蓄力发射）
pub fn player_laser_system(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    audio_resources: Res<GameAudioResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &RotationTimer, &PlayerTank), With<PlayerTank>>,
    mut charge_query: Query<(Entity, &mut LaserCharge)>,
    mut energy_ball_query: Query<(Entity, &mut Sprite, &EnergyBall)>,
    sound_query: Query<(Entity, &LaserChargeSound)>,
    mut player_info: ResMut<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_trackers: ResMut<crate::resources::GameTrackers>,
    font_resources: Res<GameTextureResources>,
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
        game_trackers
            .insufficient_energy_tracker
            .tick_all(time.delta());

        // 处理蓄力逻辑
        if has_charge {
            // 根据玩家类型预先获取对应的激光资源
            let resources = LaserResources::from_player_type(
                player_tank.tank_type,
                &texture_resources,
                &atlas_layouts,
            );

            update_charge(
                &mut commands,
                &mut player_info,
                &mut energy_ball_query,
                &sound_query,
                entity,
                transform,
                player_tank.tank_type,
                &mut charge_query,
                &time,
                &audio_resources,
                &resources,
            );
        } else {
            // 根据玩家类型预先获取对应的激光资源
            let resources = LaserResources::from_player_type(
                player_tank.tank_type,
                &texture_resources,
                &atlas_layouts,
            );

            start_charge(
                &mut commands,
                &player_info,
                entity,
                player_tank.tank_type,
                &audio_resources,
                &mut game_trackers,
                &font_resources,
                &language,
                &resources,
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
    entity: Entity,
    tank_type: TankType,
    audio_resources: &GameAudioResources,
    game_trackers: &mut crate::resources::GameTrackers,
    font_resources: &GameTextureResources,
    language: &Language,
    resources: &LaserResources,
) {
    let player_stats = player_info
        .get_stats(tank_type)
        .expect("Player should exist");

    // 检查蓝量是否足够（需要3点蓝量）
    if player_stats.energy_points < 3 {
        // 能量不足，显示提示
        game_trackers.insufficient_energy_tracker.try_show_warning(
            commands,
            tank_type,
            font_resources,
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
    let sound_entity =
        utils::play_one_shot_sound(commands, audio_resources.laser_charge.clone(), 1.0);
    commands.entity(sound_entity).insert(LaserChargeSound);

    let energy_ball_animation_indices = AnimationIndices {
        first: 0,
        last: ENERGY_BALL_END_FRAME,
    };

    // 计算能量球位置：在炮管前方，贴紧炮管（相对坦克的局部坐标）
    // 能量球将作为坦克的子实体，会自动跟随坦克移动
    // 在局部坐标系中，炮塔在 (0, 10) 的位置，能量球应该在炮塔前方
    let energy_ball_offset = Vec3::new(
        7.0, // 侧向偏移（炮塔右侧）
        TANK_DISPLAY_SIZE.y / 2.0 + crate::physics_config::collider_sizes::BULLET.y + 5.0, // 前方偏移
        crate::constants::Z_LASER + 0.1, // Z轴偏移
    );

    // 将能量球作为坦克的子实体生成，使用局部坐标
    // 子实体不需要额外旋转，因为会继承父实体（坦克）的旋转
    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            PlayingEntity,
            EnergyBall {
                player_entity: entity,
            },
            crate::constants::EnergyBallPhase::Charging,
            AnimationMode::OneShotThenLoop {
                first: 0,
                last: ENERGY_BALL_END_FRAME,
                loop_start: 20,
                loop_end: ENERGY_BALL_END_FRAME,
            },
            Sprite {
                image: resources.energy_ball_texture.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: resources.energy_ball_texture_atlas.clone(),
                    index: energy_ball_animation_indices.first,
                }),
                custom_size: Some(resources.energy_ball_atlas_info.display_size),
                ..default()
            },
            Transform {
                translation: energy_ball_offset,
                rotation: Quat::IDENTITY, // 子实体不额外旋转，继承父实体的旋转
                scale: Vec3::ONE,
            },
            energy_ball_animation_indices,
            AnimationTimer(Timer::from_seconds(
                ANIMATION_FRAME_ENERGY_BALL,
                TimerMode::Repeating,
            )),
            CurrentAnimationFrame(0),
        ));
    });
}

/// 更新蓄力
fn update_charge(
    commands: &mut Commands,
    player_info: &mut ResMut<PlayerInfo>,
    energy_ball_query: &mut Query<(Entity, &mut Sprite, &EnergyBall)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    charge_query: &mut Query<(Entity, &mut LaserCharge)>,
    time: &Res<Time>,
    audio_resources: &GameAudioResources,
    resources: &LaserResources,
) {
    if let Ok((_, mut charge)) = charge_query.get_mut(entity) {
        charge.timer.tick(time.delta());

        // 蓄力完成，发射激光
        if charge.timer.just_finished() {
            fire_laser(
                commands,
                player_info,
                energy_ball_query,
                sound_query,
                entity,
                transform,
                tank_type,
                audio_resources,
                resources,
            );
        }
    }
}

/// 发射激光
fn fire_laser(
    commands: &mut Commands,
    player_info: &mut ResMut<PlayerInfo>,
    energy_ball_query: &mut Query<(Entity, &mut Sprite, &EnergyBall)>,
    sound_query: &Query<(Entity, &LaserChargeSound)>,
    entity: Entity,
    transform: &Transform,
    tank_type: TankType,
    audio_resources: &GameAudioResources,
    resources: &LaserResources,
) {
    // 消耗整个蓝条
    player_info.with_stats_mut(tank_type, |stats| {
        stats.energy_points = 0;
    });

    // 计算激光发射方向
    let direction = crate::utils::calculate_direction_from_rotation(&transform.rotation);

    // 计算激光初始位置
    let laser_pos = transform.translation
        + direction.extend(0.0)
            * (TANK_DISPLAY_SIZE.y / 2.0 + crate::physics_config::collider_sizes::BULLET.y);

    // 找到关联的能量球实体
    let energy_ball_entity = energy_ball_query
        .iter()
        .find(|(_, _, energy_ball)| energy_ball.player_entity == entity)
        .map(|(e, _, _)| e);

    // 生成激光
    spawn_laser(
        commands,
        resources,
        LaserSpawnParams {
            position: laser_pos,
            direction,
            owner_type: tank_type,
            energy_ball_entity,
        },
    );

    // 给能量球切换到激光阶段，更新动画循环范围
    if let Some(energy_ball_entity) = energy_ball_entity {
        commands
            .entity(energy_ball_entity)
            .insert(crate::constants::EnergyBallPhase::Lasering)
            .insert(AnimationMode::LoopRange {
                start_frame: crate::constants::ENERGY_BALL_LASER_LOOP_START,
                end_frame: crate::constants::ENERGY_BALL_LASER_LOOP_END,
            })
            .insert(CurrentAnimationFrame(
                crate::constants::ENERGY_BALL_LASER_LOOP_START,
            ));
    }

    // 播放激光音效
    utils::play_one_shot_sound(commands, audio_resources.laser.clone(), 1.0);

    // 添加相机震动效果
    commands.entity(entity).insert(CameraShake {
        timer: Timer::from_seconds(CAMERA_SHAKE_DURATION, TimerMode::Once),
        intensity: CAMERA_SHAKE_INTENSITY,
    });

    // 清理蓄力相关组件
    commands.entity(entity).remove::<LaserCharge>();

    // 清理音效
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
    laser_atlas_info: &crate::atlas::TextureAtlasInfo,
    collidables: &Query<
        (Entity, &Transform),
        Or<(
            With<EnemyTank>,
            With<Bullet>,
            With<Brick>,
            With<Steel>,
            With<Forest>,
            With<Barrier>,
            With<Sea>,
        )>,
    >,
    player_tanks: &Query<(), With<PlayerTank>>,
    commanders: &Query<(), With<Commander>>,
) -> Vec<HitResult> {
    // 激光起点（坦克炮口）
    let laser_origin = laser_start + laser_direction.extend(0.0) * LASER_POSITION_OFFSET;
    // 激光终点
    let laser_end = laser_origin + laser_direction.extend(0.0) * laser_atlas_info.display_size.y;

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
        if !(0.0..=laser_atlas_info.display_size.y).contains(&projection) {
            continue;
        }

        // 计算垂直距离（到激光线的距离）
        let perpendicular_distance = to_entity.dot(laser_normal).abs();

        // 检查是否在激光宽度范围内（使用最大的实体尺寸确保不遗漏）
        // 使用 WALL_TEXTURE_SIZE.x 作为通用尺寸，因为它足够大
        if perpendicular_distance > laser_half_width + WALL_TEXTURE_SIZE.x / 2.0 {
            continue;
        }

        // 命中！
        hit_entities.push(HitResult {
            entity,
            position: transform.translation,
        });
    }
    hit_entities
}

/// 处理激光动画结束事件
/// 在激光动画播放完最后一帧时由 animate_effects 触发
/// 执行碰撞检测、销毁命中实体、生成烟雾特效
pub fn handle_laser_end_events(
    mut commands: Commands,
    mut events: MessageReader<LaserEndEvent>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    texture_resources: Res<GameTextureResources>,
    collidables: Query<
        (Entity, &Transform),
        Or<(
            With<EnemyTank>,
            With<Bullet>,
            With<Brick>,
            With<Steel>,
            With<Forest>,
            With<Barrier>,
            With<Sea>,
        )>,
    >,
    player_tanks: Query<(), With<PlayerTank>>,
    commanders: Query<(), With<Commander>>,
) {
    for event in events.read() {
        // 根据所有者类型获取对应的激光 atlas info
        let laser_atlas_info = match event.owner_type {
            TankType::Player1 => &crate::atlas::LASER_BLUE_ATLAS,
            TankType::Player2 => &crate::atlas::LASER_RED_ATLAS,
            TankType::Enemy => unreachable!("敌方坦克没有激光大招"),
        };

        // 执行碰撞检测
        let hit_entities = check_laser_collision(
            event.start_point,
            event.direction,
            laser_atlas_info,
            &collidables,
            &player_tanks,
            &commanders,
        );

        // 销毁关联的能量球
        if let Some(energy_ball_entity) = event.energy_ball_entity {
            let () = commands.entity(energy_ball_entity).try_despawn();
        }

        // 销毁激光实体和所有标记的实体，生成烟雾特效
        for hit_result in hit_entities {
            let despawn_entity = hit_result.entity;
            let transform = hit_result.position;
            // 使用预加载的烟雾图集布局和工具函数生成烟雾特效
            let _ = utils::spawn_animated_sprite(
                &mut commands,
                texture_resources.smoke.clone(),
                atlas_layouts.smoke_atlas.clone(),
                crate::atlas::SMOKE_ATLAS.animation_indices_full(),
                ANIMATION_FRAME_SMOKE_LASER,
                Transform::from_translation(transform),
                crate::atlas::SMOKE_ATLAS.display_size,
                (
                    crate::ui::PlayingEntity,
                    crate::constants::Smoke,
                    AnimationMode::OneShot,
                ),
            );

            let () = commands.entity(despawn_entity).try_despawn();
        }
    }
}

/// 处理相机震动效果
pub fn handle_camera_shake(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut CameraShake)>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    for (entity, mut shake) in &mut query {
        shake.timer.tick(time.delta());

        if let Ok(mut camera_transform) = camera_query.single_mut() {
            // 随机生成震动偏移
            let x_offset = (rand::random::<f32>() - 0.5) * 2.0 * shake.intensity;
            let y_offset = (rand::random::<f32>() - 0.5) * 2.0 * shake.intensity;

            // 应用震动偏移到相机
            camera_transform.translation.x = x_offset;
            camera_transform.translation.y = y_offset;

            // 震动时间结束，恢复原始位置并移除组件
            if shake.timer.just_finished() {
                camera_transform.translation.x = 0.0;
                camera_transform.translation.y = 0.0;
                commands.entity(entity).remove::<CameraShake>();
            }
        }
    }
}
