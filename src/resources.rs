//! Game resources for the Tank Battle game

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::constants::TankType;

#[derive(Resource, Default)]
pub struct CanFire(pub HashSet<Entity>);

#[derive(Resource, Default)]
pub struct BulletOwners {
    pub owners: HashMap<Entity, Entity>, // 子弹实体 -> 坦克实体
}

#[derive(Resource, Default)]
pub struct StartAnimationFrames {
    pub frames: Vec<Handle<Image>>,
}

#[derive(Resource)]
pub struct FadingOut {
    pub alpha: f32,
}

impl Default for FadingOut {
    fn default() -> Self {
        Self { alpha: 1.0 }
    }
}

#[derive(Resource, Default)]
pub struct CurrentMenuSelection {
    pub selected_index: usize, // 0 = 1 Player, 1 = 2 Player, 2 = EXIT
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    #[default]
    OnePlayer,
    TwoPlayers,
}

#[derive(Resource)]
pub struct EnemyCount {
    pub total_spawned: usize, // 已生成的敌方坦克总数
    pub max_count: usize,     // 最大敌方坦克数量
    pub current_enemies: usize, // 当前存活的敌方坦克数量
}

impl Default for EnemyCount {
    fn default() -> Self {
        Self {
            total_spawned: 0,
            max_count: 20,
            current_enemies: 0,
        }
    }
}

#[derive(Resource)]
pub struct StageLevel(pub usize); // 当前关卡

impl Default for StageLevel {
    fn default() -> Self {
        Self(1) // 默认从第一关开始
    }
}

#[derive(Resource, Default)]
pub struct MenuBlinkTimer(pub Timer);

#[derive(Resource, Default)]
pub struct StageIntroTimer {
    pub fade_in: Timer,
    pub stay: Timer,
    pub fade_out: Timer,
}

#[derive(Resource, Default)]
pub struct PlayerInfo {
    pub players: HashMap<TankType, PlayerStats>,
}

#[derive(Clone)]
pub struct PlayerStats {
    pub name: String,
    pub speed: usize,
    pub fire_speed: usize,
    pub protection: usize,
    pub shells: usize,
    pub penetrate: bool,
    pub track_chain: bool,
    pub air_cushion: bool,
    pub fire_shell: bool,
    pub life_red_bar: usize,
    pub energy_blue_bar: usize,
    pub score: usize,
}

// 玩家回城计时器
#[derive(Resource, Default)]
pub struct RecallTimers {
    pub timers: HashMap<Entity, RecallTimer>,
}

pub struct RecallTimer {
    pub timer: Timer,
    pub start_position: Vec3,
}

impl RecallTimer {
    pub fn new(start_position: Vec3, duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            start_position,
        }
    }
}

// 玩家冲刺计时器
#[derive(Resource, Default)]
pub struct DashTimers {
    pub timers: HashMap<Entity, DashTimer>,
}

pub struct DashTimer {
    pub timer: Timer,
    pub direction: Vec2,
}

impl DashTimer {
    pub fn new(direction: Vec2, duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            direction,
        }
    }
}

// 蓝条恢复计时器
#[derive(Resource)]
pub struct BlueBarRegenTimer {
    pub timer: Timer,
}

impl Default for BlueBarRegenTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(5.0, TimerMode::Repeating),
        }
    }
}

// Commander 生命值资源
#[derive(Resource)]
pub struct CommanderLife {
    pub life_red_bar: usize, // max 3
}

impl Default for CommanderLife {
    fn default() -> Self {
        Self { life_red_bar: 3 }
    }
}

// 玩家属性变更事件
#[derive(Message, Clone, Copy)]
pub struct PlayerStatChanged {
    pub player_type: TankType,
    pub stat_type: StatType,
}

// 玩家属性类型
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum StatType {
    Score,
    Speed,
    Protection,
    FireSpeed,
    FireShell,
    TrackChain,
    Penetrate,
    AirCushion,
}

// Barrier 伤害追踪器，使用冷却机制防止玩家坦克频繁受伤
#[derive(Resource, Default)]
pub struct BarrierDamageTracker {
    pub cooldowns: HashMap<Entity, Timer>, // 记录每个玩家坦克的受伤冷却计时器
}

// 标记游戏实体是否已生成
#[derive(Resource, Default)]
pub struct GameEntitiesSpawned(pub bool);