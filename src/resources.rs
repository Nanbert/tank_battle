//! Game resources for the Tank Battle game

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::constants::TankType;

#[derive(Resource, Default)]
pub struct BulletTracker {
    /// 坦克实体 -> 场上子弹数量
    pub active_bullets: HashMap<Entity, usize>,
    /// 子弹实体 -> 坦克实体
    pub bullet_to_tank: HashMap<Entity, Entity>,
}

impl BulletTracker {
    /// 检查坦克是否可以射击
    pub fn can_fire(&self, tank: Entity, max: usize) -> bool {
        self.active_bullets.get(&tank).copied().unwrap_or(0) < max
    }

    /// 添加子弹
    pub fn add_bullet(&mut self, bullet: Entity, tank: Entity) {
        *self.active_bullets.entry(tank).or_insert(0) += 1;
        self.bullet_to_tank.insert(bullet, tank);
    }

    /// 移除子弹，返回所属坦克
    pub fn remove_bullet(&mut self, bullet: Entity) -> Option<Entity> {
        if let Some(tank) = self.bullet_to_tank.remove(&bullet) {
            if let Some(count) = self.active_bullets.get_mut(&tank) {
                *count -= 1;
                if *count == 0 {
                    self.active_bullets.remove(&tank);
                }
            }
            Some(tank)
        } else {
            None
        }
    }
}

#[derive(Resource, Default)]
pub struct StartAnimationFrames {
    pub frames: Vec<Handle<Image>>,
    pub texture: Handle<Image>,
    pub texture_atlas_layout: Option<Handle<TextureAtlasLayout>>,
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
pub struct EnemySpawnState {
    pub has_spawned: usize,    // 已生成数量
    pub max_count: usize,      // 总数量（每关固定20个）
    pub spawn_cooldown: Timer, // 生成冷却时间
}

impl Default for EnemySpawnState {
    fn default() -> Self {
        Self {
            has_spawned: 0,
            max_count: 20,
            spawn_cooldown: Timer::from_seconds(0.8, TimerMode::Once),
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
    pub life_red_bar: usize, // max 3
    pub energy_blue_bar: usize, // max 3
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
    Shell,
}

// Barrier 伤害追踪器，使用冷却机制防止玩家坦克频繁受伤
#[derive(Resource, Default)]
pub struct BarrierDamageTracker {
    pub cooldowns: HashMap<Entity, Timer>, // 记录每个玩家坦克的受伤冷却计时器
}

// Dash 扣血追踪器，防止一次 dash 多次扣血
#[derive(Resource, Default)]
pub struct DashDamageTracker {
    pub has_taken_damage: HashSet<Entity>, // 记录本次 dash 已经扣血的玩家坦克
}

// 标记游戏实体是否已生成
#[derive(Resource, Default)]
pub struct GameEntitiesSpawned(pub bool);