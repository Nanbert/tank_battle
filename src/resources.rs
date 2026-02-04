//! Game resources for the Tank Battle game

use bevy::prelude::*;
use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use std::time::Duration;

use crate::constants::{
    BLUE_BAR_REGEN_INTERVAL, ENEMIES_PER_LEVEL, ENEMY_SPAWN_COOLDOWN, TankType,
};
use crate::utils;

#[derive(Resource, Default)]
pub struct BulletTracker {
    /// 坦克实体 -> 场上子弹数量
    pub active_bullets: EntityHashMap<usize>,
    /// 子弹实体 -> 坦克实体
    pub bullet_to_tank: EntityHashMap<Entity>,
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
    pub has_spawned: usize,               // 已生成数量
    pub max_count: usize,                 // 总数量（每关固定20个）
    pub spawn_cooldown: Timer,            // 生成冷却时间
    pub stage_complete_delay: Timer,      // 关卡完成延迟计时器（2秒后进入下一关）
}

impl Default for EnemySpawnState {
    fn default() -> Self {
        Self {
            has_spawned: 0,
            max_count: ENEMIES_PER_LEVEL,
            spawn_cooldown: Timer::from_seconds(ENEMY_SPAWN_COOLDOWN, TimerMode::Once),
            stage_complete_delay: Timer::from_seconds(2.0, TimerMode::Once),
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

impl PlayerInfo {
    /// 获取指定玩家类型的统计数据
    pub fn get_stats(&self, tank_type: TankType) -> Option<&PlayerStats> {
        match tank_type {
            TankType::Player1 => Some(&self.player1),
            TankType::Player2 => self.player2.as_ref(),
            TankType::Enemy => None,
        }
    }

    /// 获取指定玩家类型的可变统计数据
    pub fn get_stats_mut(&mut self, tank_type: TankType) -> Option<&mut PlayerStats> {
        match tank_type {
            TankType::Player1 => Some(&mut self.player1),
            TankType::Player2 => self.player2.as_mut(),
            TankType::Enemy => None,
        }
    }

    /// 获取指定玩家类型的某个属性值
    pub fn get_stat_value<F>(&self, tank_type: TankType, getter: F) -> usize
    where
        F: FnOnce(&PlayerStats) -> usize,
    {
        self.get_stats(tank_type).map_or(0, getter)
    }

    /// 获取玩家的速度百分比 (0.0 - 1.0)
    pub fn get_speed_percent(&self, tank_type: TankType) -> f32 {
        self.get_stat_value(tank_type, |p| p.speed) as f32 / 100.0
    }

    /// 获取玩家的炮弹数量
    pub fn get_shells(&self, tank_type: TankType) -> usize {
        self.get_stat_value(tank_type, |p| p.shells)
    }

    /// 检查玩家是否有火焰炮弹能力
    pub fn has_fire_shell(&self, tank_type: TankType) -> bool {
        self.get_stats(tank_type).map_or(false, |p| p.fire_shell)
    }

    /// 检查玩家是否有穿透能力
    pub fn has_penetrate(&self, tank_type: TankType) -> bool {
        self.get_stats(tank_type).map_or(false, |p| p.penetrate)
    }

    /// 检查玩家是否有气垫能力
    pub fn has_air_cushion(&self, tank_type: TankType) -> bool {
        self.get_stats(tank_type).map_or(false, |p| p.air_cushion)
    }

    /// 恢复所有玩家 1 点能量
    pub fn recover_all_energy(&mut self) {
        self.player1.recover_energy();
        if let Some(ref mut p2) = self.player2 {
            p2.recover_energy();
        }
    }

    /// 检查是否有任何玩家需要恢复能量
    pub fn needs_energy_regen(&self) -> bool {
        self.player1.energy_points < crate::constants::MAX_ENERGY_POINTS
            || self.player2.as_ref().is_some_and(|p| p.energy_points < crate::constants::MAX_ENERGY_POINTS)
    }

    /// 对指定玩家的统计数据执行操作
    ///
    /// 如果玩家存在，则对玩家的统计数据执行给定的闭包操作
    ///
    /// # 参数
    /// - `tank_type`: 玩家类型
    /// - `f`: 对 PlayerStats 执行操作的闭包
    pub fn with_stats_mut<F>(&mut self, tank_type: TankType, f: F)
    where
        F: FnOnce(&mut PlayerStats),
    {
        if let Some(stats) = self.get_stats_mut(tank_type) {
            f(stats);
        }
    }
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

impl PlayerStats {
    /// 创建默认玩家统计
    pub fn new_default() -> Self {
        use crate::constants::*;
        Self {
            speed: INITIAL_ATTRIBUTE_VALUE,
            fire_speed: INITIAL_ATTRIBUTE_VALUE,
            protection: INITIAL_ATTRIBUTE_VALUE,
            shells: 1,
            penetrate: false,
            track_chain: false,
            air_cushion: false,
            fire_shell: false,
            life_points: MAX_LIFE_POINTS,
            energy_points: MAX_ENERGY_POINTS,
            score: 0,
        }
    }

    /// 恢复 1 点能量
    pub fn recover_energy(&mut self) {
        use crate::constants::*;
        if self.energy_points < MAX_ENERGY_POINTS {
            self.energy_points = (self.energy_points + 1).min(MAX_ENERGY_POINTS);
        }
    }
}

// 玩家回城计时器
#[derive(Resource, Default)]
pub struct RecallTimers {
    pub timers: EntityHashMap<RecallTimer>,
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
    pub timers: EntityHashMap<DashTimer>,
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
    pub cooldowns: EntityHashMap<Timer>, // 记录每个玩家坦克的受伤冷却计时器
}

// Dash 扣血追踪器，防止一次 dash 多次扣血
#[derive(Resource, Default)]
pub struct DashDamageTracker {
    pub has_taken_damage: EntityHashSet, // 记录本次 dash 已经扣血的玩家坦克
}

// 能量不足提示冷却追踪器，防止重复触发提示
// 优化：使用固定数组替代 EntityHashMap，因为玩家数量固定（最多2个）
#[derive(Resource, Default)]
pub struct InsufficientEnergyTracker {
    pub p1_cooldown: Option<Timer>, // 玩家1的冷却计时器
    pub p2_cooldown: Option<Timer>, // 玩家2的冷却计时器
}

impl InsufficientEnergyTracker {
    /// 更新所有冷却计时器
    pub fn tick_all(&mut self, delta: Duration) {
        if let Some(ref mut timer) = self.p1_cooldown {
            timer.tick(delta);
        }
        if let Some(ref mut timer) = self.p2_cooldown {
            timer.tick(delta);
        }
    }

    /// 检查并触发能量不足提示
    /// 返回 true 如果成功触发提示
    pub fn try_show_warning(
        &mut self,
        commands: &mut Commands,
        tank_type: TankType,
        font_cn: Handle<Font>,
        font_en: Handle<Font>,
        language: Language,
    ) -> bool {
        // 根据玩家类型选择对应的冷却计时器
        let cooldown = match tank_type {
            TankType::Player1 => &mut self.p1_cooldown,
            TankType::Player2 => &mut self.p2_cooldown,
            TankType::Enemy => return false, // 敌方坦克不显示提示
        };

        let can_show_warning = cooldown
            .as_ref()
            .map_or(true, |t| t.is_finished());

        if can_show_warning {
            *cooldown = Some(Timer::from_seconds(
                crate::constants::INSUFFICIENT_ENERGY_DISPLAY_DURATION,
                TimerMode::Once,
            ));

            crate::overlay_ui::spawn_insufficient_energy_warning(
                commands.reborrow(),
                font_cn,
                font_en,
                tank_type,
                language,
            );
            true
        } else {
            false
        }
    }
}

// 地形纹理图集布局资源
#[derive(Resource)]
pub struct TerrainAtlasLayouts {
    pub sea: Handle<TextureAtlasLayout>,
    pub forest: Handle<TextureAtlasLayout>,
    pub forest_fire: Handle<TextureAtlasLayout>,
}

// 背景精灵图纹理图集布局资源
#[derive(Resource)]
pub struct BackgroundAtlasLayout {
    pub layout: Handle<TextureAtlasLayout>,
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
    /// 火焰特效纹理（精灵图，叠加在子弹上）
    pub bullet_fire_effect: Handle<Image>,
    /// 穿透效果纹理（精灵图，叠加在子弹上）
    pub bullet_penetrate_effect: Handle<Image>,
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
}

impl SoundResources {
    /// 播放音效
    pub fn play(
        &self,
        commands: &mut Commands,
        audio_source: Handle<AudioSource>,
        volume: f32,
    ) {
        utils::play_one_shot_sound(commands, audio_source, volume);
    }
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
    pub background: Handle<Image>,
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




