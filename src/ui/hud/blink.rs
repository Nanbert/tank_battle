//! HUD 文本闪烁动画系统
//!
//! 处理 HUD 属性变更时的文本闪烁效果

use bevy::prelude::*;

use super::super::common;
use super::stats::*;
#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;
use crate::ui::constants::*;

// ============================================================================
// Blink Animation System
// ============================================================================

/// 触发文本闪烁计时器
/// 使用统一的 BlinkAnimation 系统（绿色闪烁，完成后移除组件）
/// 根据是否达到最大值决定最终颜色
fn trigger_blink_timer(commands: &mut Commands, entity: Entity, is_max: bool) {
    let final_color = if is_max {
        COLOR_RED  // 达到最大值时最终颜色为红色
    } else {
        COLOR_WHITE  // 否则为白色
    };

    commands
        .entity(entity)
        .insert(common::BlinkAnimation::new_with_despawn(
            TEXT_BLINK_CYCLE,
            COLOR_GREEN,
            COLOR_TRANSPARENT,
            final_color,
            false, // despawn_on_complete
        ));
}

/// 处理 HUD 属性变更事件，触发文字闪烁
/// 优化：使用组件标记匹配而不是字符串前缀匹配
pub fn handle_hud_stat_changed(
    mut events: MessageReader<PlayerStatChanged>,
    mut commands: Commands,
    player1_hud_texts: Query<(Entity, &HudStatType), With<Player1Hud>>,
    player2_hud_texts: Query<(Entity, &HudStatType), With<Player2Hud>>,
    player_info: Res<PlayerInfo>,
) {
    for event in events.read() {
        let target_stat_type: HudStatType = event.stat_type.into();

        // 获取对应的玩家统计数据
        let stats = match event.player_type {
            TankType::Player1 => &player_info.player1,
            TankType::Player2 => player_info.player2.as_ref().unwrap(),
            TankType::Enemy => continue,
        };

        // 检查是否达到最大值
        let is_max = target_stat_type.is_max(stats);

        match event.player_type {
            TankType::Player1 => {
                for (entity, stat_type) in player1_hud_texts.iter() {
                    if *stat_type == target_stat_type {
                        trigger_blink_timer(&mut commands, entity, is_max);
                        break;
                    }
                }
            }
            TankType::Player2 => {
                for (entity, stat_type) in player2_hud_texts.iter() {
                    if *stat_type == target_stat_type {
                        trigger_blink_timer(&mut commands, entity, is_max);
                        break;
                    }
                }
            }
            TankType::Enemy => {}
        }
    }
}

/// 处理 HUD 属性达到最大值时变为红色
/// 当属性达到最大值时，移除闪烁动画并设置为红色
pub fn handle_hud_stat_max_value(
    player_info: Res<PlayerInfo>,
    mut query: Query<(
        &mut TextColor,
        &HudStatType,
        Option<&Player1Hud>,
        Option<&Player2Hud>,
    )>,
) {
    for (mut color, stat_type, is_p1, is_p2) in query.iter_mut() {
        // 根据所属玩家选择对应的数据
        let stats = if is_p1.is_some() {
            &player_info.player1
        } else if is_p2.is_some() {
            player_info.player2.as_ref().unwrap()
        } else {
            continue;
        };

        // 检查是否达到最大值
        if stat_type.is_max(stats) {
            color.0 = COLOR_RED;
        }
    }
}
