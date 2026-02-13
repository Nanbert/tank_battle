//! 道具系统
//!
//! 处理道具生成、动画和玩家拾取道具的碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use avian2d::prelude::*;

use crate::constants::*;
use crate::resources::{
    CommanderLife, GameAtlasLayoutResources, GameAudioResources, GameTextureResources, PlayerInfo,
    PlayerStatChanged, Language,
};
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;

// 导入策略模式
pub use crate::powerup_strategy::{PowerUp, PowerUpResult};

/// 道具碰撞检测距离
pub const POWERUP_COLLISION_DISTANCE: f32 = 100.0;

/// 气泡特效尺寸
pub const POWERUP_BUBBLE_SIZE: f32 = 100.0;

/// 道具碰撞检测和拾取系统
///
/// 使用策略模式处理不同道具的效果，代码更简洁、可维护
pub fn handle_powerup_collision(
    mut commands: Commands,
    audio_resources: Res<GameAudioResources>,
    powerups: Query<(Entity, &Transform, &PowerUp)>,
    player_tanks: Query<(&Transform, &PlayerTank, Entity), With<PlayerTank>>,
    mut player_info: ResMut<PlayerInfo>,
    mut commander_life: ResMut<CommanderLife>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    font_resources: Res<GameTextureResources>,
    language: Res<Language>,
) {
    // 定义拾取道具的结构体，确保 entity 和 type 同步
    struct PickedPowerUp {
        entity: Entity,
        powerup_type: PowerUp,
    }

    for (tank_transform, player_tank, tank_entity) in player_tanks.iter() {
        // 查找碰撞的道具（使用工具函数简化代码）
        let picked = powerups
            .iter()
            .find(|(_, powerup_transform, _)| {
                (powerup_transform.translation - tank_transform.translation).length()
                    < POWERUP_COLLISION_DISTANCE
            })
            .map(|(entity, _, powerup_type)| PickedPowerUp {
                entity,
                powerup_type: *powerup_type,
            });

        if let Some(picked) = picked {
            // 播放道具音效
            crate::utils::play_one_shot_sound(
                &mut commands,
                audio_resources.powerup_sound.clone(),
                1.0,
            );
            let () = commands.entity(picked.entity).try_despawn();

            // 获取策略并应用效果（零成本抽象）
            let strategy = picked.powerup_type.into_strategy();
            let tank_type = player_tank.tank_type;

            // 生成弹出文字特效
            let font = font_resources.get_font(*language);
            let text = picked.powerup_type.get_floating_text(*language);
            // 根据玩家类型设置颜色：玩家1蓝色，玩家2红色
            let color = match tank_type {
                TankType::Player1 => COLOR_BLUE,
                TankType::Player2 => COLOR_RED,
                TankType::Enemy => COLOR_GREEN,
            };
            // 在坦克位置上方生成弹出文字
            let floating_position = Vec3::new(
                tank_transform.translation.x,
                tank_transform.translation.y + 120.0,
                Z_UI_TEXT + 2.0,
            );
            crate::ui::overlay::spawn_floating_text(
                &mut commands,
                &text,
                floating_position,
                color,
                &font,
            );

            // 处理指挥官生命变化（汉堡道具）
            if strategy.affects_commander() && commander_life.life_points < COMMANDER_LIFE_MAX {
                commander_life.life_points += 1;
            }

            // 应用道具效果到玩家属性
            player_info.with_stats_mut(tank_type, |player_stats| {
                let result = strategy.apply(player_stats);

                // 发送属性变更事件
                if let PowerUpResult::StatChanged(stat_type) = result {
                    stat_changed_events.write(PlayerStatChanged {
                        player_type: tank_type,
                        stat_type,
                    });
                }
            });

            // 更新碰撞过滤组（气垫道具）
            if strategy.update_filter_groups() {
                // 气垫效果：允许与海洋碰撞但不被阻挡
                // memberships=layer0|layer1, filters=layer0（只与默认层碰撞）
                commands.entity(tank_entity).insert(CollisionLayers::new(
                    LayerMask::from(0b11u32), // 属于 layer0 和 layer1
                    LayerMask::from(0b01u32), // 只与 layer0 碰撞
                ));
            }
        }
    }
}

/// 在地图随机位置生成道具
pub fn spawn_powerup_random_position(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    global_rng: &mut crate::global_rng::GlobalRng,
) {
    let powerup_types = [
        PowerUp::SpeedUp,
        PowerUp::Protection,
        PowerUp::FireSpeed,
        PowerUp::FireShell,
        PowerUp::TrackChain,
        PowerUp::Penetrate,
        PowerUp::Repair,
        PowerUp::Hamburger,
        PowerUp::AirCushion,
        PowerUp::Shell,
    ];

    let powerup_type = powerup_types[global_rng.gen_range(0..powerup_types.len())];

    // 定义禁止区域
    // 上方：坦克高度区域（MAP_TOP_Y - TANK_DISPLAY_SIZE.y 到 MAP_TOP_Y）
    // 下方：commander高度区域（MAP_BOTTOM_Y 到 MAP_BOTTOM_Y + COMMANDER_SIZE.y）
    let top_forbidden_y = MAP_TOP_Y - TANK_DISPLAY_SIZE.y;
    let bottom_forbidden_y = MAP_BOTTOM_Y + COMMANDER_SIZE.y;

    // 在随机位置生成道具（在地图范围内），避开禁止区域
    let x = global_rng.gen_range_f32(MAP_LEFT_X + 100.0..MAP_RIGHT_X - 100.0);
    let y = global_rng.gen_range_f32(bottom_forbidden_y + 100.0..top_forbidden_y - 100.0);
    let position = Vec3::new(x, y, crate::constants::Z_FOREST);

    spawn_powerup(
        commands,
        texture_resources,
        atlas_layouts,
        powerup_type,
        position,
    );
}

/// 生成单个道具（使用统一的动画系统）
pub fn spawn_powerup(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    powerup_type: PowerUp,
    position: Vec3,
) {
    let (texture, atlas_info, atlas_layout) = powerup_type.get_texture_resources(texture_resources, atlas_layouts);

    crate::utils::spawn_animated_sprite(
        commands,
        texture,
        atlas_layout,
        atlas_info.animation_indices_full(),
        crate::constants::POWER_UP_ANIMATION_FRAME,
        Transform::from_translation(Vec3::new(position.x, position.y, Z_FOREST)),
        atlas_info.display_size,
        (powerup_type, PlayingEntity, AnimationMode::Looping),
    );
}

/// 测试用：在指定关卡强制生成指定道具
#[allow(dead_code)]
pub fn spawn_test_powerup(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    stage_level: Res<crate::resources::StageLevel>,
) {
    // 配置：指定关卡和道具类型
    const TEST_STAGE: usize = 1;
    const TEST_POWERUP: PowerUp = PowerUp::AirCushion;

    // 只在指定关卡生成
    if stage_level.0 != TEST_STAGE {
        return;
    }

    // 在地图中心附近生成
    let position = Vec3::new(0.0, 0.0, Z_FOREST);
    spawn_powerup(
        &mut commands,
        &texture_resources,
        &atlas_layouts,
        TEST_POWERUP,
        position,
    );
}

/// 更新履带特效
/// 根据玩家是否拥有 track_chain 能力，动态添加或移除履带子实体
pub fn update_track_chain_effect(
    mut commands: Commands,
    player_tanks: Query<(Entity, Option<&Children>, &PlayerTank), With<PlayerTank>>,
    track_chain_entities: Query<(), With<crate::constants::TrackChainEffect>>,
    player_info: Res<PlayerInfo>,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
) {
    for (entity, children, player_tank) in player_tanks.iter() {
        // 检查玩家是否有 track_chain 能力
        let has_track_chain = player_info
            .get_stat_value(player_tank.tank_type, |stats| stats.track_chain as usize)
            > 0;

        // 检查是否已经有履带特效子实体
        let has_track_chain_sprite =
            children.is_some_and(|c| c.iter().any(|child| track_chain_entities.contains(child)));

        if has_track_chain && !has_track_chain_sprite {
            // 创建履带特效
            let child_entity = crate::utils::spawn_animated_sprite(
                &mut commands,
                texture_resources.track_chain_effect.clone(),
                atlas_layouts.track_chain_effect.clone(),
                crate::atlas::TRACK_CHAIN_ATLAS.animation_indices_full(),
                crate::constants::TRACK_CHAIN_ANIMATION_FRAME,
                Transform::from_translation(Vec3::new(0.0, 0.0, crate::constants::Z_DEFAULT + 0.1)),
                crate::constants::TANK_DISPLAY_SIZE,
                (
                    crate::constants::TrackChainEffect,
                    AnimationMode::Conditional {
                        tank_type: player_tank.tank_type,
                    },
                ),
            );

            // 将履带特效设为坦克的子实体
            commands.entity(entity).add_child(child_entity);
        } else if !has_track_chain && has_track_chain_sprite {
            // 移除所有履带特效子实体
            crate::utils::cleanup_children_by_marker(
                &mut commands,
                children,
                &track_chain_entities,
            );
        }
    }
}

/// 更新低血量烟雾特效
/// 当玩家生命值 ≤ 1 时动态添加烟雾子实体，否则移除
pub fn update_low_health_smoke_effects(
    mut commands: Commands,
    player_tanks: Query<(Entity, Option<&Children>, &PlayerTank), With<PlayerTank>>,
    smoke_entities: Query<(), With<crate::constants::LowHealthSmokeEffect>>,
    player_info: Res<PlayerInfo>,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
) {
    for (entity, children, player_tank) in player_tanks.iter() {
        // 检查玩家是否处于低血量状态（生命值 ≤ 1 且 > 0）
        let is_low_health = player_info
            .get_stats(player_tank.tank_type)
            .map_or(false, |stats| stats.life_points <= 1 && stats.life_points > 0);

        // 检查是否已经有烟雾特效子实体
        let has_smoke_sprite =
            children.is_some_and(|c| c.iter().any(|child| smoke_entities.contains(child)));

        if is_low_health && !has_smoke_sprite {
            // 创建烟雾特效
            let child_entity = crate::utils::spawn_animated_sprite(
                &mut commands,
                texture_resources.tank_smoke_effect.clone(),
                atlas_layouts.tank_smoke_effect.clone(),
                crate::atlas::TANK_SMOKE_ATLAS.animation_indices_full(),
                crate::constants::LOW_HEALTH_SMOKE_ANIMATION_FRAME,
                Transform::from_translation(Vec3::new(0.0, 0.0, crate::constants::Z_DEFAULT + 0.2)),
                Vec2::new(80.0, 80.0), // 缩小到坦克的 2/3
                (
                    crate::constants::LowHealthSmokeEffect,
                    AnimationMode::Looping,
                ),
            );

            // 将烟雾特效设为坦克的子实体
            commands.entity(entity).add_child(child_entity);
        } else if !is_low_health && has_smoke_sprite {
            // 移除所有烟雾特效子实体
            crate::utils::cleanup_children_by_marker(&mut commands, children, &smoke_entities);
        }
    }
}
