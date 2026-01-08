//! Game constants for the Tank Battle game

use bevy::prelude::*;
use crate::resources::PlayerInfoData;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
pub const ORIGINAL_WIDTH: f32 = 1600.0; // 原游戏区域宽度
pub const ORIGINAL_HEIGHT: f32 = 1200.0; // 原游戏区域高度
pub const LEFT_PADDING: f32 = 230.0; // 左侧留白
pub const RIGHT_PADDING: f32 = 230.0; // 右侧留白
pub const TOP_PADDING: f32 = 100.0; // 上方留白
pub const BOTTOM_PADDING: f32 = 0.0; // 下方留白
pub const VERTICAL_OFFSET: f32 = -50.0; // 垂直偏移，向下平移50像素
pub const ARENA_WIDTH: f32 = ORIGINAL_WIDTH + LEFT_PADDING + RIGHT_PADDING; // 总宽度
pub const ARENA_HEIGHT: f32 = ORIGINAL_HEIGHT + TOP_PADDING + BOTTOM_PADDING; // 总高度
pub const TANK_WIDTH: f32 = 87.0;
pub const TANK_HEIGHT: f32 = 87.0;
pub const TANK_SPEED: f32 = 300.0;
pub const BULLET_SPEED: f32 = 900.0;
pub const BULLET_SIZE: f32 = 10.0;

pub const ENEMY_BORN_PLACES: [Vec3; 3] = [
    Vec3::new(-ORIGINAL_WIDTH / 2.0 + TANK_WIDTH / 2.0, ORIGINAL_HEIGHT/2.0 - TANK_HEIGHT / 2.0 + VERTICAL_OFFSET, 0.0),
    Vec3::new(0.0, ORIGINAL_HEIGHT/2.0 - TANK_HEIGHT / 2.0 + VERTICAL_OFFSET, 0.0),
    Vec3::new(ORIGINAL_WIDTH/2.0 - TANK_WIDTH / 2.0, ORIGINAL_HEIGHT/2.0 - TANK_HEIGHT / 2.0 + VERTICAL_OFFSET, 0.0),
];

pub const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.5, 0.5); // 蓝绿色
pub const START_SCREEN_BACKGROUND_COLOR: Color = Color::srgb(17.0/255.0, 81.0/255.0, 170.0/255.0);

pub const FORTRESS_SIZE: f32 = 60.0;
pub const WALL_THICKNESS: f32 = 15.0;

pub const DIRECTIONS: [Vec2; 4] = [
    Vec2::new(0.0, 1.0),   // 上
    Vec2::new(0.0, -1.0),  // 下
    Vec2::new(-1.0, 0.0),  // 左
    Vec2::new(1.0, 0.0),   // 右
];

/// UI 元素类型枚举
#[derive(Clone, Copy)]
pub enum UIElementType {
    /// 静态文本，直接显示固定字符串
    StaticText(&'static str),
    /// 动态文本，需要根据 PlayerInfoData 生成
    DynamicText(fn(&PlayerInfoData) -> String),
    /// 玩家分数文本
    ScoreText(usize), // 0 = 玩家1, 1 = 玩家2
    /// 玩家头像
    PlayerAvatar(usize), // 0 = 玩家1, 1 = 玩家2
    /// 血条
    HealthBar(usize), // 0 = 玩家1, 1 = 玩家2
    /// 蓝条
    BlueBar(usize), // 0 = 玩家1, 1 = 玩家2
}

/// UI 元素配置
#[derive(Clone)]
pub struct UIElementConfig {
    pub element_type: UIElementType,
    pub x_offset: f32,
    pub y_offset: f32,
    pub font_size: f32,
    pub is_player_info: bool, // 是否是 PlayerInfoText（用于闪烁效果）
}

/// 玩家1 UI 元素配置
pub const PLAYER1_UI_ELEMENTS: &[UIElementConfig] = &[
    // 玩家1名称
    UIElementConfig {
        element_type: UIElementType::StaticText("player1"),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 80.0,
        font_size: 32.0,
        is_player_info: false,
    },
    // Speed
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| {
            if info.speed >= 100 {
                "Speed:MAX".to_string()
            } else {
                format!("Speed:{}%", info.speed)
            }
        }),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 130.0,
        font_size: 24.0,
        is_player_info: true,
    },
    // Fire Speed
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| {
            if info.fire_speed >= 100 {
                "Fire Speed:MAX".to_string()
            } else {
                format!("Fire Speed:{}%", info.fire_speed)
            }
        }),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 180.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Protection
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| {
            if info.protection >= 100 {
                "Protection:MAX".to_string()
            } else {
                format!("Protection:{}%", info.protection)
            }
        }),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 230.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Shells
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Shells: {}", info.shells)),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 280.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Pnetrate
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Pnetrate: {}", if info.penetrate { "Yes" } else { "No" })),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 100.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Track Chain
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Track Chain:{}", if info.track_chain { "Yes" } else { "No" })),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 150.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Air Cushion
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Air Cushion:{}", if info.air_cushion { "Yes" } else { "No" })),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 200.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Fire Shell
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Fire Shell:{}", if info.fire_shell { "Yes" } else { "No" })),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 250.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Effects
    UIElementConfig {
        element_type: UIElementType::StaticText("Effects"),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 300.0,
        font_size: 32.0,
        is_player_info: false,
    },
    // 玩家1分数
    UIElementConfig {
        element_type: UIElementType::ScoreText(0),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 50.0,
        font_size: 28.0,
        is_player_info: false,
    },
    // 玩家1头像
    UIElementConfig {
        element_type: UIElementType::PlayerAvatar(0),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 150.0,
        font_size: 0.0,
        is_player_info: false,
    },
    // 玩家1血条
    UIElementConfig {
        element_type: UIElementType::HealthBar(0),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 215.0,
        font_size: 0.0,
        is_player_info: false,
    },
    // 玩家1蓝条
    UIElementConfig {
        element_type: UIElementType::BlueBar(0),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 230.0,
        font_size: 0.0,
        is_player_info: false,
    },
];

/// 玩家2 UI 元素配置（与玩家1相同，但位置在右侧）
pub const PLAYER2_UI_ELEMENTS: &[UIElementConfig] = &[
    // 玩家2名称
    UIElementConfig {
        element_type: UIElementType::StaticText("player2"),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 80.0,
        font_size: 32.0,
        is_player_info: false,
    },
    // Speed
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| {
            if info.speed >= 100 {
                "Speed:MAX".to_string()
            } else {
                format!("Speed:{}%", info.speed)
            }
        }),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 130.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Fire Speed
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| {
            if info.fire_speed >= 100 {
                "Fire Speed:MAX".to_string()
            } else {
                format!("Fire Speed:{}%", info.fire_speed)
            }
        }),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 180.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Protection
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| {
            if info.protection >= 100 {
                "Protection:MAX".to_string()
            } else {
                format!("Protection:{}%", info.protection)
            }
        }),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 230.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Shells
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Shells: {}", info.shells)),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET - 280.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Pnetrate
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Pnetrate: {}", if info.penetrate { "Yes" } else { "No" })),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 100.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Track Chain
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Track Chain:{}", if info.track_chain { "Yes" } else { "No" })),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 150.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Air Cushion
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Air Cushion:{}", if info.air_cushion { "Yes" } else { "No" })),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 200.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Fire Shell
    UIElementConfig {
        element_type: UIElementType::DynamicText(|info| format!("Fire Shell:{}", if info.fire_shell { "Yes" } else { "No" })),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 250.0,
        font_size: 24.0,
        is_player_info: false,
    },
    // Effects
    UIElementConfig {
        element_type: UIElementType::StaticText("Effects"),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: VERTICAL_OFFSET + 300.0,
        font_size: 32.0,
        is_player_info: false,
    },
    // 玩家2分数
    UIElementConfig {
        element_type: UIElementType::ScoreText(1),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 50.0,
        font_size: 28.0,
        is_player_info: false,
    },
    // 玩家2头像
    UIElementConfig {
        element_type: UIElementType::PlayerAvatar(1),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 150.0,
        font_size: 0.0,
        is_player_info: false,
    },
    // 玩家2血条
    UIElementConfig {
        element_type: UIElementType::HealthBar(1),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 215.0,
        font_size: 0.0,
        is_player_info: false,
    },
    // 玩家2蓝条
    UIElementConfig {
        element_type: UIElementType::BlueBar(1),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0,
        y_offset: ARENA_HEIGHT / 2.0 - 230.0,
        font_size: 0.0,
        is_player_info: false,
    },
];

/// 其他游戏信息 UI 元素配置
pub const OTHER_GAME_INFO_UI_ELEMENTS: &[UIElementConfig] = &[
    // Commander Life
    UIElementConfig {
        element_type: UIElementType::StaticText("Commander Life:"),
        x_offset: -ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0 + 350.0,
        y_offset: ARENA_HEIGHT / 2.0 - 50.0,
        font_size: 28.0,
        is_player_info: false,
    },
    // Enemy Left
    UIElementConfig {
        element_type: UIElementType::StaticText("Enemy Left:20/20"),
        x_offset: ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0 - 300.0,
        y_offset: ARENA_HEIGHT / 2.0 - 50.0,
        font_size: 28.0,
        is_player_info: false,
    },
];