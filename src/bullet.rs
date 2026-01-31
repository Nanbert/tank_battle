//! 子弹系统模块
//!
//! 处理子弹的生成、移动、碰撞检测和销毁逻辑

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::effects;

use crate::constants::*;
use crate::resources::{AmbienceResources, BulletTracker, BulletResources, EffectResources, PlayerInfo, PlayerStatChanged, PlayerStats, StatType, TerrainAtlasLayouts, SoundResources};
use bevy::audio::Volume;

/// 获取玩家统计数据的不可变引用
fn get_player_stats_ref<'a>(
    player_info: &'a PlayerInfo,
    player_type: TankType,
) -> &'a PlayerStats {
    match player_type {
        TankType::Player1 => &player_info.player1,
        TankType::Player2 => player_info.player2.as_ref().expect("Player2 should exist"),
        TankType::Enemy => unreachable!("Enemy tank should not request player stats"),
    }
}

/// 获取玩家统计数据的可变引用
fn get_player_stats_mut<'a>(
    player_info: &'a mut PlayerInfo,
    player_type: TankType,
) -> &'a mut PlayerStats {
    match player_type {
        TankType::Player1 => &mut player_info.player1,
        TankType::Player2 => player_info.player2.as_mut().expect("Player2 should exist"),
        TankType::Enemy => unreachable!("Enemy tank should not request player stats"),
    }
}

/// 碰撞类型枚举
#[derive(Clone, Copy, PartialEq, Eq)]
enum CollisionType {
    PlayerBulletHitEnemy,
    EnemyBulletHitPlayer,
}

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
/// 子弹实体标记组件
#[derive(Component)]
pub struct Bullet;

/// 子弹所有者组件
#[derive(Component)]
pub struct BulletOwner {
    pub owner_type: TankType,
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
) -> Entity {
    // 根据坦克类型选择子弹纹理
    let bullet_texture = match params.owner_type {
        TankType::Player1 => bullet_resources.bullet_player1.clone(),
        TankType::Player2 => bullet_resources.bullet_player2.clone(),
        TankType::Enemy => bullet_resources.bullet_enemy.clone(),
    };

    // 计算子弹旋转角度（纹理是横向的，需要根据射击方向旋转）
    // 假设纹理默认向右（0度），需要根据方向计算旋转角度
    let angle = params.direction.y.atan2(params.direction.x);
    let rotation = Quat::from_rotation_z(angle);

    commands
        .spawn((
            Bullet,
            PlayingEntity,
            BulletOwner {
                owner_type: params.owner_type,
            },
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
        .id()
}

/// 敌方坦克射击系统
pub fn enemy_shoot_system(
    mut commands: Commands,
    bullet_resources: Res<BulletResources>,
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
    mut bullet_tracker: ResMut<BulletTracker>,
    player_info: Res<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (entity, transform, rotation_timer, player_tank, mut fire_config) in &mut query {
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
        let shoot_key = match player_tank.tank_type {
            TankType::Player1 => KeyCode::KeyJ,
            TankType::Player2 => KeyCode::Numpad1,
            TankType::Enemy => continue,
        };

        if !keyboard.pressed(shoot_key) {
            continue;
        }

        // 获取玩家属性
        let player_stats = match player_tank.tank_type {
            TankType::Player1 => &player_info.player1,
            TankType::Player2 => player_info.player2.as_ref().expect("Player2 should exist"),
            TankType::Enemy => unreachable!(),
        };

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
        );

        // 记录子弹的所有者
        bullet_tracker.add_bullet(bullet_entity, entity);

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

/// 查找碰撞中的子弹和坦克
pub fn find_bullet_and_tank_in_collision(
    e1: Entity,
    e2: Entity,
    bullets: &Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
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

/// 获取碰撞类型
/// 返回 None 表示子弹应该穿过（不销毁）
fn get_collision_type(
    bullet_owner_type: TankType,
    tank_entity: Entity,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    player_tanks: &Query<&PlayerTank, With<PlayerTank>>,
) -> Option<CollisionType> {
    let is_player_tank = player_tanks.get(tank_entity).is_ok();
    let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();

    let is_player_bullet = matches!(bullet_owner_type, TankType::Player1 | TankType::Player2);

    // 规则：
    // 1. 玩家子弹打到敌方坦克 -> 子弹消失
    // 2. 敌方子弹打到玩家坦克 -> 子弹消失
    // 3. 敌方子弹打到敌方坦克 -> 子弹穿过（不消失）
    // 4. 玩家子弹打到玩家坦克 -> 子弹穿过（不消失）
    match (is_player_bullet, is_enemy_tank, is_player_tank) {
        (true, true, false) => Some(CollisionType::PlayerBulletHitEnemy),
        (false, false, true) => Some(CollisionType::EnemyBulletHitPlayer),
        _ => None,
    }
}

/// 子弹与地形碰撞检测系统
/// 使用 Rapier 碰撞事件进行碰撞检测
pub fn bullet_terrain_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    forests: Query<(Entity, &Transform), With<Forest>>,
    bricks: Query<(), With<Brick>>,
    steels: Query<(), With<Steel>>,
    player_info: Res<PlayerInfo>,
    mut bullet_tracker: ResMut<BulletTracker>,
    sound_resources: Res<SoundResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 提取子弹和地形实体
        let Some((bullet_entity, terrain_entity)) = extract_bullet_and_terrain(*e1, *e2, &bullets) else {
            continue;
        };

        // 获取子弹信息
        let Ok((_, bullet_owner, bullet_transform)) = bullets.get(bullet_entity) else {
            continue;
        };

        // 处理不同地形类型的碰撞
        handle_terrain_collision(
            &mut commands,
            &mut effect_events,
            terrain_entity,
            bullet_entity,
            bullet_transform,
            &bullet_owner,
            &forests,
            &bricks,
            &steels,
            &player_info,
            &mut bullet_tracker,
            &sound_resources,
        );
    }
}

/// 提取子弹和地形实体
fn extract_bullet_and_terrain(
    e1: Entity,
    e2: Entity,
    bullets: &Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
) -> Option<(Entity, Entity)> {
    if bullets.get(e1).is_ok() {
        Some((e1, e2))
    } else if bullets.get(e2).is_ok() {
        Some((e2, e1))
    } else {
        None
    }
}

/// 处理地形碰撞
fn handle_terrain_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    terrain_entity: Entity,
    bullet_entity: Entity,
    bullet_transform: &Transform,
    bullet_owner: &BulletOwner,
    forests: &Query<(Entity, &Transform), With<Forest>>,
    bricks: &Query<(), With<Brick>>,
    steels: &Query<(), With<Steel>>,
    player_info: &Res<PlayerInfo>,
    bullet_tracker: &mut BulletTracker,
    sound_resources: &SoundResources,
) {
    // 处理森林碰撞
    if let Ok((forest_entity, forest_transform)) = forests.get(terrain_entity) {
        handle_forest_collision(
            commands,
            effect_events,
            forest_entity,
            forest_transform,
            bullet_owner.owner_type,
            player_info,
        );
        return;
    }

    // 处理砖块碰撞
    if bricks.get(terrain_entity).is_ok() {
        handle_brick_collision(
            commands,
            effect_events,
            terrain_entity,
            bullet_entity,
            bullet_transform,
            bullet_tracker,
            sound_resources,
        );
        return;
    }

    // 处理钢铁碰撞
    if steels.get(terrain_entity).is_ok() {
        handle_steel_collision(
            commands,
            effect_events,
            terrain_entity,
            bullet_entity,
            bullet_transform,
            bullet_owner.owner_type,
            player_info,
            bullet_tracker,
            sound_resources,
        );
    }
}

/// 处理森林碰撞
fn handle_forest_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    forest_entity: Entity,
    forest_transform: &Transform,
    owner_type: TankType,
    player_info: &Res<PlayerInfo>,
) {
    let is_player_bullet = !matches!(owner_type, TankType::Enemy);
    if !is_player_bullet {
        return;
    }

    let player_stats = get_player_stats_ref(&player_info, owner_type);

    if !player_stats.fire_shell {
        return;
    }

    effect_events.write(EffectEvent::ForestFire {
        position: forest_transform.translation,
    });
    let () = commands.entity(forest_entity).try_despawn();
}

/// 处理砖块碰撞
fn handle_brick_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    brick_entity: Entity,
    bullet_entity: Entity,
    bullet_transform: &Transform,
    bullet_tracker: &mut BulletTracker,
    sound_resources: &SoundResources,
) {
    commands.spawn((
        AudioPlayer::new(sound_resources.brick_hit.clone()),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(VOLUME_HALF)),
    ));

    effect_events.write(EffectEvent::Spark {
        position: bullet_transform.translation,
    });

    let () = commands.entity(brick_entity).try_despawn();

    despawn_bullet(commands, bullet_tracker, bullet_entity);
}

/// 处理钢铁碰撞
fn handle_steel_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    steel_entity: Entity,
    bullet_entity: Entity,
    bullet_transform: &Transform,
    owner_type: TankType,
    player_info: &Res<PlayerInfo>,
    bullet_tracker: &mut BulletTracker,
    sound_resources: &SoundResources,
) {
    effect_events.write(EffectEvent::Spark {
        position: bullet_transform.translation,
    });

    if matches!(owner_type, TankType::Enemy) {
        commands.spawn(AudioPlayer::new(sound_resources.hit.clone()));
        despawn_bullet(commands, bullet_tracker, bullet_entity);
        return;
    }

    let player_stats = get_player_stats_ref(&player_info, owner_type);

    if player_stats.penetrate {
        commands.spawn(AudioPlayer::new(sound_resources.metal_crash.clone()));
        let () = commands.entity(steel_entity).try_despawn();
    } else {
        commands.spawn(AudioPlayer::new(sound_resources.hit.clone()));
    }

    despawn_bullet(commands, bullet_tracker, bullet_entity);
}

/// 子弹与坦克碰撞检测系统
pub fn bullet_tank_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    mut bullet_tracker: ResMut<BulletTracker>,
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    enemy_tanks_with_transform: Query<(Entity, &Transform), With<EnemyTank>>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
    player_tanks_with_transform: Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: Query<(Entity, &PlayerUI)>,
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut controllers: Query<&mut KinematicCharacterController>,
    sound_resources: Res<SoundResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 提取子弹和坦克实体
        let Some((bullet_entity, tank_entity)) =
            find_bullet_and_tank_in_collision(*e1, *e2, &bullets, &enemy_tanks_with_transform, &player_tanks)
        else { continue; };

        let bullet_owner_info = bullets.get(bullet_entity).expect("Bullet should exist in bullets query").1;

        // 获取碰撞类型，如果返回 None 则子弹穿过（不销毁）
        let collision_type = match get_collision_type(
            bullet_owner_info.owner_type,
            tank_entity,
            &enemy_tanks_with_transform,
            &player_tanks,
        ) {
            Some(ct) => ct,
            None => continue,
        };

        match collision_type {
            CollisionType::PlayerBulletHitEnemy => {
                handle_player_bullet_hit_enemy(
                    &mut commands,
                    &mut effect_events,
                    &enemy_tanks_with_transform,
                    &mut player_info,
                    &mut stat_changed_events,
                    bullet_owner_info.owner_type,
                    tank_entity,
                    &sound_resources,
                );
            }
            CollisionType::EnemyBulletHitPlayer => {
                let player_index = player_tanks.get(tank_entity).expect("PlayerTank should have tank_type").tank_type;
                handle_enemy_bullet_hit_player(
                    &mut commands,
                    &mut effect_events,
                    &sound_resources,
                    &player_tanks_with_transform,
                    &player_avatars,
                    &mut player_info,
                    &mut stat_changed_events,
                    &mut controllers,
                    player_index,
                    tank_entity,
                );
            }
        }

        despawn_bullet(&mut commands, &mut bullet_tracker, bullet_entity);
    }
}

/// 处理玩家子弹击中敌方坦克
fn handle_player_bullet_hit_enemy(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    enemy_tanks_with_transform: &Query<(Entity, &Transform), With<EnemyTank>>,
    mut player_info: &mut ResMut<PlayerInfo>,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    player_type: TankType,
    tank_entity: Entity,
    sound_resources: &SoundResources,
) {
    // 卫语句：敌方坦克不应该有这个分支
    if player_type == TankType::Enemy {
        return;
    }

    // 获取敌方坦克位置并生成爆炸特效
    if let Ok((_, tank_transform)) = enemy_tanks_with_transform.get(tank_entity) {
        effect_events.write(EffectEvent::Explosion {
            position: tank_transform.translation,
        });
    }

    // 播放中弹音效
    commands.spawn(AudioPlayer::new(sound_resources.hit.clone()));

    // 销毁敌方坦克
    let () = commands.entity(tank_entity).try_despawn();

    // 增加分数
    let player_stats = get_player_stats_mut(&mut player_info, player_type);
    player_stats.score += 100;
    stat_changed_events.write(PlayerStatChanged {
        player_type,
        stat_type: StatType::Score,
    });
}

/// 处理敌方子弹击中玩家坦克
fn handle_enemy_bullet_hit_player(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    sound_resources: &SoundResources,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: &Query<(Entity, &PlayerUI)>,
    mut player_info: &mut ResMut<PlayerInfo>,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    controllers: &mut Query<&mut KinematicCharacterController>,
    player_index: TankType,
    tank_entity: Entity,
) {
    // 播放中弹音效
    commands.spawn(AudioPlayer::new(sound_resources.hit.clone()));

    // 发送火花特效事件
    if let Ok((_, tank_transform)) = player_tanks_with_transform.get(tank_entity) {
        effect_events.write(EffectEvent::Spark {
            position: tank_transform.translation,
        });
    }

    // 获取玩家统计数据
    let player_stats = get_player_stats_mut(&mut player_info, player_index);

    // 检查玩家是否有特效或额外子弹
    let has_fire_shell = player_stats.fire_shell;
    let has_track_chain = player_stats.track_chain;
    let has_penetrate = player_stats.penetrate;
    let has_air_cushion = player_stats.air_cushion;
    let has_shells = player_stats.shells > 1;

    if has_fire_shell || has_track_chain || has_penetrate || has_air_cushion || has_shells {
        remove_player_powerup(
            player_stats,
            player_index,
            stat_changed_events,
            controllers,
            tank_entity,
            has_fire_shell,
            has_track_chain,
            has_penetrate,
            has_air_cushion,
            has_shells,
        );
    } else {
        damage_player_tank(
            commands,
            effect_events,
            player_tanks_with_transform,
            player_avatars,
            player_stats,
            player_index,
            tank_entity,
        );
    }
}

/// 移除玩家道具
/// 按优先级移除：fire_shell > track_chain > penetrate > air_cushion > shells
fn remove_player_powerup(
    player_stats: &mut PlayerStats,
    player_index: TankType,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    controllers: &mut Query<&mut KinematicCharacterController>,
    tank_entity: Entity,
    has_fire_shell: bool,
    has_track_chain: bool,
    has_penetrate: bool,
    has_air_cushion: bool,
    has_shells: bool,
) {
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
        if let Ok(mut controller) = controllers.get_mut(tank_entity) {
            controller.filter_groups = None;
        }
        stat_changed_events.write(PlayerStatChanged {
            player_type: player_index,
            stat_type: StatType::AirCushion,
        });
    } else if has_shells {
        player_stats.shells -= 1;
        stat_changed_events.write(PlayerStatChanged {
            player_type: player_index,
            stat_type: StatType::Shell,
        });
    }
}

/// 对玩家坦克造成伤害
fn damage_player_tank(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: &Query<(Entity, &PlayerUI)>,
    player_stats: &mut PlayerStats,
    player_index: TankType,
    tank_entity: Entity,
) {
    if player_stats.life_points > 0 {
        player_stats.life_points -= 1;
    }

    if player_stats.life_points == 0 {
        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(tank_entity) {
            effect_events.write(EffectEvent::Explosion {
                position: tank_transform.translation,
            });
        }

        let () = commands.entity(tank_entity).try_despawn();

        for (avatar_entity, player_idx) in player_avatars.iter() {
            if player_idx.player_type == player_index {
                commands.entity(avatar_entity).insert(PlayerDead);
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
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    commanders: Query<(Entity, &Transform), With<crate::constants::Commander>>,
    mut commander_life: ResMut<crate::resources::CommanderLife>,
    mut bullet_tracker: ResMut<BulletTracker>,
    sound_resources: Res<SoundResources>,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = event else { continue };

        // 判断是否是子弹与司令官的碰撞
        let (bullet_entity, commander_entity) =
            if bullets.get(*e1).is_ok() && commanders.get(*e2).is_ok() {
                (*e1, *e2)
            } else if bullets.get(*e2).is_ok() && commanders.get(*e1).is_ok() {
                (*e2, *e1)
            } else {
                continue;
            };

        // 获取子弹信息
        let Ok((_, bullet_owner_info, bullet_transform)) = bullets.get(bullet_entity) else {
            continue;
        };

        // 只处理敌方子弹击中司令官的情况
        if bullet_owner_info.owner_type != TankType::Enemy {
            continue;
        }

        // 获取司令官位置
        let Ok((_, _commander_transform)) = commanders.get(commander_entity) else {
            continue;
        };

        // 碰撞确认，播放受击音效
        commands.spawn(AudioPlayer::new(sound_resources.commander_get_shot.clone()));

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
            commands.spawn(AudioPlayer::new(sound_resources.commander_death.clone()));
        }

        despawn_bullet(&mut commands, &mut bullet_tracker, bullet_entity);
    }
}
