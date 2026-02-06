//! Game resources for the Tank Battle game

use bevy::ecs::entity::{EntityHashMap, EntityHashSet};
use bevy::prelude::*;
use std::time::Duration;

use crate::constants::{
    BLUE_BAR_REGEN_INTERVAL, ENEMIES_PER_LEVEL, ENEMY_SPAWN_COOLDOWN, TankType,
};

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

    /// 添加子弹，确保数据一致性
    ///
    /// # Panics
    /// 如果子弹已经存在于追踪器中（重复添加）
    pub fn add_bullet(&mut self, bullet: Entity, tank: Entity) {
        // 防御性检查：确保子弹未被重复添加
        if self.bullet_to_tank.contains_key(&bullet) {
            panic!("Bullet entity {:?} already exists in tracker", bullet);
        }

        *self.active_bullets.entry(tank).or_insert(0) += 1;
        self.bullet_to_tank.insert(bullet, tank);
    }

    /// 移除子弹，返回所属坦克
    ///
    /// 确保两个映射表保持同步，即使 bullet_to_tank 中没有对应的条目
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

    /// 清理所有子弹追踪数据
    ///
    /// 用于关卡重置时清空追踪器
    pub fn clear(&mut self) {
        self.active_bullets.clear();
        self.bullet_to_tank.clear();
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
    pub has_spawned: usize,          // 已生成数量
    pub max_count: usize,            // 总数量（每关固定20个）
    pub spawn_cooldown: Timer,       // 生成冷却时间
    pub stage_complete_delay: Timer, // 关卡完成延迟计时器（2秒后进入下一关）
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
        self.get_stats(tank_type).is_some_and(|p| p.fire_shell)
    }

    /// 检查玩家是否有穿透能力
    pub fn has_penetrate(&self, tank_type: TankType) -> bool {
        self.get_stats(tank_type).is_some_and(|p| p.penetrate)
    }

    /// 检查玩家是否有气垫能力
    pub fn has_air_cushion(&self, tank_type: TankType) -> bool {
        self.get_stats(tank_type).is_some_and(|p| p.air_cushion)
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
            || self
                .player2
                .as_ref()
                .is_some_and(|p| p.energy_points < crate::constants::MAX_ENERGY_POINTS)
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

#[derive(Clone, Default, PartialEq)]
pub struct PlayerStats {
    pub speed: usize,
    pub fire_speed: usize,
    pub protection: usize,
    pub shells: usize,
    pub penetrate: bool,
    pub track_chain: bool,
    pub air_cushion: bool,
    pub fire_shell: bool,
    pub life_points: usize,   // max 3
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

        let can_show_warning = cooldown.as_ref().is_none_or(|t| t.is_finished());

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

// ==================== 游戏资源管理 ====================

/// 统一的纹理资源结构体
/// 合并所有游戏纹理资源，减少资源结构体数量
#[derive(Resource)]
pub struct GameTextureResources {
    // 字体
    pub cn: Handle<Font>,
    pub en: Handle<Font>,
    // 玩家坦克
    pub player1: Handle<Image>,
    pub player2: Handle<Image>,
    pub single_barrel: Handle<Image>,
    pub double_barrel: Handle<Image>,
    // 司令官
    pub commander: Handle<Image>,
    pub commander_dead: Handle<Image>,
    pub avatar: Handle<Image>,
    pub avatar_death: Handle<Image>,
    pub avatar_commander_dead: Handle<Image>,
    // 子弹
    pub bullet_player1: Handle<Image>,
    pub bullet_player2: Handle<Image>,
    pub bullet_enemy: Handle<Image>,
    pub bullet_fire_effect: Handle<Image>,
    pub bullet_penetrate_effect: Handle<Image>,
    // 特效
    pub explosion: Handle<Image>,
    pub spark: Handle<Image>,
    pub smoke: Handle<Image>,
    pub bubble: Handle<Image>,
    pub energy_blue_ball: Handle<Image>,
    pub energy_red_ball: Handle<Image>,
    pub forest_fire: Handle<Image>,
    pub laser_blue: Handle<Image>,
    pub laser_red: Handle<Image>,
    // 地图
    pub brick: Handle<Image>,
    pub steel: Handle<Image>,
    pub tree: Handle<Image>,
    pub sea: Handle<Image>,
    pub barrier: Handle<Image>,
    // 敌方坦克
    pub enemy_born: Handle<Image>,
    pub enemy_tank: Handle<Image>,
    // 道具
    pub speed_up_icon: Handle<Image>,
    pub protection_icon: Handle<Image>,
    pub fire_speed_icon: Handle<Image>,
    pub fire_shell_icon: Handle<Image>,
    pub track_chain_icon: Handle<Image>,
    pub track_chain_effect: Handle<Image>,
    pub penetrate_icon: Handle<Image>,
    pub repair_icon: Handle<Image>,
    pub hamburger_icon: Handle<Image>,
    pub air_cushion_icon: Handle<Image>,
    pub shell_icon: Handle<Image>,
    // 菜单
    pub background: Handle<Image>,
    pub music_note: Handle<Image>,
}

impl GameTextureResources {
    /// 根据语言获取对应的字体
    pub fn get_font(&self, language: Language) -> Handle<Font> {
        match language {
            Language::Chinese => self.cn.clone(),
            Language::English => self.en.clone(),
        }
    }
}

/// 统一的音频资源结构体
/// 合并所有游戏音频资源，减少资源结构体数量
#[derive(Resource)]
pub struct GameAudioResources {
    // 音效
    pub explosion: Handle<AudioSource>,
    pub brick_hit: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub metal_crash: Handle<AudioSource>,
    pub laser_charge: Handle<AudioSource>,
    pub laser: Handle<AudioSource>,
    pub powerup_sound: Handle<AudioSource>,
    pub commander_get_shot: Handle<AudioSource>,
    pub commander_death: Handle<AudioSource>,
    pub player_shot: Handle<AudioSource>,
    // 环境音效
    pub burn_tree: Handle<AudioSource>,
    pub sea_ambience: Handle<AudioSource>,
    pub music_note_000: Handle<AudioSource>,
    pub music_note_001: Handle<AudioSource>,
    pub music_note_002: Handle<AudioSource>,
    pub music_note_003: Handle<AudioSource>,
    pub tree_ambience: Handle<AudioSource>,
}

impl GameAudioResources {
    // 注意：不再提供 play 方法，直接使用 utils::play_one_shot_sound
}

/// 统一的图集布局资源结构体
/// 合并所有纹理图集布局资源，减少资源结构体数量
#[derive(Resource)]
pub struct GameAtlasLayoutResources {
    // 地形
    pub sea: Handle<TextureAtlasLayout>,
    pub forest: Handle<TextureAtlasLayout>,
    pub forest_fire: Handle<TextureAtlasLayout>,
    // 背景
    pub background: Handle<TextureAtlasLayout>,
    // 子弹特效
    pub fire_effect: Handle<TextureAtlasLayout>,
    pub penetrate_effect: Handle<TextureAtlasLayout>,
    // 烟雾特效
    pub smoke_atlas: Handle<TextureAtlasLayout>,
    // 爆炸特效
    pub explosion: Handle<TextureAtlasLayout>,
    pub spark: Handle<TextureAtlasLayout>,
    // 指挥官
    pub commander: Handle<TextureAtlasLayout>,
    pub music_note: Handle<TextureAtlasLayout>,
    pub player_avatar: Handle<TextureAtlasLayout>,
    // 敌方出生
    pub enemy_born: Handle<TextureAtlasLayout>,
    pub enemy_tank: Handle<TextureAtlasLayout>,
    // 激光
    pub laser_blue: Handle<TextureAtlasLayout>,
    pub laser_red: Handle<TextureAtlasLayout>,
    // 能量球
    pub energy_blue_ball: Handle<TextureAtlasLayout>,
    pub energy_red_ball: Handle<TextureAtlasLayout>,
    // 道具
    pub speed_up_icon: Handle<TextureAtlasLayout>,
    pub protection_icon: Handle<TextureAtlasLayout>,
    pub fire_speed_icon: Handle<TextureAtlasLayout>,
    pub fire_shell_icon: Handle<TextureAtlasLayout>,
    pub track_chain_icon: Handle<TextureAtlasLayout>,
    pub track_chain_effect: Handle<TextureAtlasLayout>,
    pub penetrate_icon: Handle<TextureAtlasLayout>,
    pub repair_icon: Handle<TextureAtlasLayout>,
    pub hamburger_icon: Handle<TextureAtlasLayout>,
    pub air_cushion_icon: Handle<TextureAtlasLayout>,
    pub shell_icon: Handle<TextureAtlasLayout>,
}
