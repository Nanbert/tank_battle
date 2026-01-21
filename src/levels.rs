//! Level definitions for the game

use crate::map::TerrainType;

/// 关卡地图数据（使用 u8 数字表示地形）
pub type LevelMapRaw = [[u8; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

/// 转换后的关卡地图数据（使用 TerrainType 枚举）
pub type LevelMap = [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

/// 将原始数字地图转换为 TerrainType 地图
const fn convert_level_map(raw: &LevelMapRaw) -> LevelMap {
    let mut result = [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS];
    let mut row = 0;
    while row < crate::map::MAP_ROWS {
        let mut col = 0;
        while col < crate::map::MAP_COLS {
            result[row][col] = TerrainType::from_u8(raw[row][col]);
            col += 1;
        }
        row += 1;
    }
    result
}

/// 第1关地图
/// 地形数字对照：
/// 0 = 空地
/// 1 = 树林（坦克可穿过，提供掩护）
/// 2 = 海（子弹可穿过，坦克不可）
/// 3 = 砖块（可破坏，1发子弹）
/// 4 = 钢铁（不可破坏）
/// 5 = 屏障（可破坏，2发子弹）
pub const LEVEL_1_RAW: LevelMapRaw = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [3, 0, 0, 3, 0, 3, 0, 0, 3, 0, 0, 0, 3, 0, 0, 0],
    [3, 3, 0, 3, 0, 3, 0, 3, 3, 3, 0, 3, 3, 3, 0, 0],
    [3, 0, 3, 3, 0, 3, 3, 0, 0, 0, 3, 0, 0, 0, 3, 0],
    [3, 0, 0, 3, 3, 3, 3, 0, 0, 0, 3, 0, 0, 0, 3, 0],
    [4, 0, 4, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 4, 0, 4],
    [1, 3, 0, 3, 0, 3, 3, 3, 3, 0, 0, 0, 3, 1, 3, 1],
    [1, 3, 0, 3, 0, 3, 0, 0, 3, 0, 0, 0, 3, 1, 3, 1],
    [1, 3, 0, 3, 4, 3, 0, 0, 3, 0, 0, 4, 3, 1, 3, 1],
    [1, 3, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 3, 1],
    [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0],
];
pub const LEVEL_2_RAW: LevelMapRaw = [
    [0, 0, 0, 4, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 4, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 4, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 1, 0, 0, 0, 0, 0, 3, 0, 2, 0, 0, 1, 0],
    [1, 0, 0, 1, 0, 0, 0, 4, 0, 3, 0, 2, 3, 0, 1, 1],
    [0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 2, 3, 0, 1, 0],
    [3, 3, 2, 4, 0, 0, 0, 0, 4, 0, 3, 0, 3, 0, 1, 0],
    [0, 3, 0, 3, 0, 0, 0, 3, 3, 3, 3, 3, 3, 0, 0, 2],
    [4, 3, 0, 3, 0, 0, 0, 3, 3, 3, 0, 0, 3, 0, 4, 2],
    [2, 3, 0, 3, 0, 0, 0, 3, 3, 3, 0, 0, 2, 2, 3, 2],
    [2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 2],
    [2, 3, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 3, 3],
];

/// 转换后的第1关地图
pub const LEVEL_1: LevelMap = convert_level_map(&LEVEL_1_RAW);
pub const LEVEL_2: LevelMap = convert_level_map(&LEVEL_2_RAW);

/// 获取指定关卡的地图数据
pub const fn get_level(level: usize) -> &'static LevelMap {
    match level {
        1 => &LEVEL_1,
        2 => &LEVEL_2,
        _ => &LEVEL_1, // 默认返回第1关
    }
}
