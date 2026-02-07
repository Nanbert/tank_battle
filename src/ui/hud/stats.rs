//! HUD 统计数据相关类型和配置
//!
//! 定义了玩家统计数据、HUD 统计类型、元素配置等

use bevy::prelude::*;

#[allow(clippy::wildcard_imports)]
use crate::resources::*;
use crate::ui::localization::*;
use crate::ui::constants::*;

// ============================================================================
// Player Stats
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

/// HUD 元素配置
pub struct HudElementConfig {
    /// Y 位置偏移
    pub y_position: crate::ui::HudYPosition,
    /// 统计类型
    pub stat_type: HudStatType,
}

/// 百分比属性配置表
pub const PERCENT_STAT_CONFIGS: [HudElementConfig; 3] = [
    HudElementConfig {
        y_position: HudYPosition::Speed,
        stat_type: HudStatType::Speed,
    },
    HudElementConfig {
        y_position: HudYPosition::FireSpeed,
        stat_type: HudStatType::FireSpeed,
    },
    HudElementConfig {
        y_position: HudYPosition::Protection,
        stat_type: HudStatType::Protection,
    },
];

/// 效果属性配置表
pub const EFFECT_STAT_CONFIGS: [HudElementConfig; 4] = [
    HudElementConfig {
        y_position: HudYPosition::FireShell,
        stat_type: HudStatType::FireShell,
    },
    HudElementConfig {
        y_position: HudYPosition::Penetrate,
        stat_type: HudStatType::Penetrate,
    },
    HudElementConfig {
        y_position: HudYPosition::TrackChain,
        stat_type: HudStatType::TrackChain,
    },
    HudElementConfig {
        y_position: HudYPosition::AirCushion,
        stat_type: HudStatType::AirCushion,
    },
];

// ============================================================================
// StatType Conversions
// ============================================================================

/// StatType 到 HudStatType 的转换
impl From<StatType> for HudStatType {
    fn from(stat_type: StatType) -> Self {
        match stat_type {
            StatType::Speed => HudStatType::Speed,
            StatType::FireSpeed => HudStatType::FireSpeed,
            StatType::Protection => HudStatType::Protection,
            StatType::Shell => HudStatType::Shells,
            StatType::FireShell => HudStatType::FireShell,
            StatType::Penetrate => HudStatType::Penetrate,
            StatType::TrackChain => HudStatType::TrackChain,
            StatType::AirCushion => HudStatType::AirCushion,
            StatType::Score => HudStatType::Score,
        }
    }
}

// ============================================================================
// HUD Stat Value Representation
// ============================================================================

/// HUD 属性值的统一表示
pub enum StatValue {
    /// 百分比值（速度、射速、护盾）
    Percent(usize),
    /// 计数值（炮弹数、分数）
    Count(usize),
    /// 布尔值（道具状态）
    Bool(bool),
}

impl StatValue {
    /// 格式化属性值为字符串
    pub fn format(self, is_max: bool, label_on: &'static str, label_off: &'static str) -> String {
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

// ============================================================================
// HudStatType Methods
// ============================================================================

impl HudStatType {
    /// 获取属性的标签文本
    pub fn get_label(self, language: Language) -> &'static str {
        match self {
            HudStatType::Speed => HUD_LABEL_SPEED.get(language),
            HudStatType::FireSpeed => HUD_LABEL_FIRE_SPEED.get(language),
            HudStatType::Protection => HUD_LABEL_PROTECTION.get(language),
            HudStatType::Shells => HUD_LABEL_SHELLS.get(language),
            HudStatType::FireShell => HUD_LABEL_FIRE_SHELL.get(language),
            HudStatType::Penetrate => HUD_LABEL_PENETRATE.get(language),
            HudStatType::TrackChain => HUD_LABEL_TRACK_CHAIN.get(language),
            HudStatType::AirCushion => HUD_LABEL_AIR_CUSHION.get(language),
            HudStatType::Score => HUD_LABEL_SCORE.get(language),
        }
    }

    /// 获取属性的值（从 PlayerStats 中）
    pub fn get_value(self, stats: &PlayerStats) -> StatValue {
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
    pub fn is_max(self, stats: &PlayerStats) -> bool {
        match self {
            HudStatType::Speed => stats.speed >= HUD_MAX_PERCENT,
            HudStatType::FireSpeed => stats.fire_speed >= HUD_MAX_PERCENT,
            HudStatType::Protection => stats.protection >= HUD_MAX_PERCENT,
            HudStatType::Shells => stats.shells >= HUD_MAX_SHELLS,
            HudStatType::FireShell => stats.fire_shell,
            HudStatType::Penetrate => stats.penetrate,
            HudStatType::TrackChain => stats.track_chain,
            HudStatType::AirCushion => stats.air_cushion,
            HudStatType::Score => false,
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// 更新单个玩家 HUD 文本
pub fn update_single_player_text(
    stats: &PlayerStats,
    stat_type: HudStatType,
    language: Language,
) -> String {
    let prefix = stat_type.get_label(language);
    let value = stat_type.get_value(stats);
    let is_max = stat_type.is_max(stats);
    let label_on = HUD_ON.get(language);
    let label_off = HUD_OFF.get(language);

    let formatted_value = value.format(is_max, label_on, label_off);
    format!("{}{}", prefix, formatted_value)
}

/// 判断 HUD 属性是否达到最大值或On状态（用于动画系统）
pub fn is_hud_stat_at_max_value(
    stat_type: HudStatType,
    player_info: &PlayerInfo,
    player_hud: Option<&super::super::constants::Player1Hud>,
    player_hud2: Option<&super::super::constants::Player2Hud>,
) -> bool {
    // 根据所属玩家选择对应的数据
    let stats = if player_hud.is_some() {
        &player_info.player1
    } else if player_hud2.is_some() {
        player_info.player2.as_ref().unwrap()
    } else {
        return false;
    };

    // 使用统一的 is_max 方法
    stat_type.is_max(stats)
}