//! 子弹系统模块
//!
//! 处理子弹的生成、移动、碰撞检测和销毁逻辑

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::*;

/// 特效事件枚举
/// 用于解耦碰撞逻辑和特效生成
#[derive(Event, Clone, Copy, Message)]
pub enum EffectEvent {
    Explosion {
        position: Vec3,
    },
    Spark {
        position: Vec3,
    },
    ForestFire {
        position: Vec3,
    },
}

/// 子弹实体标记组件
#[derive(Component)]
pub struct Bullet;

/// 子弹所有者组件
#[derive(Component)]
pub struct BulletOwner {
    pub owner_type: TankType,
}

/// 子弹销毁标记组件
#[derive(Component)]
pub struct BulletDespawnMarker;

/// 子弹生成参数
pub struct BulletSpawnParams {
    pub position: Vec3,
    pub direction: Vec2,
    pub speed: f32,
    pub owner_type: TankType,
}

/// 生成子弹实体
pub fn spawn_bullet(
    commands: &mut Commands,
    asset_server: &AssetServer,
    params: BulletSpawnParams,
) -> Entity {
    // 根据坦克类型选择子弹纹理
    let bullet_texture = match params.owner_type {
        TankType::Player1 => asset_server.load("texture/bullets/bullet_player1.png"),
        TankType::Player2 => asset_server.load("texture/bullets/bullet_player2.png"),
        TankType::Enemy => asset_server.load("texture/bullets/bullet_enemy.png"),
    };

    // 计算子弹旋转角度（纹理是横向的，需要根据射击方向旋转）
    // 假设纹理默认向右（0度），需要根据方向计算旋转角度
    let angle = params.direction.y.atan2(params.direction.x);
    let rotation = Quat::from_rotation_z(angle);

    commands.spawn((
        Bullet,
        PlayingEntity,
        BulletOwner {
            owner_type: params.owner_type,
        },
        Sprite {
            image: bullet_texture,
            custom_size: Some(Vec2::new(60.0, 40.0)), // 子弹尺寸：长60像素，宽40像素
            ..default()
        },
        Transform {
            translation: params.position,
            rotation,
            ..default()
        },
        Velocity {
            linvel: params.direction * params.speed,
            angvel: 0.0,
        },
        RigidBody::KinematicVelocityBased,
        Collider::cuboid(60.0 / 2.0, 40.0 / 2.0), // 使用矩形碰撞体匹配子弹尺寸
        LockedAxes::ROTATION_LOCKED,
        Sensor,
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC | ActiveCollisionTypes::KINEMATIC_STATIC,
    ))
    .id()
}

/// 敌方坦克射击系统
pub fn enemy_shoot_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
                    &asset_server,
                    BulletSpawnParams {
                        position: bullet_pos,
                        direction,
                        speed: BULLET_SPEED,
                        owner_type: TankType::Enemy,
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
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &RotationTimer, &PlayerTank, Option<&mut PlayerShootCooldown>), With<PlayerTank>>,
    mut bullet_owners: ResMut<BulletOwners>,
    player_info: Res<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (entity, transform, rotation_timer, player_tank, mut shoot_cooldown) in &mut query {
        // 检查是否正在旋转
        if rotation_timer.0.elapsed() < rotation_timer.0.duration() {
            continue;
        }

        // 更新射击冷却时间
        if let Some(ref mut cooldown) = shoot_cooldown {
            cooldown.timer.tick(time.delta());
            if !cooldown.timer.is_finished() {
                continue;
            }
        }

        // 检查是否按下射击键
        let shoot_key = match player_tank.tank_type {
            TankType::Player1 => KeyCode::KeyJ,
            TankType::Player2 => KeyCode::Numpad1,
            TankType::Enemy => continue,
        };

        if !keyboard.pressed(shoot_key) {
            continue;
        }

        // 检查是否可以射击（当前没有子弹）
        if bullet_owners.owners.values().any(|&owner| owner == entity) {
            continue;
        }

        // 获取玩家属性
        let Some(player_stats) = player_info.players.get(&player_tank.tank_type) else {
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
            &asset_server,
            BulletSpawnParams {
                position: bullet_pos,
                direction,
                speed: bullet_speed,
                owner_type: player_tank.tank_type,
            },
        );

        // 记录子弹的所有者
        bullet_owners.owners.insert(bullet_entity, entity);

        // 设置射击冷却时间（如果没有冷却组件则添加）
        if shoot_cooldown.is_none() {
            commands.entity(entity).insert(PlayerShootCooldown {
                timer: Timer::from_seconds(0.2, TimerMode::Once),
            });
        } else if let Some(ref mut cooldown) = shoot_cooldown {
            cooldown.timer.reset();
        }
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
            commands.entity(entity).try_insert(BulletDespawnMarker);
        }
    }
}

/// 子弹统一销毁系统
/// 处理所有子弹的销毁逻辑，包括清理所有者引用和实际销毁
pub fn bullet_despawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &BulletDespawnMarker, &BulletOwner), With<Bullet>>,
    mut can_fire: ResMut<CanFire>,
    mut bullet_owners: ResMut<BulletOwners>,
) {
    for (entity, _marker, _owner) in &mut query {
        // 清理所有者引用，允许坦克再次射击
        if let Some(tank_entity) = bullet_owners.owners.remove(&entity) {
            can_fire.0.insert(tank_entity);
        }

        // 销毁子弹实体
        commands.entity(entity).despawn();
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
    bullet_owner_type: TankType,
    tank_entity: Entity,
    enemy_tanks: &Query<(), With<EnemyTank>>,
    player_tanks: &Query<&PlayerTank, With<PlayerTank>>,
) -> bool {
    let is_player_tank = player_tanks.get(tank_entity).is_ok();
    let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();

    let is_player_bullet = matches!(bullet_owner_type, TankType::Player1 | TankType::Player2);
    let is_enemy_bullet = matches!(bullet_owner_type, TankType::Enemy);

    // 规则：
    // 1. 玩家子弹打到敌方坦克 -> 子弹消失
    // 2. 敌方子弹打到玩家坦克 -> 子弹消失
    // 3. 敌方子弹打到敌方坦克 -> 子弹穿过（不消失）
    // 4. 玩家子弹打到玩家坦克 -> 子弹穿过（不消失）
    (is_player_bullet && is_enemy_tank) || (is_enemy_bullet && is_player_tank)
}

/// 子弹与地形碰撞检测系统
/// 使用 Rapier 碰撞事件替代手动距离判断，性能从 O(n²) 提升到 O(n log n)
pub fn bullet_terrain_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    asset_server: Res<AssetServer>,
    _texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    forests: Query<(Entity, &Transform), With<Forest>>,
    bricks: Query<(), With<Brick>>,
    steels: Query<(), With<Steel>>,
    player_info: Res<PlayerInfo>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // 判断是否是子弹与地形的碰撞
            let (bullet_entity, terrain_entity) = if bullets.get(*e1).is_ok() {
                (*e1, *e2)
            } else if bullets.get(*e2).is_ok() {
                (*e2, *e1)
            } else {
                continue;
            };

            // 获取子弹信息
            let (bullet_owner, bullet_transform) = match bullets.get(bullet_entity) {
                Ok((_, owner, transform)) => (owner, transform),
                Err(_) => continue,
            };

            // 判断地形类型并处理碰撞
            if let Ok((forest_entity, forest_transform)) = forests.get(terrain_entity) {
                // 子弹与树林碰撞
                let player_index = bullet_owner.owner_type;
                if player_index != TankType::Enemy {
                    if let Some(player_stats) = player_info.players.get(&player_index) {
                        if player_stats.fire_shell {
                            // 发送树林燃烧特效事件
                            effect_events.write(EffectEvent::ForestFire {
                                position: forest_transform.translation,
                            });
                            // 销毁树林，不销毁子弹
                            commands.entity(forest_entity).despawn();
                        }
                    }
                }
            } else if bricks.get(terrain_entity).is_ok() {
                // 子弹与砖块碰撞
                // 播放砖块击中音效
                let brick_hit_sound: Handle<AudioSource> = asset_server.load("brick_hit.ogg");
                commands.spawn(AudioPlayer::new(brick_hit_sound));

                // 发送火花特效事件
                effect_events.write(EffectEvent::Spark {
                    position: bullet_transform.translation,
                });

                // 销毁砖块和标记子弹销毁
                commands.entity(terrain_entity).despawn();
                commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
            } else if steels.get(terrain_entity).is_ok() {
                // 子弹与钢铁碰撞
                let player_index = bullet_owner.owner_type;
                if player_index != TankType::Enemy {
                    if let Some(player_stats) = player_info.players.get(&player_index) {
                        if player_stats.penetrate {
                            // 播放金属破碎音效
                            let metal_crash_sound: Handle<AudioSource> = asset_server.load("metal_crash.ogg");
                            commands.spawn(AudioPlayer::new(metal_crash_sound));

                            // 发送火花特效事件
                            effect_events.write(EffectEvent::Spark {
                                position: bullet_transform.translation,
                            });

                            // 销毁钢铁和标记子弹销毁
                            commands.entity(terrain_entity).despawn();
                            commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
                        } else {
                            // 没有 penetrate 效果，只播放击中音效
                            let hit_sound: Handle<AudioSource> = asset_server.load("hit_sound.ogg");
                            commands.spawn(AudioPlayer::new(hit_sound));

                            // 发送火花特效事件
                            effect_events.write(EffectEvent::Spark {
                                position: bullet_transform.translation,
                            });

                            // 只标记子弹销毁
                            commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
                        }
                    }
                } else {
                    // 敌方子弹，只播放击中音效并标记子弹销毁
                    let hit_sound: Handle<AudioSource> = asset_server.load("hit_sound.ogg");
                    commands.spawn(AudioPlayer::new(hit_sound));

                    // 发送火花特效事件
                    effect_events.write(EffectEvent::Spark {
                        position: bullet_transform.translation,
                    });

                    // 只标记子弹销毁
                    commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
                }
            }
        }
    }
}

/// 子弹与坦克碰撞检测系统
pub fn bullet_tank_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    _texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    enemy_tanks: Query<(), With<EnemyTank>>,
    enemy_tanks_with_transform: Query<(Entity, &Transform), With<EnemyTank>>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
    player_tanks_with_transform: Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: Query<(Entity, &PlayerUI)>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut controllers: Query<&mut KinematicCharacterController>,
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
                let bullet_owner_info = bullets.get(bullet_entity).unwrap().1;

                if should_bullet_destroy(bullet_owner_info.owner_type, tank_entity, &enemy_tanks, &player_tanks) {
                    // 检查是否是玩家子弹击中敌方坦克
                    let is_player_bullet = matches!(bullet_owner_info.owner_type, TankType::Player1 | TankType::Player2);
                    let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();
                    let is_player_tank = player_tanks.get(tank_entity).is_ok();

                    if is_player_bullet && is_enemy_tank {
                        // 获取敌方坦克的位置
                        if let Ok((_, tank_transform)) = enemy_tanks_with_transform.get(tank_entity) {
                            // 发送爆炸特效事件
                            effect_events.write(EffectEvent::Explosion {
                                position: tank_transform.translation,
                            });
                        }

                        // 销毁敌方坦克
                        commands.entity(tank_entity).despawn();

                        // 减少当前敌方坦克计数
                        enemy_spawn_state.active_count -= 1;

                        // 增加分数
                        let player_type = bullet_owner_info.owner_type;
                        if player_type == TankType::Enemy {
                            continue; // 敌方坦克不应该有这个分支
                        }
                        if let Some(player_stats) = player_info.players.get_mut(&player_type) {
                            player_stats.score += 100;

                            // 发送分数变更事件
                            stat_changed_events.write(PlayerStatChanged {
                                player_type: player_type,
                                stat_type: StatType::Score,
                            });
                        }
                    } else if !is_player_bullet && is_player_tank {
                        let player_index = match player_tanks.get(tank_entity) {
                            Ok(pt) => pt.tank_type,
                            Err(_) => continue,
                        };

                        // 敌方子弹击中玩家坦克
                        // 播放中弹音效
                        let hit_sound: Handle<AudioSource> = asset_server.load("hit_sound.ogg");
                        commands.spawn(AudioPlayer::new(hit_sound));

                        // 发送火花特效事件
                        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(tank_entity) {
                            effect_events.write(EffectEvent::Spark {
                                position: tank_transform.translation,
                            });
                        }

                        // 扣除对应玩家的生命值
                        if let Some(player_stats) = player_info.players.get_mut(&player_index) {
                            // 检查玩家是否有 fire_shell、track_chain、penetrate 或 air_cushion 特效
                            let has_fire_shell = player_stats.fire_shell;
                            let has_track_chain = player_stats.track_chain;
                            let has_penetrate = player_stats.penetrate;
                            let has_air_cushion = player_stats.air_cushion;

                            if has_fire_shell || has_track_chain || has_penetrate || has_air_cushion {
                                // 有特效，移除其中一个特效（优先级任意）
                                if has_fire_shell {
                                    player_stats.fire_shell = false;
                                    stat_changed_events.write(PlayerStatChanged {
                                        player_type: player_index,
                                        stat_type: StatType::FireShell,
                                    });
                                } else if has_track_chain {
                                    player_stats.track_chain = false;
                                    stat_changed_events.write(PlayerStatChanged {
                                        player_type: player_index,
                                        stat_type: StatType::TrackChain,
                                    });
                                } else if has_penetrate {
                                    player_stats.penetrate = false;
                                    stat_changed_events.write(PlayerStatChanged {
                                        player_type: player_index,
                                        stat_type: StatType::Penetrate,
                                    });
                                } else if has_air_cushion {
                                    player_stats.air_cushion = false;
                                    // 恢复 filter_groups，与海（GROUP_2）碰撞
                                    if let Ok(mut controller) = controllers.get_mut(tank_entity) {
                                        controller.filter_groups = None;
                                    }
                                    stat_changed_events.write(PlayerStatChanged {
                                        player_type: player_index,
                                        stat_type: StatType::AirCushion,
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
                                        // 发送爆炸特效事件
                                        effect_events.write(EffectEvent::Explosion {
                                            position: tank_transform.translation,
                                        });
                                    }

                                    // 销毁玩家坦克
                                    commands.entity(tank_entity).despawn();

                                    // 标记对应玩家的头像为死亡状态
                                    for (avatar_entity, player_idx) in player_avatars.iter() {
                                        if player_idx.player_type == player_index {
                                            commands.entity(avatar_entity).insert(PlayerDead);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // 标记子弹销毁
                    commands.entity(bullet_entity).insert(BulletDespawnMarker);
                }
            }
        }
    }
}

/// 特效处理系统
/// 监听特效事件并生成对应的视觉效果
pub fn handle_effect_events(
    mut events: MessageReader<EffectEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in events.read() {
        match event {
            EffectEvent::Explosion { position } => {
                crate::spawn_explosion(&mut commands, &asset_server, &mut texture_atlas_layouts, *position);
            }
            EffectEvent::Spark { position } => {
                crate::spawn_spark(&mut commands, &asset_server, *position);
            }
            EffectEvent::ForestFire { position } => {
                crate::spawn_forest_fire(&mut commands, &asset_server, &mut texture_atlas_layouts, *position);
            }
        }
    }
}