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
#[derive(Component, Clone)]
pub struct SpeedText;

/// 射速文本标记
#[derive(Component, Clone)]
pub struct FireSpeedText;

/// 护盾文本标记
#[derive(Component, Clone)]
pub struct ProtectionText;

/// 炮弹数量文本标记
#[derive(Component, Clone)]
pub struct ShellsText;

/// 分数文本标记
#[derive(Component, Clone)]
pub struct ScoreText;

/// 穿透效果文本标记
#[derive(Component, Clone)]
pub struct PenetrateText;

/// 履带链效果文本标记
#[derive(Component, Clone)]
pub struct TrackChainText;

/// 气垫效果文本标记
#[derive(Component, Clone)]
pub struct AirCushionText;

/// 火焰炮弹效果文本标记
#[derive(Component, Clone)]
pub struct FireShellText;

/// 效果标题标记
#[derive(Component, Clone)]
pub struct EffectsTitle;

/// 玩家头像标记
#[derive(Component, Clone)]
pub struct PlayerAvatar;

/// 血条标记
#[derive(Component, Clone)]
pub struct HealthBar;

/// 蓝条标记
#[derive(Component, Clone)]
pub struct BlueBar;

/// 血条前景标记
#[derive(Component, Clone)]
pub struct HealthBarForeground;

/// 蓝条前景标记
#[derive(Component, Clone)]
pub struct BlueBarForeground;

// ============================================================================
// HUD Text Prefix Constants
// ============================================================================

/// HUD 文本前缀常量
const HUD_PREFIX_SPEED: &str = "Speed:";
const HUD_PREFIX_FIRE_SPEED: &str = "Fire Speed:";
const HUD_PREFIX_PROTECTION: &str = "Protection:";
const HUD_PREFIX_SHELLS: &str = "Shells:";
const HUD_PREFIX_SCORES: &str = "Scores";
const HUD_PREFIX_PENETRATE: &str = "Penetrate:";
const HUD_PREFIX_TRACK_CHAIN: &str = "Track Chain:";
const HUD_PREFIX_AIR_CUSHION: &str = "Air Cushion:";
const HUD_PREFIX_FIRE_SHELL: &str = "Fire Shell:";



// ============================================================================
// HUD Spawn Functions
// ============================================================================

/// 生成玩家 HUD
fn spawn_player_hud(
    mut commands: Commands,
    font_resources: Res<FontResources>,
    commander_resources: Res<CommanderResources>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    player1_hud_query: Query<(), With<Player1Hud>>,
    player2_hud_query: Query<(), With<Player2Hud>>,
) {
    // 只在 HUD 不存在时才创建，以保留颜色状态
    // 关卡切换时，白色背景会自然遮挡 HUD

    let font = font_resources.en.clone();

    // 玩家1 HUD
    if player1_hud_query.is_empty() {
        spawn_single_player_hud(
            &mut commands,
            &font,
            &commander_resources,
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
            &commander_resources,
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
    commander_resources: &CommanderResources,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &PlayerInfo,
    player_type: TankType,
    x_pos: f32,
    marker: impl Component + Clone,
) {
    let stats = get_player_stats_for_spawn(player_info, player_type);

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
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_NAME, Z_UI),
    ));

    // 属性文本（百分比类型）
    spawn_percent_text(
        commands,
        marker.clone(),
        SpeedText,
        font,
        HUD_PREFIX_SPEED,
        stats.speed,
        HUD_Y_SPEED,
        x_pos,
    );
    spawn_percent_text(
        commands,
        marker.clone(),
        FireSpeedText,
        font,
        HUD_PREFIX_FIRE_SPEED,
        stats.fire_speed,
        HUD_Y_FIRE_SPEED,
        x_pos,
    );
    spawn_percent_text(
        commands,
        marker.clone(),
        ProtectionText,
        font,
        HUD_PREFIX_PROTECTION,
        stats.protection,
        HUD_Y_PROTECTION,
        x_pos,
    );

    // 炮弹数量
    spawn_stat_text(
        commands,
        marker.clone(),
        ShellsText,
        font,
        &format!("{} {}", HUD_PREFIX_SHELLS, stats.shells),
        HUD_Y_SHELLS,
        x_pos,
    );

    // 效果标题
    commands.spawn((
        marker.clone(),
        EffectsTitle,
        Text2d("Effects".to_string()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_EFFECTS_TITLE, Z_UI),
    ));

    // 效果文本（布尔类型）
    spawn_effect_text(
        commands,
        marker.clone(),
        FireShellText,
        font,
        HUD_PREFIX_FIRE_SHELL,
        stats.fire_shell,
        HUD_Y_FIRE_SHELL,
        x_pos,
    );
    spawn_effect_text(
        commands,
        marker.clone(),
        PenetrateText,
        font,
        HUD_PREFIX_PENETRATE,
        stats.penetrate,
        HUD_Y_PENETRATE,
        x_pos,
    );
    spawn_effect_text(
        commands,
        marker.clone(),
        TrackChainText,
        font,
        HUD_PREFIX_TRACK_CHAIN,
        stats.track_chain,
        HUD_Y_TRACK_CHAIN,
        x_pos,
    );
    spawn_effect_text(
        commands,
        marker.clone(),
        AirCushionText,
        font,
        HUD_PREFIX_AIR_CUSHION,
        stats.air_cushion,
        HUD_Y_AIR_CUSHION,
        x_pos,
    );

    // 分数
    commands.spawn((
        marker.clone(),
        ScoreText,
        Text2d(format!("Scores: {}", stats.score)),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_SCORE, Z_UI),
    ));

    // 玩家头像（使用精灵图）
    let player_avatar_texture = commander_resources.avatar.clone();
    let player_avatar_tile_size = UVec2::new(
        PLAYER_AVATAR_TILE_WIDTH as u32,
        PLAYER_AVATAR_TILE_HEIGHT as u32,
    );
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
            custom_size: Some(Vec2::new(PLAYER_AVATAR_DISPLAY_WIDTH, PLAYER_AVATAR_DISPLAY_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_AVATAR, Z_UI),
        player_avatar_animation_indices,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));

    // 血条背景
    spawn_bar(
        commands,
        marker.clone(),
        COLOR_GRAY,
        x_pos,
        WINDOW_TOP_Y - HUD_BAR_Y_OFFSET_HEALTH,
        HUD_BAR_WIDTH,
        Z_UI,
    );

    // 血条前景（红色）
    let health_width = HUD_BAR_WIDTH * (stats.life_points as f32 / HUD_MAX_LIFE_POINTS);
    commands.spawn((
        marker.clone(),
        HealthBar,
        HealthBarForeground,
        Sprite {
            color: COLOR_RED,
            custom_size: Some(Vec2::new(health_width, HUD_BAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(
            x_pos - HUD_BAR_WIDTH / 2.0 + health_width / 2.0,
            WINDOW_TOP_Y - HUD_BAR_Y_OFFSET_HEALTH,
            Z_UI + 0.1,
        ),
    ));

    // 蓝条背景
    spawn_bar(
        commands,
        marker.clone(),
        COLOR_GRAY,
        x_pos,
        WINDOW_TOP_Y - HUD_BAR_Y_OFFSET_BLUE,
        HUD_BAR_WIDTH,
        Z_UI,
    );

    // 蓝条前景（蓝色）
    let blue_width = HUD_BAR_WIDTH * (stats.energy_points as f32 / HUD_MAX_LIFE_POINTS);
    commands.spawn((
        marker.clone(),
        BlueBar,
        BlueBarForeground,
        Sprite {
            color: COLOR_BLUE,
            custom_size: Some(Vec2::new(blue_width, HUD_BAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(
            x_pos - HUD_BAR_WIDTH / 2.0 + blue_width / 2.0,
            WINDOW_TOP_Y - HUD_BAR_Y_OFFSET_BLUE,
            Z_UI + 0.1,
        ),
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
// HUD Helper Functions
// ============================================================================

/// 生成血条或蓝条
fn spawn_bar<T: Component + Clone>(
    commands: &mut Commands,
    marker: T,
    color: Color,
    x: f32,
    y: f32,
    width: f32,
    z: f32,
) {
    commands.spawn((
        marker,
        Sprite {
            color,
            custom_size: Some(Vec2::new(width, HUD_BAR_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(x, y, z),
    ));
}

/// 生成属性文本
fn spawn_stat_text<T1: Component, T2: Component>(
    commands: &mut Commands,
    marker1: T1,
    marker2: T2,
    font: &Handle<Font>,
    text: &str,
    y_offset: f32,
    x_pos: f32,
) {
    commands.spawn((
        marker1,
        marker2,
        Text2d(text.to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - y_offset, Z_UI),
    ));
}

/// 生成效果开关文本
fn spawn_effect_text<T1: Component, T2: Component>(
    commands: &mut Commands,
    marker1: T1,
    marker2: T2,
    font: &Handle<Font>,
    prefix: &str,
    value: bool,
    y_offset: f32,
    x_pos: f32,
) {
    spawn_stat_text(
        commands,
        marker1,
        marker2,
        font,
        &format!("{}: {}", prefix, if value { "On" } else { "Off" }),
        y_offset,
        x_pos,
    );
}

/// 生成百分比属性文本
fn spawn_percent_text<T1: Component, T2: Component>(
    commands: &mut Commands,
    marker1: T1,
    marker2: T2,
    font: &Handle<Font>,
    prefix: &str,
    value: usize,
    y_offset: f32,
    x_pos: f32,
) {
    let text = if value >= HUD_MAX_PERCENT {
        format!("{}MAX", prefix)
    } else {
        format!("{}{}%", prefix, value)
    };
    spawn_stat_text(commands, marker1, marker2, font, &text, y_offset, x_pos);
}

/// 获取玩家统计数据（用于 spawn_single_player_hud）
fn get_player_stats_for_spawn(player_info: &PlayerInfo, player_type: TankType) -> PlayerStats {
    match player_type {
        TankType::Player1 => player_info.player1.clone(),
        TankType::Player2 => player_info
            .player2
            .as_ref()
            .cloned()
            .unwrap_or_default(),
        TankType::Enemy => PlayerStats::default(),
    }
}

// ============================================================================
// HUD Helper Functions
// ============================================================================

/// 格式化百分比属性值
fn format_percent_value(value: usize, is_max: bool) -> String {
    if is_max {
        "MAX".to_string()
    } else {
        format!("{}%", value)
    }
}

/// 格式化布尔值
fn format_bool_value(value: bool) -> &'static str {
    if value { "On" } else { "Off" }
}

/// 更新血条或蓝条
fn update_bar(
    sprite: &mut Sprite,
    transform: &mut Transform,
    value: f32,
    max_value: f32,
    base_x: f32,
    bar_width: f32,
) {
    let width = bar_width * (value / max_value);
    sprite.custom_size = Some(Vec2::new(width, HUD_BAR_HEIGHT));
    transform.translation.x = base_x - bar_width / 2.0 + width / 2.0;
}

/// 更新单个玩家的文本
fn update_single_player_text(stats: &PlayerStats, text: &mut String) {
    // 百分比属性
    let percent_attrs = [
        (HUD_PREFIX_SPEED, stats.speed),
        (HUD_PREFIX_FIRE_SPEED, stats.fire_speed),
        (HUD_PREFIX_PROTECTION, stats.protection),
    ];

    for (prefix, value) in percent_attrs {
        if text.starts_with(prefix) {
            *text = format!(
                "{}{}",
                prefix,
                format_percent_value(value, value >= HUD_MAX_PERCENT)
            );
            return;
        }
    }

    // 布尔效果属性
    let effect_attrs = [
        (HUD_PREFIX_FIRE_SHELL, stats.fire_shell),
        (HUD_PREFIX_PENETRATE, stats.penetrate),
        (HUD_PREFIX_TRACK_CHAIN, stats.track_chain),
        (HUD_PREFIX_AIR_CUSHION, stats.air_cushion),
    ];

    for (prefix, value) in effect_attrs {
        if text.starts_with(prefix) {
            *text = format!("{}: {}", prefix, format_bool_value(value));
            return;
        }
    }

    // 其他属性
    if text.starts_with(HUD_PREFIX_SHELLS) {
        *text = format!("{} {}", HUD_PREFIX_SHELLS, stats.shells);
    } else if text.starts_with("Scores") {
        *text = format!("Scores: {}", stats.score);
    }
}

// ============================================================================
// HUD Update Functions
// ============================================================================

/// 更新单个玩家的 HUD（内部辅助函数）
fn update_single_player_hud(
    stats: &PlayerStats,
    x_pos: f32,
    text_query: &mut Query<(&mut Text2d, Option<&Player1Hud>, Option<&Player2Hud>)>,
    bar_query: &mut Query<(
        &mut Sprite,
        &mut Transform,
        Option<&HealthBarForeground>,
        Option<&BlueBarForeground>,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    is_player1: bool,
) {
    // 更新文本
    for (mut text, is_p1, is_p2) in text_query.iter_mut() {
        if (is_player1 && is_p1.is_some()) || (!is_player1 && is_p2.is_some()) {
            update_single_player_text(stats, &mut text.0);
        }
    }

    // 更新血条和蓝条
    for (mut sprite, mut transform, is_health_foreground, is_blue_foreground, is_p1, is_p2) in
        bar_query.iter_mut()
    {
        if (is_player1 && is_p1.is_some()) || (!is_player1 && is_p2.is_some()) {
            if is_health_foreground.is_some() {
                update_bar(
                    &mut sprite,
                    &mut transform,
                    stats.life_points as f32,
                    HUD_MAX_LIFE_POINTS,
                    x_pos,
                    HUD_BAR_WIDTH,
                );
            } else if is_blue_foreground.is_some() {
                update_bar(
                    &mut sprite,
                    &mut transform,
                    stats.energy_points as f32,
                    HUD_MAX_LIFE_POINTS,
                    x_pos,
                    HUD_BAR_WIDTH,
                );
            }
        }
    }
}

/// 更新玩家 HUD（统一处理玩家1和玩家2）
pub fn update_player_hud(
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    mut text_query: Query<(&mut Text2d, Option<&Player1Hud>, Option<&Player2Hud>)>,
    mut bar_query: Query<(
        &mut Sprite,
        &mut Transform,
        Option<&HealthBarForeground>,
        Option<&BlueBarForeground>,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
) {
    // 更新玩家1 HUD
    update_single_player_hud(
        &player_info.player1,
        WINDOW_LEFT_X + 115.0,
        &mut text_query,
        &mut bar_query,
        true,
    );

    // 更新玩家2 HUD（仅在双人模式下）
    if *game_mode == GameMode::TwoPlayers {
        if let Some(stats2) = &player_info.player2 {
            update_single_player_hud(
                stats2,
                WINDOW_RIGHT_X - 115.0,
                &mut text_query,
                &mut bar_query,
                false,
            );
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
            Option<&Player1Hud>,
            Option<&Player2Hud>,
        ),
    >,
    player_info: Res<PlayerInfo>,
) {
    for (entity, mut timer, mut color, text, is_p1, is_p2) in &mut query {
        timer.tick(time.delta());

        // 判断是否达到最大值或On状态
        let is_max = is_hud_stat_at_max_value(&text.0, &player_info, is_p1, is_p2);

        if is_max {
            // 达到最大值：保持红色，移除闪烁计时器
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = COLOR_RED; // 红色
        } else if timer.is_finished() {
            // 闪烁结束，移除计时器组件
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = COLOR_WHITE;
        } else {
            // 未达到最大值：闪烁效果
            let elapsed = timer.elapsed_secs();
            let cycle = elapsed % TEXT_BLINK_CYCLE;

            if cycle < TEXT_BLINK_CYCLE / 2.0 {
                // 亮状态：绿色
                color.0 = COLOR_GREEN;
            } else {
                // 灭状态：透明
                color.0 = COLOR_TRANSPARENT;
            }
        }
    }
}

/// 获取 HUD 属性类型对应的前缀
fn get_hud_stat_prefix(stat_type: StatType) -> &'static str {
    match stat_type {
        StatType::Speed => HUD_PREFIX_SPEED,
        StatType::Protection => HUD_PREFIX_PROTECTION,
        StatType::FireSpeed => HUD_PREFIX_FIRE_SPEED,
        StatType::FireShell => HUD_PREFIX_FIRE_SHELL,
        StatType::TrackChain => HUD_PREFIX_TRACK_CHAIN,
        StatType::Penetrate => HUD_PREFIX_PENETRATE,
        StatType::AirCushion => HUD_PREFIX_AIR_CUSHION,
        StatType::Shell => HUD_PREFIX_SHELLS,
        StatType::Score => HUD_PREFIX_SCORES,
    }
}

/// 判断单个玩家 HUD 属性是否达到最大值或On状态
fn is_player_stat_at_max(text: &str, stats: &PlayerStats) -> bool {
    if text.starts_with(HUD_PREFIX_SHELLS) {
        stats.shells >= HUD_MAX_SHELLS
    } else if text.starts_with(HUD_PREFIX_SPEED) {
        stats.speed >= HUD_MAX_PERCENT
    } else if text.starts_with(HUD_PREFIX_PROTECTION) {
        stats.protection >= HUD_MAX_PERCENT
    } else if text.starts_with(HUD_PREFIX_FIRE_SPEED) {
        stats.fire_speed >= HUD_MAX_PERCENT
    } else if text.starts_with(HUD_PREFIX_FIRE_SHELL) {
        stats.fire_shell
    } else if text.starts_with(HUD_PREFIX_AIR_CUSHION) {
        stats.air_cushion
    } else if text.starts_with(HUD_PREFIX_TRACK_CHAIN) {
        stats.track_chain
    } else if text.starts_with(HUD_PREFIX_PENETRATE) {
        stats.penetrate
    } else {
        false
    }
}

/// 判断 HUD 属性是否达到最大值或On状态（用于动画系统）
fn is_hud_stat_at_max_value(
    text: &str,
    player_info: &PlayerInfo,
    player_hud: Option<&Player1Hud>,
    player_hud2: Option<&Player2Hud>,
) -> bool {
    // 根据所属玩家选择对应的数据
    if player_hud.is_some() {
        is_player_stat_at_max(text, &player_info.player1)
    } else if player_hud2.is_some() {
        if let Some(stats2) = &player_info.player2 {
            is_player_stat_at_max(text, stats2)
        } else {
            false
        }
    } else {
        false
    }
}

/// 生成顶部 HUD（关卡信息、司令官血条、敌方坦克数量）
fn spawn_top_hud(mut commands: Commands, font_resources: &FontResources, stage_level: &Res<StageLevel>) {
    let font = font_resources.en.clone();
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
        TextColor(COLOR_YELLOW), // 黄色
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
        TextColor(COLOR_WHITE),
        Transform::from_xyz(commander_text_x - 42.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
    // Commander 血条（与玩家血条长度相同：160像素），放在文字正右方
    commands.spawn((
        PlayingEntity,
        CommanderHealthBar,
        CommanderHealthBarOriginalPosition(commander_text_x + 172.0), // 文字右侧
        Sprite {
            color: COLOR_RED,
            custom_size: Some(Vec2::new(COMMANDER_BAR_WIDTH, COMMANDER_BAR_HEIGHT)),
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
        TextColor(COLOR_WHITE),
        Transform::from_xyz(WINDOW_RIGHT_X - 465.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
}

/// 生成所有 HUD（只在第一关时生成）
pub fn spawn_hud(
    mut commands: Commands,
    font_resources: Res<FontResources>,
    commander_resources: Res<CommanderResources>,
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

    spawn_top_hud(commands.reborrow(), &font_resources, &stage_level);
    spawn_player_hud(commands, font_resources, commander_resources, texture_atlas_layouts, player_info, game_mode, player1_hud_query, player2_hud_query);
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
        sprite.custom_size = Some(Vec2::new(health_width, HUD_BAR_HEIGHT));
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
    commander_resources: Res<CommanderResources>,
    mut query: Query<(&mut Sprite, Has<PlayerDead>), With<PlayerAvatar>>,
) {
    let avatar_death_path = AssetPath::from("texture/avatar_death.png");
    for (mut sprite, is_dead) in &mut query {
        if is_dead && sprite.image.path() != Some(&avatar_death_path) {
            // 切换到死亡头像纹理
            sprite.image = commander_resources.avatar_death.clone();
            sprite.texture_atlas = None; // 死亡头像不需要动画
            sprite.custom_size = Some(Vec2::new(160.0, 147.0));
        }
    }
}

/// 处理司令官阵亡时更换纹理和头像
pub fn handle_commander_death(
    commander_resources: Res<CommanderResources>,
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
        sprite.image = commander_resources.dead_texture.clone();
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
        sprite.image = commander_resources.avatar_commander_dead.clone();
        // 移除纹理图集，因为死亡头像纹理是单张图片
        sprite.texture_atlas = None;
    }
}


