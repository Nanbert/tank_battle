//! 子弹系统模块
//!
//! 处理子弹的生成、移动、碰撞检测和销毁逻辑

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::effects;
use crate::resources::{
    GameAtlasLayoutResources, GameAudioResources, GameTextureResources, GameTrackers, PlayerInfo,
    PlayerStatChanged, StatType,
};
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;
use crate::utils;

/// 特效事件枚举
/// 用于解耦碰撞逻辑和特效生成
#[derive(Event, Clone, Message)]
pub enum EffectEvent {
    Explosion {
        position: Vec3,
    },
    Spark {
        position: Vec3,
        audio_handle: Handle<AudioSource>,
        volume: f32,
    },
    ForestFire {
        position: Vec3,
    },
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
    texture_resources: &GameTextureResources,
    params: BulletSpawnParams,
    player_info: &Res<PlayerInfo>,
    atlas_layouts: &GameAtlasLayoutResources,
) -> Entity {
    // 根据坦克类型选择子弹纹理
    let bullet_texture = texture_resources.get_bullet_texture(params.owner_type);

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
                custom_size: Some(BULLET_DISPLAY_SIZE), // 子弹尺寸：长60像素，宽40像素
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
            Collider::cuboid(BULLET_DISPLAY_SIZE.x / 2.0, BULLET_DISPLAY_SIZE.y / 2.0), // 使用矩形碰撞体匹配子弹尺寸
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
        let fire_effect_atlas = atlas_layouts.fire_effect.clone();
        let animation_indices = AnimationIndices {
            first: 0,
            last: crate::atlas::FIRE_EFFECT_ATLAS.total_frames - 1,
        };

        commands.entity(bullet_entity).with_children(|parent| {
            parent.spawn((
                crate::constants::FireEffect,
                crate::constants::AnimationMode::Looping,
                Sprite::from_atlas_image(
                    texture_resources.bullet_fire_effect.clone(),
                    TextureAtlas {
                        layout: fire_effect_atlas,
                        index: animation_indices.first,
                    },
                ),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.1), // 略高于子弹
                    rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), // 旋转 90 度
                    scale: Vec3::splat(1.65),              // 放大到 165% (因为纹理缩小到33%)
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
        let penetrate_effect_atlas = atlas_layouts.penetrate_effect.clone();
        let animation_indices = AnimationIndices {
            first: 0,
            last: crate::atlas::PENETRATE_EFFECT_ATLAS.total_frames - 1,
        };

        commands.entity(bullet_entity).with_children(|parent| {
            parent.spawn((
                PenetrateEffect,
                crate::constants::AnimationMode::Looping,
                Sprite::from_atlas_image(
                    texture_resources.bullet_penetrate_effect.clone(),
                    TextureAtlas {
                        layout: penetrate_effect_atlas,
                        index: animation_indices.first,
                    },
                ),
                Transform {
                    translation: Vec3::new(0.0, 0.0, 0.2), // 略高于火焰特效
                    rotation: Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2), // 旋转 -90 度
                    scale: Vec3::splat(0.15),              // 缩小到 0.15 倍 (原0.3的1/2)
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
    texture_resources: Res<GameTextureResources>,
    player_info: Res<PlayerInfo>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &crate::constants::EnemyTank,
            &TankFireConfig,
        ),
        With<EnemyTank>,
    >,
    mut game_trackers: ResMut<GameTrackers>,
) {
    for (entity, transform, enemy_tank, fire_config) in &mut query {
        // 检查是否可以射击
        if !game_trackers
            .bullets
            .can_fire(entity, fire_config.max_bullets)
        {
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
            let bullet_pos = transform.translation
                + direction.extend(0.0)
                    * (TANK_DISPLAY_SIZE.y / 2.0 + crate::constants::BULLET_COLLIDER_SIZE);

            // 生成子弹
            let bullet_entity = spawn_bullet(
                &mut commands,
                &texture_resources,
                BulletSpawnParams {
                    position: bullet_pos,
                    direction,
                    speed: BULLET_SPEED,
                    owner_type: TankType::Enemy,
                },
                &player_info,
                &atlas_layouts,
            );

            // 记录子弹的所有者
            game_trackers.bullets.add_bullet(bullet_entity, entity);
        }
    }
}

/// 玩家坦克射击系统
pub fn player_shoot_system(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    time: Res<Time>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
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
    mut game_trackers: ResMut<GameTrackers>,
    player_info: Res<PlayerInfo>,
    keyboard: Res<ButtonInput<KeyCode>>,
    barrel_query: Query<(), With<Barrel>>,
    audio_resources: Res<GameAudioResources>,
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
        let player_stats = player_info
            .get_stats(player_tank.tank_type)
            .expect("Player should exist");

        // 检查是否可以射击（使用 player_stats.shells 作为最大子弹数）
        if !game_trackers.bullets.can_fire(entity, player_stats.shells) {
            continue;
        }

        // 计算子弹发射方向（基于坦克当前的旋转角度）
        let direction = crate::utils::calculate_direction_from_rotation(&transform.rotation);

        // 计算子弹初始位置（坦克前方）
        let bullet_pos = transform.translation
            + direction.extend(0.0)
                * (TANK_DISPLAY_SIZE.y / 2.0 + crate::constants::BULLET_COLLIDER_SIZE);
        // 玩家子弹速度 = PLAYER_BULLET_SPEED × (1 + fire_speed百分比/100)
        let fire_speed_bonus = player_stats.fire_speed as f32 / 100.0;
        let bullet_speed = PLAYER_BULLET_SPEED * (1.0 + fire_speed_bonus);

        // 生成子弹
        let bullet_entity = spawn_bullet(
            &mut commands,
            &texture_resources,
            BulletSpawnParams {
                position: bullet_pos,
                direction,
                speed: bullet_speed,
                owner_type: player_tank.tank_type,
            },
            &player_info,
            &atlas_layouts,
        );

        // 记录子弹的所有者
        game_trackers.bullets.add_bullet(bullet_entity, entity);

        // 播放玩家射击音效，音量 0.4
        utils::play_one_shot_sound(&mut commands, audio_resources.player_shot.clone(), 0.4);

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
fn despawn_bullet(
    commands: &mut Commands,
    game_trackers: &mut GameTrackers,
    bullet_entity: Entity,
) {
    game_trackers.bullets.remove_bullet(bullet_entity);
    let () = commands.entity(bullet_entity).try_despawn();
}

/// 子弹边界检查系统
pub fn bullet_bounds_check_system(
    mut commands: Commands,
    mut game_trackers: ResMut<GameTrackers>,
    mut query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in &mut query {
        let x = transform.translation.x;
        let y = transform.translation.y;

        // 检查子弹是否超出游戏窗口边界
        if !(MAP_LEFT_X..=MAP_RIGHT_X).contains(&x) || !(MAP_BOTTOM_Y..=MAP_TOP_Y).contains(&y) {
            despawn_bullet(&mut commands, &mut game_trackers, entity);
        }
    }
}

/// 提取子弹和另一个实体的碰撞信息
/// 返回 (子弹实体, 另一个实体, 子弹引用, 子弹变换)
fn extract_bullet_collision<'a>(
    e1: Entity,
    e2: Entity,
    bullets: &'a Query<(Entity, &Bullet, &Transform), With<Bullet>>,
) -> Option<(Entity, Entity, &'a Bullet, &'a Transform)> {
    if let Some((bullet_entity, other_entity)) =
        crate::utils::extract_collision_pair(e1, e2, bullets)
    {
        bullets
            .get(bullet_entity)
            .ok()
            .map(|(_, bullet, transform)| (bullet_entity, other_entity, bullet, transform))
    } else {
        None
    }
}

/// 查找碰撞中的子弹和坦克，同时返回子弹信息
pub fn find_bullet_and_tank_in_collision<'a>(
    e1: Entity,
    e2: Entity,
    bullets: &'a Query<(Entity, &Bullet, &Transform), With<Bullet>>,
    all_tanks: &Query<(), Or<(With<EnemyTank>, With<PlayerTank>)>>,
) -> Option<(Entity, Entity, &'a Bullet)> {
    if let Ok((_, bullet, _)) = bullets.get(e1)
        && all_tanks.contains(e2)
    {
        return Some((e1, e2, bullet));
    }
    if let Ok((_, bullet, _)) = bullets.get(e2)
        && all_tanks.contains(e1)
    {
        return Some((e2, e1, bullet));
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
    mut game_trackers: ResMut<GameTrackers>,
    audio_resources: Res<GameAudioResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else {
            continue;
        };

        // 提取子弹和地形实体
        let Some((bullet_entity, terrain_entity, bullet, bullet_transform)) =
            extract_bullet_collision(*e1, *e2, &bullets)
        else {
            continue;
        };

        // 跳过已被标记销毁的实体（被激光击中的）
        if despawned_entities.contains(terrain_entity) {
            continue;
        }

        // 处理森林碰撞
        if let Ok((forest_entity, forest_transform)) = forests.get(terrain_entity) {
            if bullet.is_player()
                && let Some(player_stats) = player_info.get_stats(bullet.owner_type())
                && player_stats.fire_shell
            {
                effect_events.write(EffectEvent::ForestFire {
                    position: forest_transform.translation,
                });
                let () = commands.entity(forest_entity).try_despawn();
            }
            continue;
        }

        // 处理砖块碰撞
        if bricks.get(terrain_entity).is_ok() {
            effect_events.write(EffectEvent::Spark {
                position: bullet_transform.translation,
                audio_handle: audio_resources.brick_hit.clone(),
                volume: VOLUME_HALF,
            });
            let () = commands.entity(terrain_entity).try_despawn();
            despawn_bullet(&mut commands, &mut game_trackers, bullet_entity);
            continue;
        }

        // 处理钢铁碰撞
        if steels.get(terrain_entity).is_ok() {
            if bullet.is_enemy() {
                effect_events.write(EffectEvent::Spark {
                    position: bullet_transform.translation,
                    audio_handle: audio_resources.hit.clone(),
                    volume: 1.0,
                });
            } else if let Some(player_stats) = player_info.get_stats(bullet.owner_type()) {
                if player_stats.penetrate {
                    effect_events.write(EffectEvent::Spark {
                        position: bullet_transform.translation,
                        audio_handle: audio_resources.metal_crash.clone(),
                        volume: 1.0,
                    });
                    let () = commands.entity(terrain_entity).try_despawn();
                } else {
                    effect_events.write(EffectEvent::Spark {
                        position: bullet_transform.translation,
                        audio_handle: audio_resources.hit.clone(),
                        volume: 1.0,
                    });
                    utils::play_one_shot_sound(&mut commands, audio_resources.hit.clone(), 1.0);
                }
            }

            despawn_bullet(&mut commands, &mut game_trackers, bullet_entity);
        }
    }
}

/// 子弹与坦克碰撞检测系统
pub fn bullet_tank_collision_system(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<EffectEvent>,
    mut game_trackers: ResMut<GameTrackers>,
    bullets: Query<(Entity, &Bullet, &Transform), With<Bullet>>,
    all_tanks: Query<(), Or<(With<EnemyTank>, With<PlayerTank>)>>,
    enemy_tanks: Query<(Entity, &Transform), With<EnemyTank>>,
    player_tanks: Query<(&PlayerTank, &Transform), With<PlayerTank>>,
    despawned_entities: Query<(), With<DespawnMarker>>,
    mut player_info: ResMut<PlayerInfo>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut controllers: Query<&mut KinematicCharacterController>,
    audio_resources: Res<GameAudioResources>,
) {
    for event in collision_events.read() {
        // 卫语句：只处理 Started 事件
        let CollisionEvent::Started(e1, e2, _) = event else {
            continue;
        };

        // 提取子弹和坦克实体，同时获取子弹信息
        let Some((bullet_entity, tank_entity, bullet)) =
            find_bullet_and_tank_in_collision(*e1, *e2, &bullets, &all_tanks)
        else {
            continue;
        };

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
            utils::play_one_shot_sound(&mut commands, audio_resources.hit.clone(), 1.0);
            let () = commands.entity(tank_entity).try_despawn();
            // 增加分数
            player_info.with_stats_mut(player_type, |player_stats| {
                player_stats.score += 100;
                stat_changed_events.write(PlayerStatChanged {
                    player_type,
                    stat_type: StatType::Score,
                });
            });
            despawn_bullet(&mut commands, &mut game_trackers, bullet_entity);
            continue;
        }

        // 敌方子弹击中玩家坦克
        if bullet.is_enemy()
            && let Ok((player_tank, tank_transform)) = player_tanks.get(tank_entity)
        {
            let player_type = player_tank.tank_type;

            effect_events.write(EffectEvent::Spark {
                position: tank_transform.translation,
                audio_handle: audio_resources.hit.clone(),
                volume: 1.0,
            });

            let mut need_update_filter_groups = false;
            let mut need_despawn = false;
            let mut explosion_position = None;

            player_info.with_stats_mut(player_type, |player_stats| {
                // 按优先级移除道具：fire_shell > track_chain > penetrate > air_cushion > shells
                let has_fire_shell = player_stats.fire_shell;
                let has_track_chain = player_stats.track_chain;
                let has_penetrate = player_stats.penetrate;
                let has_air_cushion = player_stats.air_cushion;
                let has_shells = player_stats.shells > 1;

                if has_fire_shell {
                    player_stats.fire_shell = false;
                    stat_changed_events.write(PlayerStatChanged {
                        player_type,
                        stat_type: StatType::FireShell,
                    });
                } else if has_track_chain {
                    player_stats.track_chain = false;
                    stat_changed_events.write(PlayerStatChanged {
                        player_type,
                        stat_type: StatType::TrackChain,
                    });
                } else if has_penetrate {
                    player_stats.penetrate = false;
                    stat_changed_events.write(PlayerStatChanged {
                        player_type,
                        stat_type: StatType::Penetrate,
                    });
                } else if has_air_cushion {
                    player_stats.air_cushion = false;
                    need_update_filter_groups = true;
                    stat_changed_events.write(PlayerStatChanged {
                        player_type,
                        stat_type: StatType::AirCushion,
                    });
                } else if has_shells {
                    player_stats.shells -= 1;
                    stat_changed_events.write(PlayerStatChanged {
                        player_type,
                        stat_type: StatType::Shell,
                    });
                } else {
                    // 造成伤害
                    if player_stats.life_points > 0 {
                        player_stats.life_points -= 1;
                    }
                    if player_stats.life_points == 0 {
                        explosion_position = Some(tank_transform.translation);
                        need_despawn = true;
                    }
                }
            });

            // 更新 filter_groups
            if need_update_filter_groups
                && let Ok(mut controller) = controllers.get_mut(tank_entity)
            {
                controller.filter_groups = None;
            }

            // 销毁坦克
            if need_despawn {
                if let Some(pos) = explosion_position {
                    effect_events.write(EffectEvent::Explosion { position: pos });
                }
                let () = commands.entity(tank_entity).try_despawn();
            }

            despawn_bullet(&mut commands, &mut game_trackers, bullet_entity);
        }
    }
}

/// 特效处理系统
/// 监听特效事件并生成对应的视觉效果
pub fn handle_effect_events(
    mut events: MessageReader<EffectEvent>,
    mut commands: Commands,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    texture_resources: Res<GameTextureResources>,
    audio_resources: Res<GameAudioResources>,
) {
    for event in events.read() {
        match event {
            EffectEvent::Explosion { position } => {
                effects::spawn_explosion(
                    &mut commands,
                    &texture_resources,
                    &atlas_layouts,
                    &audio_resources,
                    *position,
                );
            }
            EffectEvent::Spark {
                position,
                audio_handle,
                volume,
            } => {
                effects::spawn_spark(
                    &mut commands,
                    &texture_resources,
                    &atlas_layouts,
                    audio_handle.clone(),
                    *volume,
                    *position,
                );
            }
            EffectEvent::ForestFire { position } => {
                effects::spawn_forest_fire(
                    &mut commands,
                    &texture_resources,
                    &atlas_layouts,
                    &audio_resources,
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
    mut game_trackers: ResMut<GameTrackers>,
    audio_resources: Res<GameAudioResources>,
) {
    for event in collision_events.read() {
        let CollisionEvent::Started(e1, e2, _) = event else {
            continue;
        };

        // 提取子弹信息
        let Some((bullet_entity, other_entity, bullet, bullet_transform)) =
            extract_bullet_collision(*e1, *e2, &bullets)
        else {
            continue;
        };

        // 验证另一个实体是司令官
        if commanders.get(other_entity).is_err() {
            continue;
        }

        // 只处理敌方子弹击中司令官的情况
        if !bullet.is_enemy() {
            continue;
        }

        // 碰撞确认，播放受击音效
        utils::play_one_shot_sound(
            &mut commands,
            audio_resources.commander_get_shot.clone(),
            1.0,
        );

        // 发送火花特效事件
        effect_events.write(EffectEvent::Spark {
            position: bullet_transform.translation,
            audio_handle: audio_resources.commander_get_shot.clone(),
            volume: 1.0,
        });

        // 减少司令官生命值
        if commander_life.life_points > 0 {
            commander_life.life_points -= 1;
        }

        // 检查是否是致命伤（生命值归零）
        if commander_life.life_points == 0 {
            // 播放死亡音效
            utils::play_one_shot_sound(&mut commands, audio_resources.commander_death.clone(), 1.0);
        }

        despawn_bullet(&mut commands, &mut game_trackers, bullet_entity);
    }
}
