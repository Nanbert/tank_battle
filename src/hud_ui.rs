//! HUD (Head-Up Display) 模块
//!
//! 处理游戏内的 HUD 显示，包括玩家状态、血条、蓝条等

use bevy::asset::AssetPath;
use bevy::prelude::*;

#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;

// ============================================================================
// HUD Constants
// ============================================================================

/// 血条和蓝条的总宽度
const HUD_BAR_WIDTH: f32 = 150.0;
/// 血条和蓝条的高度
const HUD_BAR_HEIGHT: f32 = 15.0;

// ============================================================================
// HUD Marker Components
// ============================================================================

/// 玩家1 HUD 容器标记
#[derive(Component, Clone)]
pub struct Player1Hud;

/// 玩家2 HUD 容器标记
#[derive(Component, Clone)]
pub struct Player2Hud;

/// 玩家名称文本标记
#[derive(Component)]
pub struct PlayerNameText;

/// 速度文本标记
#[derive(Component)]
pub struct SpeedText;

/// 射速文本标记
#[derive(Component)]
pub struct FireSpeedText;

/// 护盾文本标记
#[derive(Component)]
pub struct ProtectionText;

/// 炮弹数量文本标记
#[derive(Component)]
pub struct ShellsText;

/// 分数文本标记
#[derive(Component)]
pub struct ScoreText;

/// 穿透效果文本标记
#[derive(Component)]
pub struct PenetrateText;

/// 履带链效果文本标记
#[derive(Component)]
pub struct TrackChainText;

/// 气垫效果文本标记
#[derive(Component)]
pub struct AirCushionText;

/// 火焰炮弹效果文本标记
#[derive(Component)]
pub struct FireShellText;

/// 效果标题标记
#[derive(Component)]
pub struct EffectsTitle;

/// 玩家头像标记
#[derive(Component)]
pub struct PlayerAvatar;

/// 血条标记
#[derive(Component)]
pub struct HealthBar;

/// 蓝条标记
#[derive(Component)]
pub struct BlueBar;

// ============================================================================
// HUD Spawn Functions
// ============================================================================

/// 生成玩家 HUD
fn spawn_player_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    player1_hud_query: Query<(), With<Player1Hud>>,
    player2_hud_query: Query<(), With<Player2Hud>>,
) {
    // 只在 HUD 不存在时才创建，以保留颜色状态
    // 关卡切换时，白色背景会自然遮挡 HUD

    let font: Handle<Font> = asset_server.load(FONT_EN);

    // 玩家1 HUD
    if player1_hud_query.is_empty() {
        spawn_single_player_hud(
            &mut commands,
            &font,
            &asset_server,
            &mut texture_atlas_layouts,
            &player_info,
            TankType::Player1,
            WINDOW_LEFT_X + 115.0,
            Player1Hud,
        );
    }

    // 玩家2 HUD（仅在双人模式下）
    if *game_mode == GameMode::TwoPlayers && player2_hud_query.is_empty() {
        spawn_single_player_hud(
            &mut commands,
            &font,
            &asset_server,
            &mut texture_atlas_layouts,
            &player_info,
            TankType::Player2,
            WINDOW_RIGHT_X - 115.0,
            Player2Hud,
        );
    }
}

/// 生成单个玩家的 HUD
fn spawn_single_player_hud(
    commands: &mut Commands,
    font: &Handle<Font>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &PlayerInfo,
    player_type: TankType,
    x_pos: f32,
    marker: impl Component + Clone,
) {
    let default_stats = PlayerStats {
    name: String::new(),
    speed: 0,
    fire_speed: 0,
    protection: 0,
    shells: 0,
    penetrate: false,
    track_chain: false,
    air_cushion: false,
    fire_shell: false,
    life_points: 0,
    energy_points: 0,
    score: 0,
};
let stats = match player_type {
    TankType::Player1 => &player_info.player1,
    TankType::Player2 => player_info.player2.as_ref().unwrap_or(&default_stats),
    TankType::Enemy => &default_stats,
};

    // 玩家名称
    commands.spawn((
        marker.clone(),
        PlayerNameText,
        Text2d(stats.name.clone()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 780.0, Z_UI),
    ));

    // Speed
    commands.spawn((
        marker.clone(),
        SpeedText,
        Text2d(format!(
            "Speed:{}",
            if stats.speed >= 100 {
                "MAX".to_string()
            } else {
                format!("{}%", stats.speed)
            }
        )),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 830.0, Z_UI),
    ));

    // Fire Speed
    commands.spawn((
        marker.clone(),
        FireSpeedText,
        Text2d(format!(
            "Fire Speed:{}",
            if stats.fire_speed >= 100 {
                "MAX".to_string()
            } else {
                format!("{}%", stats.fire_speed)
            }
        )),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 880.0, Z_UI),
    ));

    // Protection
    commands.spawn((
        marker.clone(),
        ProtectionText,
        Text2d(format!(
            "Protection:{}",
            if stats.protection >= 100 {
                "MAX".to_string()
            } else {
                format!("{}%", stats.protection)
            }
        )),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 930.0, Z_UI),
    ));

    // Shells
    commands.spawn((
        marker.clone(),
        ShellsText,
        Text2d(format!("Shells: {}", stats.shells)),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 980.0, Z_UI),
    ));

    // Penetrate
    commands.spawn((
        marker.clone(),
        PenetrateText,
        Text2d(format!(
            "Penetrate: {}",
            if stats.penetrate { "On" } else { "Off" }
        )),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 420.0, Z_UI),
    ));

    // Track Chain
    commands.spawn((
        marker.clone(),
        TrackChainText,
        Text2d(format!(
            "Track Chain: {}",
            if stats.track_chain { "On" } else { "Off" }
        )),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 470.0, Z_UI),
    ));

    // Air Cushion
    commands.spawn((
        marker.clone(),
        AirCushionText,
        Text2d(format!(
            "Air Cushion: {}",
            if stats.air_cushion { "On" } else { "Off" }
        )),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 520.0, Z_UI),
    ));

    // Fire Shell
    commands.spawn((
        marker.clone(),
        FireShellText,
        Text2d(format!(
            "Fire Shell: {}",
            if stats.fire_shell { "On" } else { "Off" }
        )),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 370.0, Z_UI),
    ));

    // Effects 标题
    commands.spawn((
        marker.clone(),
        EffectsTitle,
        Text2d("Effects".to_string()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 320.0, Z_UI),
    ));

    // 分数
    commands.spawn((
        marker.clone(),
        ScoreText,
        Text2d(format!(
            "Scores{}: {}",
            if player_type == TankType::Player1 {
                "1"
            } else {
                "2"
            },
            stats.score
        )),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 50.0, Z_UI),
    ));

    // 玩家头像（使用精灵图）
    let player_avatar_texture: Handle<Image> = asset_server.load(TEXTURE_AVATAR);
    let player_avatar_tile_size = UVec2::new(160, 147);
    let player_avatar_texture_atlas = TextureAtlasLayout::from_grid(player_avatar_tile_size, 13, 3, None, None);
    let player_avatar_texture_atlas_layout = texture_atlas_layouts.add(player_avatar_texture_atlas);
    let player_avatar_animation_indices = AnimationIndices { first: 0, last: 32 };
    commands.spawn((
        marker.clone(),
        PlayerAvatar,
        Sprite {
            image: player_avatar_texture,
            texture_atlas: Some(TextureAtlas {
                layout: player_avatar_texture_atlas_layout,
                index: 0,
            }),
            custom_size: Some(Vec2::new(160.0, 147.0)),
            ..default()
        },
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 150.0, Z_UI),
        player_avatar_animation_indices,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));

    // 血条背景
    commands.spawn((
        marker.clone(),
        HealthBar,
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(HUD_BAR_WIDTH, HUD_BAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 235.0, Z_UI),
    ));

    // 血条前景（红色）
    let health_width = HUD_BAR_WIDTH * (stats.life_points as f32 / 3.0);
    commands.spawn((
        marker.clone(),
        HealthBar,
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(health_width, HUD_BAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(x_pos - HUD_BAR_WIDTH / 2.0 + health_width / 2.0, WINDOW_TOP_Y - 235.0, Z_UI + 0.1),
    ));

    // 蓝条背景
    commands.spawn((
        marker.clone(),
        BlueBar,
        Sprite {
            color: Color::srgb(0.3, 0.3, 0.3),
            custom_size: Some(Vec2::new(HUD_BAR_WIDTH, HUD_BAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - 250.0, Z_UI),
    ));

    // 蓝条前景（蓝色）
    let blue_width = HUD_BAR_WIDTH * (stats.energy_points as f32 / 3.0);
    commands.spawn((
        marker.clone(),
        BlueBar,
        Sprite {
            color: Color::srgb(0.0, 0.5, 1.0),
            custom_size: Some(Vec2::new(blue_width, HUD_BAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(x_pos - HUD_BAR_WIDTH / 2.0 + blue_width / 2.0, WINDOW_TOP_Y - 250.0, Z_UI + 0.1),
    ));
}

/// 销毁所有 HUD
pub fn despawn_hud(
    mut commands: Commands,
    top_hud_query: Query<Entity, Or<(With<StageText>, With<CommanderText>, With<CommanderHealthBar>, With<EnemyCountText>)>>,
    player_hud_query: Query<Entity, Or<(With<Player1Hud>, With<Player2Hud>)>>,
) {
    // 销毁顶部 HUD
    for entity in top_hud_query.iter() {
        let () = commands.entity(entity).try_despawn();
    }

    // 销毁玩家 HUD
    for entity in player_hud_query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}

// ============================================================================
// HUD Update Functions
// ============================================================================

/// 更新玩家1 HUD
pub fn update_player1_hud(
    player_info: Res<PlayerInfo>,
    mut text_query: Query<(&mut Text2d, Option<&SpeedText>, Option<&FireSpeedText>, Option<&ProtectionText>, Option<&ShellsText>, Option<&ScoreText>, Option<&AirCushionText>, Option<&PenetrateText>, Option<&TrackChainText>, Option<&FireShellText>), With<Player1Hud>>,
    mut bar_query: Query<(&mut Sprite, &mut Transform, Option<&HealthBar>, Option<&BlueBar>), With<Player1Hud>>,
) {
    let stats = &player_info.player1;

    // 更新所有文本
    for (mut text, is_speed, is_fire_speed, is_protection, is_shells, is_score, is_air_cushion, is_penetrate, is_track_chain, is_fire_shell) in text_query.iter_mut() {
        if is_speed.is_some() {
            text.0 = format!(
                "Speed:{}",
                if stats.speed >= 100 {
                    "MAX".to_string()
                } else {
                    format!("{}%", stats.speed)
                }
            );
        } else if is_fire_speed.is_some() {
            text.0 = format!(
                "Fire Speed:{}",
                if stats.fire_speed >= 100 {
                    "MAX".to_string()
                } else {
                    format!("{}%", stats.fire_speed)
                }
            );
        } else if is_protection.is_some() {
            text.0 = format!(
                "Protection:{}",
                if stats.protection >= 100 {
                    "MAX".to_string()
                } else {
                    format!("{}%", stats.protection)
                }
            );
        } else if is_shells.is_some() {
            text.0 = format!("Shells: {}", stats.shells);
        } else if is_score.is_some() {
            text.0 = format!("Scores1: {}", stats.score);
        } else if is_air_cushion.is_some() {
            text.0 = format!("Air Cushion:{}", if stats.air_cushion { "On" } else { "Off" });
        } else if is_penetrate.is_some() {
            text.0 = format!("Penetrate: {}", if stats.penetrate { "On" } else { "Off" });
        } else if is_track_chain.is_some() {
            text.0 = format!("Track Chain:{}", if stats.track_chain { "On" } else { "Off" });
        } else if is_fire_shell.is_some() {
            text.0 = format!("Fire Shell:{}", if stats.fire_shell { "On" } else { "Off" });
        }
    }

// 更新血条和蓝条（更新所有血条/蓝条精灵）
    for (mut sprite, mut transform, is_health_bar, is_blue_bar) in bar_query.iter_mut() {
        let color = sprite.color.to_srgba();
        if is_health_bar.is_some() && color.red > 0.5 {
            // 这是红色血条前景
            let health_width = HUD_BAR_WIDTH * (stats.life_points as f32 / 3.0);
            sprite.custom_size = Some(Vec2::new(health_width, HUD_BAR_HEIGHT));
            // 更新位置以保持左对齐
            transform.translation.x = WINDOW_LEFT_X + 115.0 - HUD_BAR_WIDTH / 2.0 + health_width / 2.0;
        } else if is_blue_bar.is_some() && color.blue > 0.5 {
            // 这是蓝色蓝条前景
            let blue_width = HUD_BAR_WIDTH * (stats.energy_points as f32 / 3.0);
            sprite.custom_size = Some(Vec2::new(blue_width, HUD_BAR_HEIGHT));
            // 更新位置以保持左对齐
            transform.translation.x = WINDOW_LEFT_X + 115.0 - HUD_BAR_WIDTH / 2.0 + blue_width / 2.0;
        }
    }
}

/// 更新玩家2 HUD
pub fn update_player2_hud(
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    mut text_query: Query<(&mut Text2d, Option<&SpeedText>, Option<&FireSpeedText>, Option<&ProtectionText>, Option<&ShellsText>, Option<&ScoreText>, Option<&AirCushionText>, Option<&PenetrateText>, Option<&TrackChainText>, Option<&FireShellText>), With<Player2Hud>>,
    mut bar_query: Query<(&mut Sprite, &mut Transform, Option<&HealthBar>, Option<&BlueBar>), With<Player2Hud>>,
) {
    if *game_mode != GameMode::TwoPlayers {
        return;
    }

    let Some(stats) = player_info.player2.as_ref() else {
        return;
    };

    // 更新所有文本
    for (mut text, is_speed, is_fire_speed, is_protection, is_shells, is_score, is_air_cushion, is_penetrate, is_track_chain, is_fire_shell) in text_query.iter_mut() {
        if is_speed.is_some() {
            text.0 = format!(
                "Speed:{}",
                if stats.speed >= 100 {
                    "MAX".to_string()
                } else {
                    format!("{}%", stats.speed)
                }
            );
        } else if is_fire_speed.is_some() {
            text.0 = format!(
                "Fire Speed:{}",
                if stats.fire_speed >= 100 {
                    "MAX".to_string()
                } else {
                    format!("{}%", stats.fire_speed)
                }
            );
        } else if is_protection.is_some() {
            text.0 = format!(
                "Protection:{}",
                if stats.protection >= 100 {
                    "MAX".to_string()
                } else {
                    format!("{}%", stats.protection)
                }
            );
        } else if is_shells.is_some() {
            text.0 = format!("Shells: {}", stats.shells);
        } else if is_score.is_some() {
            text.0 = format!("Scores2: {}", stats.score);
        } else if is_air_cushion.is_some() {
            text.0 = format!("Air Cushion:{}", if stats.air_cushion { "On" } else { "Off" });
        } else if is_penetrate.is_some() {
            text.0 = format!("Penetrate: {}", if stats.penetrate { "On" } else { "Off" });
        } else if is_track_chain.is_some() {
            text.0 = format!("Track Chain:{}", if stats.track_chain { "On" } else { "Off" });
        } else if is_fire_shell.is_some() {
            text.0 = format!("Fire Shell:{}", if stats.fire_shell { "On" } else { "Off" });
        }
    }

    // 更新血条和蓝条（更新所有血条/蓝条精灵）
    for (mut sprite, mut transform, is_health_bar, is_blue_bar) in bar_query.iter_mut() {
        let color = sprite.color.to_srgba();
        if is_health_bar.is_some() && color.red > 0.5 {
            // 这是红色血条前景
            let health_width = HUD_BAR_WIDTH * (stats.life_points as f32 / 3.0);
            sprite.custom_size = Some(Vec2::new(health_width, HUD_BAR_HEIGHT));
            // 更新位置以保持左对齐
            transform.translation.x = WINDOW_RIGHT_X - 115.0 - HUD_BAR_WIDTH / 2.0 + health_width / 2.0;
        } else if is_blue_bar.is_some() && color.blue > 0.5 {
            // 这是蓝色蓝条前景
            let blue_width = HUD_BAR_WIDTH * (stats.energy_points as f32 / 3.0);
            sprite.custom_size = Some(Vec2::new(blue_width, HUD_BAR_HEIGHT));
            // 更新位置以保持左对齐
            transform.translation.x = WINDOW_RIGHT_X - 115.0 - HUD_BAR_WIDTH / 2.0 + blue_width / 2.0;
        }
    }
}

// ============================================================================
// HUD Text Blink System
// ============================================================================

/// 处理 HUD 属性变更事件，触发文字闪烁
pub fn handle_hud_stat_changed(
    mut events: MessageReader<PlayerStatChanged>,
    mut commands: Commands,
    player1_hud_texts: Query<(Entity, &Text2d), With<Player1Hud>>,
    player2_hud_texts: Query<(Entity, &Text2d), With<Player2Hud>>,
) {
    for event in events.read() {
        let prefix = get_hud_stat_prefix(event.stat_type);

        match event.player_type {
            TankType::Player1 => {
                for (entity, text) in player1_hud_texts.iter() {
                    if text.0.starts_with(prefix) {
                        commands
                            .entity(entity)
                            .insert(PlayerInfoBlinkTimer(Timer::from_seconds(
                                TEXT_BLINK_CYCLE,
                                TimerMode::Once,
                            )));
                        break;
                    }
                }
            }
            TankType::Player2 => {
                for (entity, text) in player2_hud_texts.iter() {
                    if text.0.starts_with(prefix) {
                        commands
                            .entity(entity)
                            .insert(PlayerInfoBlinkTimer(Timer::from_seconds(
                                TEXT_BLINK_CYCLE,
                                TimerMode::Once,
                            )));
                        break;
                    }
                }
            }
            TankType::Enemy => {}
        }
    }
}

/// 动画化 HUD 文本闪烁效果
pub fn animate_hud_text(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut PlayerInfoBlinkTimer,
            &mut TextColor,
            &Text2d,
        ),
        Or<(With<Player1Hud>, With<Player2Hud>)>,
    >,
    player_info: Res<PlayerInfo>,
) {
    for (entity, mut timer, mut color, text) in &mut query {
        timer.tick(time.delta());

        // 判断是否达到最大值或On状态
        let is_max = is_hud_stat_at_max_value(&text.0, &player_info);

        if is_max {
            // 达到最大值：保持红色，移除闪烁计时器
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = Color::srgb(1.0, 0.0, 0.0); // 红色
        } else if timer.is_finished() {
            // 闪烁结束，移除计时器组件
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = Color::srgb(1.0, 1.0, 1.0);
        } else {
            // 未达到最大值：闪烁效果
            let elapsed = timer.elapsed_secs();
            let cycle = elapsed % TEXT_BLINK_CYCLE;

            if cycle < TEXT_BLINK_CYCLE / 2.0 {
                // 亮状态：绿色
                color.0 = Color::srgb(0.0, 1.0, 0.0);
            } else {
                // 灭状态：透明
                color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
            }
        }
    }
}

/// 获取 HUD 属性类型对应的前缀
fn get_hud_stat_prefix(stat_type: StatType) -> &'static str {
    match stat_type {
        StatType::Speed => "Speed:",
        StatType::Protection => "Protection:",
        StatType::FireSpeed => "Fire Speed:",
        StatType::FireShell => "Fire Shell:",
        StatType::TrackChain => "Track Chain:",
        StatType::Penetrate => "Penetrate:",
        StatType::AirCushion => "Air Cushion:",
        StatType::Shell => "Shells:",
        StatType::Score => "Scores",
    }
}

/// 判断 HUD 属性是否达到最大值或On状态
fn is_hud_stat_at_max_value(text: &str, player_info: &PlayerInfo) -> bool {
    // 检查玩家1和玩家2的属性
    let players = [&player_info.player1]
        .into_iter()
        .chain(player_info.player2.as_ref().into_iter());
    for player_stats in players {
        if text.starts_with("Shells:") {
            if player_stats.shells >= 2 {
                return true;
            }
        } else if text.starts_with("Speed:") {
            if player_stats.speed >= 100 {
                return true;
            }
        } else if text.starts_with("Protection:") {
            if player_stats.protection >= 100 {
                return true;
            }
        } else if text.starts_with("Fire Speed:") {
            if player_stats.fire_speed >= 100 {
                return true;
            }
        } else if text.starts_with("Fire Shell:") {
            if player_stats.fire_shell {
                return true;
            }
        } else if text.starts_with("Air Cushion:") {
            if player_stats.air_cushion {
                return true;
            }
        } else if text.starts_with("Track Chain:") {
            if player_stats.track_chain {
                return true;
            }
        } else if text.starts_with("Penetrate:") {
            if player_stats.penetrate {
                return true;
            }
        }
    }
    false
}

/// 生成顶部 HUD（关卡信息、司令官血条、敌方坦克数量）
fn spawn_top_hud(mut commands: Commands, asset_server: &Res<AssetServer>, stage_level: &Res<StageLevel>) {
    let font: Handle<Font> = asset_server.load(FONT_EN);
    // 其他游戏信息 UI 元素配置
    let commander_text_x = WINDOW_LEFT_X + 435.0; // 往左平移30像素

    // 关卡信息显示在顶部中心
    commands.spawn((
        PlayingEntity,
        StageText,
        Text2d(format!("Stage {}", stage_level.0)),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色
        Transform::from_xyz(0.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));

    commands.spawn((
        PlayingEntity,
        CommanderText,
        Text2d("Commander Life:".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(commander_text_x - 42.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
    // Commander 血条（与玩家血条长度相同：160像素），放在文字正右方
    commands.spawn((
        PlayingEntity,
        CommanderHealthBar,
        CommanderHealthBarOriginalPosition(commander_text_x + 172.0), // 文字右侧
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(160.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(commander_text_x + 172.0, WINDOW_TOP_Y - 50.0, 1.0), // 与文字同一Y坐标
    ));
    commands.spawn((
        PlayingEntity,
        EnemyCountText,
        Text2d("Enemy Left: 5/5".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(WINDOW_RIGHT_X - 465.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
}

/// 生成所有 HUD（只在第一关时生成）
pub fn spawn_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    player1_hud_query: Query<(), With<Player1Hud>>,
    player2_hud_query: Query<(), With<Player2Hud>>,
    top_hud_query: Query<Entity, Or<(With<StageText>, With<CommanderText>, With<CommanderHealthBar>, With<EnemyCountText>)>>,
    player_hud_query: Query<Entity, Or<(With<Player1Hud>, With<Player2Hud>)>>,
    stage_level: Res<StageLevel>,
) {
    // 先清理现有的 HUD，以防万一
    for entity in top_hud_query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
    for entity in player_hud_query.iter() {
        let () = commands.entity(entity).try_despawn();
    }

    spawn_top_hud(commands.reborrow(), &asset_server, &stage_level);
    spawn_player_hud(commands, asset_server, texture_atlas_layouts, player_info, game_mode, player1_hud_query, player2_hud_query);
}

/// 更新关卡信息文本
pub fn update_stage_text(
    stage_level: Res<StageLevel>,
    mut stage_text_query: Query<&mut Text2d, With<StageText>>,
) {
    for mut text in &mut stage_text_query {
        text.0 = format!("Stage {}", stage_level.0);
    }
}

/// 更新 Commander 血条
pub fn update_commander_health_bar(
    commander_life: Res<CommanderLife>,
    mut health_bars: Query<
        (
            &mut Sprite,
            &CommanderHealthBarOriginalPosition,
            &mut Transform,
        ),
        With<CommanderHealthBar>,
    >,
) {
    for (mut sprite, original_pos, mut transform) in &mut health_bars {
        let health_width = (commander_life.life_points as f32 / 3.0) * COMMANDER_BAR_WIDTH;
        sprite.custom_size = Some(Vec2::new(health_width, BAR_HEIGHT));
        transform.translation.x = original_pos.0 - (COMMANDER_BAR_WIDTH - health_width) / 2.0;
    }
}

/// 更新敌方坦克数量显示
pub fn update_enemy_count_display(
    enemy_spawn_state: Res<crate::resources::EnemySpawnState>,
    mut query: Query<&mut Text2d, With<EnemyCountText>>,
) {
    let remaining = enemy_spawn_state.max_count - enemy_spawn_state.has_spawned;

    for mut text in &mut query {
        text.0 = format!("Enemy Left: {}/{}", remaining, enemy_spawn_state.max_count);
    }
}

/// 动画玩家头像（只在存活时播放）
pub fn animate_player_avatar(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
            Has<PlayerDead>,
        ),
        With<PlayerAvatar>,
    >,
) {
    for (mut timer, mut sprite, indices, mut current_frame, is_dead) in &mut query {
        // 玩家已死亡，不播放动画
        if is_dead {
            continue;
        }

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

/// 处理玩家头像死亡状态
pub fn handle_player_avatar_death(
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut Sprite, Has<PlayerDead>), With<PlayerAvatar>>,
) {
    let avatar_death_path = AssetPath::from("texture/avatar_death.png");
    for (mut sprite, is_dead) in &mut query {
        if is_dead && sprite.image.path() != Some(&avatar_death_path) {
            // 切换到死亡头像纹理
            let avatar_dead_texture: Handle<Image> = asset_server.load(TEXTURE_AVATAR_DEATH);
            sprite.image = avatar_dead_texture.clone();
            sprite.texture_atlas = None; // 死亡头像不需要动画
            sprite.custom_size = Some(Vec2::new(160.0, 147.0));
        }
    }
}

/// 处理司令官阵亡时更换纹理和头像
pub fn handle_commander_death(
    asset_server: Res<AssetServer>,
    commander_life: Res<CommanderLife>,
    mut queries: ParamSet<(
        Query<&mut Sprite, With<Commander>>,
        Query<&mut Sprite, With<PlayerAvatar>>,
        Query<&mut AnimationTimer, With<Commander>>,
        Query<&mut AnimationTimer, With<CommanderMusicAnimation>>,
    )>,
    mut has_handled: Local<bool>,
) {
    // 只在司令官生命值归零时执行一次
    if commander_life.life_points != 0 {
        *has_handled = false;
        return;
    }

    // 如果已经处理过，跳过
    if *has_handled {
        return;
    }

    *has_handled = true;

    // 更换司令官纹理为死亡纹理
    for mut sprite in &mut queries.p0() {
        sprite.image = asset_server.load(TEXTURE_COMMANDER_DEAD);
        // 移除纹理图集，因为死亡纹理是单张图片
        sprite.texture_atlas = None;
    }

    // 停止司令官动画
    for mut timer in &mut queries.p2() {
        timer.pause();
    }

    // 停止司令官音乐动画
    for mut timer in &mut queries.p3() {
        timer.pause();
    }

    // 更换所有玩家头像为死亡头像
    for mut sprite in &mut queries.p1() {
        sprite.image = asset_server.load(TEXTURE_AVATAR_COMMANDER_DEAD);
        // 移除纹理图集，因为死亡头像纹理是单张图片
        sprite.texture_atlas = None;
    }
}


