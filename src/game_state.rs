//! 游戏状态管理模块
//!
//! 处理游戏关卡、游戏结束、游戏重置等状态管理

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::*;
pub fn handle_game_over_delay(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer), With<GameOverTimer>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut timer) in &mut query {
        timer.tick(time.delta());
        if timer.is_finished() {
            let _ = commands.entity(entity).try_despawn();
            next_state.set(GameState::GameOver);
        }
    }
}

pub fn check_game_over(
    mut commands: Commands,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    commander_life: Res<CommanderLife>,
    existing_timers: Query<(), With<GameOverTimer>>,
) {
    // 如果已经存在 GameOverTimer，说明已经触发了 GameOver，不再重复触发
    if !existing_timers.is_empty() {
        return;
    }

    // 检测 Commander 血量是否为 0
    if commander_life.life_red_bar == 0 {
        // 启动 Game Over 延迟计时器（1.2秒），等待爆炸动画完成
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
        ));
        return;
    }

    // 检测所有玩家生命值是否都为 0
    let all_players_dead = if player_info.players.is_empty() {
        false
    } else {
        match *game_mode {
            GameMode::OnePlayer => {
                player_info.players.get(&TankType::Player1).map_or(false, |p| p.life_red_bar == 0)
            }
            GameMode::TwoPlayers => {
                player_info.players.get(&TankType::Player1).map_or(false, |p| p.life_red_bar == 0)
                    && player_info.players.get(&TankType::Player2).map_or(false, |p| p.life_red_bar == 0)
            }
        }
    };

    if all_players_dead {
        // 启动 Game Over 延迟计时器（1.2秒）
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
        ));
    }
}

fn setup_fade_out(
    mut fading_out: ResMut<FadingOut>,
) {
    fading_out.alpha = 1.0;
}

// 文本更新函数类型
type TextUpdateFn = fn(&PlayerStats, TankType) -> Option<String>;

fn get_text_update_fn(prefix: &str) -> TextUpdateFn {
    match prefix {
        s if s.starts_with("Scores") => |stats, _| {
            Some(format!("Scores: {}", stats.score))
        },
        s if s.starts_with("Speed") => |stats, _| {
            Some(if stats.speed < 100 {
                format!("Speed: {}%", stats.speed)
            } else {
                "Speed: Max".to_string()
            })
        },
        s if s.starts_with("Shells") => |stats, _| {
            Some(format!("Shells: {}", stats.shells))
        },
        s if s.starts_with("Protection") => |stats, _| {
            Some(if stats.protection < 100 {
                format!("Protection: {}%", stats.protection)
            } else {
                "Protection: Max".to_string()
            })
        },
        s if s.starts_with("Fire Speed") => |stats, _| {
            Some(if stats.fire_speed < 100 {
                format!("Fire Speed: {}%", stats.fire_speed)
            } else {
                "Fire Speed: Max".to_string()
            })
        },
        s if s.starts_with("Fire Shell") => |stats, _| {
            if stats.fire_shell {
                Some("Fire Shell: On".to_string())
            } else {
                Some("Fire Shell: Off".to_string())
            }
        },
        s if s.starts_with("Air Cushion") => |stats, _| {
            if stats.air_cushion {
                Some("Air Cushion: On".to_string())
            } else {
                Some("Air Cushion: Off".to_string())
            }
        },
        s if s.starts_with("Track Chain") => |stats, _| {
            if stats.track_chain {
                Some("Track Chain: On".to_string())
            } else {
                Some("Track Chain: Off".to_string())
            }
        },
        s if s.starts_with("Penetrate") => |stats, _| {
            if stats.penetrate {
                Some("Penetrate: On".to_string())
            } else {
                Some("Penetrate: Off".to_string())
            }
        },
        _ => |_, _| None,
    }
}

fn update_player_info_display(
    changed_player_info: Res<PlayerInfo>,
    mut text2ds: Query<(&PlayerUI, &mut Text2d), With<Text2d>>,
    mut bar_queries: ParamSet<(
        Query<(&mut Sprite, &HealthBarOriginalPosition, &mut Transform, &PlayerUI), With<HealthBar>>,
        Query<(&mut Sprite, &BlueBarOriginalPosition, &mut Transform, &PlayerUI), With<BlueBar>>,
    )>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
) {
    for player_tank in player_tanks {
        if let Some(player_stats) = changed_player_info.players.get(&player_tank.tank_type) {
            // 更新文本信息
            for (player_index, mut text) in &mut text2ds {
                if player_tank.tank_type != player_index.player_type {
                    continue;
                }
                let update_fn = get_text_update_fn(&text.0);
                if let Some(new_text) = update_fn(player_stats, player_index.player_type) {
                    text.0 = new_text;
                }
            }

            // 更新血条
            for (mut sprite, original_pos, mut transform, player_index) in &mut bar_queries.p0() {
                if player_tank.tank_type != player_index.player_type {
                    continue;
                }
                // 血条总宽度 160，生命值 3，每条代表 1/3
                let health_width = (player_stats.life_red_bar as f32 / 3.0) * 160.0;
                sprite.custom_size = Some(Vec2::new(health_width, 10.0));

                // 左对齐：将血条向左移动，使其从左边界开始
                // 原始位置是中心点，需要向左偏移 (160 - health_width) / 2
                let offset = (160.0 - health_width) / 2.0;
                transform.translation.x = original_pos.0 - offset;
            }

            // 更新蓝条
            for (mut sprite, original_pos, mut transform, player_index) in &mut bar_queries.p1() {
                if player_tank.tank_type != player_index.player_type {
                    continue;
                }
                // 蓝条总宽度 160，能量值 100
                let blue_width = (player_stats.energy_blue_bar as f32 / 3.0) * 160.0;
                sprite.custom_size = Some(Vec2::new(blue_width, 10.0));

                // 左对齐：将蓝条向左移动，使其从左边界开始
                // 原始位置是中心点，需要向左偏移 (160 - blue_width) / 2
                let offset = (160.0 - blue_width) / 2.0;
                transform.translation.x = original_pos.0 - offset;
            }
        }
    }
}

fn update_commander_health_bar(
    changed_commander_life: Res<CommanderLife>,
    mut health_bars: Query<(&mut Sprite, &CommanderHealthBarOriginalPosition, &mut Transform), With<CommanderHealthBar>>,
) {
    for (mut sprite, original_pos, mut transform) in &mut health_bars {
        let health_width = (changed_commander_life.life_red_bar as f32 / 3.0) * 160.0;
        sprite.custom_size = Some(Vec2::new(health_width, 10.0));
        transform.translation.x = original_pos.0 - (160.0 - health_width) / 2.0;
    }
}

fn update_blue_bar_regen(
    time: Res<Time>,
    mut regen_timer: ResMut<BlueBarRegenTimer>,
    mut player_info: ResMut<PlayerInfo>,
) {
    // 检查是否有玩家蓝条不满
    let any_player_needs_regen = player_info.players.values().any(|p| p.energy_blue_bar < 3);

    // 只有当有玩家蓝条不满时才更新计时器
    if any_player_needs_regen {
        regen_timer.timer.tick(time.delta());

        // 当计时器触发时，恢复1点蓝条
        if regen_timer.timer.just_finished() {
            for player_stats in player_info.players.values_mut() {
                if player_stats.energy_blue_bar < 3 {
                    player_stats.energy_blue_bar = (player_stats.energy_blue_bar + 1).min(3);
                }
            }
        }
    } else {
        // 所有玩家蓝条都满时，重置计时器
        regen_timer.timer.reset();
    }
}

fn update_menu_blink(
    time: Res<Time>,
    fading_out: Res<FadingOut>,
    menu_selection: Res<CurrentMenuSelection>,
    mut blink_timer: ResMut<MenuBlinkTimer>,
    mut text_query: Query<(&MenuOption, &mut TextColor), Without<MenuArrow>>,
    game_state: Res<State<GameState>>,
) {
    // 在 FadingOut 状态下闪烁 + 淡出
    if *game_state.get() == GameState::FadingOut {
        blink_timer.0.tick(time.delta());

        // 初始化计时器（0.2秒闪烁）
        if blink_timer.0.duration().is_zero() {
            blink_timer.0 = Timer::from_seconds(0.2, TimerMode::Repeating);
        }

        if blink_timer.0.just_finished() {
            for (option, mut text_color) in &mut text_query {
                if option.index == menu_selection.selected_index {
                    // 当前选中的选项闪烁
                    // 出现时使用当前淡出透明度，消失时完全透明
                    let linear = text_color.0.to_linear();
                    let alpha = if linear.alpha < 0.5 {
                        // 当前不可见，切换到可见（使用当前淡出透明度）
                        fading_out.alpha
                    } else {
                        // 当前可见，切换到不可见（完全透明）
                        0.0
                    };
                    text_color.0 = Color::srgb(1.0, 1.0, 0.0).with_alpha(alpha);
                }
            }
        }
    } else {
        // 在 StartScreen 状态下，选中的选项保持黄色常亮
        for (option, mut text_color) in &mut text_query {
            if option.index == menu_selection.selected_index {
                text_color.0 = Color::srgb(1.0, 1.0, 0.0);
            }
        }
    }
}

pub fn cleanup_playing_entities(
    mut commands: Commands,
    playing_entities: Query<Entity, With<PlayingEntity>>,
    mut player_info: ResMut<PlayerInfo>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    mut stage_level: ResMut<StageLevel>,
    mut commander_life: ResMut<CommanderLife>,
    mut entities_spawned: ResMut<GameEntitiesSpawned>,
) {
    // 清理所有游戏实体
    for entity in playing_entities.iter() {
        commands.entity(entity).try_despawn();
    }

    // 重置玩家信息
    player_info.players.clear();

    // 重置敌方坦克计数
    enemy_spawn_state.has_spawned = 0;
    enemy_spawn_state.spawn_cooldown.reset();

    // 重置关卡数
    stage_level.0 = 1;

    // 重置 Commander 生命值
    commander_life.life_red_bar = 3;

    // 重置游戏实体生成标志
    entities_spawned.0 = false;
}

pub fn check_stage_complete(
    enemy_spawn_state: Res<EnemySpawnState>,
    enemies: Query<(), With<EnemyTank>>,
    player_info: Res<PlayerInfo>,
    commander_life: Res<CommanderLife>,
    game_mode: Res<GameMode>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stage_level: ResMut<StageLevel>,
) {
    // 检查是否完成关卡：已生成所有敌方坦克且当前没有存活的敌方坦克
    let current_enemy_count = enemies.iter().count();
    if enemy_spawn_state.has_spawned >= enemy_spawn_state.max_count && current_enemy_count == 0 {
        // 检查玩家是否阵亡
        let all_players_dead = if player_info.players.is_empty() {
            false
        } else {
            match *game_mode {
                GameMode::OnePlayer => {
                    player_info.players.get(&TankType::Player1).map_or(false, |p| p.life_red_bar == 0)
                }
                GameMode::TwoPlayers => {
                    player_info.players.get(&TankType::Player1).map_or(false, |p| p.life_red_bar == 0)
                        && player_info.players.get(&TankType::Player2).map_or(false, |p| p.life_red_bar == 0)
                }
            }
        };

        // 如果玩家或 Commander 已阵亡，不能进入下一关
        if all_players_dead || commander_life.life_red_bar == 0 {
            return;
        }

        // 进入下一关
        stage_level.0 += 1;
        next_state.set(GameState::StageIntro);
    }
}

pub fn reset_for_next_stage(
    mut commands: Commands,
    playing_entities: Query<Entity, With<PlayingEntity>>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    mut entities_spawned: ResMut<GameEntitiesSpawned>,
) {
    // 清理所有游戏实体
    for entity in playing_entities.iter() {
        commands.entity(entity).try_despawn();
    }

    // 重置敌方坦克计数
    enemy_spawn_state.has_spawned = 0;
    enemy_spawn_state.spawn_cooldown.reset();

    // 重置游戏实体生成标志
    entities_spawned.0 = false;
}

fn animate_sea(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Sea>>,
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

fn play_sea_ambience(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    seas: Query<&Transform, With<Sea>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<SeaAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在海附近
    let mut is_near_sea = false;
    const DETECTION_RADIUS: f32 = 150.0; // 海检测半径

    for player_transform in player_tanks.iter() {
        for sea_transform in seas.iter() {
            let distance = player_transform.translation.distance(sea_transform.translation);
            if distance < DETECTION_RADIUS {
                is_near_sea = true;
                break;
            }
        }
        if is_near_sea {
            break;
        }
    }

    if is_near_sea {
        // 如果在海附近但没有播放音效，则播放
        if ambience_players.is_empty() {
            let sea_ambience_sound: Handle<AudioSource> = asset_server.load(SOUND_SEA_AMBIENCE);
            commands.spawn((
                AudioPlayer::new(sea_ambience_sound),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.3)),
                SeaAmbiencePlayer,
            ));
        }
    } else {
        // 如果不在海附近但有播放音效，则停止
        for (entity, _) in ambience_players.iter() {
            let _ = commands.entity(entity).try_despawn();
        }
    }
}

    fn update_air_cushion_effect(
    
        mut commands: Commands,
    
        asset_server: Res<AssetServer>,
    
        player_tanks: Query<(Entity, Option<&Children>, Has<crate::constants::BubbleEffect>), With<PlayerTank>>,
    
        bubble_effects: Query<&crate::constants::BubbleEffect>,
    
    ) {
    
        for (entity, children, has_bubble_effect) in player_tanks.iter() {
    
            if has_bubble_effect {
    
                // 检查是否已经有气泡特效子实体
    
                let has_bubble_sprite = if let Some(children) = children {
    
                    children.iter().any(|child| bubble_effects.contains(child))
    
                } else {
    
                    false
    
                };
    
        
    
                if !has_bubble_sprite {
    
                    // 加载气泡纹理并缩放到 100x100
    
                                        let bubble_texture: Handle<Image> = asset_server.load(TEXTURE_BUBBLE);
    
                                        
    
                                        
    
                                        // 创建气泡特效实体
    
                                        
    
                                        
    
                                                                                commands.entity(entity).with_children(|parent| {
    
                                        
    
                                        
    
                                                                                    parent.spawn((
    
                                        
    
                                        
    
                                                                                        Sprite {
    
                                        
    
                                        
    
                                                                                            image: bubble_texture,
    
                                        
    
                                        
    
                                                                                            custom_size: Some(Vec2::new(100.0, 100.0)),
    
                                        
    
                                        
    
                                                                                            ..default()
    
                                        
    
                                        
    
                                                                                        },
    
                            Transform::from_xyz(0.0, 0.0, 1.0), // 在坦克中心
    
                            crate::constants::BubbleEffect,
    
                        ));
    
                    });
    
                }
    
            } else {
    
                // 移除所有气泡特效子实体
    
                if let Some(children) = children {

                    for child in children.iter() {

                        if bubble_effects.contains(child) {

                            let _ = commands.entity(child).try_despawn();

                        }

                    }

                }

            }

        }

    }
