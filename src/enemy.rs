//! 敌方坦克系统模块
//!
//! 处理敌方坦克的生成、移动、碰撞检测和动画

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[allow(clippy::wildcard_imports)]
use crate::constants::*;
use crate::resources::{EnemyResources, EnemySpawnState};
use crate::utils;

/// 敌方坦克出生动画系统
pub fn enemy_spawn_system(
    time: Res<Time>,
    mut commands: Commands,
    enemy_resources: Res<EnemyResources>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    enemy_entities: Query<Entity, Or<(With<EnemyTank>, With<EnemyBornAnimation>)>>,
) {
    // 更新生成冷却时间
    enemy_spawn_state.spawn_cooldown.tick(time.delta());

    // 动态获取当前场上敌方坦克数量（包括已生成的和正在出生动画中的）
    let current_enemy_count = enemy_entities.iter().count();

    // 检查是否需要生成新敌人
    // 条件：未达到总数上限 + 场上敌人数量少于4个 + 冷却时间已结束
    if enemy_spawn_state.has_spawned < enemy_spawn_state.max_count
        && current_enemy_count < MAX_ENEMY_ON_SCREEN
        && enemy_spawn_state.spawn_cooldown.is_finished()
    {
        // 生成敌方坦克出生动画
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..ENEMY_BORN_PLACES.len());
        let position = ENEMY_BORN_PLACES[random_index];
        spawn_enemy_born_animation(
            &mut commands,
            &enemy_resources,
            &mut texture_atlas_layouts,
            position,
        );

        // 更新计数
        enemy_spawn_state.has_spawned += 1;

        // 重置冷却时间
        enemy_spawn_state.spawn_cooldown.reset();
    }
}

/// 生成敌方坦克出生动画
pub fn spawn_enemy_born_animation(
    commands: &mut Commands,
    enemy_resources: &Res<EnemyResources>,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) -> Entity {
    let enemy_born_texture = enemy_resources.enemy_born.clone();
    let enemy_born_tile_size = UVec2::new(ENEMY_BORN_TILE_SIZE as u32, ENEMY_BORN_TILE_SIZE as u32);
    let enemy_born_texture_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, enemy_born_tile_size, 5, 3);
    let enemy_born_animation_indices = AnimationIndices {
        first: 0,
        last: ENEMY_BORN_END_FRAME,
    };

    commands
        .spawn((
            EnemyBornAnimation,
            PlayingEntity,
            Sprite {
                image: enemy_born_texture,
                texture_atlas: Some(TextureAtlas {
                    layout: enemy_born_texture_atlas,
                    index: enemy_born_animation_indices.first,
                }),
                custom_size: Some(Vec2::new(
                    ENEMY_BORN_ANIMATION_SIZE,
                    ENEMY_BORN_ANIMATION_SIZE,
                )),
                ..default()
            },
            Transform::from_translation(position),
            enemy_born_animation_indices,
            AnimationTimer(Timer::from_seconds(
                ANIMATION_FRAME_ENEMY_BORN,
                TimerMode::Repeating,
            )),
            CurrentAnimationFrame(0),
            BornPosition(position), // 记录出生位置
        ))
        .id()
}

/// 敌方坦克出生动画系统
pub fn animate_enemy_born_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
            &BornPosition,
        ),
        With<EnemyBornAnimation>,
    >,
    enemy_resources: Res<EnemyResources>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame, born_position) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            let current = current_frame.0;
            let spawn_frame = 10; // 第10帧生成坦克

            if current >= indices.last {
                // 动画播放完毕，销毁出生动画实体
                let () = commands.entity(entity).try_despawn();
            } else {
                // 继续播放动画
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;

                // 在动画播放到 2/3 时生成敌方坦克
                if next_index == spawn_frame {
                    // 加载敌方坦克纹理和创建精灵图
                    let enemy_texture = enemy_resources.enemy_tank.clone();
                    let enemy_tile_size =
                        UVec2::new(ENEMY_TILE_WIDTH as u32, ENEMY_TILE_HEIGHT as u32);
                    let enemy_texture_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, enemy_tile_size, 2, 1);
                    let enemy_animation_indices = AnimationIndices { first: 0, last: 1 };

                    // 生成敌方坦克
                    let _enemy_entity = commands
                        .spawn_empty()
                        .insert(EnemyTank {
                            direction: Vec2::new(0.0, -1.0),
                        })
                        .insert(PlayingEntity)
                        .insert(TankFireConfig::default())
                        .insert(DirectionChangeTimer(Timer::from_seconds(
                            ENEMY_DIRECTION_CHANGE_INTERVAL,
                            TimerMode::Once,
                        )))
                        .insert(CollisionCooldownTimer(Timer::from_seconds(
                            ENEMY_SPAWN_COOLDOWN,
                            TimerMode::Once,
                        )))
                        .insert(RotationTimer(Timer::from_seconds(
                            ENEMY_ROTATION_TIME,
                            TimerMode::Once,
                        )))
                        .insert(TargetRotation {
                            angle: ENEMY_ANGLE_OFFSET_DEGREES.to_radians(),
                        })
                        .insert(AnimationTimer(Timer::from_seconds(
                            ANIMATION_FRAME_ENEMY_MOVE,
                            TimerMode::Repeating,
                        )))
                        .insert(Sprite {
                            image: enemy_texture,
                            texture_atlas: Some(TextureAtlas {
                                layout: enemy_texture_atlas,
                                index: enemy_animation_indices.first,
                            }),
                            custom_size: Some(Vec2::new(
                                ENEMY_TANK_DISPLAY_WIDTH,
                                ENEMY_TANK_DISPLAY_HEIGHT,
                            )),
                            ..default()
                        })
                        .insert(Transform::from_translation(born_position.0))
                        .insert(enemy_animation_indices)
                        .insert(CurrentAnimationFrame(0))
                        .insert(Velocity {
                            linvel: Vec2::new(0.0, -TANK_SPEED),
                            angvel: 0.0,
                        })
                        .insert(RigidBody::Dynamic)
                        .insert(Collider::cuboid(
                            ENEMY_COLLIDER_HALF_WIDTH,
                            ENEMY_COLLIDER_HALF_HEIGHT,
                        ))
                        .insert(ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS)
                        .insert(
                            ActiveCollisionTypes::default()
                                | ActiveCollisionTypes::DYNAMIC_DYNAMIC
                                | ActiveCollisionTypes::DYNAMIC_STATIC,
                        )
                        .insert(LockedAxes::ROTATION_LOCKED)
                        .insert(GravityScale(0.0))
                        .insert(Friction::new(0.0))
                        .insert(Restitution::new(0.0))
                        .id();
                }
            }
        }
    }
}

/// 收集敌方坦克碰撞事件
/// 使用事件驱动模式，只在碰撞发生时处理，避免每帧主动查询
pub fn collect_enemy_collisions(
    mut events: MessageReader<CollisionEvent>,
    mut collision_cache: ResMut<EnemyCollisionCache>,
    all_tanks: Query<Entity, Or<(With<EnemyTank>, With<PlayerTank>)>>,
) {
    for event in events.read() {
        // 只处理碰撞开始事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 判断是否是敌方坦克参与的碰撞
        let enemy_entity = if all_tanks.contains(*e1) {
            *e1
        } else if all_tanks.contains(*e2) {
            *e2
        } else {
            continue;
        };

        // 缓存碰撞标记，具体法线从 ContactForceEvent 获取
        collision_cache.insert(enemy_entity, Vec2::ZERO);
    }
}

/// 收集接触力事件，获取碰撞法线
/// ContactForceEvent 提供了详细的接触力信息，包括最大力的方向
pub fn collect_contact_forces(
    mut events: MessageReader<ContactForceEvent>,
    mut collision_cache: ResMut<EnemyCollisionCache>,
    enemy_tanks: Query<(), With<EnemyTank>>,
) {
    for event in events.read() {
        let entity = event.collider1;

        // 只处理敌方坦克的接触力
        if !enemy_tanks.contains(entity) {
            continue;
        }

        // 从接触力事件中获取最大力的方向作为碰撞法线
        let direction = event.max_force_direction;
        if direction.length() > 0.0 {
            let normal = direction.normalize();
            collision_cache.insert(entity, normal);
        }
    }
}

/// 敌方坦克移动系统
#[allow(clippy::type_complexity)]
pub fn move_enemy_tanks(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Velocity,
        &mut EnemyTank,
        &mut DirectionChangeTimer,
        &mut CollisionCooldownTimer,
        &mut Transform,
        &mut RotationTimer,
        &mut TargetRotation,
    )>,
    mut collision_cache: ResMut<EnemyCollisionCache>,
) {
    for (
        entity,
        mut velocity,
        mut enemy_tank,
        mut direction_timer,
        mut collision_cooldown,
        mut transform,
        mut rotation_timer,
        mut target_rotation,
    ) in &mut query
    {
        // 更新碰撞冷却计时器
        collision_cooldown.tick(time.delta());

        // 只在冷却时间结束后才检测碰撞
        if collision_cooldown.is_finished() {
            // 优先使用事件缓存（事件驱动模式）
            if let Some(collision_normal) = collision_cache.take(entity) {
                if collision_normal.length() > 0.0 {
                    enemy_tank.direction = get_new_direction(collision_normal);
                    direction_timer.reset();
                    collision_cooldown.reset();
                }
            }

            // 边界碰撞仍然需要手动检测（边界不是物理实体，无碰撞事件）
            if let Some(boundary_normal) = check_boundary_collision(
                &transform,
                ENEMY_COLLIDER_HALF_WIDTH,
                ENEMY_COLLIDER_HALF_HEIGHT,
                enemy_tank.direction,
            ) {
                enemy_tank.direction = get_new_direction(boundary_normal);
                direction_timer.reset();
                collision_cooldown.reset();
            }
        }

        // 更新方向计时器
        direction_timer.tick(time.delta());

        // 如果计时器结束，有10%几率随机转向
        if direction_timer.just_finished() {
            handle_random_direction_change(&mut enemy_tank, &mut direction_timer);
        }

        // 更新坦克移动
        update_enemy_tank_movement(
            *enemy_tank,
            &mut velocity,
            &mut target_rotation,
            &mut rotation_timer,
        );

        // 更新旋转计时器
        rotation_timer.tick(time.delta());

        // 平滑旋转
        let current_rotation = transform.rotation;
        let target_rotation = Quat::from_rotation_z(target_rotation.angle);

        if current_rotation.angle_between(target_rotation) > 0.01 && !rotation_timer.is_finished() {
            // 使用 slerp 进行平滑旋转
            let progress = rotation_timer.elapsed_secs() / rotation_timer.duration().as_secs_f32();
            let eased_progress = progress * progress * 2.0f32.mul_add(-progress, 3.0); // 缓动函数
            transform.rotation = current_rotation.slerp(target_rotation, eased_progress);
        } else if current_rotation.angle_between(target_rotation) > 0.01 {
            // 旋转完成，直接设置为目标角度
            transform.rotation = target_rotation;
        }

        // 限制敌方坦克在地图边界内
        transform.translation.x = transform.translation.x.clamp(
            MAP_LEFT_X + ENEMY_TANK_DISPLAY_WIDTH / 2.0,
            MAP_RIGHT_X - ENEMY_TANK_DISPLAY_WIDTH / 2.0,
        );
        transform.translation.y = transform.translation.y.clamp(
            MAP_BOTTOM_Y + ENEMY_TANK_DISPLAY_HEIGHT / 2.0,
            MAP_TOP_Y - ENEMY_TANK_DISPLAY_HEIGHT / 2.0,
        );
    }
}

/// 检测边界碰撞
/// 返回碰撞法线方向（指向边界外部）
fn check_boundary_collision(
    transform: &Transform,
    collider_half_width: f32,
    collider_half_height: f32,
    current_direction: Vec2,
) -> Option<Vec2> {
    const BOUNDARY_BUFFER: f32 = 20.0;

    let x = transform.translation.x;
    let y = transform.translation.y;

    // 左边界：朝左且距离过近
    if x - collider_half_width < MAP_LEFT_X + BOUNDARY_BUFFER && current_direction.x < -0.5 {
        return Some(DIRECTIONS[1]); // 返回右方向
    }

    // 右边界：朝右且距离过近
    if x + collider_half_width > MAP_RIGHT_X - BOUNDARY_BUFFER && current_direction.x > 0.5 {
        return Some(DIRECTIONS[2]); // 返回左方向
    }

    // 上边界：朝上且距离过近
    if y + collider_half_height > MAP_TOP_Y - BOUNDARY_BUFFER && current_direction.y > 0.5 {
        return Some(DIRECTIONS[0]); // 返回下方向
    }

    // 下边界：朝下且距离过近
    if y - collider_half_height < MAP_BOTTOM_Y + BOUNDARY_BUFFER && current_direction.y < -0.5 {
        return Some(DIRECTIONS[3]); // 返回上方向
    }

    None
}

/// 根据碰撞法线获取新的移动方向
/// 分析碰撞法线，选择一个可用的移动方向（避免碰撞方向）
fn get_new_direction(collision_normal: Vec2) -> Vec2 {
    let mut rng = rand::rng();

    // 比较绝对值，确定主要碰撞方向
    let x_abs = collision_normal.x.abs();
    let y_abs = collision_normal.y.abs();

    // 确定被阻挡的方向索引
    let blocked_index = if x_abs > y_abs {
        // 水平方向碰撞
        if collision_normal.x > 0.0 { 1 } else { 2 }
    } else {
        // 垂直方向碰撞
        if collision_normal.y > 0.0 { 3 } else { 0 }
    };

    // 从三个可用方向中随机选择一个
    // 可用索引：0,1,2,3 中除了 blocked_index 的三个
    let available: [usize; 3] = [0, 1, 2, 3]
        .into_iter()
        .filter(|&i| i != blocked_index)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap_or([0, 1, 2]);

    DIRECTIONS[available[rng.random_range(0..3)]]
}

/// 处理随机方向改变
fn handle_random_direction_change(
    enemy_tank: &mut EnemyTank,
    direction_timer: &mut DirectionChangeTimer,
) {
    let mut rng = rand::rng();
    if rng.random::<f32>() < ENEMY_RANDOM_TURN_PROBABILITY {
        let random_index = rng.random_range(0..DIRECTIONS.len());
        enemy_tank.direction = DIRECTIONS[random_index];
    }
    direction_timer.reset();
}

/// 更新敌方坦克移动
fn update_enemy_tank_movement(
    enemy_tank: EnemyTank,
    velocity: &mut Velocity,
    target_rotation: &mut TargetRotation,
    rotation_timer: &mut RotationTimer,
) {
    if enemy_tank.direction.length() > 0.0 {
        let angle = enemy_tank.direction.y.atan2(enemy_tank.direction.x);
        let target_angle = angle - ENEMY_ANGLE_OFFSET_DEGREES.to_radians();

        // 检查是否需要转向
        let current_euler = target_rotation.angle;
        let angle_diff = std::f32::consts::PI.mul_add(3.0, target_angle - current_euler)
            % (std::f32::consts::PI * 2.0)
            - std::f32::consts::PI;

        if angle_diff.abs() > ANGLE_DIFF_THRESHOLD {
            // 需要转向，设置速度为0实现原地转向
            velocity.linvel = Vec2::ZERO;
            target_rotation.angle = target_angle;
            rotation_timer.reset();
        } else {
            // 不需要转向，正常移动
            velocity.linvel = enemy_tank.direction * TANK_SPEED;
        }
    }
}

/// 销毁所有敌方坦克
pub fn despawn_enemy_tank(
    mut commands: Commands,
    enemy_tanks: Query<Entity, With<EnemyTank>>,
) {
    for entity in enemy_tanks.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}

/// 重置敌方坦克生成状态
pub fn reset_enemy_spawn_state(mut enemy_spawn_state: ResMut<EnemySpawnState>) {
    // 重置敌方坦克计数
    enemy_spawn_state.has_spawned = 0;
    enemy_spawn_state.spawn_cooldown.reset();
}
