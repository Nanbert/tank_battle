//! Game constants for the Tank Battle game

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use crate::resources::PlayerStats;

// 碰撞分组常量
pub const SEA_GROUP: Group = Group::GROUP_2;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
pub const MAP_WIDTH: f32 = 1600.0; // 地图区域宽度
pub const MAP_HEIGHT: f32 = 1200.0; // 地图区域高度
pub const LEFT_PADDING: f32 = 230.0; // 左侧留白
pub const RIGHT_PADDING: f32 = 230.0; // 右侧留白
pub const TOP_PADDING: f32 = 100.0; // 上方留白
pub const BOTTOM_PADDING: f32 = 0.0; // 下方不留白
pub const TANK_WIDTH: f32 = 87.0;
pub const TANK_HEIGHT: f32 = 87.0;
pub const TANK_SPEED: f32 = 200.0;
pub const PLAYER_TANK_SPEED: f32 = 150.0;
pub const BULLET_SPEED: f32 = 900.0;
pub const PLAYER_BULLET_SPEED: f32 = 600.0;
pub const BULLET_SIZE: f32 = 10.0;
pub const RECALL_TIME: f32 = 4.0; // 回城时间（秒）
pub const VERTICAL_OFFSET: f32 = (BOTTOM_PADDING-TOP_PADDING) / 2.0; // 由于下边不留白，会导致坐标垂直便移-50
pub const WINDOW_WIDTH: f32 = MAP_WIDTH + LEFT_PADDING + RIGHT_PADDING; // 总宽度
pub const WINDOW_HEIGHT: f32 = MAP_HEIGHT + TOP_PADDING + BOTTOM_PADDING; // 总高度
pub const WINDOW_LEFT_X:f32 = -WINDOW_WIDTH / 2.0;
pub const WINDOW_RIGHT_X:f32 = WINDOW_WIDTH / 2.0;
pub const WINDOW_TOP_Y:f32 = WINDOW_HEIGHT / 2.0;
//pub const WINDOW_BOTTOM_Y:f32 = -WINDOW_HEIGHT / 2.0;
pub const MAP_LEFT_X:f32 = -MAP_WIDTH / 2.0;
pub const MAP_RIGHT_X:f32 = MAP_WIDTH / 2.0;
pub const MAP_TOP_Y:f32 = MAP_HEIGHT / 2.0 + VERTICAL_OFFSET;
pub const MAP_BOTTOM_Y:f32 = -MAP_HEIGHT / 2.0 + VERTICAL_OFFSET;


pub const ENEMY_BORN_PLACES: [Vec3; 3] = [
    Vec3::new(MAP_LEFT_X + TANK_WIDTH / 2.0, MAP_TOP_Y - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(0.0, MAP_TOP_Y - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(MAP_RIGHT_X - TANK_WIDTH / 2.0, MAP_TOP_Y - TANK_HEIGHT / 2.0, 0.0),
];

pub const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.5, 0.5); // 蓝绿色
pub const START_SCREEN_BACKGROUND_COLOR: Color = Color::srgb(17.0/255.0, 81.0/255.0, 170.0/255.0);

pub const COMMANDER_WIDTH: f32 = 100.0;
pub const COMMANDER_HEIGHT: f32 = 100.0;

pub const DIRECTIONS: [Vec2; 4] = [
    Vec2::new(0.0, 1.0),   // 上
    Vec2::new(0.0, -1.0),  // 下
    Vec2::new(-1.0, 0.0),  // 左
    Vec2::new(1.0, 0.0),   // 右
];

// 冲刺相关常量
pub const DASH_DURATION: f32 = 0.2; // 冲刺持续时间（秒）
pub const DASH_DISTANCE: f32 = TANK_HEIGHT * 2.0; // 冲刺距离（两个坦克长度）

// 关卡俏皮话，根据关卡序号选择（索引从0开始）
pub const STAGE_QUOTES: [&str; 17] = [
    "The brave commander will not retreat even when hit. He will stand firm in place,\nwaiting for his soldiers to rescue him.",
    "When you fire while turning, your bullet might not go straight!\nThough controlling bullet trajectory is quite difficult.",
    "Beware, enemy tanks aren't fools either - they can shoot diagonally too.",
    "Both enemy and our factories have constrained our tanks to fire straight,\nincreasing hit rate. After all, a shell is expensive.",
    "All tanks move in straight lines, not like crabs. This is to commemorate\nthe countless tanks sacrificed in the tank battles of the last century.",
    "The cunning enemy tanks have upgraded their shells,\nmaking our shells unable to intercept theirs. This is truly frustrating.",
    "When all your stats reach max, please share the power-ups with your teammates,\nyou greedy person.",
    "Our power-ups have been sprayed with invisible paint - only you can see them,\nthe enemy cannot, even if they're right next to them.",
    "It is said that in the tank battles of the last century, there was a period\nwhen enemies could also pick up our power-ups. That was truly a disaster.",
    "Our shells have been specially treated - when they encounter the commander,\nthey pass through without harming him. It is said this was strongly requested\nby the commander, because in the tank battles of the last century,\ncountless commanders died at the hands of their own troops. Truly pitiful.",
    "Our commander has investigated the enemy numbers in advance -\nthere are at most a few hundred enemies. The war will have an end.",
    "In the tank battles of the last century, the enemies seemed endless,\nand no one lived to see the end of the war.",
    "In the tank battles of the last century, the super bomb power-up would destroy\nmany enemy tanks, yet the destroyed tanks would not count towards your\nbattle record. This is truly strange.",
    "When you're alone, you can request an extra tank from the commander.\nYou can tell the commander that the extra tank can help block some shells for you.",
    "When dashing, you must strike from the front or side. When attacking from behind,\nyou're moving in the same direction, so the impact force may not be enough.",
    "When dashing, if there are obstacles or enemies, make sure to maintain a certain distance\nto more successfully trigger the dash destruction effect.",
    "Enemies destroyed by laser do not count towards your score.\nThe commander's reason is that laser damages the flowers and grass.\nThis is truly ridiculous.",
];

/// UI 元素类型枚举
#[derive(Clone, Copy)]
pub enum UIElementType {
    NormalText(fn(&PlayerStats) -> String),
    PlayerAvatar,
    HealthBar, 
    BlueBar
}

/// UI 元素配置
#[derive(Clone)]
pub struct UIElementConfig {
    pub element_type: UIElementType,
    pub x_pos: f32,
    pub y_pos: f32,
    pub font_size: f32,
}

/// 玩家1 UI 元素配置
pub const PLAYER1_UI_ELEMENTS: &[UIElementConfig] = &[
    // 玩家1名称
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {info.name.clone()}),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 780.0,
        font_size: 32.0,
    },
    // Speed
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            if info.speed >= 100 {
                "Speed:MAX".to_string()
            } else {
                format!("Speed:{}%", info.speed)
            }
        }),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 830.0,
        font_size: 24.0,
    },
    // Fire Speed
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            if info.fire_speed >= 100 {
                "Fire Speed:MAX".to_string()
            } else {
                format!("Fire Speed:{}%", info.fire_speed)
            }
        }),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 880.0,
        font_size: 24.0,
    },
    // Protection
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            if info.protection >= 100 {
                "Protection:MAX".to_string()
            } else {
                format!("Protection:{}%", info.protection)
            }
        }),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 930.0,
        font_size: 24.0,
    },
    // Shells
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Shells: {}", info.shells)),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 980.0,
        font_size: 24.0,
    },
    // Penetrate
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Penetrate: {}", if info.penetrate { "On" } else { "Off" })),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 420.0,
        font_size: 24.0,
    },
    // Track Chain
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Track Chain:{}", if info.track_chain { "On" } else { "Off" })),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 470.0,
        font_size: 24.0,
    },
    // Air Cushion
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Air Cushion:{}", if info.air_cushion { "On" } else { "Off" })),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 520.0,
        font_size: 24.0,
    },
    // Fire Shell
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Fire Shell:{}", if info.fire_shell { "On" } else { "Off" })),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 370.0,
        font_size: 24.0,
    },
    // Effects
    UIElementConfig {
        element_type: UIElementType::NormalText(|_| {"Effects".to_string()}),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 320.0,
        font_size: 32.0,
    },
    // 玩家1分数
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Scores1: {}", info.score)),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 50.0,
        font_size: 28.0,
    },
    // 玩家1头像
    UIElementConfig {
        element_type: UIElementType::PlayerAvatar,
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 150.0,
        font_size: 0.0,
    },
    // 玩家1血条
    UIElementConfig {
        element_type: UIElementType::HealthBar,
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 235.0,
        font_size: 0.0,
    },
    // 玩家1蓝条
    UIElementConfig {
        element_type: UIElementType::BlueBar,
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 250.0,
        font_size: 0.0,
    },
];

/// 玩家2 UI 元素配置（与玩家1相同，但位置在右侧）
pub const PLAYER2_UI_ELEMENTS: &[UIElementConfig] = &[
    // 玩家2名称
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {info.name.clone()}),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 780.0,
        font_size: 32.0,
    },
    // Speed
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            if info.speed >= 100 {
                "Speed:MAX".to_string()
            } else {
                format!("Speed:{}%", info.speed)
            }
        }),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 830.0,
        font_size: 24.0,
    },
    // Fire Speed
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            if info.fire_speed >= 100 {
                "Fire Speed:MAX".to_string()
            } else {
                format!("Fire Speed:{}%", info.fire_speed)
            }
        }),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 880.0,
        font_size: 24.0,
    },
    // Protection
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            if info.protection >= 100 {
                "Protection:MAX".to_string()
            } else {
                format!("Protection:{}%", info.protection)
            }
        }),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 930.0,
        font_size: 24.0,
    },
    // Shells
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Shells: {}", info.shells)),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 980.0,
        font_size: 24.0,
    },
    // Penetrate
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Penetrate: {}", if info.penetrate { "On" } else { "Off" })),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 420.0,
        font_size: 24.0,
    },
    // Track Chain
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Track Chain:{}", if info.track_chain { "On" } else { "Off" })),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 470.0,
        font_size: 24.0,
    },
    // Air Cushion
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Air Cushion:{}", if info.air_cushion { "On" } else { "Off" })),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 520.0,
        font_size: 24.0,
    },
    // Fire Shell
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Fire Shell:{}", if info.fire_shell { "On" } else { "Off" })),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 370.0,
        font_size: 24.0,
    },
    // Effects
    UIElementConfig {
        element_type: UIElementType::NormalText(|_| {"Effects".to_string()}),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 320.0,
        font_size: 32.0,
    },
    // 玩家2分数
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| format!("Scores2: {}", info.score)),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 50.0,
        font_size: 28.0,
    },
    // 玩家2头像
    UIElementConfig {
        element_type: UIElementType::PlayerAvatar,
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 150.0,
        font_size: 0.0,
    },
    // 玩家2血条
    UIElementConfig {
        element_type: UIElementType::HealthBar,
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 235.0,
        font_size: 0.0,
    },
    // 玩家2蓝条
    UIElementConfig {
        element_type: UIElementType::BlueBar,
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 250.0,
        font_size: 0.0,
    },
];

//Component
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    StartScreen,
    FadingOut,
    StageIntro,
    Playing,
    Paused,
    GameOver,
}

#[derive(Component)]
pub struct StartScreenUI;

#[derive(Component)]
pub struct MenuOption {
    pub index: usize,
}

#[derive(Component)]
pub struct MenuArrow;

#[derive(Component)]
pub struct PauseUI;

#[derive(Component)]
pub struct GameOverUI;

#[derive(Component)]
pub struct StageIntroUI;

#[derive(Component, Copy, Clone)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct CurrentAnimationFrame(pub usize);

/// 待销毁标记
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DespawnMarker;

#[derive(Component, Deref, DerefMut)]
pub struct DirectionChangeTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct CollisionCooldownTimer(pub Timer);

#[derive(Component, Deref, DerefMut)]
pub struct RotationTimer(pub Timer);

#[derive(Component)]
pub struct TargetRotation {
    pub angle: f32,
}

#[derive(Component, Copy, Clone)]
pub struct EnemyTank {
    pub direction: Vec2,
}

#[derive(Component)]
pub struct EnemyBornAnimation;

/// 坦克类型枚举
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum TankType {
    Player1,
    Player2,
    Enemy,
}

#[derive(Component)]
pub struct PlayerTank {
    pub tank_type: TankType, // TankType::Player1 或 TankType::Player2
}

#[derive(Component)]
pub struct PlayerAvatar;

//用来标记，文字，头像等信息属于哪个玩家
#[derive(Component)]
pub struct PlayerUI {
    pub player_type: TankType,
}

#[derive(Component)]
pub struct PlayerDead;

// 玩家正在冲刺标记
#[derive(Component)]
pub struct IsDashing;

// 玩家气垫船特效标记
#[derive(Component)]
pub struct BubbleEffect;

#[derive(Component)]
pub struct Explosion;

#[derive(Component)]
pub struct Laser;

#[derive(Component)]
pub struct Spark;

#[derive(Component)]
pub struct Smoke;

#[derive(Component)]
pub struct GameOverTimer;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Forest;

#[derive(Component)]
pub struct ForestFire;

#[derive(Component)]
pub struct TreeAmbiencePlayer;

#[derive(Component)]
pub struct Sea;

#[derive(Component)]
pub struct SeaAmbiencePlayer;

#[derive(Component)]
pub struct CommanderAmbiencePlayer;

#[derive(Component)]
pub struct CommanderMusicAnimation;

#[derive(Component)]
pub struct Barrier;

pub const BARRIER_WIDTH: f32 = 100.0;
pub const BARRIER_HEIGHT: f32 = 100.0;

#[derive(Component)]
pub struct Brick;

pub const BRICK_WIDTH: f32 = 50.0;
pub const BRICK_HEIGHT: f32 = 50.0;

#[derive(Component)]
pub struct Steel;

pub const STEEL_WIDTH: f32 = 50.0;
pub const STEEL_HEIGHT: f32 = 50.0;

#[derive(Component)]
pub struct Commander;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
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
    pub const fn texture_path(self) -> &'static str {
        match self {
            Self::SpeedUp => "power_up/speed_up.png",
            Self::Protection => "power_up/protection.png",
            Self::FireSpeed => "power_up/fire_speed.png",
            Self::FireShell => "power_up/fire_shell.png",
            Self::TrackChain => "power_up/track_chain.png",
            Self::Penetrate => "power_up/penetrate.png",
            Self::Repair => "power_up/repair.png",
            Self::Hamburger => "power_up/hamburger.png",
            Self::AirCushion => "power_up/air_cushion.png",
            Self::Shell => "power_up/shell.png",
        }
    }
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct BlueBar;

#[derive(Component)]
pub struct BlueBarOriginalPosition(pub f32); // 记录蓝条的原始 X 位置

#[derive(Component)]
pub struct HealthBarOriginalPosition(pub f32); // 记录血条的原始 X 位置

#[derive(Component)]
pub struct CommanderHealthBar;

#[derive(Component)]
pub struct CommanderHealthBarOriginalPosition(pub f32); // 记录 Commander 血条的原始 X 位置

#[derive(Component)]
pub struct EnemyCountText;

// 标记游戏过程中所有的Entity
#[derive(Component)]
pub struct PlayingEntity;

/// 后坐力组件
#[derive(Component)]
pub struct RecoilForce {
    pub original_pos: Vec3,  // 原始位置
    pub target_offset: Vec2, // 目标偏移量
    pub timer: Timer,        // 后坐力持续时间
}

/// 激光蓄力组件
#[derive(Component)]
pub struct LaserCharge {
    pub timer: Timer,  // 蓄力计时器
    pub tank_type: TankType,  // 坦克类型
}

/// 激光蓄力进度条组件
#[derive(Component)]
pub struct LaserChargeProgressBar {
    pub tank_type: TankType,
    pub player_entity: Entity,
}

/// 激光蓄力音效组件
#[derive(Component)]
pub struct LaserChargeSound;

#[derive(Component, Deref, DerefMut)]
pub struct PlayerInfoBlinkTimer(pub Timer);

#[derive(Resource, Deref, DerefMut)]
pub struct PlayerRespawnTimer(pub Timer);

/// 坦克射击配置
#[derive(Component)]
pub struct TankFireConfig {
    pub max_bullets: usize,  // 最大同时子弹数
    pub cooldown: Timer,     // 射击冷却时间
}

impl Default for TankFireConfig {
    fn default() -> Self {
        Self {
            max_bullets: 1,
            cooldown: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}
