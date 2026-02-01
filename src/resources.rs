//! Game resources for the Tank Battle game

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::constants::{
    BLUE_BAR_REGEN_INTERVAL, ENEMIES_PER_LEVEL, ENEMY_SPAWN_COOLDOWN, TankType,
};

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
    pub textures: Vec<Handle<Image>>,
    pub texture_atlas_layouts: Vec<Handle<TextureAtlasLayout>>,
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

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    #[default]
    Chinese,
    English,
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
            max_count: ENEMIES_PER_LEVEL,
            spawn_cooldown: Timer::from_seconds(ENEMY_SPAWN_COOLDOWN, TimerMode::Once),
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
    pub player1: PlayerStats,
    pub player2: Option<PlayerStats>,
}

#[derive(Clone, Default)]
pub struct PlayerStats {
    pub speed: usize,
    pub fire_speed: usize,
    pub protection: usize,
    pub shells: usize,
    pub penetrate: bool,
    pub track_chain: bool,
    pub air_cushion: bool,
    pub fire_shell: bool,
    pub life_points: usize,    // max 3
    pub energy_points: usize, // max 3
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
            timer: Timer::from_seconds(BLUE_BAR_REGEN_INTERVAL, TimerMode::Repeating),
        }
    }
}

// Commander 生命值资源
#[derive(Resource)]
pub struct CommanderLife {
    pub life_points: usize, // max 3
}

impl Default for CommanderLife {
    fn default() -> Self {
        Self { life_points: 3 }
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

// 能量不足提示冷却追踪器，防止重复触发提示
#[derive(Resource, Default)]
pub struct InsufficientEnergyTracker {
    pub cooldowns: HashMap<Entity, Timer>, // 记录每个玩家坦克的能量不足提示冷却计时器
}

// 地形纹理图集布局资源
#[derive(Resource)]
pub struct TerrainAtlasLayouts {
    pub sea: Handle<TextureAtlasLayout>,
    pub forest: Handle<TextureAtlasLayout>,
    pub forest_fire: Handle<TextureAtlasLayout>,
}

// 子弹资源
#[derive(Resource)]
pub struct BulletResources {
    /// 玩家1子弹纹理
    pub bullet_player1: Handle<Image>,
    /// 玩家2子弹纹理
    pub bullet_player2: Handle<Image>,
    /// 敌方子弹纹理
    pub bullet_enemy: Handle<Image>,
}

// ==================== 游戏资源管理 ====================

/// 字体资源
#[derive(Resource)]
pub struct FontResources {
    pub cn: Handle<Font>,
    pub en: Handle<Font>,
}

/// 玩家坦克资源
#[derive(Resource)]
pub struct PlayerTankResources {
    pub player1: Handle<Image>,
    pub player2: Handle<Image>,
    pub single_barrel: Handle<Image>,
    pub double_barrel: Handle<Image>,
}

/// 司令官资源
#[derive(Resource)]
pub struct CommanderResources {
    pub texture: Handle<Image>,
    pub dead_texture: Handle<Image>,
    pub avatar: Handle<Image>,
    pub avatar_death: Handle<Image>,
    pub avatar_commander_dead: Handle<Image>,
}

/// 音效资源
#[derive(Resource)]
pub struct SoundResources {
    pub explosion: Handle<AudioSource>,
    pub brick_hit: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub metal_crash: Handle<AudioSource>,
    pub laser_charge: Handle<AudioSource>,
    pub laser: Handle<AudioSource>,
    pub commander_get_shot: Handle<AudioSource>,
    pub commander_death: Handle<AudioSource>,
    pub player_shot: Handle<AudioSource>,
    pub enemy_shot: Handle<AudioSource>,
}

/// 特效纹理资源
#[derive(Resource)]
pub struct EffectResources {
    pub explosion: Handle<Image>,
    pub spark: Handle<Image>,
    pub smoke: Handle<Image>,
    pub bubble: Handle<Image>,
    pub energy_blue_ball: Handle<Image>,
    pub energy_red_ball: Handle<Image>,
    pub forest_fire: Handle<Image>,
}

/// 地图纹理资源
#[derive(Resource)]
pub struct MapResources {
    pub brick: Handle<Image>,
    pub steel: Handle<Image>,
    pub tree: Handle<Image>,
    pub sea: Handle<Image>,
    pub barrier: Handle<Image>,
}

/// 敌方坦克资源
#[derive(Resource)]
pub struct EnemyResources {
    pub enemy_born: Handle<Image>,
    pub enemy_tank: Handle<Image>,
}

/// 道具纹理资源
#[derive(Resource)]
pub struct PowerUpResources {
    pub speed_up: Handle<Image>,
    pub protection: Handle<Image>,
    pub fire_speed: Handle<Image>,
    pub fire_shell: Handle<Image>,
    pub track_chain: Handle<Image>,
    pub track_train: Handle<Image>,
    pub penetrate: Handle<Image>,
    pub repair: Handle<Image>,
    pub hamburger: Handle<Image>,
    pub air_cushion: Handle<Image>,
    pub shell: Handle<Image>,
}

/// 激光纹理资源
#[derive(Resource)]
pub struct LaserResources {
    pub laser_blue: Handle<Image>,
    pub laser_red: Handle<Image>,
}

/// 菜单背景纹理资源
#[derive(Resource)]
pub struct MenuResources {
    pub background_part1: Handle<Image>,
    pub background_part2: Handle<Image>,
    pub background_part3: Handle<Image>,
}

/// 司令官音乐纹理资源
#[derive(Resource)]
pub struct CommanderMusicResources {
    pub music_note: Handle<Image>,
}

/// 环境音效资源
#[derive(Resource)]
pub struct AmbienceResources {
    pub burn_tree: Handle<AudioSource>,
    pub sea_ambience: Handle<AudioSource>,
    pub commander_music_000: Handle<AudioSource>,
    pub commander_music_001: Handle<AudioSource>,
    pub commander_music_002: Handle<AudioSource>,
    pub commander_music_003: Handle<AudioSource>,
    pub tree_ambience: Handle<AudioSource>,
}
