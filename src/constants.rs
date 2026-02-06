//! Game constants for the Tank Battle game

use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::resources::Language;

// 字体路径常量
pub const FONT_CN: &str = "font/LiuHuanKaTongShouShu1.5-2.ttf";
pub const FONT_EN: &str = "font/ChelaOne-Regular-2.ttf";

// 音频文件路径常量
pub const SOUND_BRICK_HIT: &str = "music/brick_hit.ogg";
pub const SOUND_BURN_TREE: &str = "music/burn_tree.ogg";
pub const SOUND_COMMANDER_DEATH: &str = "music/commander_death.ogg";
pub const SOUND_COMMANDER_GET_SHOT: &str = "music/commander_get_shot.ogg";
pub const SOUND_MUSIC_NOTE_000: &str = "music/music_note_000.ogg";
pub const SOUND_MUSIC_NOTE_001: &str = "music/music_note_001.ogg";
pub const SOUND_MUSIC_NOTE_002: &str = "music/music_note_002.ogg";
pub const SOUND_MUSIC_NOTE_003: &str = "music/music_note_003.ogg";
pub const SOUND_EXPLOSION: &str = "music/explosion_sound.ogg";
pub const SOUND_HIT: &str = "music/hit_sound.ogg";
pub const SOUND_LASER_CHARGE: &str = "music/laser_charge.ogg";
pub const SOUND_LASER: &str = "music/laser.ogg";
pub const SOUND_METAL_CRASH: &str = "music/metal_crash.ogg";
pub const SOUND_POWERUP: &str = "music/powerup_sound.ogg";
pub const SOUND_SEA_AMBIENCE: &str = "music/sea_ambience.ogg";
pub const SOUND_TREE_AMBIENCE: &str = "music/tree_ambience.ogg";
pub const SOUND_PLAYER_SHOT: &str = "music/player_shot.ogg";

// 子弹纹理路径常量（静态）
pub const TEXTURE_BULLET_PLAYER1: &str = "texture/bullets/bullet_player1.png";
pub const TEXTURE_BULLET_PLAYER2: &str = "texture/bullets/bullet_player2.png";
pub const TEXTURE_BULLET_ENEMY: &str = "texture/bullets/bullet_enemy.png";

// 动画帧间隔常量
pub const FIRE_EFFECT_ANIMATION_FRAME: f32 = 0.03;
pub const PENETRATE_EFFECT_ANIMATION_FRAME: f32 = 0.05;
pub const BACKGROUND_ANIMATION_FRAME: f32 = 0.03;
pub const POWER_UP_ANIMATION_FRAME: f32 = 0.1; // 道具动画帧间隔

// 地图纹理路径常量（静态）
pub const TEXTURE_BARRIER: &str = "maps/barrier.png";
pub const TEXTURE_BRICK: &str = "maps/brick.png";
pub const TEXTURE_STEEL: &str = "maps/steel.png";

// 收款码路径常量
pub const IMAGE_ALIPAY: &str = "alipay.png";
pub const IMAGE_WECHAT: &str = "wechat.png";
pub const PAYMENT_CODE_SIZE: f32 = 400.0;

// 特效纹理路径常量（静态）
pub const TEXTURE_BUBBLE: &str = "effect/BubbleBlue.png";

// 角色纹理路径常量（静态）
pub const TEXTURE_COMMANDER_DEAD: &str = "texture/commander_dead.png";
pub const TEXTURE_SINGLE_BARREL: &str = "texture/single_barrel.png";
pub const TEXTURE_DOUBLE_BARREL: &str = "texture/double_barrel.png";
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
pub const TANK_SPEED: f32 = 200.0;
pub const PLAYER_TANK_SPEED: f32 = 150.0;
pub const BULLET_SPEED: f32 = 900.0;
pub const PLAYER_BULLET_SPEED: f32 = 600.0;

pub const RECALL_TIME: f32 = 2.0; // 回城时间（秒）
pub const VERTICAL_OFFSET: f32 = (BOTTOM_PADDING - TOP_PADDING) / 2.0; // 由于下边不留白，会导致坐标垂直便移-50
pub const WINDOW_WIDTH: u32 = (MAP_WIDTH + LEFT_PADDING + RIGHT_PADDING) as u32; // 总宽度
pub const WINDOW_HEIGHT: u32 = (MAP_HEIGHT + TOP_PADDING + BOTTOM_PADDING) as u32; // 总高度
pub const WINDOW_SIZE: Vec2 = Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32); // 窗口尺寸
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
        MAP_LEFT_X + TANK_DISPLAY_SIZE.x / 2.0,
        MAP_TOP_Y - TANK_DISPLAY_SIZE.y / 2.0,
        0.0,
    ),
    Vec3::new(0.0, MAP_TOP_Y - TANK_DISPLAY_SIZE.y / 2.0, 0.0),
    Vec3::new(
        MAP_RIGHT_X - TANK_DISPLAY_SIZE.x / 2.0,
        MAP_TOP_Y - TANK_DISPLAY_SIZE.y / 2.0,
        0.0,
    ),
];

// ==================== 履带道具常量 ====================
pub const TRACK_CHAIN_ANIMATION_FRAME: f32 = 0.1; // 履带动画帧间隔

// ==================== 颜色常量 ====================
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

// ==================== 尺寸常量（合并版） ====================
pub const COMMANDER_SIZE: Vec2 = Vec2::new(100.0, 100.0);
pub const BARRIER_SIZE: Vec2 = Vec2::new(100.0, 100.0);
pub const TANK_DISPLAY_SIZE: Vec2 = Vec2::new(80.0, 90.0); // 玩家/敌方/炮管显示尺寸
pub const BULLET_DISPLAY_SIZE: Vec2 = Vec2::new(60.0, 40.0);
pub const BULLET_COLLIDER_SIZE: f32 = 10.0; // 子弹碰撞体大小
pub const HUD_BAR_SIZE: Vec2 = Vec2::new(150.0, 15.0);
pub const COMMANDER_BAR_SIZE: Vec2 = Vec2::new(160.0, 15.0);

// 碰撞体尺寸（半尺寸）
pub const ENEMY_COLLIDER_HALF_SIZE: Vec2 = Vec2::new(38.0, 43.0);
pub const PLAYER_COLLIDER_HALF_SIZE: Vec2 = Vec2::new(35.0, 35.0);
pub const WALL_COLLIDER_SIZE: Vec2 = Vec2::new(46.0, 46.0); // 砖块/钢铁
pub const WALL_TEXTURE_SIZE: Vec2 = Vec2::new(50.0, 50.0); // 砖块/钢铁
pub const FOREST_COLLIDER_HALF: f32 = 131.0; // 森林碰撞体半宽/高

// 菜单 Y 坐标数组


// HUD Y 坐标数组
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

pub const HUD_Y_POSITIONS: [f32; 15] = [
    780.0,  // Name
    830.0,  // Speed
    880.0,  // FireSpeed
    930.0,  // Protection
    980.0,  // Shells
    320.0,  // EffectsTitle
    370.0,  // FireShell
    420.0,  // Penetrate
    470.0,  // TrackChain
    520.0,  // AirCushion
    610.0,  // InsufficientEnergy
    50.0,   // Score
    150.0,  // Avatar
    235.0,  // BarHealth
    250.0,  // BarBlue
];



pub const DIRECTION_UP: Vec2 = Vec2::new(0.0, 1.0);
pub const DIRECTION_DOWN: Vec2 = Vec2::new(0.0, -1.0);
pub const DIRECTION_LEFT: Vec2 = Vec2::new(-1.0, 0.0);
pub const DIRECTION_RIGHT: Vec2 = Vec2::new(1.0, 0.0);

pub const DIRECTIONS: [Vec2; 4] = [
    DIRECTION_UP,
    DIRECTION_DOWN,
    DIRECTION_LEFT,
    DIRECTION_RIGHT,
];

// 冲刺相关常量
pub const DASH_DURATION: f32 = 0.2; // 冲刺持续时间（秒）
pub const DASH_DISTANCE: f32 = TANK_DISPLAY_SIZE.y * 2.0; // 冲刺距离（两个坦克长度）

// 关卡俏皮话中文版
pub const STAGE_QUOTES_CN: [&str; 17] = [
    "勇敢的司令官即使被击中也不会撤退。他会坚守原地，\n等待士兵们来营救他。",
    "当你转身射击时，炮弹可能不会直行！\n虽然控制炮弹轨迹确实很困难。",
    "小心，敌方坦克有时也会甩狙",
    "敌方和我方工厂都限制了我们的坦克只能直射，\n以提高命中率。毕竟，炮弹很贵。",
    "所有坦克都直线移动，这不是为了学习螃蟹。而是为了纪念\n上个世纪坦克大战中牺牲的无数坦克。",
    "狡猾的敌方坦克升级了他们的炮弹，\n使我们的炮弹无法拦截他们的炮弹。这真令人沮丧。",
    "当你的所有属性都达到最大值时，请与你的队友分享道具，\n别做贪婪的人。",
    "我们的补给品喷了隐形漆——只有你能看到它们，\n敌人看不到，即使它们就在旁边。",
    "据说在上个世纪的坦克大战中，有一段时间\n敌人也能捡起我们的道具。那真是一场灾难。",
    "我们的炮弹经过特殊处理——当它们遇到司令官时，\n会穿过去而不伤害他。据说这是司令官强烈要求的，\n因为在上个世纪的坦克大战中，无数司令官死于自己人之手。那是一个悲剧。",
    "我们的司令官已经提前调查了敌人的数量——\n最多,额。。可能只有几百个敌人。战争会有结束的一天。",
    "在上个世纪的坦克大战中，敌人似乎无穷无尽，\n没有人活着看到战争的结束。",
    "在上个世纪的坦克大战中，超级炸弹道具会摧毁\n许多敌方坦克，但被摧毁的坦克不计入你的战斗记录。这真的很奇怪。",
    "当你独自一人时，可以向司令官要求一辆额外的坦克。\n你可以告诉司令官，额外的坦克可以帮你挡一些炮弹。",
    "冲刺时，你必须从正面或侧面攻击。从后面攻击时，\n你朝同一方向移动，所以冲击力可能不够。",
    "冲刺时，如果有障碍物或敌人，请确保保持一定距离，\n以更成功地触发冲刺破坏效果。",
    "被激光摧毁的敌人不计入你的分数。\n司令官的理由是激光会损坏花草树木。这真荒谬。",
];

pub const STAGE_QUOTES_EN: [&str; 17] = [
    "The brave commander will not retreat even when hit.\nHe will hold his ground and wait for soldiers to rescue him.",
    "When you turn to shoot, the shells may not go straight!\nAlthough controlling shell trajectory is indeed difficult.",
    "Be careful, enemy tanks sometimes snap-shoot too.",
    "Both enemy and our factories limited our tanks to direct fire\nto improve hit rates. After all, shells are expensive.",
    "All tanks move in straight lines, not to learn to walk like crabs.\nBut to commemorate the countless tanks sacrificed in the last century's tank battle.",
    "The cunning enemy tanks upgraded their shells,\nso our shells cannot intercept theirs. This is truly frustrating.",
    "When all your attributes reach maximum value, please share power-ups with your teammate.\nDon't be greedy.",
    "Our supplies were sprayed with invisible paint—only you can see them,\nenemies can't, even if they're right next to them.",
    "It is said that in the last century's tank battle, for a while,\nenemies could also pick up our power-ups. That was truly a disaster.",
    "Our shells are specially treated—when they encounter the commander,\nthey pass through without harming him. The commander strongly requested this,\nbecause in the last century's tank battle, countless commanders died at the hands of their own people. That was a tragedy.",
    "Our commander has investigated the enemy count in advance—\nat most... well... maybe only a few hundred enemies. The war will have an end one day.",
    "In the last century's tank battle, enemies seemed endless,\nand no one lived to see the end of the war.",
    "In the last century's tank battle, super bomb power-ups would destroy\nmany enemy tanks, but destroyed tanks don't count towards your battle record. That's really strange.",
    "When you're alone, you can ask the commander for an extra tank.\nYou can tell the commander that the extra tank can help block some shells for you.",
    "When dashing, you must attack from the front or side. When attacking from behind,\nyou move in the same direction, so the impact force may not be enough.",
    "When dashing, if there are obstacles or enemies, please ensure you maintain a certain distance,\nto more successfully trigger the dash destruction effect.",
    "Enemies destroyed by lasers don't count towards your score.\nThe commander's reason is that lasers damage flowers and trees. That's absurd.",
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

/// 玩家按键绑定配置
#[derive(Clone, Copy)]
pub struct PlayerKeyBindings {
    pub up: KeyCode,
    pub down: KeyCode,
    pub left: KeyCode,
    pub right: KeyCode,
    pub shoot: KeyCode,
    pub dash: KeyCode,
    pub recall: KeyCode,
    pub laser: KeyCode,
}

impl PlayerKeyBindings {
    /// 玩家1按键绑定 (WASD + J/K/I/L)
    pub fn player1() -> Self {
        Self {
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            left: KeyCode::KeyA,
            right: KeyCode::KeyD,
            shoot: KeyCode::KeyJ,
            dash: KeyCode::KeyK,
            recall: KeyCode::KeyI,
            laser: KeyCode::KeyL,
        }
    }

    /// 玩家2按键绑定 (方向键 + 小键盘)
    pub fn player2() -> Self {
        Self {
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
            shoot: KeyCode::Numpad1,
            dash: KeyCode::Numpad2,
            recall: KeyCode::Numpad4,
            laser: KeyCode::Numpad3,
        }
    }

    /// 检查是否在移动
    pub fn is_moving(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.pressed(self.up)
            || keyboard.pressed(self.down)
            || keyboard.pressed(self.left)
            || keyboard.pressed(self.right)
    }

    /// 获取移动方向
    pub fn get_direction(&self, keyboard: &ButtonInput<KeyCode>) -> Vec2 {
        match (
            keyboard.pressed(self.up),
            keyboard.pressed(self.down),
            keyboard.pressed(self.left),
            keyboard.pressed(self.right),
        ) {
            (true, false, false, false) => Vec2::new(0.0, 1.0),
            (false, true, false, false) => Vec2::new(0.0, -1.0),
            (false, false, true, false) => Vec2::new(-1.0, 0.0),
            (false, false, false, true) => Vec2::new(1.0, 0.0),
            _ => Vec2::ZERO,
        }
    }

    /// 检查是否按下射击键
    pub fn is_shooting(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.pressed(self.shoot)
    }

    /// 检查是否按下回城键
    pub fn is_recalling(&self, keyboard: &ButtonInput<KeyCode>) -> bool {
        keyboard.pressed(self.recall)
    }
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

impl AnimationIndices {
    /// 从第一帧播放到最后一帧（total_frames - 1）
    pub fn from_total_frames(total_frames: usize) -> Self {
        Self {
            first: 0,
            last: total_frames - 1,
        }
    }

    /// 从 TextureAtlasInfo 创建（使用 total_frames）
    pub fn from_atlas_info(atlas_info: &crate::atlas::TextureAtlasInfo) -> Self {
        Self::from_total_frames(atlas_info.total_frames)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Resource, Deref, DerefMut)]
pub struct CurrentAnimationFrame(pub usize);

/// 动画模式枚举
/// 定义动画的播放模式
#[derive(Component, Clone, Copy, PartialEq, Default)]
pub enum AnimationMode {
    /// 循环播放（用于敌方坦克、火焰特效、穿透特效、森林、海洋、司令官音乐动画等）
    #[default]
    Looping,
    /// 播放一次后停止（用于爆炸、烟雾、火花、森林火焰等）
    OneShot,
    /// 播放一次完整动画，然后循环播放指定帧范围
    LoopRange {
        /// 循环起始帧
        start_frame: usize,
        /// 循环结束帧
        end_frame: usize,
    },
    /// 先播放一次完整动画，完成后再循环播放指定帧范围（用于能量球蓄力动画）
    OneShotThenLoop {
        /// 一次性播放的起始帧
        first: usize,
        /// 一次性播放的结束帧
        last: usize,
        /// 循环播放的起始帧
        loop_start: usize,
        /// 循环播放的结束帧
        loop_end: usize,
    },
    /// 条件动画，只有条件满足时才播放（例如：履带动画只在移动时播放）
    Conditional {
        /// 玩家坦克类型（用于查询按键状态）
        tank_type: TankType,
    },
    /// 在指定帧触发事件后继续播放（用于敌方出生动画等）
    AtFrameWithEvent {
        /// 触发事件的帧索引
        trigger_frame: usize,
        /// 触发的事件类型
        event_type: AnimationEventType,
    },
}

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

/// 动画事件类型枚举
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AnimationEventType {
    /// 生成敌方坦克
    SpawnEnemy,
    /// 激光动画结束事件
    LaserAnimationEnd {
        direction: Vec2,
        start_point: Vec3,
        owner_type: TankType,
        energy_ball_entity: Option<Entity>,
    },
}

/// 激光动画结束事件
/// 由 animate_effects 在激光动画结束时发送
#[derive(Message, Clone, Debug)]
pub struct LaserEndEvent {
    /// 激光方向
    pub direction: Vec2,
    /// 激光起点
    pub start_point: Vec3,
    /// 激光所有者类型
    pub owner_type: TankType,
    /// 关联的能量球实体
    pub energy_ball_entity: Option<Entity>,
}

/// 坦克类型枚举
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TankType {
    Player1,
    Player2,
    Enemy,
}

impl TankType {
    /// 获取指定玩家类型的按键绑定
    pub fn get_key_bindings(self) -> PlayerKeyBindings {
        match self {
            TankType::Player1 => PlayerKeyBindings::player1(),
            TankType::Player2 => PlayerKeyBindings::player2(),
            TankType::Enemy => panic!("Enemy tank has no key bindings"),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct PlayerTank {
    pub tank_type: TankType, // TankType::Player1 或 TankType::Player2
}

//用来标记，文字，头像等信息属于哪个玩家
#[derive(Component)]
pub struct PlayerUI {
    pub player_type: TankType,
}

// 玩家正在冲刺标记
#[derive(Component)]
pub struct IsDashing;

// 玩家气垫船特效标记
#[derive(Component)]
pub struct BubbleEffect;

// 玩家履带特效标记
#[derive(Component)]
pub struct TrackChainEffect;

// 火焰特效标记（叠加在子弹上的火焰特效）
#[derive(Component)]
pub struct FireEffect;

// 穿透特效标记（叠加在子弹上的穿透特效）
#[derive(Component)]
pub struct PenetrateEffect;

// 炮管组件标记
#[derive(Component)]
pub struct Barrel;

#[derive(Component)]
pub struct Explosion;

#[derive(Component)]
pub struct Laser;

/// 用于事件驱动模式，缓存碰撞事件和法线信息
#[derive(Resource, Default)]
pub struct EnemyCollisionCache {
    /// 存储 Entity -> 碰撞法线的映射
    pub collisions: EntityHashMap<Vec2>,
}

impl EnemyCollisionCache {
    /// 插入碰撞法线
    pub fn insert(&mut self, entity: Entity, normal: Vec2) {
        self.collisions.insert(entity, normal);
    }

    /// 取出并移除碰撞法线
    pub fn take(&mut self, entity: Entity) -> Option<Vec2> {
        self.collisions.remove(&entity)
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.collisions.clear();
    }
}

/// 本地化文本结构
/// 用于存储多语言文本
#[derive(Clone, Copy)]
pub struct LocalizedText {
    pub cn: &'static str,
    pub en: &'static str,
}

impl LocalizedText {
    /// 根据语言获取对应文本
    pub fn get(&self, language: Language) -> &'static str {
        match language {
            Language::Chinese => self.cn,
            Language::English => self.en,
        }
    }
}

#[derive(Component)]
pub struct Spark;

#[derive(Component)]
pub struct Smoke;

#[derive(Component)]
pub struct EnergyBall {
    pub player_entity: Entity,
}

/// 能量球阶段枚举
/// 标记能量球当前所处的动画阶段
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum EnergyBallPhase {
    /// 蓄力阶段：播放0-64帧，然后循环50-64帧
    Charging,
    /// 激光阶段：循环81-84帧
    Lasering,
}

#[derive(Component)]
pub struct GameOverTimer;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Forest;

#[derive(Component)]
pub struct ForestFire;

#[derive(Component, Default)]
pub struct TreeAmbiencePlayer;

#[derive(Component)]
pub struct Sea;

#[derive(Component, Default)]
pub struct SeaAmbiencePlayer;

#[derive(Component, Default)]
pub struct CommanderAmbiencePlayer;

#[derive(Component)]
pub struct MusicNoteAnimation;

#[derive(Component)]
pub struct Barrier;

#[derive(Component)]
pub struct Brick;

#[derive(Component)]
pub struct Steel;

#[derive(Component)]
pub struct Commander;

#[derive(Component)]
pub struct CommanderText;

#[derive(Component)]
pub struct CommanderHealthBar;

#[derive(Component)]
pub struct CommanderHealthBarOriginalPosition(pub f32); // 记录 Commander 血条的原始 X 位置

#[derive(Component)]
pub struct EnemyCountText;

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

/// 炮管后坐力组件
#[derive(Component)]
pub struct BarrelRecoilForce {
    pub timer: Timer, // 后坐力持续时间
}

/// 激光蓄力组件
#[derive(Component)]
pub struct LaserCharge {
    pub timer: Timer,        // 蓄力计时器
    pub tank_type: TankType, // 坦克类型
}

/// 激光蓄力音效组件
#[derive(Component)]
pub struct LaserChargeSound;

#[derive(Component, Deref, DerefMut)]
pub struct PlayerInfoBlinkTimer(pub Timer);

/// 能量不足提示文本组件
#[derive(Component)]
pub struct InsufficientEnergyText;

/// 能量不足提示计时器
#[derive(Component, Deref, DerefMut)]
pub struct InsufficientEnergyTimer(pub Timer);

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

// ==================== 玩家常量 ====================
pub const MAX_LIFE_POINTS: usize = 3; // 最大生命值
pub const MAX_ENERGY_POINTS: usize = 3; // 最大能量值
pub const BARRIER_DAMAGE_COOLDOWN: f32 = 2.0; // 屏障伤害冷却时间（秒）

// 检测半径常量（用于海、司令官、森林的检测）
pub const DETECTION_RADIUS: f32 = 100.0;

// ==================== 动画时间常量 ====================
pub const ANIMATION_FRAME_EXPLOSION: f32 = 0.01; // 爆炸动画帧间隔
pub const ANIMATION_FRAME_SPARK: f32 = 0.02; // 火花动画帧间隔
pub const ANIMATION_FRAME_FOREST_FIRE: f32 = 0.15; // 森林火灾动画帧间隔，1.5秒播完10帧
pub const ANIMATION_FRAME_LASER: f32 = 0.06; // 激光动画帧间隔，12帧共0.72秒
pub const ANIMATION_FRAME_ENEMY_BORN: f32 = 0.1; // 敌方坦克出生动画帧间隔
pub const ANIMATION_FRAME_ENEMY_MOVE: f32 = 0.1; // 敌方坦克移动动画帧间隔
pub const ANIMATION_FRAME_SMOKE: f32 = 0.1; // 烟雾动画帧间隔
pub const ANIMATION_FRAME_ENERGY_BALL: f32 = 0.02; // 能量球动画帧间隔
pub const ANIMATION_FRAME_MUSIC_NOTE: f32 = 0.1; // 音符动画帧间隔
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
pub const ENEMY_DIRECTION_CHANGE_INTERVAL: f32 = 2.0; // 敌方坦克方向改变间隔
pub const ENEMY_ROTATION_TIME: f32 = 0.8; // 敌方坦克旋转时间
pub const LASER_CHARGE_TIME: f32 = 3.0; // 激光蓄力时间
pub const BLUE_BAR_REGEN_INTERVAL: f32 = 5.0; // 蓝条恢复间隔
pub const INSUFFICIENT_ENERGY_DISPLAY_DURATION: f32 = 2.0; // 能量不足提示显示时长（秒）

// ==================== 尺寸常量 ====================
// 进度条
pub const PROGRESS_BAR_HEIGHT: f32 = 8.0; // 回城进度条高度
pub const PROGRESS_BAR_Y_OFFSET: f32 = 20.0; // 回城进度条Y偏移
pub const PROGRESS_BAR_INITIAL_WIDTH: f32 = 100.0; // 回城进度条初始宽度

// 墙壁和地形
pub const BRICK_GROUP_OFFSET: f32 = 25.0; // 砖块组偏移

// 砖块组四个方向的位置偏移常量
pub const BRICK_GROUP_TOP_LEFT: Vec2 = Vec2::new(-BRICK_GROUP_OFFSET, BRICK_GROUP_OFFSET);
pub const BRICK_GROUP_TOP_RIGHT: Vec2 = Vec2::new(BRICK_GROUP_OFFSET, BRICK_GROUP_OFFSET);
pub const BRICK_GROUP_BOTTOM_LEFT: Vec2 = Vec2::new(-BRICK_GROUP_OFFSET, -BRICK_GROUP_OFFSET);
pub const BRICK_GROUP_BOTTOM_RIGHT: Vec2 = Vec2::new(BRICK_GROUP_OFFSET, -BRICK_GROUP_OFFSET);

pub const CHARACTER_CONTROLLER_OFFSET: f32 = 0.01; // CharacterController offset
pub const CHARACTER_CONTROLLER_MAX_HEIGHT: f32 = 5.0; // CharacterController max_height
pub const CHARACTER_CONTROLLER_MIN_WIDTH: f32 = 0.5; // CharacterController min_width

// 激光

pub const LASER_COLLISION_WIDTH: f32 = 70.0; // 激光碰撞宽度（略窄于坦克车身）
pub const LASER_POSITION_OFFSET: f32 = -40.0; // 激光位置偏移（炮口向前的距离）
pub const RECOIL_DISTANCE_FACTOR: f32 = 0.3; // 后坐力距离系数（坦克整体）
pub const BARREL_RECOIL_DISTANCE: f32 = 10.0; // 炮管后坐力距离（像素）
pub const BARREL_RECOIL_DURATION: f32 = 0.15; // 炮管后坐力持续时间（秒）

// 玩家坦克
pub const PLAYER_SPAWN_OFFSET: f32 = 50.0; // 玩家出生位置偏移

// ==================== 速度和角度常量 ====================
pub const ANGLE_DIFF_THRESHOLD: f32 = 0.01; // 角度差阈值
pub const ANGLE_DIFF_RESET_THRESHOLD: f32 = 0.1; // 角度差阈值（重置计时器）
pub const ROTATION_SPEED_FACTOR: f32 = 0.5; // 转向时速度系数
pub const ANGLE_OFFSET_DEGREES: f32 = 90.0; // 角度偏移（度）
pub const ENEMY_ANGLE_OFFSET_DEGREES: f32 = 270.0; // 敌方坦克角度偏移（度）

// ==================== 游戏数值常量 ====================
pub const MAX_ENEMY_ON_SCREEN: usize = 4; // 场上最大敌方坦克数
pub const ENERGY_BALL_END_FRAME: usize = 64; // 能量球动画结束帧（65帧：0-64）
pub const ENERGY_BALL_LASER_LOOP_START: usize = 81; // 激光阶段循环起始帧
pub const ENERGY_BALL_LASER_LOOP_END: usize = 84; // 激光阶段循环结束帧
pub const ENEMIES_PER_LEVEL: usize = 5; // 每关敌方坦克总数
pub const ENEMY_SHOOT_PROBABILITY: f32 = 0.01; // 敌方坦克射击概率
pub const ENEMY_RANDOM_TURN_PROBABILITY: f32 = 0.4; // 随机转向概率
pub const INITIAL_ATTRIBUTE_VALUE: usize = 40; // 初始属性值
pub const MAX_ATTRIBUTE_VALUE: usize = 100; // 最大属性值

// ==================== 比例和音量常量 ====================
pub const VOLUME_HALF: f32 = 0.5; // 音效音量
pub const VOLUME_MUSIC_NOTE: f32 = 0.4; // 音符动画音量

// ==================== Z轴层级常量 ====================
pub const Z_SEA: f32 = -0.5; // 海水层级
pub const Z_DEFAULT: f32 = 0.0; // 默认层级
pub const Z_LASER: f32 = 0.9; // 激光层级
pub const Z_FOREST: f32 = 1.0; // 森林层级
pub const Z_PROGRESS_BAR: f32 = 2.0; // 进度条层级
pub const Z_UI: f32 = 10.0; // UI层级
pub const Z_STAGE_INTRO_BG: f32 = 100.0; // 关卡介绍层级
pub const Z_STAGE_INTRO_TEXT: f32 = 101.0; // 关卡介绍文字层级

// HUD 数值常量
pub const HUD_MAX_PERCENT: usize = 100; // 最大百分比
pub const HUD_MAX_LIFE_POINTS: f32 = 3.0; // 最大生命值
pub const HUD_MAX_SHELLS: usize = 2; // 最大炮弹数

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
pub const FONT_SIZE_INSUFFICIENT_ENERGY: f32 = 24.0; // 能量不足提示字体大小（与HUD字体一致）
