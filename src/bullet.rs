//! 子弹系统模块
//!
//! 处理子弹的生成、移动、碰撞检测和销毁逻辑

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::*;

/// 子弹实体标记组件
#[derive(Component)]
pub struct Bullet;

/// 子弹所有者组件，记录子弹是由哪个坦克发射的
#[derive(Component)]
pub struct BulletOwner {
    pub owner: Entity,
}

/// 子弹类型枚举
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BulletType {
    Player,
    Enemy,
}

/// 子弹生成参数
pub struct BulletSpawnParams {
    pub position: Vec3,
    pub direction: Vec2,
    pub speed: f32,
    pub owner: Entity,
    pub bullet_type: BulletType,
}

/// 生成子弹实体
pub fn spawn_bullet(
    commands: &mut Commands,
    params: BulletSpawnParams,
) -> Entity {
    commands.spawn((
        Bullet,
        PlayingEntity,
        BulletOwner { owner: params.owner },
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
            ..default()
        },
        Transform::from_translation(params.position),
        Velocity {
            linvel: params.direction * params.speed,
            angvel: 0.0,
        },
        RigidBody::KinematicVelocityBased,
        Collider::ball(BULLET_SIZE / 200.0),
        LockedAxes::ROTATION_LOCKED,
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
    ))
    .id()
}

/// 敌方坦克射击系统
pub fn enemy_shoot_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Velocity), With<EnemyTank>>,
    mut can_fire: ResMut<CanFire>,
    mut bullet_owners: ResMut<BulletOwners>,
) {
    for (entity, transform, velocity) in &mut query {
        // 检查是否可以射击（当前没有子弹）
        if can_fire.0.contains(&entity) {
            // 随机射击，每帧有 0.5% 的概率射击
            let mut rng = rand::rng();
            if rng.random::<f32>() < 0.005 {
                // 计算子弹发射方向（基于坦克当前移动方向）
                let direction = if velocity.linvel.length() > 0.0 {
                    velocity.linvel.normalize()
                } else {
                    Vec2::new(0.0, -1.0) // 默认向下
                };

                // 计算子弹初始位置（坦克前方）
                let bullet_pos = transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

                // 生成子弹
                let bullet_entity = spawn_bullet(
                    &mut commands,
                    BulletSpawnParams {
                        position: bullet_pos,
                        direction,
                        speed: BULLET_SPEED,
                        owner: entity,
                        bullet_type: BulletType::Enemy,
                    },
                );

                // 记录子弹的所有者
                bullet_owners.owners.insert(bullet_entity, entity);

                // 标记该坦克暂时不能射击
                can_fire.0.remove(&entity);
            }
        }
    }
}

/// 玩家坦克射击系统
pub fn player_shoot_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &RotationTimer, &PlayerTank), With<PlayerTank>>,
    mut bullet_owners: ResMut<BulletOwners>,
    player_info: Res<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (entity, transform, rotation_timer, player_tank) in &mut query {
        // 检查是否正在旋转
        if rotation_timer.0.elapsed() < rotation_timer.0.duration() {
            continue;
        }

        // 检查是否按下射击键
        let shoot_key = match player_tank.index {
            0 => KeyCode::KeyJ,
            1 => KeyCode::Numpad1,
            _ => continue,
        };

        if !keyboard.pressed(shoot_key) {
            continue;
        }

        // 检查是否可以射击（当前没有子弹）
        if bullet_owners.owners.values().any(|&owner| owner == entity) {
            continue;
        }

        // 获取玩家属性
        let Some(player_stats) = player_info.players.get(player_tank.index) else {
            continue;
        };

        // 计算子弹发射方向（基于坦克当前的旋转角度）
        // 坦克旋转时使用：angle - 270.0_f32.to_radians()
        // 因此需要补偿：actual_angle = euler_angle + 270.0_f32.to_radians()
        let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
        let actual_angle = euler_angle + 270.0_f32.to_radians();
        let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

        // 计算子弹初始位置（坦克前方）
        let bullet_pos = transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

        // 玩家子弹速度 = PLAYER_BULLET_SPEED × (1 + fire_speed百分比/100)
        let fire_speed_bonus = player_stats.fire_speed as f32 / 100.0;
        let bullet_speed = PLAYER_BULLET_SPEED * (1.0 + fire_speed_bonus);

        // 生成子弹
        let bullet_entity = spawn_bullet(
            &mut commands,
            BulletSpawnParams {
                position: bullet_pos,
                direction,
                speed: bullet_speed,
                owner: entity,
                bullet_type: BulletType::Player,
            },
        );

        // 记录子弹的所有者
        bullet_owners.owners.insert(bullet_entity, entity);
    }
}

/// 子弹边界检查系统
pub fn bullet_bounds_check_system(mut commands: Commands, mut query: Query<(Entity, &Transform), With<Bullet>>) {
    for (entity, transform) in &mut query {
        let x = transform.translation.x;
        let y = transform.translation.y;

        // 检查子弹是否超出游戏窗口边界
        if !(MAP_LEFT_X..=MAP_RIGHT_X).contains(&x)
            || !(MAP_BOTTOM_Y..=MAP_TOP_Y).contains(&y)
        {
            commands.entity(entity).despawn();
        }
    }
}

/// 子弹销毁清理系统
pub fn bullet_cleanup_system(
    mut removed_bullets: RemovedComponents<Bullet>,
    mut can_fire: ResMut<CanFire>,
    mut bullet_owners: ResMut<BulletOwners>,
) {
    // 当子弹被销毁时，只允许该子弹所属的坦克可以再次射击
    for bullet_entity in removed_bullets.read() {
        if let Some(tank_entity) = bullet_owners.owners.remove(&bullet_entity) {
            can_fire.0.insert(tank_entity);
        }
    }
}

/// 查找碰撞中的子弹和坦克
pub fn find_bullet_and_tank_in_collision(
    e1: Entity,
    e2: Entity,
    bullets: &Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    enemy_tanks: &Query<(), With<EnemyTank>>,
    player_tanks: &Query<&PlayerTank, With<PlayerTank>>,
) -> Option<(Entity, Entity)> {
    if bullets.get(e1).is_ok() && (enemy_tanks.get(e2).is_ok() || player_tanks.get(e2).is_ok()) {
        return Some((e1, e2));
    } else if bullets.get(e2).is_ok()
        && (enemy_tanks.get(e1).is_ok() || player_tanks.get(e1).is_ok())
    {
        return Some((e2, e1));
    }
    None
}

/// 判断子弹是否应该销毁
pub fn should_bullet_destroy(
    bullet_owner: Entity,
    tank_entity: Entity,
    enemy_tanks: &Query<(), With<EnemyTank>>,
    player_tanks: &Query<&PlayerTank, With<PlayerTank>>,
) -> bool {
    let is_player_bullet = player_tanks.get(bullet_owner).is_ok();
    let is_enemy_bullet = enemy_tanks.get(bullet_owner).is_ok();
    let is_player_tank = player_tanks.get(tank_entity).is_ok();
    let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();

    // 规则：
    // 1. 玩家子弹打到敌方坦克 -> 子弹消失
    // 2. 敌方子弹打到玩家坦克 -> 子弹消失
    // 3. 敌方子弹打到敌方坦克 -> 子弹穿过（不消失）
    // 4. 玩家子弹打到玩家坦克 -> 子弹穿过（不消失）
    (is_player_bullet && is_enemy_tank) || (is_enemy_bullet && is_player_tank)
}

/// 子弹与地形碰撞检测系统
pub fn bullet_terrain_collision_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
    enemy_tanks: Query<(), With<EnemyTank>>,
    forests: Query<(Entity, &Transform), With<Forest>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    player_info: Res<PlayerInfo>,
) {
    let mut bullets_to_despawn: Vec<Entity> = Vec::new();
    let mut forests_to_despawn: Vec<Entity> = Vec::new();
    let mut bricks_to_despawn: Vec<Entity> = Vec::new();
    let mut steels_to_despawn: Vec<Entity> = Vec::new();

    // 检查子弹与树林的碰撞
    for (forest_entity, forest_transform) in forests.iter() {
        for (bullet_entity, bullet_owner, bullet_transform) in bullets.iter() {
            let distance = (bullet_transform.translation - forest_transform.translation).length();
            if distance < FOREST_WIDTH / 2.0 {
                // 检查子弹所有者是否具有 fire_shell 效果
                if let Ok(player_tank) = player_tanks.get(bullet_owner.owner) {
                    if let Some(player_stats) = player_info.players.get(player_tank.index) {
                        if player_stats.fire_shell {
                            // 生成树林燃烧动画
                            crate::spawn_forest_fire(
                                &mut commands,
                                &asset_server,
                                &mut texture_atlas_layouts,
                                forest_transform.translation,
                            );

                            // 标记需要销毁的实体（只销毁树林，不销毁子弹，让子弹穿过）
                            forests_to_despawn.push(forest_entity);
                        }
                    }
                }
            }
        }
    }

    // 检查子弹与砖块的碰撞
    for (brick_entity, brick_transform) in bricks.iter() {
        for (bullet_entity, _bullet_owner, bullet_transform) in bullets.iter() {
            let distance = (bullet_transform.translation - brick_transform.translation).length();
            if distance < BRICK_WIDTH / 2.0 {
                // 播放砖块击中音效
                let brick_hit_sound: Handle<AudioSource> = asset_server.load("brick_hit.ogg");
                commands.spawn(AudioPlayer::new(brick_hit_sound));

                // 生成火花效果
                                            crate::spawn_spark(&mut commands, &asset_server, brick_transform.translation);
                // 标记需要销毁的实体（砖块和子弹）
                bricks_to_despawn.push(brick_entity);
                bullets_to_despawn.push(bullet_entity);
            }
        }
    }

    // 检查子弹与钢铁的碰撞
    for (steel_entity, steel_transform) in steels.iter() {
        for (bullet_entity, bullet_owner, bullet_transform) in bullets.iter() {
            let distance = (bullet_transform.translation - steel_transform.translation).length();
            if distance < STEEL_WIDTH / 2.0 {
                // 检查子弹所有者是否具有 penetrate 效果
                if let Ok(player_tank) = player_tanks.get(bullet_owner.owner) {
                    if let Some(player_stats) = player_info.players.get(player_tank.index) {
                        if player_stats.penetrate {
                            // 播放金属破碎音效
                            let metal_crash_sound: Handle<AudioSource> =
                                asset_server.load("metal_crash.ogg");
                            commands.spawn(AudioPlayer::new(metal_crash_sound));

                            // 生成火花效果
                            crate::spawn_spark(&mut commands, &asset_server, steel_transform.translation);

                            // 标记需要销毁的实体（钢铁和子弹）
                            steels_to_despawn.push(steel_entity);
                            bullets_to_despawn.push(bullet_entity);
                        } else {
                            // 没有 penetrate 效果，只播放击中音效
                            let hit_sound: Handle<AudioSource> = asset_server.load("hit_sound.ogg");
                            commands.spawn(AudioPlayer::new(hit_sound));

                            // 生成火花效果
                            crate::spawn_spark(&mut commands, &asset_server, steel_transform.translation);

                            // 只销毁子弹，不销毁钢铁
                            bullets_to_despawn.push(bullet_entity);
                        }
                    }
                } else if enemy_tanks.get(bullet_owner.owner).is_ok() {
                    // 敌方子弹，只播放击中音效并销毁子弹
                    let hit_sound: Handle<AudioSource> = asset_server.load("hit_sound.ogg");
                    commands.spawn(AudioPlayer::new(hit_sound));

                    // 生成火花效果
                    crate::spawn_spark(&mut commands, &asset_server, steel_transform.translation);

                    // 只销毁子弹，不销毁钢铁
                    bullets_to_despawn.push(bullet_entity);
                }
            }
        }
    }

    // 销毁标记的实体
    for entity in forests_to_despawn {
        commands.entity(entity).despawn();
    }
    for entity in bricks_to_despawn {
        commands.entity(entity).despawn();
    }
    for entity in steels_to_despawn {
        commands.entity(entity).despawn();
    }
    for entity in bullets_to_despawn {
        commands.entity(entity).despawn();
    }
}

/// 子弹与坦克碰撞检测系统
pub fn bullet_tank_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    enemy_tanks: Query<(), With<EnemyTank>>,
    enemy_tanks_with_transform: Query<(Entity, &Transform), With<EnemyTank>>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
    player_tanks_with_transform: Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: Query<(Entity, &PlayerIndex)>,
    mut enemy_count: ResMut<EnemyCount>,
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // 检查是否是子弹与坦克的碰撞
            if let Some((bullet_entity, tank_entity)) = find_bullet_and_tank_in_collision(
                *e1,
                *e2,
                &bullets,
                &enemy_tanks,
                &player_tanks,
            ) {
                let bullet_owner = bullets.get(bullet_entity).unwrap().1.owner;

                if should_bullet_destroy(bullet_owner, tank_entity, &enemy_tanks, &player_tanks) {
                    // 检查是否是玩家子弹击中敌方坦克
                    let is_player_bullet = player_tanks.get(bullet_owner).is_ok();
                    let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();
                    let is_player_tank = player_tanks.get(tank_entity).is_ok();

                    if is_player_bullet && is_enemy_tank {
                        // 获取敌方坦克的位置
                        if let Ok((_, tank_transform)) = enemy_tanks_with_transform.get(tank_entity) {
                            // 生成爆炸效果
                            crate::spawn_explosion(
                                &mut commands,
                                &asset_server,
                                &mut texture_atlas_layouts,
                                tank_transform.translation,
                            );
                        }

                        // 销毁敌方坦克
                        commands.entity(tank_entity).despawn();

                        // 减少当前敌方坦克计数
                        enemy_count.current_enemies -= 1;

                        // 增加分数
                        let player_tank = player_tanks.get(bullet_owner).expect("无法获取玩家坦克!");
                        if let Some(player_stats) = player_info.players.get_mut(player_tank.index) {
                            player_stats.score += 100;

                            // 发送分数变更事件
                            stat_changed_events.write(PlayerStatChanged {
                                player_index: player_tank.index,
                                stat_type: StatType::Score,
                            });
                        }

                        // 检查是否需要重新生成敌方坦克
                        if enemy_count.total_spawned < enemy_count.max_count {
                            // 生成敌方坦克出生动画（动画完成后会自动生成敌方坦克）
                            let mut rng = rand::rng();
                            let random_index = rng.random_range(0..ENEMY_BORN_PLACES.len());
                            let position = ENEMY_BORN_PLACES[random_index];
                            crate::spawn_enemy_born_animation(
                                &mut commands,
                                &asset_server,
                                &mut texture_atlas_layouts,
                                position,
                            );

                            // 增加已生成计数
                            enemy_count.total_spawned += 1;
                        }
                    } else if !is_player_bullet && is_player_tank {
                        let player_tank = player_tanks.get(tank_entity).expect("无法获取玩家坦克!");
                        // 敌方子弹击中玩家坦克
                        // 播放中弹音效
                        let hit_sound: Handle<AudioSource> = asset_server.load("hit_sound.ogg");
                        commands.spawn(AudioPlayer::new(hit_sound));

                        // 生成火花效果
                        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(tank_entity) {
                            crate::spawn_spark(&mut commands, &asset_server, tank_transform.translation);
                        }

                        // 扣除对应玩家的生命值
                        if let Some(player_stats) = player_info.players.get_mut(player_tank.index) {
                            // 检查玩家是否有 fire_shell、track_chain 或 penetrate 特效
                            let has_fire_shell = player_stats.fire_shell;
                            let has_track_chain = player_stats.track_chain;
                            let has_penetrate = player_stats.penetrate;

                            if has_fire_shell || has_track_chain || has_penetrate {
                                // 有特效，移除其中一个特效（优先级任意）
                                if has_fire_shell {
                                    player_stats.fire_shell = false;
                                    stat_changed_events.write(PlayerStatChanged {
                                        player_index: player_tank.index,
                                        stat_type: StatType::FireShell,
                                    });
                                } else if has_track_chain {
                                    player_stats.track_chain = false;
                                    stat_changed_events.write(PlayerStatChanged {
                                        player_index: player_tank.index,
                                        stat_type: StatType::TrackChain,
                                    });
                                } else if has_penetrate {
                                    player_stats.penetrate = false;
                                    stat_changed_events.write(PlayerStatChanged {
                                        player_index: player_tank.index,
                                        stat_type: StatType::Penetrate,
                                    });
                                }
                            } else {
                                // 没有特效，正常扣血
                                if player_stats.life_red_bar > 0 {
                                    player_stats.life_red_bar -= 1;
                                }
                                if player_stats.life_red_bar == 0 {
                                    // 获取玩家坦克的位置
                                    if let Ok((_, tank_transform)) =
                                        player_tanks_with_transform.get(tank_entity)
                                    {
                                        // 生成爆炸效果
                                        crate::spawn_explosion(
                                            &mut commands,
                                            &asset_server,
                                            &mut texture_atlas_layouts,
                                            tank_transform.translation,
                                        );
                                    }

                                    // 销毁玩家坦克
                                    commands.entity(tank_entity).despawn();

                                    // 标记对应玩家的头像为死亡状态
                                    for (avatar_entity, player_index) in player_avatars.iter() {
                                        if player_index.0 == player_tank.index {
                                            commands.entity(avatar_entity).insert(PlayerDead);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 销毁子弹
                    commands.entity(bullet_entity).despawn();
                }
            }
        }
    }
}