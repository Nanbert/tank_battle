//! HUD 文本闪烁动画系统
//!
//! 处理 HUD 属性变更时的文本闪烁效果

use bevy::prelude::*;

#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;
use crate::ui::constants::*;
use super::stats::*;

// ============================================================================
// Blink Animation System
// ============================================================================

/// 触发文本闪烁计时器
fn trigger_blink_timer(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .insert(UiTimer::new(TEXT_BLINK_CYCLE, TimerMode::Once));
}

/// 处理 HUD 属性变更事件，触发文字闪烁
/// 优化：使用组件标记匹配而不是字符串前缀匹配
pub fn handle_hud_stat_changed(
    mut events: MessageReader<PlayerStatChanged>,
    mut commands: Commands,
    player1_hud_texts: Query<(Entity, &HudStatType), With<Player1Hud>>,
    player2_hud_texts: Query<(Entity, &HudStatType), With<Player2Hud>>,
) {
    for event in events.read() {
        let target_stat_type: HudStatType = event.stat_type.into();

        match event.player_type {
            TankType::Player1 => {
                for (entity, stat_type) in player1_hud_texts.iter() {
                    if *stat_type == target_stat_type {
                        trigger_blink_timer(&mut commands, entity);
                        break;
                    }
                }
            }
            TankType::Player2 => {
                for (entity, stat_type) in player2_hud_texts.iter() {
                    if *stat_type == target_stat_type {
                        trigger_blink_timer(&mut commands, entity);
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