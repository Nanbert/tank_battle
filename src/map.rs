//! Map module for terrain and level management

use bevy::prelude::*;

/// 地形类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainType {
    /// 空地（可通行）
    Empty = 0,
    /// 树林（坦克可穿过，提供掩护）
    Forest = 1,
    /// 海（子弹可穿过，坦克不可）
    Sea = 2,
    /// 砖块（可破坏，1发子弹）
    Brick = 3,
    /// 钢铁（不可破坏）
    Steel = 4,
    /// 屏障（可破坏，2发子弹）
    Barrier = 5,
}

impl TerrainType {
    /// 从数字转换为地形类型（const 函数，可用于常量）
    pub const fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::Empty,
            1 => Self::Forest,
            2 => Self::Sea,
            3 => Self::Brick,
            4 => Self::Steel,
            5 => Self::Barrier,
            _ => Self::Empty,
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