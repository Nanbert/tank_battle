//! HUD 生成函数
//!
//! 处理所有 HUD 元素的生成

use bevy::prelude::*;

use super::stats::*;
#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;
use crate::ui::common;
use crate::ui::constants::*;
use crate::ui::localization::*;

// ============================================================================
// HUD 查询类型别名
// ============================================================================

/// 顶部 HUD 查询（关卡文本、司令官文本、血条、敌方数量）
pub type TopHudQuery<'w, 's> = Query<
    'w,
    's,
    Entity,
    Or<(
        With<StageText>,
        With<CommanderText>,
        With<CommanderHealthBar>,
        With<EnemyCountText>,
    )>,
>;

/// 玩家 HUD 查询（玩家1和玩家2的HUD）
pub type PlayerHudQuery<'w, 's> = Query<'w, 's, Entity, Or<(With<Player1Hud>, With<Player2Hud>)>>;

// ============================================================================
// Language Helper Functions
// ============================================================================

/// 辅助函数：获取玩家名称
fn get_player_name(player_type: TankType, language: Language) -> &'static str {
    match player_type {
        TankType::Player1 => HUD_PLAYER1_NAME.get(language),
        TankType::Player2 => HUD_PLAYER2_NAME.get(language),
        TankType::Enemy => "Enemy",
    }
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
// Helper Functions
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
    common::spawn_bar(
        commands,
        None, // 无纹理，使用纯色
        color,
        Vec3::new(x, y, z),
        Vec2::new(width, HUD_BAR_SIZE.y),
        z,
        marker,
    );
}

// 根据配置生成单个 HUD 文本元素（数据驱动版本）
fn spawn_hud_text_element<M1: Component, M2: Component>(
    commands: &mut Commands,
    marker1: M1,
    marker2: M2,
    font: &Handle<Font>,
    config: &HudElementConfig,
    stats: &PlayerStats,
    x_pos: f32,
    language: Language,
) {
    let prefix = config.stat_type.get_label(language);
    let value = config.stat_type.get_value(stats);
    let is_max = config.stat_type.is_max(stats);
    let label_on = HUD_ON.get(language);
    let label_off = HUD_OFF.get(language);

    let formatted_value = value.format(is_max, label_on, label_off);
    let text = format!("{}{}", prefix, formatted_value);

    commands.spawn((
        marker1,
        marker2,
        config.stat_type,
        Text2d(text),
        common::create_text_font(font, FONT_SIZE_INSTRUCTION),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, common::hud_y_position(config.y_position), Z_UI),
    ));
}

// ============================================================================
// Player HUD Spawn Functions
// ============================================================================

/// 生成单个玩家的 HUD
///
/// 此函数为指定玩家创建完整的 HUD 界面，包括：
/// - 玩家名称
/// - 属性文本（速度、射速、护盾）
/// - 炮弹数量
/// - 效果标题和状态（火焰炮弹、穿透、履带链、气垫）
/// - 分数
/// - 玩家头像（动画精灵）
/// - 血条（背景和前景）
/// - 蓝条（背景和前景）
///
/// # 参数
/// - `commands`: Bevy 命令队列
/// - `font`: 字体句柄
/// - `commander_resources`: 游戏纹理资源（用于头像）
/// - `atlas_layouts`: 纹理图集布局资源
/// - `player_info`: 玩家信息资源
/// - `player_type`: 玩家类型（玩家1或玩家2）
/// - `x_pos`: HUD 的 X 坐标
/// - `marker`: 标记组件（用于标识 HUD 所属玩家）
/// - `language`: 当前语言设置
pub fn spawn_single_player_hud(
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
        common::create_text_font(font, FONT_SIZE_HUD_NAME),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(x_pos, common::hud_y_position(HudYPosition::Name), Z_UI),
    ));

    // 属性文本（百分比类型 - 数据驱动，使用循环）
    for config in &PERCENT_STAT_CONFIGS {
        match config.stat_type {
            HudStatType::Speed => {
                spawn_hud_text_element(
                    commands,
                    marker.clone(),
                    SpeedText,
                    font,
                    config,
                    &stats,
                    x_pos,
                    language,
                );
            }
            HudStatType::FireSpeed => {
                spawn_hud_text_element(
                    commands,
                    marker.clone(),
                    FireSpeedText,
                    font,
                    config,
                    &stats,
                    x_pos,
                    language,
                );
            }
            HudStatType::Protection => {
                spawn_hud_text_element(
                    commands,
                    marker.clone(),
                    ProtectionText,
                    font,
                    config,
                    &stats,
                    x_pos,
                    language,
                );
            }
            _ => {}
        }
    }

    // 炮弹数量
    let shells_config = HudElementConfig {
        y_position: HudYPosition::Shells,
        stat_type: HudStatType::Shells,
    };
    spawn_hud_text_element(
        commands,
        marker.clone(),
        ShellsText,
        font,
        &shells_config,
        &stats,
        x_pos,
        language,
    );

    // 效果标题
    commands.spawn((
        marker.clone(),
        EffectsTitle,
        Text2d(HUD_EFFECTS_TITLE.get(language).to_string()),
        common::create_text_font(font, FONT_SIZE_HUD_NAME),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(
            x_pos,
            common::hud_y_position(HudYPosition::EffectsTitle),
            Z_UI,
        ),
    ));

    // 效果文本（布尔类型 - 数据驱动，使用循环）
    for config in &EFFECT_STAT_CONFIGS {
        match config.stat_type {
            HudStatType::FireShell => {
                spawn_hud_text_element(
                    commands,
                    marker.clone(),
                    FireShellText,
                    font,
                    config,
                    &stats,
                    x_pos,
                    language,
                );
            }
            HudStatType::Penetrate => {
                spawn_hud_text_element(
                    commands,
                    marker.clone(),
                    PenetrateText,
                    font,
                    config,
                    &stats,
                    x_pos,
                    language,
                );
            }
            HudStatType::TrackChain => {
                spawn_hud_text_element(
                    commands,
                    marker.clone(),
                    TrackChainText,
                    font,
                    config,
                    &stats,
                    x_pos,
                    language,
                );
            }
            HudStatType::AirCushion => {
                spawn_hud_text_element(
                    commands,
                    marker.clone(),
                    AirCushionText,
                    font,
                    config,
                    &stats,
                    x_pos,
                    language,
                );
            }
            _ => {}
        }
    }

    // 分数
    let score_config = HudElementConfig {
        y_position: HudYPosition::Score,
        stat_type: HudStatType::Score,
    };
    spawn_hud_text_element(
        commands,
        marker.clone(),
        ScoreText,
        font,
        &score_config,
        &stats,
        x_pos,
        language,
    );

    // 玩家头像
    crate::utils::spawn_animated_sprite(
        commands,
        commander_resources.avatar.clone(),
        atlas_layouts.player_avatar.clone(),
        crate::atlas::PLAYER_AVATAR_ATLAS.animation_indices_full(),
        0.2,
        Transform::from_translation(common::hud_position(x_pos, HudYPosition::Avatar, Z_UI)),
        crate::atlas::PLAYER_AVATAR_ATLAS.display_size,
        (
            marker.clone(),
            PlayerAvatar,
            PlayerUI { player_type },
            AnimationMode::Looping,
        ),
    );

    // 血条
    spawn_bar(
        commands,
        marker.clone(),
        COLOR_DARK_GRAY,
        x_pos,
        common::hud_y_position(HudYPosition::BarHealth),
        HUD_BAR_SIZE.x,
        Z_UI,
    );
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
            common::hud_y_position(HudYPosition::BarHealth),
            Z_UI + HUD_FOREGROUND_Z_OFFSET,
        ),
    ));

    // 蓝条
    spawn_bar(
        commands,
        marker.clone(),
        COLOR_DARK_GRAY,
        x_pos,
        common::hud_y_position(HudYPosition::BarBlue),
        HUD_BAR_SIZE.x,
        Z_UI,
    );
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
            common::hud_y_position(HudYPosition::BarBlue),
            Z_UI + HUD_FOREGROUND_Z_OFFSET,
        ),
    ));
}

/// 生成玩家 HUD
pub fn spawn_player_hud(
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
        WINDOW_LEFT_X + HUD_PLAYER_OFFSET,
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
            WINDOW_RIGHT_X - HUD_PLAYER_OFFSET,
            Player2Hud,
            *language,
        );
    }
}

/// 生成顶部 HUD（关卡信息、司令官血条、敌方坦克数量）
pub fn spawn_top_hud(
    mut commands: Commands,
    font_resources: &GameTextureResources,
    stage_level: &Res<StageLevel>,
    language: Language,
) {
    let font = common::get_font(&font_resources, language);
    let commander_text_x = WINDOW_LEFT_X + HUD_COMMANDER_TEXT_X;

    // 关卡信息显示在顶部中心（使用格式化本地化文本）
    let stage_text = STAGE_TEXT.format(language, stage_level.0);
    commands.spawn((
        PlayingEntity,
        StageText,
        Text2d(stage_text),
        common::create_text_font(&font, FONT_SIZE_SCORE),
        TextColor(COLOR_YELLOW),
        Transform::from_xyz(0.0, WINDOW_TOP_Y - TOP_HUD_Y_OFFSET, Z_UI_TEXT),
    ));

    // Commander 文本和血条
    let commander_text = HUD_COMMANDER_LIFE.get(language);
    // Commander 文字（在血条左侧）
    commands.spawn((
        PlayingEntity,
        CommanderText,
        Text2d(commander_text.to_string()),
        common::create_text_font(&font, FONT_SIZE_SCORE),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(
            commander_text_x - HUD_COMMANDER_TEXT_OFFSET,
            WINDOW_TOP_Y - TOP_HUD_Y_OFFSET,
            Z_UI_TEXT,
        ),
    ));
    // Commander 血条
    commands.spawn((
        PlayingEntity,
        CommanderHealthBar,
        CommanderHealthBarOriginalPosition(commander_text_x + HUD_COMMANDER_BAR_OFFSET),
        Sprite {
            color: COLOR_RED,
            custom_size: Some(COMMANDER_BAR_SIZE),
            ..default()
        },
        Transform::from_xyz(
            commander_text_x + HUD_COMMANDER_BAR_OFFSET,
            WINDOW_TOP_Y - TOP_HUD_Y_OFFSET,
            Z_UI_TEXT,
        ),
    ));

    // 敌方剩余数量显示在右侧（初始值）
    let enemy_count_text = ENEMY_COUNT_TEXT.format_named(
        language,
        &[
            ("remaining", ENEMY_COUNT_INITIAL),
            ("total", ENEMY_COUNT_INITIAL),
        ],
    );
    commands.spawn((
        PlayingEntity,
        EnemyCountText,
        Text2d(enemy_count_text),
        common::create_text_font(&font, FONT_SIZE_SCORE),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(
            WINDOW_RIGHT_X - HUD_ENEMY_COUNT_OFFSET,
            WINDOW_TOP_Y - TOP_HUD_Y_OFFSET,
            Z_UI_TEXT,
        ),
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
    top_hud_query: TopHudQuery,
    player_hud_query: PlayerHudQuery,
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
