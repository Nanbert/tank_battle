//! 道具效果策略模式实现
//!
//! 使用枚举实现零成本抽象的策略模式，避免堆分配

use bevy::prelude::Component;

use crate::constants::*;
use crate::resources::{PlayerStats, StatType};

/// 道具效果应用结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpResult {
    /// 属性改变，需要发送事件
    StatChanged(StatType),
    /// 属性未改变，不发送事件
    NoStatChange,
    /// 指挥官生命改变
    CommanderLifeChanged,
}

/// 道具效果策略（零成本抽象枚举）
///
/// 每个变体对应一种道具效果，无需堆分配
#[derive(Debug, Clone, Copy, Default)]
pub enum PowerUpStrategy {
    /// 速度提升
    SpeedUp,
    /// 护甲提升
    Protection,
    /// 射速提升
    FireSpeed,
    /// 火焰子弹
    FireShell,
    /// 履带链
    TrackChain,
    /// 穿透子弹
    Penetrate,
    /// 修理
    Repair,
    /// 汉堡（恢复指挥官生命）
    Hamburger,
    /// 气垫
    AirCushion,
    /// 弹壳
    Shell,
    /// 默认值
    #[default]
    None,
}

impl PowerUpStrategy {
    /// 应用道具效果到玩家属性
    ///
    /// 返回效果应用结果，用于决定是否发送事件和更新 UI
    pub fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        match self {
            PowerUpStrategy::SpeedUp => {
                if stats.speed < MAX_ATTRIBUTE_VALUE {
                    stats.speed =
                        (stats.speed + POWERUP_ATTRIBUTE_INCREASE).min(MAX_ATTRIBUTE_VALUE);
                }
                PowerUpResult::StatChanged(StatType::Speed)
            }
            PowerUpStrategy::Protection => {
                if stats.protection < MAX_ATTRIBUTE_VALUE {
                    stats.protection =
                        (stats.protection + POWERUP_ATTRIBUTE_INCREASE).min(MAX_ATTRIBUTE_VALUE);
                }
                PowerUpResult::StatChanged(StatType::Protection)
            }
            PowerUpStrategy::FireSpeed => {
                if stats.fire_speed < MAX_ATTRIBUTE_VALUE {
                    stats.fire_speed =
                        (stats.fire_speed + POWERUP_ATTRIBUTE_INCREASE).min(MAX_ATTRIBUTE_VALUE);
                }
                PowerUpResult::StatChanged(StatType::FireSpeed)
            }
            PowerUpStrategy::FireShell => {
                stats.fire_shell = true;
                PowerUpResult::StatChanged(StatType::FireShell)
            }
            PowerUpStrategy::TrackChain => {
                stats.track_chain = true;
                PowerUpResult::StatChanged(StatType::TrackChain)
            }
            PowerUpStrategy::Penetrate => {
                stats.penetrate = true;
                PowerUpResult::StatChanged(StatType::Penetrate)
            }
            PowerUpStrategy::Repair => {
                if stats.life_points < COMMANDER_LIFE_MAX {
                    stats.life_points += 1;
                }
                PowerUpResult::NoStatChange
            }
            PowerUpStrategy::Hamburger => PowerUpResult::CommanderLifeChanged,
            PowerUpStrategy::AirCushion => {
                stats.air_cushion = true;
                PowerUpResult::StatChanged(StatType::AirCushion)
            }
            PowerUpStrategy::Shell => {
                if stats.shells < 2 {
                    stats.shells += 1;
                }
                PowerUpResult::StatChanged(StatType::Shell)
            }
            PowerUpStrategy::None => PowerUpResult::NoStatChange,
        }
    }

    /// 是否需要更新碰撞过滤组
    ///
    /// 某些道具（如 AirCushion）需要修改坦克的碰撞检测
    pub const fn update_filter_groups(&self) -> bool {
        matches!(self, PowerUpStrategy::AirCushion)
    }

    /// 是否影响指挥官生命
    ///
    /// 某些道具（如 Hamburger）会影响指挥官而非玩家
    pub const fn affects_commander(&self) -> bool {
        matches!(self, PowerUpStrategy::Hamburger)
    }
}

/// 道具类型枚举（从 powerup.rs 移动到这里）
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUp {
    SpeedUp,
    Protection,
    FireSpeed,
    FireShell,
    TrackChain,
    Penetrate,
    Repair,
    Hamburger,
    AirCushion,
    Shell,
}

impl PowerUp {
    /// 将道具类型转换为策略枚举（零成本）
    pub const fn into_strategy(self) -> PowerUpStrategy {
        match self {
            PowerUp::SpeedUp => PowerUpStrategy::SpeedUp,
            PowerUp::Protection => PowerUpStrategy::Protection,
            PowerUp::FireSpeed => PowerUpStrategy::FireSpeed,
            PowerUp::FireShell => PowerUpStrategy::FireShell,
            PowerUp::TrackChain => PowerUpStrategy::TrackChain,
            PowerUp::Penetrate => PowerUpStrategy::Penetrate,
            PowerUp::Repair => PowerUpStrategy::Repair,
            PowerUp::Hamburger => PowerUpStrategy::Hamburger,
            PowerUp::AirCushion => PowerUpStrategy::AirCushion,
            PowerUp::Shell => PowerUpStrategy::Shell,
        }
    }

    /// 获取道具的弹出文本
    pub fn get_floating_text(&self, language: crate::resources::Language) -> String {
        use crate::ui::localization::*;
        match self {
            PowerUp::SpeedUp => {
                POWERUP_FLOATING_SPEED_UP.format(language, POWERUP_ATTRIBUTE_INCREASE)
            }
            PowerUp::Protection => {
                POWERUP_FLOATING_PROTECTION.format(language, POWERUP_ATTRIBUTE_INCREASE)
            }
            PowerUp::FireSpeed => {
                POWERUP_FLOATING_FIRE_SPEED.format(language, POWERUP_ATTRIBUTE_INCREASE)
            }
            PowerUp::Repair => POWERUP_FLOATING_REPAIR.get(language).to_string(),
            PowerUp::Hamburger => POWERUP_FLOATING_HAMBURGER.get(language).to_string(),
            PowerUp::Shell => POWERUP_FLOATING_SHELL.get(language).to_string(),
            PowerUp::FireShell => POWERUP_FLOATING_FIRE_SHELL.get(language).to_string(),
            PowerUp::TrackChain => POWERUP_FLOATING_TRACK_CHAIN.get(language).to_string(),
            PowerUp::Penetrate => POWERUP_FLOATING_PENETRATE.get(language).to_string(),
            PowerUp::AirCushion => POWERUP_FLOATING_AIR_CUSHION.get(language).to_string(),
        }
    }

    /// 获取道具的纹理资源
    pub fn get_texture_resources<'a>(
        &self,
        texture_resources: &'a crate::resources::GameTextureResources,
        atlas_layouts: &'a crate::resources::GameAtlasLayoutResources,
    ) -> (
        bevy::prelude::Handle<bevy::prelude::Image>,
        &'static crate::atlas::TextureAtlasInfo,
        bevy::prelude::Handle<bevy::prelude::TextureAtlasLayout>,
    ) {
        match self {
            PowerUp::SpeedUp => (
                texture_resources.speed_up_icon.clone(),
                &crate::atlas::POWER_UP_SPEED_UP_ATLAS,
                atlas_layouts.speed_up_icon.clone(),
            ),
            PowerUp::Protection => (
                texture_resources.protection_icon.clone(),
                &crate::atlas::POWER_UP_PROTECTION_ATLAS,
                atlas_layouts.protection_icon.clone(),
            ),
            PowerUp::FireSpeed => (
                texture_resources.fire_speed_icon.clone(),
                &crate::atlas::POWER_UP_FIRE_SPEED_ATLAS,
                atlas_layouts.fire_speed_icon.clone(),
            ),
            PowerUp::FireShell => (
                texture_resources.fire_shell_icon.clone(),
                &crate::atlas::POWER_UP_FIRE_SHELL_ATLAS,
                atlas_layouts.fire_shell_icon.clone(),
            ),
            PowerUp::TrackChain => (
                texture_resources.track_chain_icon.clone(),
                &crate::atlas::POWER_UP_TRACK_CHAIN_ATLAS,
                atlas_layouts.track_chain_icon.clone(),
            ),
            PowerUp::Penetrate => (
                texture_resources.penetrate_icon.clone(),
                &crate::atlas::POWER_UP_PENETRATE_ATLAS,
                atlas_layouts.penetrate_icon.clone(),
            ),
            PowerUp::Repair => (
                texture_resources.repair_icon.clone(),
                &crate::atlas::POWER_UP_REPAIR_ATLAS,
                atlas_layouts.repair_icon.clone(),
            ),
            PowerUp::Hamburger => (
                texture_resources.hamburger_icon.clone(),
                &crate::atlas::POWER_UP_HAMBURGER_ATLAS,
                atlas_layouts.hamburger_icon.clone(),
            ),
            PowerUp::AirCushion => (
                texture_resources.air_cushion_icon.clone(),
                &crate::atlas::POWER_UP_AIR_CUSHION_ATLAS,
                atlas_layouts.air_cushion_icon.clone(),
            ),
            PowerUp::Shell => (
                texture_resources.shell_icon.clone(),
                &crate::atlas::POWER_UP_SHELL_ATLAS,
                atlas_layouts.shell_icon.clone(),
            ),
        }
    }
}
