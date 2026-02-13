//! 道具效果策略模式实现
//!
//! 使用策略模式处理不同道具的效果，符合开闭原则和单一职责原则

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

/// 道具效果策略 trait
///
/// 每个道具类型实现此 trait，定义其具体效果
pub trait PowerUpEffect {
    /// 应用道具效果到玩家属性
    ///
    /// 返回效果应用结果，用于决定是否发送事件和更新 UI
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult;

    /// 是否需要更新碰撞过滤组
    ///
    /// 某些道具（如 AirCushion）需要修改坦克的碰撞检测
    fn update_filter_groups(&self) -> bool {
        false
    }

    /// 是否影响指挥官生命
    ///
    /// 某些道具（如 Hamburger）会影响指挥官而非玩家
    fn affects_commander(&self) -> bool {
        false
    }
}

/// 速度提升策略
pub struct SpeedUpStrategy;

impl PowerUpEffect for SpeedUpStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        if stats.speed < MAX_ATTRIBUTE_VALUE {
            stats.speed = (stats.speed + POWERUP_ATTRIBUTE_INCREASE)
                .min(MAX_ATTRIBUTE_VALUE);
        }
        PowerUpResult::StatChanged(StatType::Speed)
    }
}

/// 护甲提升策略
pub struct ProtectionStrategy;

impl PowerUpEffect for ProtectionStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        if stats.protection < MAX_ATTRIBUTE_VALUE {
            stats.protection = (stats.protection + POWERUP_ATTRIBUTE_INCREASE)
                .min(MAX_ATTRIBUTE_VALUE);
        }
        PowerUpResult::StatChanged(StatType::Protection)
    }
}

/// 射速提升策略
pub struct FireSpeedStrategy;

impl PowerUpEffect for FireSpeedStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        if stats.fire_speed < MAX_ATTRIBUTE_VALUE {
            stats.fire_speed = (stats.fire_speed + POWERUP_ATTRIBUTE_INCREASE)
                .min(MAX_ATTRIBUTE_VALUE);
        }
        PowerUpResult::StatChanged(StatType::FireSpeed)
    }
}

/// 火焰子弹策略
pub struct FireShellStrategy;

impl PowerUpEffect for FireShellStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        stats.fire_shell = true;
        PowerUpResult::StatChanged(StatType::FireShell)
    }
}

/// 履带链策略
pub struct TrackChainStrategy;

impl PowerUpEffect for TrackChainStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        stats.track_chain = true;
        PowerUpResult::StatChanged(StatType::TrackChain)
    }
}

/// 穿透子弹策略
pub struct PenetrateStrategy;

impl PowerUpEffect for PenetrateStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        stats.penetrate = true;
        PowerUpResult::StatChanged(StatType::Penetrate)
    }
}

/// 修理策略
pub struct RepairStrategy;

impl PowerUpEffect for RepairStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        if stats.life_points < COMMANDER_LIFE_MAX {
            stats.life_points += 1;
        }
        PowerUpResult::NoStatChange
    }
}

/// 汉堡策略（恢复指挥官生命）
pub struct HamburgerStrategy;

impl PowerUpEffect for HamburgerStrategy {
    fn apply(&self, _stats: &mut PlayerStats) -> PowerUpResult {
        PowerUpResult::CommanderLifeChanged
    }

    fn affects_commander(&self) -> bool {
        true
    }
}

/// 气垫策略
pub struct AirCushionStrategy;

impl PowerUpEffect for AirCushionStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        stats.air_cushion = true;
        PowerUpResult::StatChanged(StatType::AirCushion)
    }

    fn update_filter_groups(&self) -> bool {
        true
    }
}

/// 弹壳策略（增加子弹数量）
pub struct ShellStrategy;

impl PowerUpEffect for ShellStrategy {
    fn apply(&self, stats: &mut PlayerStats) -> PowerUpResult {
        if stats.shells < 2 {
            stats.shells += 1;
        }
        PowerUpResult::StatChanged(StatType::Shell)
    }
}

/// 根据道具类型获取对应的策略
///
/// 这是策略模式的工厂函数，用于创建策略实例
pub fn get_strategy(powerup: PowerUp) -> Box<dyn PowerUpEffect> {
    match powerup {
        PowerUp::SpeedUp => Box::new(SpeedUpStrategy),
        PowerUp::Protection => Box::new(ProtectionStrategy),
        PowerUp::FireSpeed => Box::new(FireSpeedStrategy),
        PowerUp::FireShell => Box::new(FireShellStrategy),
        PowerUp::TrackChain => Box::new(TrackChainStrategy),
        PowerUp::Penetrate => Box::new(PenetrateStrategy),
        PowerUp::Repair => Box::new(RepairStrategy),
        PowerUp::Hamburger => Box::new(HamburgerStrategy),
        PowerUp::AirCushion => Box::new(AirCushionStrategy),
        PowerUp::Shell => Box::new(ShellStrategy),
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
    /// 获取道具的弹出文本
    pub fn get_floating_text(&self, language: crate::resources::Language) -> String {
        use crate::ui::localization::*;
        match self {
            PowerUp::SpeedUp => POWERUP_FLOATING_SPEED_UP.format(language, POWERUP_ATTRIBUTE_INCREASE),
            PowerUp::Protection => POWERUP_FLOATING_PROTECTION.format(language, POWERUP_ATTRIBUTE_INCREASE),
            PowerUp::FireSpeed => POWERUP_FLOATING_FIRE_SPEED.format(language, POWERUP_ATTRIBUTE_INCREASE),
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
    ) -> (bevy::prelude::Handle<bevy::prelude::Image>, &'static crate::atlas::TextureAtlasInfo, bevy::prelude::Handle<bevy::prelude::TextureAtlasLayout>) {
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