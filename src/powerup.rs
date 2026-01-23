//! 道具系统
//!
//! 处理道具动画和玩家拾取道具的碰撞检测

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::{PlayerInfo, CommanderLife, PlayerStatChanged, StatType};

/// 道具动画系统
pub fn animate_powerup(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<PowerUp>>,
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

/// 道具碰撞检测和拾取系统
pub fn handle_powerup_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    powerups: Query<(Entity, &Transform, &PowerUp)>,
    player_tanks: Query<(&Transform, &PlayerTank, Entity), With<PlayerTank>>,
    mut controllers: Query<&mut KinematicCharacterController>,
    mut player_info: ResMut<PlayerInfo>,
    mut commander_life: ResMut<CommanderLife>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
) {
    for (tank_transform, player_tank, tank_entity) in player_tanks.iter() {
        let mut picked_powerup: Option<PowerUp> = None;
        let mut powerup_entity_to_despawn: Option<Entity> = None;

        // 检查道具碰撞
        for (powerup_entity, powerup_transform, powerup_type) in powerups.iter() {
            let distance = (powerup_transform.translation - tank_transform.translation).length();
            if distance < 81.0 {
                picked_powerup = Some(*powerup_type);
                powerup_entity_to_despawn = Some(powerup_entity);
            }
        }

        if let Some(powerup_type) = picked_powerup {
            let powerup_entity = powerup_entity_to_despawn.unwrap();

            // 播放道具音效
            let powerup_sound: Handle<AudioSource> = asset_server.load(SOUND_POWERUP);
            commands.spawn(AudioPlayer::new(powerup_sound));
            let () = commands.entity(powerup_entity).try_despawn();

            // 根据道具类型应用效果并发送事件
            if let Some(player_stats) = player_info.players.get_mut(&player_tank.tank_type) {
                let stat_type = match powerup_type {
                    PowerUp::SpeedUp => {
                        if player_stats.speed < 100 {
                            player_stats.speed += 20;
                        }
                        Some(StatType::Speed)
                    }
                    PowerUp::Protection => {
                        if player_stats.protection < 100 {
                            player_stats.protection += 20;
                        }
                        Some(StatType::Protection)
                    }
                    PowerUp::FireSpeed => {
                        if player_stats.fire_speed < 100 {
                            player_stats.fire_speed += 20;
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
                        if player_stats.life_red_bar < 3 {
                            player_stats.life_red_bar += 1;
                        }
                        None // 修理道具不需要闪烁文字
                    }
                    PowerUp::Hamburger => {
                        if commander_life.life_red_bar < 3 {
                            commander_life.life_red_bar += 1;
                        }
                        None // 汉堡道具不影响玩家属性，不发送事件
                    }
                    PowerUp::AirCushion => {
                        player_stats.air_cushion = true;
                        // 更新 filter_groups，排除海（GROUP_2）
                        // 玩家坦克不设置 memberships（默认所有组），filters 设置为不包含 GROUP_2
                        if let Ok(mut controller) = controllers.get_mut(tank_entity) {
                            controller.filter_groups = Some(CollisionGroups::new(Group::all(), Group::all() & !SEA_GROUP));
                        }
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

                // 发送属性变更事件（如果有）
                if let Some(st) = stat_type {
                    stat_changed_events.write(PlayerStatChanged {
                        player_type: player_tank.tank_type,
                        stat_type: st,
                    });
                }
            }
        }
    }
}
