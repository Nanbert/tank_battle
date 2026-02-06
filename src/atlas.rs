//! 纹理图集（Atlas）模块
//!
//! 定义纹理图集布局信息常量和相关结构体

use bevy::prelude::*;

use crate::constants::{TANK_DISPLAY_SIZE, COMMANDER_SIZE};

/// 纹理图布局信息
#[derive(Clone, Copy)]
pub struct TextureAtlasInfo {
    pub texture_path: &'static str,
    pub tile_size: Vec2,
    pub columns: usize,
    pub rows: usize,
    pub display_size: Vec2,
    pub total_frames: usize,
}

impl TextureAtlasInfo {
    /// 将 tile_size 转换为 UVec2
    pub fn to_uvec2(&self) -> UVec2 {
        UVec2::new(self.tile_size.x as u32, self.tile_size.y as u32)
    }

    /// 创建纹理图集布局
    pub fn create_layout(&self) -> TextureAtlasLayout {
        TextureAtlasLayout::from_grid(self.to_uvec2(), self.columns as u32, self.rows as u32, None, None)
    }

    /// 添加纹理图集布局到资源管理器
    pub fn add_to_assets(&self, texture_atlas_layouts: &mut Assets<TextureAtlasLayout>) -> Handle<TextureAtlasLayout> {
        let layout = self.create_layout();
        texture_atlas_layouts.add(layout)
    }

    /// 从资源服务器加载纹理
    pub fn load_texture(&self, asset_server: &AssetServer) -> Handle<Image> {
        asset_server.load(self.texture_path)
    }

    /// 创建播放全部帧的 AnimationIndices
    pub fn animation_indices_full(&self) -> crate::constants::AnimationIndices {
        crate::constants::AnimationIndices::from_atlas_info(self)
    }

    /// 创建 TextureAtlasInfo
    pub const fn new(
        texture_path: &'static str,
        tile_size: Vec2,
        columns: usize,
        rows: usize,
    ) -> Self {
        Self {
            texture_path,
            tile_size,
            columns,
            rows,
            display_size: tile_size,
            total_frames: columns * rows,
        }
    }

    /// 设置 display_size
    pub const fn with_display_size(mut self, display_size: Vec2) -> Self {
        self.display_size = display_size;
        self
    }
}

// ==================== 纹理图常量 ====================

// 火焰特效精灵图常量
pub const FIRE_EFFECT_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "texture/bullets/fire_effect.png",
        Vec2::new(64.0, 64.0),
        5,
        7,
    );
    atlas.total_frames = 32; // 火焰特效总帧数（最后一行只有2帧）
    atlas
};

// 穿透效果精灵图常量
pub const PENETRATE_EFFECT_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "texture/bullets/penetrate_effect.png",
        Vec2::new(256.0, 256.0),
        4,
        2,
    );
    atlas.total_frames = 8; // 穿透特效总帧数（2×4=8）
    atlas
};

// 开始界面背景精灵图常量
pub const BACKGROUND_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "texture/background.png",
        Vec2::new(960.0, 960.0),
        5,
        5,
    );
    atlas.display_size = Vec2::new(2060.0, 1300.0);
    atlas
};

// 烟雾精灵图常量
pub const SMOKE_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "effect/smoke_sprite.png",
        Vec2::new(100.0, 100.0),
        5,
        3,
    );
    atlas.display_size = Vec2::new(100.0, 100.0);
    atlas.total_frames = 15; // 烟雾总帧数
    atlas
};

// 履带精灵图常量（2列1行，横向排列，138x77，每帧69x77）
pub const TRACK_CHAIN_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "texture/track_train.png",
    Vec2::new(69.0, 77.0),
    2,  // columns: 2列
    1,  // rows: 1行
)
.with_display_size(TANK_DISPLAY_SIZE);

// 爆炸精灵图常量
// 实际尺寸: 2048 x 2048, 8行8列, 每帧 256 x 256
pub const EXPLOSION_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "effect/explosion.png",
        Vec2::new(256.0, 256.0),
        8,
        8,
    );
    atlas.display_size = Vec2::new(300.0, 300.0);
    atlas
};

// 火花精灵图常量（击中特效）
// 实际尺寸: 1024 x 1024, 4行4列, 每帧 256 x 256
pub const SPARK_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "effect/hit_spark.png",
        Vec2::new(256.0, 256.0),
        4,
        4,
    );
    atlas.display_size = Vec2::new(200.0, 200.0);
    atlas
};

// 敌方出生动画精灵图常量
// 实际尺寸: 1700 x 1020, 3行5列, 每帧 340 x 340
pub const ENEMY_BORN_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "effect/enemy_born.png",
        Vec2::new(340.0, 340.0),
        5,
        3,
    );
    atlas.display_size = Vec2::new(100.0, 100.0);
    atlas.total_frames = 13; // 总共 13 帧
    atlas
};

// 敌方坦克精灵图常量
pub const ENEMY_TANK1_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "enemy_tank/enemy_tank1_sprite.png",
    Vec2::new(137.0, 183.0),
    2,
    1,
)
.with_display_size(TANK_DISPLAY_SIZE);

// 玩家坦克精灵图常量
pub const PLAYER_TANK1_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "texture/player_tank1.png",
    Vec2::new(293.0, 328.0),
    2,
    1,
)
.with_display_size(TANK_DISPLAY_SIZE);

// 玩家坦克2精灵图常量
pub const PLAYER_TANK2_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "texture/player_tank2.png",
    Vec2::new(293.0, 328.0),
    2,
    1,
)
.with_display_size(TANK_DISPLAY_SIZE);

// 指挥官精灵图常量
// 实际尺寸: 1400 x 1200, 10行10列, 每帧 140 x 120
pub const COMMANDER_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "texture/commander.png",
    Vec2::new(140.0, 120.0),
    10,
    10,
)
.with_display_size(COMMANDER_SIZE);

// 激光精灵图常量（蓝色）
// 实际尺寸: 2048 x 2048, 4行4列, 每帧 512 x 512
pub const LASER_BLUE_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "effect/texture_laser_blue.png",
    Vec2::new(512.0, 683.0),
    4,
    3,
).with_display_size(Vec2::new(512.0, 1366.0));

// 激光精灵图常量（红色）
// 实际尺寸: 2048 x 2048, 4行4列, 每帧 512 x 512
pub const LASER_RED_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "effect/texture_laser_red.png",
    Vec2::new(512.0, 683.0),
    4,
    3,
).with_display_size(Vec2::new(512.0, 1366.0));

// 玩家头像精灵图常量
// 实际尺寸: 2080 x 441, 3行13列, 每帧 160 x 147
pub const PLAYER_AVATAR_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "texture/avatar.png",
        Vec2::new(160.0, 147.0),
        13,
        3,
    );
    atlas.display_size = Vec2::new(160.0, 147.0);
    atlas.total_frames = 33;
    atlas
};

// 海水精灵图常量
// 实际尺寸: 300 x 100, 1行3列, 每帧 100 x 100
pub const SEA_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "maps/sea_sheet.png",
    Vec2::new(100.0, 100.0),
    3,
    1,
);

// 森林精灵图常量
// 实际尺寸: 1310 x 131, 1行10列, 每帧 131 x 131
pub const FOREST_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "maps/tree.png",
    Vec2::new(131.0, 131.0),
    10,
    1,
);

// 森林燃烧精灵图常量
// 实际尺寸: 1310 x 131, 1行10列, 每帧 131 x 131
pub const FOREST_FIRE_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "maps/tree_fire_sheet.png",
    Vec2::new(131.0, 131.0),
    10,
    1,
);

// 能量球精灵图常量（蓝色）
// 实际尺寸: 11475 x 2440, 5行17列, 每帧 675 x 488
pub const ENERGY_BALL_BLUE_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "effect/energy_blue_ball.png",
        Vec2::new(675.0, 488.0),
        17,
        5,
    );
    atlas.display_size = Vec2::new(405.0, 293.0);
    atlas
};

// 能量球精灵图常量（红色）
// 实际尺寸: 11475 x 2440, 5行17列, 每帧 675 x 488
pub const ENERGY_BALL_RED_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "effect/energy_red_ball.png",
        Vec2::new(675.0, 488.0),
        17,
        5,
    );
    atlas.display_size = Vec2::new(405.0, 293.0);
    atlas
};

// 音符精灵图常量
// 实际尺寸: 1400 x 120, 1行10列, 每帧 140 x 120
pub const MUSIC_NOTE_ATLAS: TextureAtlasInfo = {
    let mut atlas = TextureAtlasInfo::new(
        "effect/music_note_sheet.png",
        Vec2::new(140.0, 120.0),
        10,
        1,
    );
    atlas.display_size = Vec2::new(70.0, 60.0);
    atlas
};

// ==================== 道具精灵图常量 ====================
// 所有道具使用相同的尺寸和布局（3列1行）
pub const POWER_UP_SPEED_UP_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/speed_up.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_PROTECTION_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/protection.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_FIRE_SPEED_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/fire_speed.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_FIRE_SHELL_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/fire_shell.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_TRACK_CHAIN_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/track_chain.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_PENETRATE_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/penetrate.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_REPAIR_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/repair.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_HAMBURGER_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/hamburger.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_AIR_CUSHION_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/air_cushion.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);

pub const POWER_UP_SHELL_ATLAS: TextureAtlasInfo = TextureAtlasInfo::new(
    "power_up/shell.png",
    Vec2::new(87.0, 69.0),
    3,
    1,
);
