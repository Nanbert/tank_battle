//! UI 常量模块
//!
//! 包含所有 UI 相关的常量和标记组件

use bevy::prelude::*;

// ============================================================================
// HUD Marker 组件
// ============================================================================

/// 玩家1 HUD 容器标记
#[derive(Component, Clone)]
pub struct Player1Hud;

/// 玩家2 HUD 容器标记
#[derive(Component, Clone)]
pub struct Player2Hud;

/// 玩家名称文本标记
#[derive(Component)]
pub struct PlayerNameText;

/// 玩家头像标记
#[derive(Component, Clone)]
pub struct PlayerAvatar;

/// 效果标题标记
#[derive(Component, Clone)]
pub struct EffectsTitle;

// ============================================================================
// Stat Text Markers
// ============================================================================

/// 速度文本标记
#[derive(Component, Clone)]
pub struct SpeedText;

/// 射速文本标记
#[derive(Component, Clone)]
pub struct FireSpeedText;

/// 护盾文本标记
#[derive(Component, Clone)]
pub struct ProtectionText;

/// 炮弹数量文本标记
#[derive(Component, Clone)]
pub struct ShellsText;

/// 分数文本标记
#[derive(Component, Clone)]
pub struct ScoreText;

// ============================================================================
// Effect Text Markers
// ============================================================================

/// 穿透效果文本标记
#[derive(Component, Clone)]
pub struct PenetrateText;

/// 履带链效果文本标记
#[derive(Component, Clone)]
pub struct TrackChainText;

/// 气垫效果文本标记
#[derive(Component, Clone)]
pub struct AirCushionText;

/// 火焰炮弹效果文本标记
#[derive(Component, Clone)]
pub struct FireShellText;

// ============================================================================
// Bar Markers
// ============================================================================

/// 血条标记
#[derive(Component, Clone)]
pub struct HealthBar;

/// 血条前景标记
#[derive(Component, Clone)]
pub struct HealthBarForeground;

/// 蓝条标记
#[derive(Component, Clone)]
pub struct BlueBar;

/// 蓝条前景标记
#[derive(Component, Clone)]
pub struct BlueBarForeground;

// ============================================================================
// Top HUD Markers
// ============================================================================

/// 关卡文本标记
#[derive(Component)]
pub struct StageText;

/// 司令官文本标记
#[derive(Component)]
pub struct CommanderText;

/// 司令官血条标记
#[derive(Component)]
pub struct CommanderHealthBar;

/// 司令官血条原始位置（用于更新血条宽度）
#[derive(Component)]
pub struct CommanderHealthBarOriginalPosition(pub f32);

/// 敌方剩余数量文本标记
#[derive(Component)]
pub struct EnemyCountText;

// ============================================================================
// Animation Markers
// ============================================================================

/// 通用 UI 计时器
///
/// 用于 UI 元素的计时效果，如闪烁、淡入淡出等
#[derive(Component, Deref, DerefMut)]
pub struct UiTimer(pub Timer);

impl UiTimer {
    /// 创建新的 UI 计时器
    pub fn new(duration: f32, mode: TimerMode) -> Self {
        Self(Timer::from_seconds(duration, mode))
    }
}

/// HUD 文本闪烁计时器（类型别名）
pub type PlayerInfoBlinkTimer = UiTimer;

/// 能量不足提示文本组件
#[derive(Component)]
pub struct InsufficientEnergyText;

// ============================================================================
// Screen Markers
// ============================================================================

/// 开始界面 UI 标记
#[derive(Component)]
pub struct StartScreenUI;

/// 菜单选项组件
#[derive(Component)]
pub struct MenuOption {
    pub index: usize,
}

/// 暂停界面 UI 标记
#[derive(Component)]
pub struct PauseUI;

/// 游戏结束界面 UI 标记
#[derive(Component)]
pub struct GameOverUI;

/// 关卡介绍界面 UI 标记
#[derive(Component)]
pub struct StageIntroUI;

/// 关于界面 UI 标记
#[derive(Component)]
pub struct AboutUI;

/// 致谢界面 UI 标记
#[derive(Component)]
pub struct CreditsUI;

/// 标记游戏过程中所有的 Entity
#[derive(Component)]
pub struct PlayingEntity;

// ============================================================================
// HUD 布局常量
// ============================================================================

/// HUD 血条尺寸
pub const HUD_BAR_SIZE: Vec2 = Vec2::new(150.0, 15.0);

/// 司令官血条尺寸
pub const COMMANDER_BAR_SIZE: Vec2 = Vec2::new(160.0, 15.0);

/// HUD Y 坐标数组
#[derive(Clone, Copy)]
pub enum HudYPosition {
    Name,
    Speed,
    FireSpeed,
    Protection,
    Shells,
    EffectsTitle,
    FireShell,
    Penetrate,
    TrackChain,
    AirCushion,
    InsufficientEnergy,
    Score,
    Avatar,
    BarHealth,
    BarBlue,
}

impl HudYPosition {
    /// 获取对应的 Y 坐标值
    pub const fn offset(self) -> f32 {
        match self {
            Self::Name => 780.0,
            Self::Speed => 830.0,
            Self::FireSpeed => 880.0,
            Self::Protection => 930.0,
            Self::Shells => 980.0,
            Self::EffectsTitle => 320.0,
            Self::FireShell => 370.0,
            Self::Penetrate => 420.0,
            Self::TrackChain => 470.0,
            Self::AirCushion => 520.0,
            Self::InsufficientEnergy => 610.0,
            Self::Score => 50.0,
            Self::Avatar => 150.0,
            Self::BarHealth => 235.0,
            Self::BarBlue => 250.0,
        }
    }
}

/// HUD 数值常量
pub const HUD_MAX_PERCENT: usize = 100; // 最大百分比
pub const HUD_MAX_LIFE_POINTS: f32 = 3.0; // 最大生命值
pub const HUD_MAX_SHELLS: usize = 2; // 最大炮弹数

/// HUD 布局位置常量
pub const HUD_PLAYER_OFFSET: f32 = 115.0; // 玩家 HUD X 坐标偏移量
pub const HUD_COMMANDER_TEXT_OFFSET: f32 = 42.0; // 司令官血条文本偏移量
pub const HUD_COMMANDER_BAR_OFFSET: f32 = 172.0; // 司令官血条偏移量
pub const HUD_ENEMY_COUNT_OFFSET: f32 = 465.0; // 敌方剩余数量文本偏移量
pub const HUD_COMMANDER_TEXT_X: f32 = 435.0; // 司令官血条文本 X 坐标
pub const HUD_COMMANDER_MAX_LIFE: f32 = 3.0; // 司令官最大生命值

// ============================================================================
// UI 字体大小常量
// ============================================================================

pub const FONT_SIZE_SMALL: f32 = 18.0; // 小字体
pub const FONT_SIZE_MEDIUM: f32 = 22.0; // 中等字体
pub const FONT_SIZE_INSTRUCTION: f32 = 24.0; // 说明文字字体
pub const FONT_SIZE_SCORE: f32 = 28.0; // 分数字体
pub const FONT_SIZE_UI: f32 = 30.0; // UI字体
pub const FONT_SIZE_HUD_NAME: f32 = 32.0; // HUD玩家名称字体大小
pub const FONT_SIZE_OPTION: f32 = 50.0; // 选项字体
pub const FONT_SIZE_TITLE: f32 = 60.0; // 标题字体
pub const FONT_SIZE_CREDITS_TITLE: f32 = 70.0; // 标题字体
pub const FONT_SIZE_MENU: f32 = 80.0; // 菜单字体
pub const FONT_SIZE_GAME_OVER: f32 = 100.0; // 大标题字体
pub const FONT_SIZE_INSUFFICIENT_ENERGY: f32 = 24.0; // 能量不足提示字体大小（与HUD字体一致）

// ============================================================================
// UI 时间常量
// ============================================================================

/// 菜单闪烁周期
pub const MENU_BLINK_PERIOD: f32 = 0.5;

/// 文字闪烁周期
pub const TEXT_BLINK_CYCLE: f32 = 0.6;

/// Game Over 延迟
pub const GAME_OVER_DELAY: f32 = 1.2;

// ============================================================================
// Z 轴层级常量
// ============================================================================

pub const Z_UI: f32 = 10.0; // UI层级
pub const Z_STAGE_INTRO_BG: f32 = 100.0; // 关卡介绍层级
pub const Z_STAGE_INTRO_TEXT: f32 = 101.0; // 关卡介绍文字层级

// ============================================================================
// 从 constants.rs 迁移的 UI 相关常量
// ============================================================================

// 颜色常量
pub const COLOR_BACKGROUND: Color = Color::srgb(0.0, 0.5, 0.5); // 蓝绿色
pub const COLOR_BLACK: Color = Color::srgb(0.0, 0.0, 0.0); // 黑色
pub const COLOR_WHITE: Color = Color::srgb(1.0, 1.0, 1.0); // 白色
pub const COLOR_RED: Color = Color::srgb(1.0, 0.0, 0.0); // 红色
pub const COLOR_GREEN: Color = Color::srgb(0.0, 1.0, 0.0); // 绿色
pub const COLOR_BLUE: Color = Color::srgb(0.0, 0.5, 1.0); // 蓝色
pub const COLOR_YELLOW: Color = Color::srgb(1.0, 1.0, 0.0); // 黄色
pub const COLOR_GRAY: Color = Color::srgb(146.0 / 255.0, 159.0 / 255.0, 167.0 / 255.0); // 开始界面背景色
pub const COLOR_DARK_GRAY: Color = Color::srgb(0.3, 0.3, 0.3); // 血条空槽背景色（深灰）
pub const COLOR_TRANSPARENT: Color = Color::srgba(1.0, 1.0, 1.0, 0.0); // 透明白色
pub const COLOR_TRANSPARENT_BLACK: Color = Color::srgba(0.0, 0.0, 0.0, 0.0); // 透明黑色
pub const COLOR_GOLD: Color = Color::srgb(1.0, 0.84, 0.0); // 金色

// UI 时间常量（关卡相关）
pub const STAGE_FADE_IN_DURATION: f32 = 1.0; // 关卡淡入时间
pub const STAGE_FADE_HOLD_DURATION: f32 = 1.0; // 关卡停留时间
pub const STAGE_FADE_OUT_DURATION: f32 = 1.0; // 关卡淡出时间

// 其他 UI 常量
pub const PAYMENT_CODE_SIZE: f32 = 400.0; // 收款码尺寸