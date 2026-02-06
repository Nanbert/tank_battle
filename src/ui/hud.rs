//! HUD (Head-Up Display) 模块
//!
//! 处理游戏内的 HUD 显示，包括玩家状态、血条、蓝条等

use bevy::prelude::*;

#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;
use super::common;

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

// ============================================================================
// HUD Stat Type Components
// ============================================================================

/// HUD 属性类型枚举
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum HudStatType {
    Speed,
    FireSpeed,
    Protection,
    Shells,
    FireShell,
    Penetrate,
    TrackChain,
    AirCushion,
    Score,
}

/// HUD 属性值的统一表示
enum StatValue {
    /// 百分比值（速度、射速、护盾）
    Percent(usize),
    /// 计数值（炮弹数、分数）
    Count(usize),
    /// 布尔值（道具状态）
    Bool(bool),
}

impl HudStatType {
    /// 获取属性的标签文本
    fn get_label(self, language: Language) -> &'static str {
        match self {
            HudStatType::Speed => HUD_STAT_LABELS.speed.get(language),
            HudStatType::FireSpeed => HUD_STAT_LABELS.fire_speed.get(language),
            HudStatType::Protection => HUD_STAT_LABELS.protection.get(language),
            HudStatType::Shells => HUD_STAT_LABELS.shells.get(language),
            HudStatType::FireShell => HUD_STAT_LABELS.fire_shell.get(language),
            HudStatType::Penetrate => HUD_STAT_LABELS.penetrate.get(language),
            HudStatType::TrackChain => HUD_STAT_LABELS.track_chain.get(language),
            HudStatType::AirCushion => HUD_STAT_LABELS.air_cushion.get(language),
            HudStatType::Score => HUD_STAT_LABELS.score.get(language),
        }
    }

    /// 获取属性的值（从 PlayerStats 中）
    fn get_value(self, stats: &PlayerStats) -> StatValue {
        match self {
            HudStatType::Speed => StatValue::Percent(stats.speed),
            HudStatType::FireSpeed => StatValue::Percent(stats.fire_speed),
            HudStatType::Protection => StatValue::Percent(stats.protection),
            HudStatType::Shells => StatValue::Count(stats.shells),
            HudStatType::FireShell => StatValue::Bool(stats.fire_shell),
            HudStatType::Penetrate => StatValue::Bool(stats.penetrate),
            HudStatType::TrackChain => StatValue::Bool(stats.track_chain),
            HudStatType::AirCushion => StatValue::Bool(stats.air_cushion),
            HudStatType::Score => StatValue::Count(stats.score),
        }
    }

    /// 检查是否达到最大值
    fn is_max(self, stats: &PlayerStats) -> bool {
        match self {
            HudStatType::Speed => stats.speed >= HUD_MAX_PERCENT,
            HudStatType::FireSpeed => stats.fire_speed >= HUD_MAX_PERCENT,
            HudStatType::Protection => stats.protection >= HUD_MAX_PERCENT,
            _ => false,
        }
    }
}

impl StatValue {
    /// 格式化属性值为字符串
    fn format(self, is_max: bool, label_on: &'static str, label_off: &'static str) -> String {
        match self {
            StatValue::Percent(v) => {
                if is_max {
                    String::from("MAX")
                } else {
                    format!("{}%", v)
                }
            }
            StatValue::Count(v) => format!("{}", v),
            StatValue::Bool(v) => if v { label_on } else { label_off }.to_string(),
        }
    }
}

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
// HUD Language Configuration
// ============================================================================

#[derive(Clone, Copy)]
struct HudTextLabels {
    player1_name: LocalizedText,
    player2_name: LocalizedText,
    effects_title: LocalizedText,
    commander_life: LocalizedText,
    on_off: LocalizedText,
    off_label: LocalizedText,
}

#[derive(Clone, Copy)]
struct HudStatLabels {
    speed: LocalizedText,
    fire_speed: LocalizedText,
    protection: LocalizedText,
    shells: LocalizedText,
    fire_shell: LocalizedText,
    penetrate: LocalizedText,
    track_chain: LocalizedText,
    air_cushion: LocalizedText,
    score: LocalizedText,
}

// ==================== 本地化文本常量 ====================

// HUD 文本标签
const HUD_TEXT_LABELS: HudTextLabels = HudTextLabels {
    player1_name: LocalizedText {
        cn: "李云龙",
        en: "Li Yun Long",
    },
    player2_name: LocalizedText {
        cn: "楚云飞",
        en: "Chu Yun Fei",
    },
    effects_title: LocalizedText {
        cn: "效果",
        en: "Effects",
    },
    commander_life: LocalizedText {
        cn: "司令官生命:",
        en: "Commander Life:",
    },
    on_off: LocalizedText {
        cn: "开启",
        en: "On",
    },
    off_label: LocalizedText {
        cn: "关闭",
        en: "Off",
    },
};

// HUD 属性标签
const HUD_STAT_LABELS: HudStatLabels = HudStatLabels {
    speed: LocalizedText {
        cn: "速度:",
        en: "Speed:",
    },
    fire_speed: LocalizedText {
        cn: "射速:",
        en: "Fire Speed:",
    },
    protection: LocalizedText {
        cn: "护盾:",
        en: "Protection:",
    },
    shells: LocalizedText {
        cn: "炮弹:",
        en: "Shells:",
    },
    fire_shell: LocalizedText {
        cn: "火焰炮弹",
        en: "Fire Shell",
    },
    penetrate: LocalizedText {
        cn: "穿透",
        en: "Penetrate",
    },
    track_chain: LocalizedText {
        cn: "履带链",
        en: "Track Chain",
    },
    air_cushion: LocalizedText {
        cn: "气垫",
        en: "Air Cushion",
    },
    score: LocalizedText {
        cn: "分数:",
        en: "Scores:",
    },
};

/// 辅助函数：获取玩家名称
fn get_player_name(player_type: TankType, language: Language) -> &'static str {
    match player_type {
        TankType::Player1 => HUD_TEXT_LABELS.player1_name.get(language),
        TankType::Player2 => HUD_TEXT_LABELS.player2_name.get(language),
        TankType::Enemy => "Enemy",
    }
}

// ============================================================================
// HUD Spawn Functions
// ============================================================================

/// 生成玩家 HUD
fn spawn_player_hud(
    mut commands: Commands,
    font_resources: Res<GameTextureResources>,
    commander_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    language: Res<Language>,
) {
    let font = common::get_font(&font_resources, *language);

    // 玩家1 HUD
    spawn_single_player_hud(
        &mut commands,
        &font,
        &commander_resources,
        &atlas_layouts,
        &player_info,
        TankType::Player1,
        WINDOW_LEFT_X + 115.0,
        Player1Hud,
        *language,
    );

    // 玩家2 HUD（仅在双人模式下）
    if *game_mode == GameMode::TwoPlayers {
        spawn_single_player_hud(
            &mut commands,
            &font,
            &commander_resources,
            &atlas_layouts,
            &player_info,
            TankType::Player2,
            WINDOW_RIGHT_X - 115.0,
            Player2Hud,
            *language,
        );
    }
}

/// 生成单个玩家的 HUD
fn spawn_single_player_hud(
    commands: &mut Commands,
    font: &Handle<Font>,
    commander_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    player_info: &PlayerInfo,
    player_type: TankType,
    x_pos: f32,
    marker: impl Component + Clone,
    language: Language,
) {
    let stats = get_player_stats_for_spawn(player_info, player_type);

    // 玩家名称
    commands.spawn((
        marker.clone(),
        PlayerNameText,
        Text2d(get_player_name(player_type, language).to_string()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::Name as usize], Z_UI),
    ));

    // 属性文本（百分比类型）
    spawn_percent_text(
        commands,
        marker.clone(),
        SpeedText,
        font,
        HUD_STAT_LABELS.speed.get(language),
        stats.speed,
        HUD_Y_POSITIONS[HudYPosition::Speed as usize],
        x_pos,
        HudStatType::Speed,
    );
    spawn_percent_text(
        commands,
        marker.clone(),
        FireSpeedText,
        font,
        HUD_STAT_LABELS.fire_speed.get(language),
        stats.fire_speed,
        HUD_Y_POSITIONS[HudYPosition::FireSpeed as usize],
        x_pos,
        HudStatType::FireSpeed,
    );
    spawn_percent_text(
        commands,
        marker.clone(),
        ProtectionText,
        font,
        HUD_STAT_LABELS.protection.get(language),
        stats.protection,
        HUD_Y_POSITIONS[HudYPosition::Protection as usize],
        x_pos,
        HudStatType::Protection,
    );

    // 炮弹数量
    let prefix_shells = HUD_STAT_LABELS.shells.get(language);
    commands.spawn((
        marker.clone(),
        ShellsText,
        HudStatType::Shells,
        Text2d(format!("{} {}", prefix_shells, stats.shells)),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::Shells as usize], Z_UI),
    ));

    // 效果标题
    commands.spawn((
        marker.clone(),
        EffectsTitle,
        Text2d(HUD_TEXT_LABELS.effects_title.get(language).to_string()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::EffectsTitle as usize], Z_UI),
    ));

    // 效果文本（布尔类型）
    spawn_effect_text(
        commands,
        marker.clone(),
        FireShellText,
        font,
        HUD_STAT_LABELS.fire_shell.get(language),
        stats.fire_shell,
        HUD_Y_POSITIONS[HudYPosition::FireShell as usize],
        x_pos,
        language,
        HudStatType::FireShell,
    );
    spawn_effect_text(
        commands,
        marker.clone(),
        PenetrateText,
        font,
        HUD_STAT_LABELS.penetrate.get(language),
        stats.penetrate,
        HUD_Y_POSITIONS[HudYPosition::Penetrate as usize],
        x_pos,
        language,
        HudStatType::Penetrate,
    );
    spawn_effect_text(
        commands,
        marker.clone(),
        TrackChainText,
        font,
        HUD_STAT_LABELS.track_chain.get(language),
        stats.track_chain,
        HUD_Y_POSITIONS[HudYPosition::TrackChain as usize],
        x_pos,
        language,
        HudStatType::TrackChain,
    );
    spawn_effect_text(
        commands,
        marker.clone(),
        AirCushionText,
        font,
        HUD_STAT_LABELS.air_cushion.get(language),
        stats.air_cushion,
        HUD_Y_POSITIONS[HudYPosition::AirCushion as usize],
        x_pos,
        language,
        HudStatType::AirCushion,
    );

    // 分数
    let prefix_scores = HUD_STAT_LABELS.score.get(language);
    commands.spawn((
        marker.clone(),
        ScoreText,
        HudStatType::Score,
        Text2d(format!("{} {}", prefix_scores, stats.score)),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::Score as usize], Z_UI),
    ));

    // 玩家头像（使用精灵图）
    crate::utils::spawn_animated_sprite(
        commands,
        commander_resources.avatar.clone(),
        atlas_layouts.player_avatar.clone(),
        crate::atlas::PLAYER_AVATAR_ATLAS.animation_indices_full(),
        0.2,
        Transform::from_translation(Vec3::new(x_pos, WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::Avatar as usize], Z_UI)),
        crate::atlas::PLAYER_AVATAR_ATLAS.display_size,
        (marker.clone(), PlayerAvatar, PlayerUI { player_type }, AnimationMode::Looping),
    );

    // 血条背景
    spawn_bar(
        commands,
        marker.clone(),
        COLOR_DARK_GRAY,
        x_pos,
        WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::BarHealth as usize],
        HUD_BAR_SIZE.x,
        Z_UI,
    );

    // 血条前景（红色）
    let health_width = HUD_BAR_SIZE.x * (stats.life_points as f32 / HUD_MAX_LIFE_POINTS);
    commands.spawn((
        marker.clone(),
        HealthBar,
        HealthBarForeground,
        Sprite {
            color: COLOR_RED,
            custom_size: Some(Vec2::new(health_width, HUD_BAR_SIZE.y)),
            ..default()
        },
        Transform::from_xyz(
            x_pos - HUD_BAR_SIZE.x / 2.0 + health_width / 2.0,
            WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::BarHealth as usize],
            Z_UI + 0.1,
        ),
    ));

    // 蓝条背景
    spawn_bar(
        commands,
        marker.clone(),
        COLOR_DARK_GRAY,
        x_pos,
        WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::BarBlue as usize],
        HUD_BAR_SIZE.x,
        Z_UI,
    );

    // 蓝条前景（蓝色）
    let blue_width = HUD_BAR_SIZE.x * (stats.energy_points as f32 / HUD_MAX_LIFE_POINTS);
    commands.spawn((
        marker.clone(),
        BlueBar,
        BlueBarForeground,
        Sprite {
            color: COLOR_BLUE,
            custom_size: Some(Vec2::new(blue_width, HUD_BAR_SIZE.y)),
            ..default()
        },
        Transform::from_xyz(
            x_pos - HUD_BAR_SIZE.x / 2.0 + blue_width / 2.0,
            WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::BarBlue as usize],
            Z_UI + 0.1,
        ),
    ));
}

/// 销毁 HUD
pub fn despawn_hud(
    mut commands: Commands,
    top_hud_query: Query<
        Entity,
        Or<(
            With<StageText>,
            With<CommanderText>,
            With<CommanderHealthBar>,
            With<EnemyCountText>,
        )>,
    >,
    player_hud_query: Query<Entity, Or<(With<Player1Hud>, With<Player2Hud>)>>,
) {
    // 销毁顶部 HUD
    crate::utils::cleanup_entities(&mut commands, top_hud_query.iter());

    // 销毁玩家 HUD
    crate::utils::cleanup_entities(&mut commands, player_hud_query.iter());
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
            custom_size: Some(Vec2::new(width, HUD_BAR_SIZE.y)),
            ..default()
        },
        Transform::from_xyz(x, y, z),
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
    language: Language,
    stat_type: HudStatType,
) {
    let on_off_label = if value {
        HUD_TEXT_LABELS.on_off.get(language)
    } else {
        HUD_TEXT_LABELS.off_label.get(language)
    };
    commands.spawn((
        marker1,
        marker2,
        stat_type,
        Text2d(format!("{}: {}", prefix, on_off_label)),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - y_offset, Z_UI),
    ));
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
    stat_type: HudStatType,
) {
    let text = if value >= HUD_MAX_PERCENT {
        format!("{}MAX", prefix)
    } else {
        format!("{}{}%", prefix, value)
    };
    commands.spawn((
        marker1,
        marker2,
        stat_type,
        Text2d(text),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, WINDOW_TOP_Y - y_offset, Z_UI),
    ));
}

/// 获取玩家统计数据（用于 spawn_single_player_hud）
fn get_player_stats_for_spawn(player_info: &PlayerInfo, player_type: TankType) -> PlayerStats {
    match player_type {
        TankType::Player1 => player_info.player1.clone(),
        TankType::Player2 => player_info.player2.as_ref().cloned().unwrap_or_default(),
        TankType::Enemy => PlayerStats::default(),
    }
}

// ============================================================================
// HUD Helper Functions
// ============================================================================

/// 更新血条或蓝条
/// 优化：使用 HudStatType 方法统一处理，消除重复代码
fn update_single_player_text(
    stats: &PlayerStats,
    stat_type: HudStatType,
    language: Language,
) -> String {
    let prefix = stat_type.get_label(language);
    let value = stat_type.get_value(stats);
    let is_max = stat_type.is_max(stats);
    let label_on = HUD_TEXT_LABELS.on_off.get(language);
    let label_off = HUD_TEXT_LABELS.off_label.get(language);

    let formatted_value = value.format(is_max, label_on, label_off);
    format!("{}{}", prefix, formatted_value)
}

// ============================================================================
// HUD Update Functions
// ============================================================================

/// 更新单个玩家的 HUD（内部辅助函数）
/// 优化：使用组件标记直接查询和更新，避免重复遍历
fn update_single_player_hud(
    stats: &PlayerStats,
    x_pos: f32,
    player_hud_query: &mut Query<(
        &mut Text2d,
        &HudStatType,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    bar_query: &mut Query<(
        &mut Sprite,
        &mut Transform,
        Option<&HealthBarForeground>,
        Option<&BlueBarForeground>,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    is_player1: bool,
    language: Language,
) {
    // 更新文本
    for (mut text, stat_type, is_p1, is_p2) in player_hud_query.iter_mut() {
        if (is_player1 && is_p1.is_some()) || (!is_player1 && is_p2.is_some()) {
            text.0 = update_single_player_text(stats, *stat_type, language);
        }
    }

    // 更新血条和蓝条
    for (mut sprite, mut transform, is_health_foreground, is_blue_foreground, is_p1, is_p2) in
        bar_query.iter_mut()
    {
        if (is_player1 && is_p1.is_some()) || (!is_player1 && is_p2.is_some()) {
            if is_health_foreground.is_some() {
                common::update_bar(
                    &mut sprite,
                    &mut transform,
                    stats.life_points as f32,
                    HUD_MAX_LIFE_POINTS,
                    x_pos,
                    HUD_BAR_SIZE.x,
                    HUD_BAR_SIZE.y,
                );
            } else if is_blue_foreground.is_some() {
                common::update_bar(
                    &mut sprite,
                    &mut transform,
                    stats.energy_points as f32,
                    HUD_MAX_LIFE_POINTS,
                    x_pos,
                    HUD_BAR_SIZE.x,
                    HUD_BAR_SIZE.y,
                );
            }
        }
    }
}

/// 更新玩家 HUD（统一处理玩家1和玩家2）
/// 优化：通过系统条件 resource_changed::<PlayerInfo> 确保只在玩家信息变化时更新 HUD
/// 优化：使用组件标记直接查询和更新，避免重复遍历
pub fn update_player_hud(
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    mut player_hud_query: Query<(
        &mut Text2d,
        &HudStatType,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    mut bar_query: Query<(
        &mut Sprite,
        &mut Transform,
        Option<&HealthBarForeground>,
        Option<&BlueBarForeground>,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    language: Res<Language>,
) {
    // 更新玩家1 HUD
    update_single_player_hud(
        &player_info.player1,
        WINDOW_LEFT_X + 115.0,
        &mut player_hud_query,
        &mut bar_query,
        true,
        *language,
    );

    // 更新玩家2 HUD（仅在双人模式下）
    if *game_mode == GameMode::TwoPlayers
        && let Some(stats2) = &player_info.player2
    {
        update_single_player_hud(
            stats2,
            WINDOW_RIGHT_X - 115.0,
            &mut player_hud_query,
            &mut bar_query,
            false,
            *language,
        );
    }
}

// ============================================================================
// HUD Text Blink System
// ============================================================================

/// 处理 HUD 属性变更事件，触发文字闪烁
/// 优化：使用组件标记匹配而不是字符串前缀匹配
pub fn handle_hud_stat_changed(
    mut events: MessageReader<PlayerStatChanged>,
    mut commands: Commands,
    player1_hud_texts: Query<(Entity, &HudStatType), With<Player1Hud>>,
    player2_hud_texts: Query<(Entity, &HudStatType), With<Player2Hud>>,
) {
    for event in events.read() {
        let target_stat_type = match event.stat_type {
            StatType::Speed => HudStatType::Speed,
            StatType::FireSpeed => HudStatType::FireSpeed,
            StatType::Protection => HudStatType::Protection,
            StatType::Shell => HudStatType::Shells,
            StatType::FireShell => HudStatType::FireShell,
            StatType::Penetrate => HudStatType::Penetrate,
            StatType::TrackChain => HudStatType::TrackChain,
            StatType::AirCushion => HudStatType::AirCushion,
            StatType::Score => HudStatType::Score,
        };

        match event.player_type {
            TankType::Player1 => {
                for (entity, stat_type) in player1_hud_texts.iter() {
                    if *stat_type == target_stat_type {
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
                for (entity, stat_type) in player2_hud_texts.iter() {
                    if *stat_type == target_stat_type {
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
/// 优化：使用组件标记判断是否达到最大值，避免字符串匹配
pub fn animate_hud_text(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut PlayerInfoBlinkTimer,
        &mut TextColor,
        &HudStatType,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    player_info: Res<PlayerInfo>,
) {
    for (entity, mut timer, mut color, stat_type, is_p1, is_p2) in &mut query {
        timer.tick(time.delta());

        // 判断是否达到最大值或On状态
        let is_max = is_hud_stat_at_max_value(*stat_type, &player_info, is_p1, is_p2);

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

/// 判断 HUD 属性是否达到最大值或On状态（用于动画系统）
/// 优化：使用组件标记判断，避免字符串匹配
fn is_hud_stat_at_max_value(
    stat_type: HudStatType,
    player_info: &PlayerInfo,
    player_hud: Option<&Player1Hud>,
    player_hud2: Option<&Player2Hud>,
) -> bool {
    // 根据所属玩家选择对应的数据
    let stats = if player_hud.is_some() {
        &player_info.player1
    } else if player_hud2.is_some() {
        player_info.player2.as_ref().unwrap()
    } else {
        return false;
    };

    match stat_type {
        HudStatType::Speed => stats.speed >= HUD_MAX_PERCENT,
        HudStatType::FireSpeed => stats.fire_speed >= HUD_MAX_PERCENT,
        HudStatType::Protection => stats.protection >= HUD_MAX_PERCENT,
        HudStatType::Shells => stats.shells >= HUD_MAX_SHELLS,
        HudStatType::FireShell => stats.fire_shell,
        HudStatType::Penetrate => stats.penetrate,
        HudStatType::TrackChain => stats.track_chain,
        HudStatType::AirCushion => stats.air_cushion,
        HudStatType::Score => false, // 分数没有最大值
    }
}

/// 生成顶部 HUD（关卡信息、司令官血条、敌方坦克数量）
fn spawn_top_hud(
    mut commands: Commands,
    font_resources: &GameTextureResources,
    stage_level: &Res<StageLevel>,
    language: Language,
) {
    let font = common::get_font(&font_resources, language);
    // 其他游戏信息 UI 元素配置
    let commander_text_x = WINDOW_LEFT_X + 435.0; // 往左平移30像素

    // 关卡信息显示在顶部中心
    let stage_text = match language {
        Language::Chinese => format!("第 {} 关", stage_level.0),
        Language::English => format!("Stage {}", stage_level.0),
    };
    commands.spawn((
        PlayingEntity,
        StageText,
        Text2d(stage_text),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_YELLOW), // 黄色
        Transform::from_xyz(0.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));

    // Commander 文本和血条
    let commander_text = HUD_TEXT_LABELS.commander_life.get(language);
    // Commander 文字（在血条左侧）
    commands.spawn((
        PlayingEntity,
        CommanderText,
        Text2d(commander_text.to_string()),
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
            custom_size: Some(COMMANDER_BAR_SIZE),
            ..default()
        },
        Transform::from_xyz(commander_text_x + 172.0, WINDOW_TOP_Y - 50.0, 1.0), // 与文字同一Y坐标
    ));

    // 敌方剩余数量显示在右侧
    let enemy_count_text = match language {
        Language::Chinese => "敌方剩余: 20/20".to_string(),
        Language::English => "Enemy Left: 20/20".to_string(),
    };
    commands.spawn((
        PlayingEntity,
        EnemyCountText,
        Text2d(enemy_count_text),
        TextFont {
            font_size: 28.0,
            font,
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(WINDOW_RIGHT_X - 465.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
}

/// 生成所有 HUD（只在第一关时生成）
pub fn spawn_hud(
    mut commands: Commands,
    font_resources: Res<GameTextureResources>,
    commander_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    top_hud_query: Query<
        Entity,
        Or<(
            With<StageText>,
            With<CommanderText>,
            With<CommanderHealthBar>,
            With<EnemyCountText>,
        )>,
    >,
    player_hud_query: Query<Entity, Or<(With<Player1Hud>, With<Player2Hud>)>>,
    stage_level: Res<StageLevel>,
    language: Res<Language>,
) {
    // 先清理现有的 HUD，以防万一
    crate::utils::cleanup_entities(&mut commands, top_hud_query.iter());
    crate::utils::cleanup_entities(&mut commands, player_hud_query.iter());

    spawn_top_hud(
        commands.reborrow(),
        &font_resources,
        &stage_level,
        *language,
    );
    spawn_player_hud(
        commands,
        font_resources,
        commander_resources,
        atlas_layouts,
        player_info,
        game_mode,
        language,
    );
}

/// 更新关卡信息文本
pub fn update_stage_text(
    stage_level: Res<StageLevel>,
    mut stage_text_query: Query<&mut Text2d, With<StageText>>,
    language: Res<Language>,
) {
    for mut text in &mut stage_text_query {
        text.0 = match *language {
            Language::Chinese => format!("第 {} 关", stage_level.0),
            Language::English => format!("Stage {}", stage_level.0),
        };
    }
}

/// 更新 Commander 血条
/// 优化：使用 update_bar 函数避免代码重复
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
        common::update_bar(
            &mut sprite,
            &mut transform,
            commander_life.life_points as f32,
            3.0,
            original_pos.0,
            COMMANDER_BAR_SIZE.x,
            COMMANDER_BAR_SIZE.y,
        );
    }
}

/// 更新敌方剩余数量文本
pub fn update_enemy_count_text(
    enemy_spawn_state: Res<EnemySpawnState>,
    mut query: Query<&mut Text2d, With<EnemyCountText>>,
    language: Res<Language>,
) {
    let remaining = enemy_spawn_state.max_count - enemy_spawn_state.has_spawned;
    let max_count = enemy_spawn_state.max_count;

    let text = match *language {
        Language::Chinese => format!("敌方剩余: {}/{}", remaining, max_count),
        Language::English => format!("Enemy Left: {}/{}", remaining, max_count),
    };

    for mut text_mut in &mut query {
        text_mut.0 = text.clone();
    }
}



/// 处理玩家头像死亡状态
pub fn handle_player_avatar_death(
    texture_resources: Res<GameTextureResources>,
    player_info: Res<PlayerInfo>,
    mut player_avatars: Query<(&mut Sprite, &PlayerUI), With<PlayerAvatar>>,
    mut has_handled_player1: Local<bool>,
    mut has_handled_player2: Local<bool>,
) {
    // 处理玩家1头像
    let player1_dead = player_info.player1.life_points == 0;
    if player1_dead && !*has_handled_player1 {
        for (mut sprite, player_ui) in &mut player_avatars {
            if player_ui.player_type == TankType::Player1 {
                sprite.image = texture_resources.avatar_death.clone();
                sprite.texture_atlas = None; // 死亡头像不需要动画
                sprite.custom_size = Some(crate::atlas::PLAYER_AVATAR_ATLAS.display_size);
                break;
            }
        }
        *has_handled_player1 = true;
    } else if !player1_dead && *has_handled_player1 {
        *has_handled_player1 = false; // 重置状态，以便下次死亡时再次处理
    }

    // 处理玩家2头像（如果存在）
    if let Some(ref player2_stats) = player_info.player2 {
        let player2_dead = player2_stats.life_points == 0;
        if player2_dead && !*has_handled_player2 {
            for (mut sprite, player_ui) in &mut player_avatars {
                if player_ui.player_type == TankType::Player2 {
                    sprite.image = texture_resources.avatar_death.clone();
                    sprite.texture_atlas = None; // 死亡头像不需要动画
                    sprite.custom_size = Some(crate::atlas::PLAYER_AVATAR_ATLAS.display_size);
                    break;
                }
            }
            *has_handled_player2 = true;
        } else if !player2_dead && *has_handled_player2 {
            *has_handled_player2 = false; // 重置状态，以便下次死亡时再次处理
        }
    }
}

/// 处理司令官阵亡时更换纹理和头像
pub fn handle_commander_death(
    texture_resources: Res<GameTextureResources>,
    commander_life: Res<CommanderLife>,
    player_info: Res<PlayerInfo>,
    mut queries: ParamSet<(
        Query<&mut Sprite, With<Commander>>,
        Query<(&mut Sprite, &PlayerUI), With<PlayerAvatar>>,
        Query<&mut AnimationTimer, With<Commander>>,
        Query<&mut AnimationTimer, With<MusicNoteAnimation>>,
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
        sprite.image = texture_resources.commander_dead.clone();
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

    // 只将还活着的玩家头像设置为悲伤图片，已经死亡的玩家头像保持死亡图片
    for (mut sprite, player_ui) in &mut queries.p1() {
        let player_dead = match player_ui.player_type {
            TankType::Player1 => player_info.player1.life_points == 0,
            TankType::Player2 => player_info
                .player2
                .as_ref()
                .map(|p| p.life_points == 0)
                .unwrap_or(false),
            _ => false,
        };

        // 如果玩家已经死亡，保持死亡图片；否则设置为悲伤图片
        if player_dead {
            sprite.image = texture_resources.avatar_death.clone();
        } else {
            sprite.image = texture_resources.avatar_commander_dead.clone();
        }
        // 移除纹理图集，因为这些头像纹理都是单张图片
        sprite.texture_atlas = None;
    }
}
