//! 子弹系统模块
//!
//! 处理子弹的生成、移动、碰撞检测和销毁逻辑

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::effects;

use crate::constants::*;
use crate::resources::{BulletTracker, PlayerInfo, PlayerStatChanged, PlayerStats, StatType};
use bevy::audio::Volume;

/// 特效事件枚举
/// 用于解耦碰撞逻辑和特效生成
#[derive(Event, Clone, Copy, Message)]
pub enum EffectEvent {
    Explosion { position: Vec3 },
    Spark { position: Vec3 },
    ForestFire { position: Vec3 },
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
    asset_server: Res<AssetServer>,
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
                transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

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
            bullet_tracker.add_bullet(bullet_entity, entity);
        }
    }
}

/// 玩家坦克射击系统
pub fn player_shoot_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
        let Some(player_stats) = player_info.players.get(&player_tank.tank_type) else {
            continue;
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
            transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

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
        bullet_tracker.add_bullet(bullet_entity, entity);

        // 重置冷却时间
        fire_config.cooldown.reset();
    }
}

/// 子弹边界检查系统
pub fn bullet_bounds_check_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in &mut query {
        let x = transform.translation.x;
        let y = transform.translation.y;

        // 检查子弹是否超出游戏窗口边界
        if !(MAP_LEFT_X..=MAP_RIGHT_X).contains(&x) || !(MAP_BOTTOM_Y..=MAP_TOP_Y).contains(&y) {
            commands.entity(entity).try_insert(BulletDespawnMarker);
        }
    }
}

/// 子弹统一销毁系统
/// 处理所有子弹的销毁逻辑，包括清理所有者引用和实际销毁
pub fn bullet_despawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &BulletDespawnMarker, &BulletOwner), With<Bullet>>,
    mut bullet_tracker: ResMut<BulletTracker>,
) {
    for (entity, _marker, _owner) in &mut query {
        // 清理所有者引用，允许坦克再次射击
        bullet_tracker.remove_bullet(entity);

        // 销毁子弹实体
        let () = commands.entity(entity).try_despawn();
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
            &asset_server,
            terrain_entity,
            bullet_entity,
            bullet_transform,
            &bullet_owner,
            &forests,
            &bricks,
            &steels,
            &player_info,
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
    asset_server: &Res<AssetServer>,
    terrain_entity: Entity,
    bullet_entity: Entity,
    bullet_transform: &Transform,
    bullet_owner: &BulletOwner,
    forests: &Query<(Entity, &Transform), With<Forest>>,
    bricks: &Query<(), With<Brick>>,
    steels: &Query<(), With<Steel>>,
    player_info: &Res<PlayerInfo>,
) {
    // 处理森林碰撞
    if let Ok((forest_entity, forest_transform)) = forests.get(terrain_entity) {
        handle_forest_collision(
            commands,
            effect_events,
            forest_entity,
            forest_transform,
            &bullet_owner.owner_type,
            player_info,
        );
        return;
    }

    // 处理砖块碰撞
    if bricks.get(terrain_entity).is_ok() {
        handle_brick_collision(
            commands,
            effect_events,
            asset_server,
            terrain_entity,
            bullet_entity,
            bullet_transform,
        );
        return;
    }

    // 处理钢铁碰撞
    if steels.get(terrain_entity).is_ok() {
        handle_steel_collision(
            commands,
            effect_events,
            asset_server,
            terrain_entity,
            bullet_entity,
            bullet_transform,
            &bullet_owner.owner_type,
            player_info,
        );
    }
}

/// 处理森林碰撞
fn handle_forest_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    forest_entity: Entity,
    forest_transform: &Transform,
    owner_type: &TankType,
    player_info: &Res<PlayerInfo>,
) {
    let is_player_bullet = !matches!(owner_type, TankType::Enemy);
    if !is_player_bullet {
        return;
    }

    let Some(player_stats) = player_info.players.get(owner_type) else {
        return;
    };

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
    asset_server: &Res<AssetServer>,
    brick_entity: Entity,
    bullet_entity: Entity,
    bullet_transform: &Transform,
) {
    let brick_hit_sound: Handle<AudioSource> = asset_server.load(SOUND_BRICK_HIT);
    commands.spawn((
        AudioPlayer::new(brick_hit_sound),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(VOLUME_HALF)),
    ));

    effect_events.write(EffectEvent::Spark {
        position: bullet_transform.translation,
    });

    let () = commands.entity(brick_entity).try_despawn();
    commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
}

/// 处理钢铁碰撞
fn handle_steel_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    asset_server: &Res<AssetServer>,
    steel_entity: Entity,
    bullet_entity: Entity,
    bullet_transform: &Transform,
    owner_type: &TankType,
    player_info: &Res<PlayerInfo>,
) {
    effect_events.write(EffectEvent::Spark {
        position: bullet_transform.translation,
    });

    if matches!(owner_type, TankType::Enemy) {
        let hit_sound: Handle<AudioSource> = asset_server.load(SOUND_HIT);
        commands.spawn(AudioPlayer::new(hit_sound));
        commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
        return;
    }

    let Some(player_stats) = player_info.players.get(owner_type) else {
        let hit_sound: Handle<AudioSource> = asset_server.load(SOUND_HIT);
        commands.spawn(AudioPlayer::new(hit_sound));
        commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
        return;
    };

    if player_stats.penetrate {
        let metal_crash_sound: Handle<AudioSource> = asset_server.load(SOUND_METAL_CRASH);
        commands.spawn(AudioPlayer::new(metal_crash_sound));
        let () = commands.entity(steel_entity).try_despawn();
    } else {
        let hit_sound: Handle<AudioSource> = asset_server.load(SOUND_HIT);
        commands.spawn(AudioPlayer::new(hit_sound));
    }

    commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
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
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut controllers: Query<&mut KinematicCharacterController>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else { continue; };

        // 提取子弹和坦克实体
        let Some((bullet_entity, tank_entity)) =
            find_bullet_and_tank_in_collision(*e1, *e2, &bullets, &enemy_tanks, &player_tanks)
        else { continue; };

        let bullet_owner_info = bullets.get(bullet_entity).unwrap().1;

        // 卫语句：检查子弹是否应该销毁
        if !should_bullet_destroy(
            bullet_owner_info.owner_type,
            tank_entity,
            &enemy_tanks,
            &player_tanks,
        ) {
            continue;
        }

        let is_player_bullet = matches!(
            bullet_owner_info.owner_type,
            TankType::Player1 | TankType::Player2
        );
        let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();
        let is_player_tank = player_tanks.get(tank_entity).is_ok();

        if is_player_bullet && is_enemy_tank {
            handle_player_bullet_hit_enemy(
                &mut commands,
                &mut effect_events,
                &enemy_tanks_with_transform,
                &mut player_info,
                &mut stat_changed_events,
                bullet_owner_info.owner_type,
                tank_entity,
            );
        } else if !is_player_bullet && is_player_tank {
            let player_index = player_tanks.get(tank_entity).unwrap().tank_type;
            handle_enemy_bullet_hit_player(
                &mut commands,
                &mut effect_events,
                &asset_server,
                &player_tanks_with_transform,
                &player_avatars,
                &mut player_info,
                &mut stat_changed_events,
                &mut controllers,
                player_index,
                tank_entity,
            );
        }

        commands.entity(bullet_entity).try_insert(BulletDespawnMarker);
    }
}

/// 处理玩家子弹击中敌方坦克
fn handle_player_bullet_hit_enemy(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    enemy_tanks_with_transform: &Query<(Entity, &Transform), With<EnemyTank>>,
    player_info: &mut ResMut<PlayerInfo>,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    player_type: TankType,
    tank_entity: Entity,
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

    // 销毁敌方坦克
    let () = commands.entity(tank_entity).try_despawn();

    // 增加分数
    if let Some(player_stats) = player_info.players.get_mut(&player_type) {
        player_stats.score += 100;
        stat_changed_events.write(PlayerStatChanged {
            player_type,
            stat_type: StatType::Score,
        });
    }
}

/// 处理敌方子弹击中玩家坦克
fn handle_enemy_bullet_hit_player(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<EffectEvent>,
    asset_server: &Res<AssetServer>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: &Query<(Entity, &PlayerUI)>,
    player_info: &mut ResMut<PlayerInfo>,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    controllers: &mut Query<&mut KinematicCharacterController>,
    player_index: TankType,
    tank_entity: Entity,
) {
    // 播放中弹音效
    let hit_sound: Handle<AudioSource> = asset_server.load(SOUND_HIT);
    commands.spawn(AudioPlayer::new(hit_sound));

    // 发送火花特效事件
    if let Ok((_, tank_transform)) = player_tanks_with_transform.get(tank_entity) {
        effect_events.write(EffectEvent::Spark {
            position: tank_transform.translation,
        });
    }

    // 扣除对应玩家的生命值
    let Some(player_stats) = player_info.players.get_mut(&player_index) else {
        return;
    };

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
    if player_stats.life_red_bar > 0 {
        player_stats.life_red_bar -= 1;
    }

    if player_stats.life_red_bar == 0 {
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
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for event in events.read() {
        match event {
            EffectEvent::Explosion { position } => {
                effects::spawn_explosion(
                    &mut commands,
                    &asset_server,
                    &mut texture_atlas_layouts,
                    *position,
                );
            }
            EffectEvent::Spark { position } => {
                effects::spawn_spark(
                    &mut commands,
                    &asset_server,
                    &mut texture_atlas_layouts,
                    *position,
                );
            }
            EffectEvent::ForestFire { position } => {
                effects::spawn_forest_fire(
                    &mut commands,
                    &asset_server,
                    &mut texture_atlas_layouts,
                    *position,
                );
            }
        }
    }
}

/// 司令官与敌方子弹碰撞检测系统
/// 使用简单的 AABB 碰撞检测
pub fn bullet_commander_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    asset_server: Res<AssetServer>,
    bullets: Query<(Entity, &BulletOwner, &Transform), With<Bullet>>,
    commanders: Query<(Entity, &Transform), With<crate::constants::Commander>>,
    mut commander_life: ResMut<crate::resources::CommanderLife>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
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
            let Ok((_, bullet_owner_info, _)) = bullets.get(bullet_entity) else {
                continue;
            };

            // 只处理敌方子弹击中司令官的情况
            if bullet_owner_info.owner_type != TankType::Enemy {
                continue;
            }

            // 获取司令官位置
            let Ok((_, commander_transform)) = commanders.get(commander_entity) else {
                continue;
            };

            // 使用 AABB 碰撞检测验证碰撞
            let Ok((_, _, bullet_transform)) = bullets.get(bullet_entity) else {
                continue;
            };

            // 简单的 AABB 碰撞检测
            let commander_half_size = Vec2::new(
                crate::constants::COMMANDER_WIDTH / 2.0,
                crate::constants::COMMANDER_HEIGHT / 2.0,
            );
            let bullet_half_size = Vec2::new(BULLET_WIDTH / 2.0, BULLET_HEIGHT / 2.0); // 子弹尺寸的一半

            let commander_min = commander_transform.translation.truncate() - commander_half_size;
            let commander_max = commander_transform.translation.truncate() + commander_half_size;
            let bullet_min = bullet_transform.translation.truncate() - bullet_half_size;
            let bullet_max = bullet_transform.translation.truncate() + bullet_half_size;

            if commander_min.x <= bullet_max.x
                && commander_max.x >= bullet_min.x
                && commander_min.y <= bullet_max.y
                && commander_max.y >= bullet_min.y
            {
                // 碰撞确认，播放受击音效
                let hit_sound: Handle<AudioSource> = asset_server.load(SOUND_COMMANDER_GET_SHOT);
                commands.spawn(AudioPlayer::new(hit_sound));

                // 发送火花特效事件
                effect_events.write(EffectEvent::Spark {
                    position: bullet_transform.translation,
                });

                // 减少司令官生命值
                if commander_life.life_red_bar > 0 {
                    commander_life.life_red_bar -= 1;
                }

                // 检查是否是致命伤（生命值归零）
                if commander_life.life_red_bar == 0 {
                    // 播放死亡音效
                    let death_sound: Handle<AudioSource> = asset_server.load(SOUND_COMMANDER_DEATH);
                    commands.spawn(AudioPlayer::new(death_sound));
                }

                // 标记子弹销毁
                commands
                    .entity(bullet_entity)
                    .try_insert(BulletDespawnMarker);
            }
        }
    }
}
