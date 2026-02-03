//! 子弹系统模块
//!
//! 处理子弹的生成、移动、碰撞检测和销毁逻辑

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::effects;
use crate::constants::*;
use crate::resources::{AmbienceResources, BulletTracker, BulletResources, EffectResources, PlayerInfo, PlayerStatChanged, StatType, TerrainAtlasLayouts, SoundResources};
use crate::utils;

/// 特效事件枚举
/// 用于解耦碰撞逻辑和特效生成
#[derive(Event, Clone, Copy, Message)]
pub enum EffectEvent {
    Explosion { position: Vec3 },
    Spark { position: Vec3 },
    ForestFire { position: Vec3 },
}

/// 子弹资源缓存
/// 用于预加载子弹纹理和音效，避免重复加载
/// 子弹实体标记组件（包含所有者信息）
#[derive(Component, Copy, Clone)]
pub enum Bullet {
    Player(TankType),
    Enemy,
}

impl Bullet {
    /// 获取子弹所有者类型
    pub fn owner_type(&self) -> TankType {
        match self {
            Bullet::Player(tank_type) => *tank_type,
            Bullet::Enemy => TankType::Enemy,
        }
    }

    /// 检查是否为玩家子弹
    pub fn is_player(&self) -> bool {
        matches!(self, Bullet::Player(_))
    }

    /// 检查是否为敌方子弹
    pub fn is_enemy(&self) -> bool {
        matches!(self, Bullet::Enemy)
    }
}

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
    bullet_resources: &BulletResources,
    params: BulletSpawnParams,
    player_info: &Res<PlayerInfo>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> Entity {
    // 根据坦克类型选择子弹纹理
    let bullet_texture = match params.owner_type {
        TankType::Player1 => bullet_resources.bullet_player1.clone(),
        TankType::Player2 => bullet_resources.bullet_player2.clone(),
        TankType::Enemy => bullet_resources.bullet_enemy.clone(),
    };

    // 检查玩家是否拥有 fire_shell 能力
    let has_fire_shell = player_info.has_fire_shell(params.owner_type);

    // 计算子弹旋转角度（纹理是横向的，需要根据射击方向旋转）
    // 假设纹理默认向右（0度），需要根据方向计算旋转角度
    let angle = params.direction.y.atan2(params.direction.x);
    let rotation = Quat::from_rotation_z(angle);

    // 生成子弹实体
    let bullet_type = if matches!(params.owner_type, TankType::Enemy) {
        Bullet::Enemy
    } else {
        Bullet::Player(params.owner_type)
    };

    let bullet_entity = commands
        .spawn((
            bullet_type,
            PlayingEntity,
            Sprite {
                image: bullet_texture,
                custom_size: Some(Vec2::new(BULLET_WIDTH, BULLET_HEIGHT)), // 子弹尺寸：长60像素，宽40像素
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
            Collider::cuboid(BULLET_WIDTH / 2.0, BULLET_HEIGHT / 2.0), // 使用矩形碰撞体匹配子弹尺寸
            LockedAxes::ROTATION_LOCKED,
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::default()
                | ActiveCollisionTypes::KINEMATIC_KINEMATIC
                | ActiveCollisionTypes::KINEMATIC_STATIC,
        ))
        .id();

    // 如果玩家有 fire_shell 效果，添加火焰特效子实体
    if has_fire_shell {
        let fire_effect_tile_size = UVec2::new(FIRE_EFFECT_TILE_WIDTH as u32, FIRE_EFFECT_TILE_HEIGHT as u32);
        let fire_effect_atlas_layout = utils::create_texture_atlas(fire_effect_tile_size, FIRE_EFFECT_COLUMNS as u32, FIRE_EFFECT_ROWS as u32);
        let fire_effect_atlas = texture_atlas_layouts.add(fire_effect_atlas_layout);
        let animation_indices = AnimationIndices {
            first: 0,
            last: crate::constants::FIRE_EFFECT_TOTAL_FRAMES - 1,
        };

        commands.entity(bullet_entity).with_children(|parent| {
            parent.spawn((
                crate::constants::FireEffect,
                Sprite::from_atlas_image(
                    bullet_resources.bullet_fire_effect.clone(),
                    TextureAtlas {
                        layout: fire_effect_atlas,
                        index: animation_indices.first,
                    },
                ),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.1), // 略高于子弹
                    rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), // 旋转 90 度
                    scale: Vec3::splat(1.65), // 放大到 165% (因为纹理缩小到33%)
                },
                animation_indices,
                AnimationTimer(Timer::from_seconds(
                    crate::constants::FIRE_EFFECT_ANIMATION_FRAME,
                    TimerMode::Repeating,
                )),
                CurrentAnimationFrame(0),
            ));
        });
    }

    // 检查玩家是否拥有 penetrate 能力
    let has_penetrate = player_info.has_penetrate(params.owner_type);

    // 如果玩家有 penetrate 效果，添加穿透特效子实体
    if has_penetrate {
        let penetrate_effect_tile_size = UVec2::new(PENETRATE_EFFECT_TILE_WIDTH as u32, PENETRATE_EFFECT_TILE_HEIGHT as u32);
        let penetrate_effect_atlas_layout = utils::create_texture_atlas(penetrate_effect_tile_size, PENETRATE_EFFECT_COLUMNS as u32, PENETRATE_EFFECT_ROWS as u32);
        let penetrate_effect_atlas = texture_atlas_layouts.add(penetrate_effect_atlas_layout);
        let animation_indices = AnimationIndices {
            first: 0,
            last: crate::constants::PENETRATE_EFFECT_TOTAL_FRAMES - 1,
        };

        commands.entity(bullet_entity).with_children(|parent| {
            parent.spawn((
                PenetrateEffect,
                Sprite::from_atlas_image(
                    bullet_resources.bullet_penetrate_effect.clone(),
                    TextureAtlas {
                        layout: penetrate_effect_atlas,
                        index: animation_indices.first,
                    },
                ),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.2), // 略高于火焰特效
                    rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2), // 旋转 -90 度
                    scale: Vec3::splat(0.3), // 放大到 0.3 倍 (因为纹理缩小到33%)
                },
                animation_indices,
                AnimationTimer(Timer::from_seconds(
                    crate::constants::PENETRATE_EFFECT_ANIMATION_FRAME,
                    TimerMode::Repeating,
                )),
                CurrentAnimationFrame(0),
            ));
        });
    }

    bullet_entity
}

/// 敌方坦克射击系统
pub fn enemy_shoot_system(
    mut commands: Commands,
    bullet_resources: Res<BulletResources>,
    player_info: Res<PlayerInfo>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &crate::constants::EnemyTank,
            &TankFireConfig,
        ),
        With<EnemyTank>,
    >,
    mut bullet_tracker: ResMut<BulletTracker>,
) {
    for (entity, transform, enemy_tank, fire_config) in &mut query {
        // 检查是否可以射击
        if !bullet_tracker.can_fire(entity, fire_config.max_bullets) {
            continue;
        }

        // 随机射击，每帧有 1.0% 的概率射击
        let mut rng = rand::rng();
        if rng.random::<f32>() < ENEMY_SHOOT_PROBABILITY {
            // 计算子弹发射方向（基于坦克的预期朝向 direction）
            let direction = if enemy_tank.direction.length() > 0.0 {
                enemy_tank.direction.normalize()
            } else {
                Vec2::new(0.0, -1.0) // 默认向下
            };

            // 计算子弹初始位置（坦克前方）
            let bullet_pos =
                transform.translation + direction.extend(0.0) * (PLAYER_TANK_DISPLAY_HEIGHT / 2.0 + BULLET_SIZE);

            // 生成子弹
            let bullet_entity = spawn_bullet(
                &mut commands,
                &bullet_resources,
                BulletSpawnParams {
                    position: bullet_pos,
                    direction,
                    speed: BULLET_SPEED,
                    owner_type: TankType::Enemy,
                },
                &player_info,
                &mut texture_atlas_layouts,
            );

            // 记录子弹的所有者
            bullet_tracker.add_bullet(bullet_entity, entity);
        }
    }
}

/// 玩家坦克射击系统
pub fn player_shoot_system(
    mut commands: Commands,
    bullet_resources: Res<BulletResources>,
    time: Res<Time>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &RotationTimer,
            &PlayerTank,
            &mut TankFireConfig,
            Option<&Children>,
        ),
        With<PlayerTank>,
    >,
    mut bullet_tracker: ResMut<BulletTracker>,
    player_info: Res<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    barrel_query: Query<(), With<Barrel>>,
    sound_resources: Res<SoundResources>,
) {
    for (entity, transform, rotation_timer, player_tank, mut fire_config, children) in &mut query {
        // 检查是否正在旋转
        if rotation_timer.0.elapsed() < rotation_timer.0.duration() {
            continue;
        }

        // 更新射击冷却时间
        fire_config.cooldown.tick(time.delta());
        if !fire_config.cooldown.is_finished() {
            continue;
        }

        // 检查是否按下射击键
        let key_bindings = player_tank.tank_type.get_key_bindings();

        if !key_bindings.is_shooting(&keyboard) {
            continue;
        }

        // 获取玩家属性
        let player_stats = player_info.get_stats(player_tank.tank_type).expect("Player should exist");

        // 检查是否可以射击（使用 player_stats.shells 作为最大子弹数）
        if !bullet_tracker.can_fire(entity, player_stats.shells) {
            continue;
        }

        // 计算子弹发射方向（基于坦克当前的旋转角度）
        // 坦克旋转时使用：angle - 90.0_f32.to_radians()
        // 因此需要补偿：actual_angle = euler_angle + 90.0_f32.to_radians()
        let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
        let actual_angle = euler_angle + ANGLE_OFFSET_DEGREES.to_radians();
        let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

        // 计算子弹初始位置（坦克前方）
        let bullet_pos =
            transform.translation + direction.extend(0.0) * (PLAYER_TANK_DISPLAY_HEIGHT / 2.0 + BULLET_SIZE);

        // 玩家子弹速度 = PLAYER_BULLET_SPEED × (1 + fire_speed百分比/100)
        let fire_speed_bonus = player_stats.fire_speed as f32 / 100.0;
        let bullet_speed = PLAYER_BULLET_SPEED * (1.0 + fire_speed_bonus);

        // 生成子弹
        let bullet_entity = spawn_bullet(
            &mut commands,
            &bullet_resources,
            BulletSpawnParams {
                position: bullet_pos,
                direction,
                speed: bullet_speed,
                owner_type: player_tank.tank_type,
            },
            &player_info,
            &mut texture_atlas_layouts,
        );

        // 记录子弹的所有者
        bullet_tracker.add_bullet(bullet_entity, entity);

        // 播放玩家射击音效，音量 0.4
        sound_resources.play(&mut commands, sound_resources.player_shot.clone(), 0.4);

        // 给炮管添加后坐力效果
        if let Some(children) = children {
            for child in children {
                if barrel_query.get(*child).is_ok() {
                    commands.entity(*child).insert(BarrelRecoilForce {
                        timer: Timer::from_seconds(BARREL_RECOIL_DURATION, TimerMode::Once),
                    });
                    break;
                }
            }
        }

        // 重置冷却时间
        fire_config.cooldown.reset();
    }
}

/// 销毁子弹实体并清理所有者引用
fn despawn_bullet(commands: &mut Commands, bullet_tracker: &mut BulletTracker, bullet_entity: Entity) {
    bullet_tracker.remove_bullet(bullet_entity);
    let () = commands.entity(bullet_entity).try_despawn();
}

/// 子弹边界检查系统
pub fn bullet_bounds_check_system(
    mut commands: Commands,
    mut bullet_tracker: ResMut<BulletTracker>,
    mut query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in &mut query {
        let x = transform.translation.x;
        let y = transform.translation.y;

        // 检查子弹是否超出游戏窗口边界
        if !(MAP_LEFT_X..=MAP_RIGHT_X).contains(&x) || !(MAP_BOTTOM_Y..=MAP_TOP_Y).contains(&y) {
            despawn_bullet(&mut commands, &mut bullet_tracker, entity);
        }
    }
}

/// 查找碰撞中的子弹和坦克，同时返回子弹信息
pub fn find_bullet_and_tank_in_collision<'a>(
    e1: Entity,
    e2: Entity,
    bullets: &'a Query<(Entity, &Bullet, &Transform), With<Bullet>>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    player_tanks: &Query<(&PlayerTank, &Transform), With<PlayerTank>>,
) -> Option<(Entity, Entity, &'a Bullet)> {
    if let Ok((_, bullet, _)) = bullets.get(e1) {
        if enemy_tanks.get(e2).is_ok() || player_tanks.get(e2).is_ok() {
            return Some((e1, e2, bullet));
        }
    }
    if let Ok((_, bullet, _)) = bullets.get(e2) {
        if enemy_tanks.get(e1).is_ok() || player_tanks.get(e1).is_ok() {
            return Some((e2, e1, bullet));
        }
    }
    None
}

/// 子弹与地形碰撞检测系统
/// 使用 Rapier 碰撞事件进行碰撞检测
pub fn bullet_terrain_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    bullets: Query<(Entity, &Bullet, &Transform), With<Bullet>>,
    forests: Query<(Entity, &Transform), With<Forest>>,
    bricks: Query<(), With<Brick>>,
    steels: Query<(), With<Steel>>,
    despawned_entities: Query<(), With<DespawnMarker>>,
    player_info: Res<PlayerInfo>,
    mut bullet_tracker: ResMut<BulletTracker>,
    sound_resources: Res<SoundResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 内联提取子弹和地形实体
        let (bullet_entity, terrain_entity, bullet, bullet_transform) =
            if let Ok((_, b, t)) = bullets.get(*e1) {
                (*e1, *e2, b, t)
            } else if let Ok((_, b, t)) = bullets.get(*e2) {
                (*e2, *e1, b, t)
            } else {
                continue;
            };

        // 跳过已被标记销毁的实体（被激光击中的）
        if despawned_entities.contains(terrain_entity) {
            continue;
        }

        // 处理森林碰撞
        if let Ok((forest_entity, forest_transform)) = forests.get(terrain_entity) {
            if bullet.is_player() {
                if let Some(player_stats) = player_info.get_stats(bullet.owner_type()) {
                    if player_stats.fire_shell {
                        effect_events.write(EffectEvent::ForestFire {
                            position: forest_transform.translation,
                        });
                        let () = commands.entity(forest_entity).try_despawn();
                    }
                }
            }
            continue;
        }

        // 处理砖块碰撞
        if bricks.get(terrain_entity).is_ok() {
            sound_resources.play(&mut commands, sound_resources.brick_hit.clone(), VOLUME_HALF);
            effect_events.write(EffectEvent::Spark {
                position: bullet_transform.translation,
            });
            let () = commands.entity(terrain_entity).try_despawn();
            despawn_bullet(&mut commands, &mut bullet_tracker, bullet_entity);
            continue;
        }

        // 处理钢铁碰撞
        if steels.get(terrain_entity).is_ok() {
            effect_events.write(EffectEvent::Spark {
                position: bullet_transform.translation,
            });

            if bullet.is_enemy() {
                utils::play_one_shot_sound(&mut commands, sound_resources.hit.clone(), 1.0);
            } else if let Some(player_stats) = player_info.get_stats(bullet.owner_type()) {
                if player_stats.penetrate {
                    sound_resources.play(&mut commands, sound_resources.metal_crash.clone(), 1.0);
                    let () = commands.entity(terrain_entity).try_despawn();
                } else {
                    sound_resources.play(&mut commands, sound_resources.hit.clone(), 1.0);
                }
            }

            despawn_bullet(&mut commands, &mut bullet_tracker, bullet_entity);
        }
    }
}



/// 子弹与坦克碰撞检测系统
pub fn bullet_tank_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    mut bullet_tracker: ResMut<BulletTracker>,
    bullets: Query<(Entity, &Bullet, &Transform), With<Bullet>>,
    enemy_tanks: Query<(Entity, &Transform), With<EnemyTank>>,
    player_tanks: Query<(&PlayerTank, &Transform), With<PlayerTank>>,
    despawned_entities: Query<(), With<DespawnMarker>>,
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut controllers: Query<&mut KinematicCharacterController>,
    sound_resources: Res<SoundResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 提取子弹和坦克实体，同时获取子弹信息
        let Some((bullet_entity, tank_entity, bullet)) =
            find_bullet_and_tank_in_collision(*e1, *e2, &bullets, &enemy_tanks, &player_tanks)
        else { continue; };

        // 跳过已被标记销毁的实体（被激光击中的）
        if despawned_entities.contains(tank_entity) {
            continue;
        }

        // 玩家子弹击中敌方坦克
        if bullet.is_player() && enemy_tanks.get(tank_entity).is_ok() {
            let player_type = bullet.owner_type();
            // 生成爆炸特效
            if let Ok((_, tank_transform)) = enemy_tanks.get(tank_entity) {
                effect_events.write(EffectEvent::Explosion {
                    position: tank_transform.translation,
                });
            }
            sound_resources.play(&mut commands, sound_resources.hit.clone(), 1.0);
            let () = commands.entity(tank_entity).try_despawn();
            // 增加分数
            if let Some(player_stats) = player_info.get_stats_mut(player_type) {
                player_stats.score += 100;
                stat_changed_events.write(PlayerStatChanged {
                    player_type,
                    stat_type: StatType::Score,
                });
            }
            despawn_bullet(&mut commands, &mut bullet_tracker, bullet_entity);
            continue;
        }

        // 敌方子弹击中玩家坦克
        if bullet.is_enemy() {
            if let Ok((player_tank, tank_transform)) = player_tanks.get(tank_entity) {
                let player_type = player_tank.tank_type;

                sound_resources.play(&mut commands, sound_resources.hit.clone(), 1.0);
                effect_events.write(EffectEvent::Spark {
                    position: tank_transform.translation,
                });

                if let Some(player_stats) = player_info.get_stats_mut(player_type) {
                    // 按优先级移除道具：fire_shell > track_chain > penetrate > air_cushion > shells
                    let has_fire_shell = player_stats.fire_shell;
                    let has_track_chain = player_stats.track_chain;
                    let has_penetrate = player_stats.penetrate;
                    let has_air_cushion = player_stats.air_cushion;
                    let has_shells = player_stats.shells > 1;

                    if has_fire_shell {
                        player_stats.fire_shell = false;
                        stat_changed_events.write(PlayerStatChanged { player_type, stat_type: StatType::FireShell });
                    } else if has_track_chain {
                        player_stats.track_chain = false;
                        stat_changed_events.write(PlayerStatChanged { player_type, stat_type: StatType::TrackChain });
                    } else if has_penetrate {
                        player_stats.penetrate = false;
                        stat_changed_events.write(PlayerStatChanged { player_type, stat_type: StatType::Penetrate });
                    } else if has_air_cushion {
                        player_stats.air_cushion = false;
                        if let Ok(mut controller) = controllers.get_mut(tank_entity) {
                            controller.filter_groups = None;
                        }
                        stat_changed_events.write(PlayerStatChanged { player_type, stat_type: StatType::AirCushion });
                    } else if has_shells {
                        player_stats.shells -= 1;
                        stat_changed_events.write(PlayerStatChanged { player_type, stat_type: StatType::Shell });
                    } else {
                        // 造成伤害
                        if player_stats.life_points > 0 {
                            player_stats.life_points -= 1;
                        }
                        if player_stats.life_points == 0 {
                            effect_events.write(EffectEvent::Explosion {
                                position: tank_transform.translation,
                            });
                            let () = commands.entity(tank_entity).try_despawn();
                        }
                    }
                }

                despawn_bullet(&mut commands, &mut bullet_tracker, bullet_entity);
            }
        }
    }
}



/// 特效处理系统
/// 监听特效事件并生成对应的视觉效果
pub fn handle_effect_events(
    mut events: MessageReader<EffectEvent>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    terrain_atlas_layouts: Res<TerrainAtlasLayouts>,
    effect_resources: Res<EffectResources>,
    sound_resources: Res<SoundResources>,
    ambience_resources: Res<AmbienceResources>,
) {
    for event in events.read() {
        match event {
            EffectEvent::Explosion { position } => {
                effects::spawn_explosion(
                    &mut commands,
                    &mut texture_atlas_layouts,
                    &effect_resources,
                    &sound_resources,
                    *position,
                );
            }
            EffectEvent::Spark { position } => {
                effects::spawn_spark(
                    &mut commands,
                    &mut texture_atlas_layouts,
                    &effect_resources,
                    *position,
                );
            }
            EffectEvent::ForestFire { position } => {
                effects::spawn_forest_fire(
                    &mut commands,
                    &terrain_atlas_layouts,
                    &effect_resources,
                    &ambience_resources,
                    *position,
                );
            }
        }
    }
}

/// 司令官与敌方子弹碰撞检测系统
pub fn bullet_commander_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    bullets: Query<(Entity, &Bullet, &Transform), With<Bullet>>,
    commanders: Query<(Entity, &Transform), With<crate::constants::Commander>>,
    mut commander_life: ResMut<crate::resources::CommanderLife>,
    mut bullet_tracker: ResMut<BulletTracker>,
    sound_resources: Res<SoundResources>,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = event else { continue };

        // 判断是否是子弹与司令官的碰撞，同时获取子弹和司令官信息
        let (bullet_entity, bullet, bullet_transform) = if let (Ok((_, b, t)), Ok(_)) =
            (bullets.get(*e1), commanders.get(*e2))
        {
            (*e1, b, t)
        } else if let (Ok((_, b, t)), Ok(_)) =
            (bullets.get(*e2), commanders.get(*e1))
        {
            (*e2, b, t)
        } else {
            continue;
        };

        // 只处理敌方子弹击中司令官的情况
        if !bullet.is_enemy() {
            continue;
        }

        // 碰撞确认，播放受击音效
        sound_resources.play(&mut commands, sound_resources.commander_get_shot.clone(), 1.0);

        // 发送火花特效事件
        effect_events.write(EffectEvent::Spark {
            position: bullet_transform.translation,
        });

        // 减少司令官生命值
        if commander_life.life_points > 0 {
            commander_life.life_points -= 1;
        }

        // 检查是否是致命伤（生命值归零）
        if commander_life.life_points == 0 {
            // 播放死亡音效
            sound_resources.play(&mut commands, sound_resources.commander_death.clone(), 1.0);
        }

        despawn_bullet(&mut commands, &mut bullet_tracker, bullet_entity);
    }
}

/// 火焰特效动画系统
/// 播放叠加在子弹上的火焰特效精灵图动画
pub fn animate_fire_shell_bullet(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<crate::constants::FireEffect>,
    >,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            let current = current_frame.0;
            let next_index = if current == indices.last {
                indices.first
            } else {
                current + 1
            };
            current_frame.0 = next_index;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = next_index;
            }
        }
    }
}

/// 穿透特效动画系统
/// 播放叠加在子弹上的穿透特效精灵图动画
pub fn animate_penetrate_bullet(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<crate::constants::PenetrateEffect>,
    >,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            let current = current_frame.0;
            let next_index = if current == indices.last {
                indices.first
            } else {
                current + 1
            };
            current_frame.0 = next_index;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = next_index;
            }
        }
    }
}
