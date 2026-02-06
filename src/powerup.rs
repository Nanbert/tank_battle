//! 道具系统
//!
//! 处理道具生成、动画和玩家拾取道具的碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::{
    CommanderLife, GameAudioResources, GameTextureResources, GameAtlasLayoutResources, PlayerInfo, PlayerStatChanged,
    StatType,
};

/// 道具碰撞检测距离
pub const POWERUP_COLLISION_DISTANCE: f32 = 100.0;

/// 气泡特效尺寸
pub const POWERUP_BUBBLE_SIZE: f32 = 100.0;

/// 道具属性增加量
pub const POWERUP_ATTRIBUTE_INCREASE: usize = 20;

/// 道具类型枚举
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum PowerUp {
    SpeedUp,
    Protection,
    FireSpeed,
    FireShell,
    TrackChain,
    Penetrate,
    Repair,
    Hamburger,
    AirCushion,
    Shell,
}

/// 道具碰撞检测和拾取系统
/// 优化：使用 PickedPowerUp 结构体合并 entity 和 type，提高类型安全性
pub fn handle_powerup_collision(
    mut commands: Commands,
    audio_resources: Res<GameAudioResources>,
    powerups: Query<(Entity, &Transform, &PowerUp)>,
    player_tanks: Query<(&Transform, &PlayerTank, Entity), With<PlayerTank>>,
    mut controllers: Query<&mut KinematicCharacterController>,
    mut player_info: ResMut<PlayerInfo>,
    mut commander_life: ResMut<CommanderLife>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
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

            // 根据道具类型应用效果并发送事件
            let tank_type = player_tank.tank_type;
            let mut stat_type = None;
            let mut update_filter_groups = false;

            player_info.with_stats_mut(tank_type, |player_stats| {
                stat_type = match picked.powerup_type {
                    PowerUp::SpeedUp => {
                        if player_stats.speed < MAX_ATTRIBUTE_VALUE {
                            player_stats.speed = (player_stats.speed + POWERUP_ATTRIBUTE_INCREASE)
                                .min(MAX_ATTRIBUTE_VALUE);
                        }
                        Some(StatType::Speed)
                    }
                    PowerUp::Protection => {
                        if player_stats.protection < MAX_ATTRIBUTE_VALUE {
                            player_stats.protection = (player_stats.protection
                                + POWERUP_ATTRIBUTE_INCREASE)
                                .min(MAX_ATTRIBUTE_VALUE);
                        }
                        Some(StatType::Protection)
                    }
                    PowerUp::FireSpeed => {
                        if player_stats.fire_speed < MAX_ATTRIBUTE_VALUE {
                            player_stats.fire_speed = (player_stats.fire_speed
                                + POWERUP_ATTRIBUTE_INCREASE)
                                .min(MAX_ATTRIBUTE_VALUE);
                        }
                        Some(StatType::FireSpeed)
                    }
                    PowerUp::FireShell => {
                        player_stats.fire_shell = true;
                        Some(StatType::FireShell)
                    }
                    PowerUp::TrackChain => {
                        player_stats.track_chain = true;
                        Some(StatType::TrackChain)
                    }
                    PowerUp::Penetrate => {
                        player_stats.penetrate = true;
                        Some(StatType::Penetrate)
                    }
                    PowerUp::Repair => {
                        if player_stats.life_points < COMMANDER_LIFE_MAX {
                            player_stats.life_points += 1;
                        }
                        None // 修理道具不需要闪烁文字
                    }
                    PowerUp::Hamburger => {
                        None // 汉堡道具不影响玩家属性，不发送事件
                    }
                    PowerUp::AirCushion => {
                        player_stats.air_cushion = true;
                        update_filter_groups = true;
                        Some(StatType::AirCushion)
                    }
                    PowerUp::Shell => {
                        // 增加 1 颗子弹，最多 2 颗
                        if player_stats.shells < 2 {
                            player_stats.shells += 1;
                        }
                        Some(StatType::Shell)
                    }
                };
            });

            // 处理汉堡道具效果（修改 Commander 生命）
            if let PowerUp::Hamburger = picked.powerup_type
                && commander_life.life_points < COMMANDER_LIFE_MAX
            {
                commander_life.life_points += 1;
            }

            // 更新 filter_groups，排除海（GROUP_2）
            if update_filter_groups && let Ok(mut controller) = controllers.get_mut(tank_entity) {
                controller.filter_groups = Some(CollisionGroups::new(
                    Group::all(),
                    Group::all() & !SEA_GROUP,
                ));
            }

            // 发送属性变更事件（如果有）
            if let Some(st) = stat_type {
                stat_changed_events.write(PlayerStatChanged {
                    player_type: tank_type,
                    stat_type: st,
                });
            }
        }
    }
}

/// 第一关测试道具生成
///
/// 第一关固定生成 fire_shell 道具用于测试火焰特效效果。
/// 道具会生成在地图范围内，避开坦克出生点和司令官区域。
pub fn spawn_test_powerup_stage1(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
) {
    let powerup_type = PowerUp::FireShell; // 第一关测试用：fire_shell 用于测试火焰特效

// 定义禁止区域
    // 上方：坦克高度区域（MAP_TOP_Y - TANK_DISPLAY_SIZE.y 到 MAP_TOP_Y）
    // 下方：commander高度区域（MAP_BOTTOM_Y 到 MAP_BOTTOM_Y + COMMANDER_SIZE.y）
    let top_forbidden_y = MAP_TOP_Y - TANK_DISPLAY_SIZE.y;
    let bottom_forbidden_y = MAP_BOTTOM_Y + COMMANDER_SIZE.y;

    // 在随机位置生成道具（在地图范围内），避开禁止区域
    let mut rng = rand::rng();
    let x = rng.random_range(MAP_LEFT_X + 100.0..MAP_RIGHT_X - 100.0);
    let y = rng.random_range(bottom_forbidden_y + 100.0..top_forbidden_y - 100.0);
    let position = Vec3::new(x, y, 0.0);

    spawn_powerup(
        &mut commands,
        &texture_resources,
        &atlas_layouts,
        powerup_type,
        position,
    );
}

/// 随机生成道具
pub fn spawn_power_ups_random(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
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

    let mut rng = rand::rng();
    let powerup_type = powerup_types[rng.random_range(0..powerup_types.len())];

    // 定义禁止区域
    // 上方：坦克高度区域（MAP_TOP_Y - TANK_DISPLAY_SIZE.y 到 MAP_TOP_Y）
    // 下方：commander高度区域（MAP_BOTTOM_Y 到 MAP_BOTTOM_Y + COMMANDER_SIZE.y）
    let top_forbidden_y = MAP_TOP_Y - TANK_DISPLAY_SIZE.y;
    let bottom_forbidden_y = MAP_BOTTOM_Y + COMMANDER_SIZE.y;

    // 在随机位置生成道具（在地图范围内），避开禁止区域
    let x = rng.random_range(MAP_LEFT_X + 100.0..MAP_RIGHT_X - 100.0);
    let y = rng.random_range(bottom_forbidden_y + 100.0..top_forbidden_y - 100.0);
    let position = Vec3::new(x, y, 0.0);

    spawn_powerup(
        &mut commands,
        &texture_resources,
        &atlas_layouts,
        powerup_type,
        position,
    );
}

/// 批量生成道具
///
/// 在指定位置生成多个相同类型的道具。
///
/// 参数：
/// - commands: 命令队列
/// - texture_resources: 纹理资源
/// - atlas_layouts: 图集布局资源
/// - powerup_type: 道具类型
/// - positions: 生成位置数组
/// 生成单个道具
/// 使用统一的动画系统
fn spawn_powerup(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    powerup_type: PowerUp,
    position: Vec3,
) {
    let (texture, atlas_info, atlas_layout) = match powerup_type {
        PowerUp::SpeedUp => (
            texture_resources.speed_up_icon.clone(),
            &crate::atlas::POWER_UP_SPEED_UP_ATLAS,
            &atlas_layouts.speed_up_icon,
        ),
        PowerUp::Protection => (
            texture_resources.protection_icon.clone(),
            &crate::atlas::POWER_UP_PROTECTION_ATLAS,
            &atlas_layouts.protection_icon,
        ),
        PowerUp::FireSpeed => (
            texture_resources.fire_speed_icon.clone(),
            &crate::atlas::POWER_UP_FIRE_SPEED_ATLAS,
            &atlas_layouts.fire_speed_icon,
        ),
        PowerUp::FireShell => (
            texture_resources.fire_shell_icon.clone(),
            &crate::atlas::POWER_UP_FIRE_SHELL_ATLAS,
            &atlas_layouts.fire_shell_icon,
        ),
        PowerUp::TrackChain => (
            texture_resources.track_chain_icon.clone(),
            &crate::atlas::POWER_UP_TRACK_CHAIN_ATLAS,
            &atlas_layouts.track_chain_icon,
        ),
        PowerUp::Penetrate => (
            texture_resources.penetrate_icon.clone(),
            &crate::atlas::POWER_UP_PENETRATE_ATLAS,
            &atlas_layouts.penetrate_icon,
        ),
        PowerUp::Repair => (
            texture_resources.repair_icon.clone(),
            &crate::atlas::POWER_UP_REPAIR_ATLAS,
            &atlas_layouts.repair_icon,
        ),
        PowerUp::Hamburger => (
            texture_resources.hamburger_icon.clone(),
            &crate::atlas::POWER_UP_HAMBURGER_ATLAS,
            &atlas_layouts.hamburger_icon,
        ),
        PowerUp::AirCushion => (
            texture_resources.air_cushion_icon.clone(),
            &crate::atlas::POWER_UP_AIR_CUSHION_ATLAS,
            &atlas_layouts.air_cushion_icon,
        ),
        PowerUp::Shell => (
            texture_resources.shell_icon.clone(),
            &crate::atlas::POWER_UP_SHELL_ATLAS,
            &atlas_layouts.shell_icon,
        ),    };

    crate::utils::spawn_animated_sprite(
        commands,
        texture,
        atlas_layout.clone(),
        atlas_info.animation_indices_full(),
        crate::constants::POWER_UP_ANIMATION_FRAME,
        position,
        atlas_info.display_size,
        (powerup_type, PlayingEntity, AnimationMode::Looping),
    );
}

/// 更新履带特效

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
                        Vec3::new(0.0, 0.0, crate::constants::Z_DEFAULT + 0.1),
                        crate::constants::TANK_DISPLAY_SIZE,
                        (crate::constants::TrackChainEffect, AnimationMode::Conditional { tank_type: player_tank.tank_type }),
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
