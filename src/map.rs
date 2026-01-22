//! Map module for terrain and level management

use bevy::prelude::*;

/// 地形类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainType {
    /// 空地（可通行）
    Empty,
    /// 树林（坦克可穿过，提供掩护）
    Forest,
    /// 海（子弹可穿过，坦克不可）
    Sea,
    /// 砖块（可破坏，1发子弹）
    Brick,
    /// 砖块左半（50×100）
    BrickLeft,
    /// 砖块右半（50×100）
    BrickRight,
    /// 砖块上半（100×50）
    BrickTop,
    /// 砖块下半（100×50）
    BrickBottom,
    /// 钢铁（不可破坏）
    Steel,
    /// 钢铁左半（50×100）
    SteelLeft,
    /// 钢铁右半（50×100）
    SteelRight,
    /// 钢铁上半（100×50）
    SteelTop,
    /// 钢铁下半（100×50）
    SteelBottom,
    /// 屏障（可破坏，2发子弹）
    Barrier,
}

impl TerrainType {
    /// 从字符串转换为地形类型
    pub fn from_str(s: &str) -> Self {
        match s {
            "." => Self::Empty,
            "t" => Self::Forest,
            "s" => Self::Sea,
            "b" => Self::Brick,
            "bl" => Self::BrickLeft,
            "br" => Self::BrickRight,
            "bt" => Self::BrickTop,
            "bb" => Self::BrickBottom,
            "i" => Self::Steel,
            "il" => Self::SteelLeft,
            "ir" => Self::SteelRight,
            "it" => Self::SteelTop,
            "ib" => Self::SteelBottom,
            "a" => Self::Barrier,
            _ => Self::Empty,
        }
    }

    /// 获取地形的基础类型（忽略半块状态）
    pub fn base_type(&self) -> TerrainType {
        match self {
            Self::Brick | Self::BrickLeft | Self::BrickRight | Self::BrickTop | Self::BrickBottom => Self::Brick,
            Self::Steel | Self::SteelLeft | Self::SteelRight | Self::SteelTop | Self::SteelBottom => Self::Steel,
            other => *other,
        }
    }

    /// 获取地形的碰撞盒尺寸
    pub fn collider_size(&self) -> Vec2 {
        match self {
            Self::BrickLeft | Self::SteelLeft => Vec2::new(50.0, 100.0),
            Self::BrickRight | Self::SteelRight => Vec2::new(50.0, 100.0),
            Self::BrickTop | Self::SteelTop => Vec2::new(100.0, 50.0),
            Self::BrickBottom | Self::SteelBottom => Vec2::new(100.0, 50.0),
            _ => Vec2::new(100.0, 100.0),
        }
    }

    /// 获取地形的偏移量（相对于网格中心）
    pub fn collider_offset(&self) -> Vec2 {
        match self {
            Self::BrickLeft | Self::SteelLeft => Vec2::new(-25.0, 0.0),
            Self::BrickRight | Self::SteelRight => Vec2::new(25.0, 0.0),
            Self::BrickTop | Self::SteelTop => Vec2::new(0.0, 25.0),
            Self::BrickBottom | Self::SteelBottom => Vec2::new(0.0, -25.0),
            _ => Vec2::new(0.0, 0.0),
        }
    }
}

/// 地图配置常量
pub const MAP_ROWS: usize = 12;
pub const MAP_COLS: usize = 16;
pub const GRID_SIZE: f32 = 100.0; // 每个网格的像素大小

/// 将网格坐标转换为世界坐标
pub fn grid_to_world(row: usize, col: usize) -> Vec2 {
    let x = crate::constants::MAP_LEFT_X + col as f32 * GRID_SIZE + GRID_SIZE / 2.0;
    let y = crate::constants::MAP_TOP_Y - row as f32 * GRID_SIZE - GRID_SIZE / 2.0;
    Vec2::new(x, y)
}