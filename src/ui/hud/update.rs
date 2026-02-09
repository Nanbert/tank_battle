//! HUD 更新函数
//!
//! 处理 HUD 文本的更新

use bevy::prelude::*;

use super::spawn::{BarAnimation, PlayerHudQuery, TopHudQuery};
use super::stats::*;
#[allow(clippy::wildcard_imports)]
use crate::constants::*;
use crate::resources::*;
use crate::ui::common;
use crate::ui::constants::*;

// ============================================================================
// 血条动画系统
// ============================================================================

/// 更新血条和蓝条的动画效果
///
/// 此系统每帧运行，平滑过渡血条和蓝条的宽度变化
pub fn update_bar_animations(
    mut bars: Query<(&mut BarAnimation, &mut Sprite, &mut Transform), Or<(With<HealthBarForeground>, With<BlueBarForeground>)>>,
) {
    for (mut animation, mut sprite, mut transform) in bars.iter_mut() {
        // 更新动画
        let is_animating = animation.update();

        // 如果动画进行中或需要更新显示
        if is_animating {
            // 更新 Sprite 宽度
            if let Some(ref mut size) = sprite.custom_size {
                size.x = animation.current_width;
            }

            // 更新 Transform 位置（保持血条左对齐）
            // 位置 = 左边缘 + 宽度/2
            transform.translation.x = animation.left_edge_x + animation.current_width / 2.0;
        }
    }
}

// ============================================================================
// Despawn Functions
// ============================================================================

/// 销毁 HUD
pub fn despawn_hud(
    mut commands: Commands,
    top_hud_query: TopHudQuery,
    player_hud_query: PlayerHudQuery,
) {
    // 销毁顶部 HUD
    crate::utils::cleanup_entities(&mut commands, top_hud_query.iter());

    // 销毁玩家 HUD
    crate::utils::cleanup_entities(&mut commands, player_hud_query.iter());
}

// ============================================================================
// Player HUD Update Functions
// ============================================================================

/// 更新单个玩家的 HUD 文本（内部函数）
fn update_single_player_hud_text(
    stats: &PlayerStats,
    player_hud_query: &mut Query<(
        &mut Text2d,
        &HudStatType,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    is_player1: bool,
    language: Language,
) {
    for (mut text, stat_type, is_p1, is_p2) in player_hud_query.iter_mut() {
        if (is_player1 && is_p1.is_some()) || (!is_player1 && is_p2.is_some()) {
            text.0 = update_single_player_text(stats, *stat_type, language);
        }
    }
}

/// 更新单个玩家的 HUD 血条和蓝条（内部函数）
fn update_single_player_hud_bars(
    stats: &PlayerStats,
    bar_query: &mut Query<(
        &mut BarAnimation,
        Option<&HealthBarForeground>,
        Option<&BlueBarForeground>,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    is_player1: bool,
) {
    for (mut animation, is_health_foreground, is_blue_foreground, is_p1, is_p2) in
        bar_query.iter_mut()
    {
        if (is_player1 && is_p1.is_some()) || (!is_player1 && is_p2.is_some()) {
            if is_health_foreground.is_some() {
                let target_width = HUD_BAR_SIZE.x * (stats.life_points as f32 / HUD_MAX_LIFE_POINTS);
                animation.set_target(target_width);
            } else if is_blue_foreground.is_some() {
                let target_width = HUD_BAR_SIZE.x * (stats.energy_points as f32 / HUD_MAX_LIFE_POINTS);
                animation.set_target(target_width);
            }
        }
    }
}

/// 更新单个玩家的 HUD（组合函数）
///
/// 此函数消除了玩家1和玩家2 HUD 更新的重复逻辑，将逻辑拆分为两个独立的函数
fn update_single_player_hud(
    stats: &PlayerStats,
    _x_pos: f32,
    player_hud_query: &mut Query<(
        &mut Text2d,
        &HudStatType,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    bar_query: &mut Query<(
        &mut BarAnimation,
        Option<&HealthBarForeground>,
        Option<&BlueBarForeground>,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
    is_player1: bool,
    language: Language,
) {
    update_single_player_hud_text(stats, player_hud_query, is_player1, language);
    update_single_player_hud_bars(stats, bar_query, is_player1);
}

/// 更新玩家 HUD（统一处理玩家1和玩家2）
///
/// 此函数根据玩家信息资源更新所有 HUD 元素：
/// - 更新所有属性文本（速度、射速、护盾、炮弹、效果状态、分数）
/// - 更新血条和蓝条的宽度
///
/// # 性能优化
/// - 使用 `resource_changed::<PlayerInfo>` 条件确保只在玩家信息变化时更新
/// - 使用组件标记（Player1Hud/Player2Hud）直接查询特定玩家的 HUD 元素
/// - 分别处理玩家1和玩家2的 HUD，避免不必要的遍历
/// - 使用宏消除重复逻辑
///
/// # 参数
/// - `player_info`: 玩家信息资源
/// - `game_mode`: 游戏模式（单人或双人）
/// - `player_hud_query`: HUD 文本查询
/// - `bar_query`: 血条/蓝条查询
/// - `language`: 当前语言设置
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
        &mut BarAnimation,
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
        WINDOW_LEFT_X + HUD_PLAYER_OFFSET,
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
            WINDOW_RIGHT_X - HUD_PLAYER_OFFSET,
            &mut player_hud_query,
            &mut bar_query,
            false,
            *language,
        );
    }
}

/// 处理单个玩家头像死亡的内部函数
pub fn handle_avatar_death_internal(
    player_dead: bool,
    has_handled: &mut bool,
    tank_type: TankType,
    player_avatars: &mut Query<(&mut Sprite, &PlayerUI), With<PlayerAvatar>>,
    texture_resources: &GameTextureResources,
) {
    if player_dead && !*has_handled {
        for (mut sprite, player_ui) in player_avatars.iter_mut() {
            if player_ui.player_type == tank_type {
                sprite.image = texture_resources.avatar_death.clone();
                sprite.texture_atlas = None; // 死亡头像不需要动画
                sprite.custom_size = Some(crate::atlas::PLAYER_AVATAR_ATLAS.display_size);
                *has_handled = true;
            }
        }
    } else if !player_dead && *has_handled {
        *has_handled = false; // 重置状态，以便下次死亡时再次处理
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
    handle_avatar_death_internal(
        player1_dead,
        &mut has_handled_player1,
        TankType::Player1,
        &mut player_avatars,
        &texture_resources,
    );

    // 处理玩家2头像（如果存在）
    if let Some(ref player2_stats) = player_info.player2 {
        let player2_dead = player2_stats.life_points == 0;
        handle_avatar_death_internal(
            player2_dead,
            &mut has_handled_player2,
            TankType::Player2,
            &mut player_avatars,
            &texture_resources,
        );
    }
}

/// 处理司令官阵亡时更换纹理和头像
///
/// 此函数执行以下操作：
/// 1. 将司令官纹理替换为死亡纹理
/// 2. 停止司令官的动画计时器
/// 3. 停止司令官音乐动画
/// 4. 将玩家头像设置为悲伤图片（仅限仍存活的玩家）
/// 5. 已死亡的玩家头像保持死亡图片不变
///
/// 注意：此函数只会在司令官生命值归零时执行一次
pub fn handle_commander_death(
    texture_resources: Res<GameTextureResources>,
    commander_life: Res<CommanderLife>,
    player_info: Res<PlayerInfo>,
    mut queries: ParamSet<(
        Query<(&mut Sprite, &mut AnimationTimer), With<Commander>>,
        Query<(&mut Sprite, &PlayerUI), With<PlayerAvatar>>,
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

    // 更换司令官纹理为死亡纹理并停止动画
    for (mut sprite, mut timer) in queries.p0().iter_mut() {
        sprite.image = texture_resources.commander_dead.clone();
        // 移除纹理图集，因为死亡纹理是单张图片
        sprite.texture_atlas = None;
        timer.pause();
    }

    // 停止司令官音乐动画
    for mut timer in queries.p2().iter_mut() {
        timer.pause();
    }

    // 只将还活着的玩家头像设置为悲伤图片，已经死亡的玩家头像保持死亡图片
    for (mut sprite, player_ui) in queries.p1().iter_mut() {
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

// ============================================================================
// Top HUD Update Functions
// ============================================================================

/// 更新关卡信息文本
pub fn update_stage_text(
    stage_level: Res<StageLevel>,
    mut stage_text_query: Query<&mut Text2d, With<StageText>>,
    language: Res<Language>,
) {
    use crate::ui::localization::STAGE_TEXT;
    for mut text in &mut stage_text_query {
        text.0 = STAGE_TEXT.format(*language, stage_level.0);
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
            HUD_COMMANDER_MAX_LIFE,
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
    use crate::ui::localization::ENEMY_COUNT_TEXT;
    let remaining = enemy_spawn_state.max_count - enemy_spawn_state.has_spawned;
    let max_count = enemy_spawn_state.max_count;

    let text =
        ENEMY_COUNT_TEXT.format_named(*language, &[("remaining", remaining), ("total", max_count)]);

    for mut text_mut in &mut query {
        text_mut.0 = text.clone();
    }
}
