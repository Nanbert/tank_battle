//! Game constants for the Tank Battle game

use crate::resources::PlayerStats;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// 字体路径常量
pub const FONT_CN: &str = "font/LiuHuanKaTongShouShu1.5-2.ttf";
pub const FONT_EN: &str = "font/ChelaOne-Regular-2.ttf";

// 音频文件路径常量
pub const SOUND_BRICK_HIT: &str = "music/brick_hit.ogg";
pub const SOUND_BURN_TREE: &str = "music/burn_tree.ogg";
pub const SOUND_COMMANDER_DEATH: &str = "music/commander_death.ogg";
pub const SOUND_COMMANDER_GET_SHOT: &str = "music/commander_get_shot.ogg";
pub const SOUND_COMMANDER_MUSIC_000: &str = "music/commander_music_000.ogg";
pub const SOUND_COMMANDER_MUSIC_001: &str = "music/commander_music_001.ogg";
pub const SOUND_COMMANDER_MUSIC_002: &str = "music/commander_music_002.ogg";
pub const SOUND_COMMANDER_MUSIC_003: &str = "music/commander_music_003.ogg";
pub const SOUND_EXPLOSION: &str = "music/explosion_sound.ogg";
pub const SOUND_HIT: &str = "music/hit_sound.ogg";
pub const SOUND_LASER_CHARGE: &str = "music/laser_charge.ogg";
pub const SOUND_LASER: &str = "music/laser.ogg";
pub const SOUND_METAL_CRASH: &str = "music/metal_crash.ogg";
pub const SOUND_POWERUP: &str = "music/powerup_sound.ogg";
pub const SOUND_SEA_AMBIENCE: &str = "music/sea_ambience.ogg";
pub const SOUND_TREE_AMBIENCE: &str = "music/tree_ambience.ogg";

// 地图纹理路径常量
pub const TEXTURE_BARRIER: &str = "maps/barrier.png";
pub const TEXTURE_BRICK: &str = "maps/brick.png";
pub const TEXTURE_SEA: &str = "maps/sea_sheet.png";
pub const TEXTURE_STEEL: &str = "maps/steel.png";

// 特效纹理路径常量
pub const TEXTURE_BUBBLE: &str = "effect/BubbleBlue.png";
pub const TEXTURE_ENEMY_BORN: &str = "effect/enemy_born.png";
pub const TEXTURE_EXPLOSION: &str = "effect/explosion.png";
pub const TEXTURE_MUSIC_NOTE: &str = "effect/music_note_sheet.png";
pub const TEXTURE_SMOKE: &str = "effect/smoke_sprite.png";
pub const TEXTURE_STEEL_HIT: &str = "effect/steel_hit.png";
pub const TEXTURE_LASER_BLUE: &str = "effect/texture_laser_blue.png";
pub const TEXTURE_LASER_RED: &str = "effect/texture_laser_red.png";

// 角色纹理路径常量
pub const TEXTURE_COMMANDER: &str = "texture/commander.png";
pub const TEXTURE_COMMANDER_DEAD: &str = "texture/commander_dead.png";
pub const TEXTURE_AVATAR: &str = "texture/avatar.png";
pub const TEXTURE_PLAYER_TANK1: &str = "texture/player_tank1_sprite.png";
pub const TEXTURE_PLAYER_TANK2: &str = "texture/player_tank2_sprite.png";
pub const TEXTURE_AVATAR_DEATH: &str = "texture/avatar_death.png";
pub const TEXTURE_AVATAR_COMMANDER_DEAD: &str = "texture/avatar_commander_dead.png";

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
pub const RECALL_TIME: f32 = 2.0; // 回城时间（秒）
pub const VERTICAL_OFFSET: f32 = (BOTTOM_PADDING - TOP_PADDING) / 2.0; // 由于下边不留白，会导致坐标垂直便移-50
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub const WINDOW_WIDTH: u32 = (MAP_WIDTH + LEFT_PADDING + RIGHT_PADDING) as u32; // 总宽度
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub const WINDOW_HEIGHT: u32 = (MAP_HEIGHT + TOP_PADDING + BOTTOM_PADDING) as u32; // 总高度
pub const WINDOW_LEFT_X: f32 = -(WINDOW_WIDTH as f32) / 2.0;
pub const WINDOW_RIGHT_X: f32 = (WINDOW_WIDTH as f32) / 2.0;
pub const WINDOW_TOP_Y: f32 = (WINDOW_HEIGHT as f32) / 2.0;
//pub const WINDOW_BOTTOM_Y:f32 = -WINDOW_HEIGHT / 2.0;
pub const MAP_LEFT_X: f32 = -MAP_WIDTH / 2.0;
pub const MAP_RIGHT_X: f32 = MAP_WIDTH / 2.0;
pub const MAP_TOP_Y: f32 = MAP_HEIGHT / 2.0 + VERTICAL_OFFSET;
pub const MAP_BOTTOM_Y: f32 = -MAP_HEIGHT / 2.0 + VERTICAL_OFFSET;

pub const ENEMY_BORN_PLACES: [Vec3; 3] = [
    Vec3::new(
        MAP_LEFT_X + TANK_WIDTH / 2.0,
        MAP_TOP_Y - TANK_HEIGHT / 2.0,
        0.0,
    ),
    Vec3::new(0.0, MAP_TOP_Y - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(
        MAP_RIGHT_X - TANK_WIDTH / 2.0,
        MAP_TOP_Y - TANK_HEIGHT / 2.0,
        0.0,
    ),
];

pub const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.5, 0.5); // 蓝绿色

pub const COMMANDER_WIDTH: f32 = 100.0;
pub const COMMANDER_HEIGHT: f32 = 100.0;

pub const DIRECTIONS: [Vec2; 4] = [
    Vec2::new(0.0, 1.0),  // 上
    Vec2::new(0.0, -1.0), // 下
    Vec2::new(-1.0, 0.0), // 左
    Vec2::new(1.0, 0.0),  // 右
];

// 冲刺相关常量
pub const DASH_DURATION: f32 = 0.2; // 冲刺持续时间（秒）
pub const DASH_DISTANCE: f32 = TANK_HEIGHT * 2.0; // 冲刺距离（两个坦克长度）

// 关卡俏皮话中文版
pub const STAGE_QUOTES_CN: [&str; 17] = [
    "勇敢的司令官即使被击中也不会撤退。他会坚守原地，\n等待士兵们来营救他。",
    "当你转身射击时，炮弹可能不会直行！\n虽然控制炮弹轨迹确实很困难。",
    "小心，敌方坦克也不是傻瓜——他们也能斜向射击。",
    "敌方和我方工厂都限制了我们的坦克只能直射，\n以提高命中率。毕竟，炮弹很贵。",
    "所有坦克都直线移动，不像螃蟹一样。这是为了纪念\n上个世纪坦克大战中牺牲的无数坦克。",
    "狡猾的敌方坦克升级了他们的炮弹，\n使我们的炮弹无法拦截他们的炮弹。这真令人沮丧。",
    "当你的所有属性都达到最大值时，请与你的队友分享道具，\n你这个贪婪的人。",
    "我们的道具喷了隐形漆——只有你能看到它们，\n敌人看不到，即使它们就在旁边。",
    "据说在上个世纪的坦克大战中，有一段时间\n敌人也能捡起我们的道具。那真是一场灾难。",
    "我们的炮弹经过特殊处理——当它们遇到司令官时，\n会穿过去而不伤害他。据说这是司令官强烈要求的，\n因为在上个世纪的坦克大战中，无数司令官死于自己人之手。真可怜。",
    "我们的司令官已经提前调查了敌人的数量——\n最多只有几百个敌人。战争会有结束的一天。",
    "在上个世纪的坦克大战中，敌人似乎无穷无尽，\n没有人活着看到战争的结束。",
    "在上个世纪的坦克大战中，超级炸弹道具会摧毁\n许多敌方坦克，但被摧毁的坦克不计入你的战斗记录。这真的很奇怪。",
    "当你独自一人时，可以向司令官要求一辆额外的坦克。\n你可以告诉司令官，额外的坦克可以帮你挡一些炮弹。",
    "冲刺时，你必须从正面或侧面攻击。从后面攻击时，\n你朝同一方向移动，所以冲击力可能不够。",
    "冲刺时，如果有障碍物或敌人，请确保保持一定距离，\n以更成功地触发冲刺破坏效果。",
    "被激光摧毁的敌人不计入你的分数。\n司令官的理由是激光会损坏花草树木。这真荒谬。",
];

/// UI 元素类型枚举
#[derive(Clone, Copy)]
pub enum UIElementType {
    NormalText(fn(&PlayerStats) -> String),
    PlayerAvatar,
    HealthBar,
    BlueBar,
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
        element_type: UIElementType::NormalText(|info| info.name.clone()),
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
        element_type: UIElementType::NormalText(|info| {
            format!("Penetrate: {}", if info.penetrate { "On" } else { "Off" })
        }),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 420.0,
        font_size: 24.0,
    },
    // Track Chain
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            format!(
                "Track Chain:{}",
                if info.track_chain { "On" } else { "Off" }
            )
        }),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 470.0,
        font_size: 24.0,
    },
    // Air Cushion
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            format!(
                "Air Cushion:{}",
                if info.air_cushion { "On" } else { "Off" }
            )
        }),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 520.0,
        font_size: 24.0,
    },
    // Fire Shell
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            format!("Fire Shell:{}", if info.fire_shell { "On" } else { "Off" })
        }),
        x_pos: WINDOW_LEFT_X + 115.0,
        y_pos: WINDOW_TOP_Y - 370.0,
        font_size: 24.0,
    },
    // Effects
    UIElementConfig {
        element_type: UIElementType::NormalText(|_| "Effects".to_string()),
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
        element_type: UIElementType::NormalText(|info| info.name.clone()),
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
        element_type: UIElementType::NormalText(|info| {
            format!("Penetrate: {}", if info.penetrate { "On" } else { "Off" })
        }),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 420.0,
        font_size: 24.0,
    },
    // Track Chain
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            format!(
                "Track Chain:{}",
                if info.track_chain { "On" } else { "Off" }
            )
        }),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 470.0,
        font_size: 24.0,
    },
    // Air Cushion
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            format!(
                "Air Cushion:{}",
                if info.air_cushion { "On" } else { "Off" }
            )
        }),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 520.0,
        font_size: 24.0,
    },
    // Fire Shell
    UIElementConfig {
        element_type: UIElementType::NormalText(|info| {
            format!("Fire Shell:{}", if info.fire_shell { "On" } else { "Off" })
        }),
        x_pos: WINDOW_RIGHT_X - 115.0,
        y_pos: WINDOW_TOP_Y - 370.0,
        font_size: 24.0,
    },
    // Effects
    UIElementConfig {
        element_type: UIElementType::NormalText(|_| "Effects".to_string()),
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
    About,
    Credits,
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

#[derive(Component)]
pub struct AboutUI;

#[derive(Component)]
pub struct CreditsUI;

#[derive(Component, Resource, Copy, Clone)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Resource, Deref, DerefMut)]
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
#[derive(Debug)]
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

#[derive(Component)]
pub struct Steel;

// 纹理大小
pub const BRICK_TEXTURE_WIDTH: f32 = 50.0;
pub const BRICK_TEXTURE_HEIGHT: f32 = 50.0;
pub const STEEL_TEXTURE_WIDTH: f32 = 50.0;
pub const STEEL_TEXTURE_HEIGHT: f32 = 50.0;

// 碰撞体积大小
pub const BRICK_COLLIDER_WIDTH: f32 = 46.0;
pub const BRICK_COLLIDER_HEIGHT: f32 = 46.0;
pub const STEEL_COLLIDER_WIDTH: f32 = 46.0;
pub const STEEL_COLLIDER_HEIGHT: f32 = 46.0;

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

/// 出生位置记录组件
#[derive(Component)]
pub struct BornPosition(pub Vec3);

/// 回城进度条组件
#[derive(Component)]
pub struct RecallProgressBar {
    pub player_entity: Entity,
}

/// 玩家正在回城标记
#[derive(Component)]
pub struct IsRecalling;

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
    pub timer: Timer,        // 蓄力计时器
    pub tank_type: TankType, // 坦克类型
}

/// 激光蓄力进度条组件
#[derive(Component)]
pub struct LaserChargeProgressBar {
    pub player_entity: Entity,
}

/// 激光蓄力音效组件
#[derive(Component)]
pub struct LaserChargeSound;

#[derive(Component, Deref, DerefMut)]
pub struct PlayerInfoBlinkTimer(pub Timer);

/// 关卡信息文本标记
#[derive(Component)]
pub struct StageText;

/// 坦克射击配置
#[derive(Component)]
pub struct TankFireConfig {
    pub max_bullets: usize, // 最大同时子弹数
    pub cooldown: Timer,    // 射击冷却时间
}

impl Default for TankFireConfig {
    fn default() -> Self {
        Self {
            max_bullets: 1,
            cooldown: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}
pub const COMMANDER_LIFE_MAX: usize = 3;

// 检测半径常量（用于海、司令官、森林的检测）
pub const DETECTION_RADIUS: f32 = 100.0;

// ==================== 动画时间常量 ====================
pub const ANIMATION_FRAME_EXPLOSION: f32 = 0.01; // 爆炸动画帧间隔
pub const ANIMATION_FRAME_SPARK: f32 = 0.02; // 火花动画帧间隔
pub const ANIMATION_FRAME_LASER: f32 = 0.05; // 激光动画帧间隔
pub const ANIMATION_FRAME_ENEMY_BORN: f32 = 0.1; // 敌方坦克出生动画帧间隔
pub const ANIMATION_FRAME_ENEMY_MOVE: f32 = 0.1; // 敌方坦克移动动画帧间隔
pub const ANIMATION_FRAME_SMOKE: f32 = 0.1; // 烟雾动画帧间隔
pub const ANIMATION_FRAME_COMMANDER_MUSIC: f32 = 0.1; // 指挥官音乐动画帧间隔
pub const ANIMATION_FRAME_START_BACKGROUND: f32 = 0.15; // 开始界面背景动画帧间隔
pub const ANIMATION_FRAME_COMMANDER: f32 = 0.15; // 指挥官动画帧间隔
pub const ANIMATION_FRAME_FOREST: f32 = 0.2; // 森林动画帧间隔
pub const ANIMATION_FRAME_SEA: f32 = 0.2; // 海水动画帧间隔

// ==================== 游戏机制时间常量 ====================
pub const RECOIL_DURATION: f32 = 0.3; // 后坐力持续时间
pub const MENU_BLINK_PERIOD: f32 = 0.5; // 菜单闪烁周期
pub const TEXT_BLINK_CYCLE: f32 = 0.6; // 文字闪烁周期
pub const ENEMY_SPAWN_COOLDOWN: f32 = 0.8; // 敌方坦克生成冷却时间
pub const STAGE_FADE_IN_DURATION: f32 = 1.0; // 关卡淡入时间
pub const STAGE_FADE_HOLD_DURATION: f32 = 1.0; // 关卡停留时间
pub const STAGE_FADE_OUT_DURATION: f32 = 1.0; // 关卡淡出时间
pub const GAME_OVER_DELAY: f32 = 1.2; // Game Over 延迟
pub const FADE_OUT_SPEED: f32 = 1.5; // 淡出速度倒数
pub const FOREST_FIRE_DURATION: f32 = 1.5; // 森林燃烧动画总时长
pub const ENEMY_DIRECTION_CHANGE_INTERVAL: f32 = 2.0; // 敌方坦克方向改变间隔
pub const ENEMY_ROTATION_TIME: f32 = 0.8; // 敌方坦克旋转时间
pub const LASER_CHARGE_TIME: f32 = 4.0; // 激光蓄力时间
pub const BLUE_BAR_REGEN_INTERVAL: f32 = 5.0; // 蓝条恢复间隔

// ==================== 尺寸常量 ====================
// 进度条
pub const PROGRESS_BAR_HEIGHT: f32 = 8.0; // 回城进度条高度
pub const PROGRESS_BAR_Y_OFFSET: f32 = 20.0; // 回城进度条Y偏移
pub const PROGRESS_BAR_INITIAL_WIDTH: f32 = 100.0; // 回城进度条初始宽度

// 墙壁和地形
pub const WALL_POSITION_OFFSET_2: f32 = 5.0; // 墙壁位置偏移
pub const WALL_SCALE: f32 = 10.0; // 墙壁缩放
pub const BRICK_GROUP_OFFSET: f32 = 25.0; // 砖块组偏移
pub const CHARACTER_CONTROLLER_OFFSET: f32 = 0.01; // CharacterController offset
pub const CHARACTER_CONTROLLER_MAX_HEIGHT: f32 = 5.0; // CharacterController max_height
pub const CHARACTER_CONTROLLER_MIN_WIDTH: f32 = 0.5; // CharacterController min_width

// 激光
pub const LASER_POSITION_OFFSET: f32 = 30.0; // 激光位置偏移
pub const RECOIL_DISTANCE_FACTOR: f32 = 0.3; // 后坐力距离系数
pub const LASER_HEIGHT: f32 = 1366.0; // 激光高度
pub const LASER_COLLIDER_HALF_WIDTH: f32 = 35.0; // 激光碰撞体半宽
pub const LASER_COLLIDER_HALF_HEIGHT: f32 = 683.0; // 激光碰撞体半高
pub const LASER_CHARGE_PROGRESS_BAR_WIDTH: f32 = 100.0; // 激光蓄力进度条宽度

// 敌方坦克
pub const ENEMY_TANK_DISPLAY_WIDTH: f32 = 80.0; // 敌方坦克显示宽度
pub const ENEMY_TANK_DISPLAY_HEIGHT: f32 = 90.0; // 敌方坦克显示高度
pub const ENEMY_COLLIDER_HALF_WIDTH: f32 = 38.0; // 敌方坦克碰撞体半宽 ((80-4)/2)
pub const ENEMY_COLLIDER_HALF_HEIGHT: f32 = 43.0; // 敌方坦克碰撞体半高 ((90-4)/2)
pub const ENEMY_BORN_ANIMATION_SIZE: f32 = 100.0; // 敌方坦克出生动画尺寸
pub const ENEMY_TILE_WIDTH: f32 = 137.0; // 敌方坦克纹理瓦片宽度
pub const ENEMY_TILE_HEIGHT: f32 = 183.0; // 敌方坦克纹理瓦片高度
pub const ENEMY_BORN_TILE_SIZE: f32 = 192.0; // 敌方出生动画瓦片尺寸

// 玩家坦克
pub const PLAYER_TANK_DISPLAY_WIDTH: f32 = 80.0; // 玩家坦克显示宽度
pub const PLAYER_TANK_DISPLAY_HEIGHT: f32 = 90.0; // 玩家坦克显示高度
pub const PLAYER_COLLIDER_HALF: f32 = 35.0; // 玩家坦克碰撞体半宽/高
pub const PLAYER_SPAWN_OFFSET: f32 = 50.0; // 玩家出生位置偏移
pub const PLAYER_TILE_WIDTH: f32 = 293.0; // 玩家坦克瓦片宽度
pub const PLAYER_TILE_HEIGHT: f32 = 328.0; // 玩家坦克瓦片高度

// 子弹
pub const BULLET_WIDTH: f32 = 60.0; // 子弹宽度
pub const BULLET_HEIGHT: f32 = 40.0; // 子弹高度

// 特效
pub const SMOKE_SIZE: f32 = 100.0; // 烟雾尺寸
pub const EXPLOSION_TILE_SIZE: f32 = 512.0; // 爆炸瓦片尺寸
pub const SPARK_TILE_SIZE: f32 = 1024.0; // 火花瓦片尺寸

// 地形
pub const FOREST_COLLIDER_HALF: f32 = 131.0; // 森林碰撞体半宽/高
pub const COMMANDER_BRICK_SIZE: f32 = 50.0; // 司令官砖块大小
pub const COMMANDER_TILE_WIDTH: f32 = 140.0; // 指挥官瓦片宽度
pub const COMMANDER_TILE_HEIGHT: f32 = 120.0; // 指挥官瓦片高度

// UI
pub const PAYMENT_CODE_SIZE: f32 = 400.0; // 收款码尺寸
pub const BACKGROUND_ANIMATION_TILE_WIDTH: f32 = 2060.0; // 背景动画瓦片宽度
pub const BACKGROUND_ANIMATION_TILE_HEIGHT: f32 = 1300.0; // 背景动画瓦片高度

// 血条/蓝条
pub const BAR_TOTAL_WIDTH: f32 = 160.0; // 血条/蓝条总宽度
pub const BAR_HEIGHT: f32 = 10.0; // 血条/蓝条高度
pub const COMMANDER_BAR_WIDTH: f32 = 160.0; // 司令官血条宽度

// 道具
pub const POWERUP_COLLISION_DISTANCE: f32 = 100.0; // 道具碰撞检测距离
pub const POWERUP_BUBBLE_SIZE: f32 = 100.0; // 气泡特效尺寸

// ==================== 速度和角度常量 ====================
pub const ANGLE_DIFF_THRESHOLD: f32 = 0.01; // 角度差阈值
pub const ANGLE_DIFF_RESET_THRESHOLD: f32 = 0.1; // 角度差阈值（重置计时器）
pub const ROTATION_SPEED_FACTOR: f32 = 0.5; // 转向时速度系数
pub const ANGLE_OFFSET_DEGREES: f32 = 90.0; // 角度偏移（度）
pub const ENEMY_ANGLE_OFFSET_DEGREES: f32 = 270.0; // 敌方坦克角度偏移（度）

// ==================== 游戏数值常量 ====================
pub const MAX_ENEMY_ON_SCREEN: usize = 4; // 场上最大敌方坦克数
pub const LASER_COLLISION_FRAME_INTERVAL: u32 = 5; // 激光碰撞检测帧间隔
pub const ENEMY_BORN_END_FRAME: usize = 12; // 敌方出生动画结束帧
pub const ENEMIES_PER_LEVEL: usize = 5; // 每关敌方坦克总数
pub const ENEMY_SHOOT_PROBABILITY: f32 = 0.01; // 敌方坦克射击概率
pub const ENEMY_RANDOM_TURN_PROBABILITY: f32 = 0.4; // 随机转向概率
pub const POWERUP_ATTRIBUTE_INCREASE: usize = 20; // 道具属性增加量
pub const INITIAL_ATTRIBUTE_VALUE: usize = 40; // 初始属性值
pub const MAX_ATTRIBUTE_VALUE: usize = 100; // 最大属性值
pub const DASH_DAMAGE_COST_HIGH: usize = 2; // 高扣血量

// ==================== 比例和音量常量 ====================
pub const VOLUME_HALF: f32 = 0.5; // 音效音量
pub const VOLUME_COMMANDER_MUSIC: f32 = 0.4; // 指挥官音乐音量

// ==================== Z轴层级常量 ====================
pub const Z_SEA: f32 = -0.5; // 海水层级
pub const Z_DEFAULT: f32 = 0.0; // 默认层级
pub const Z_LASER: f32 = 0.9; // 激光层级
pub const Z_FOREST: f32 = 1.0; // 森林层级
pub const Z_PROGRESS_BAR: f32 = 2.0; // 进度条层级
pub const Z_UI: f32 = 10.0; // UI层级
pub const Z_STAGE_INTRO_BG: f32 = 100.0; // 关卡介绍层级
pub const Z_STAGE_INTRO_TEXT: f32 = 101.0; // 关卡介绍文字层级

// ==================== UI字体大小常量 ====================
pub const FONT_SIZE_SMALL: f32 = 18.0; // 小字体
pub const FONT_SIZE_INFO: f32 = 20.0; // 说明文字字体
pub const FONT_SIZE_MEDIUM: f32 = 22.0; // 中等字体
pub const FONT_SIZE_INSTRUCTION: f32 = 24.0; // 说明文字字体
pub const FONT_SIZE_SCORE: f32 = 28.0; // 分数字体
pub const FONT_SIZE_UI: f32 = 30.0; // UI字体
pub const FONT_SIZE_OPTION: f32 = 50.0; // 选项字体
pub const FONT_SIZE_TITLE: f32 = 60.0; // 标题字体
pub const FONT_SIZE_CREDITS_TITLE: f32 = 70.0; // 标题字体
pub const FONT_SIZE_MENU: f32 = 80.0; // 菜单字体
pub const FONT_SIZE_GAME_OVER: f32 = 100.0; // 大标题字体
