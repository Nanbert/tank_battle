//! Level definitions for the game

use crate::map::TerrainType;

/// 关卡地图数据
pub type LevelMap = [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

/// 第1关地图
/// 地形数字对照：
/// 0 = 空地
/// 1 = 树林（坦克可穿过，提供掩护）
/// 2 = 海（子弹可穿过，坦克不可）
/// 3 = 砖块（可破坏，1发子弹）
/// 4 = 钢铁（不可破坏）
/// 5 = 屏障（可破坏，2发子弹）
pub const LEVEL_1: LevelMap = [
    [
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
    ],
    [
        TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(3),
        TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
    ],
    [
        TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(3), TerrainType::from_u8(3),
        TerrainType::from_u8(2), TerrainType::from_u8(2), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(2),
        TerrainType::from_u8(2), TerrainType::from_u8(2), TerrainType::from_u8(2), TerrainType::from_u8(2),
    ],
    [
        TerrainType::from_u8(2), TerrainType::from_u8(2), TerrainType::from_u8(3), TerrainType::from_u8(3),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(4), TerrainType::from_u8(4),
        TerrainType::from_u8(4), TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
    ],
    [
        TerrainType::from_u8(2), TerrainType::from_u8(2), TerrainType::from_u8(3), TerrainType::from_u8(3),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(4), TerrainType::from_u8(4),
        TerrainType::from_u8(4), TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(0), TerrainType::from_u8(0),
    ],
    [
        TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(4), TerrainType::from_u8(4),
        TerrainType::from_u8(4), TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(2),
    ],
    [
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(3), TerrainType::from_u8(3), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(2),
    ],
    [
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(2),
    ],
    [
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(2),
    ],
    [
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(0),
    ],
    [
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(1),
    ],
    [
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0), TerrainType::from_u8(0),
        TerrainType::from_u8(0), TerrainType::from_u8(1), TerrainType::from_u8(1), TerrainType::from_u8(0),
    ],
];

/// 获取指定关卡的地图数据
pub const fn get_level(level: usize) -> &'static LevelMap {
    match level {
        1 => &LEVEL_1,
        _ => &LEVEL_1, // 默认返回第1关
    }
}
