//! Level definitions for the game

use crate::map::TerrainType;
use bevy::prelude::*;

/// 关卡地图数据（使用 `TerrainType` 枚举）
pub type LevelMap = [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

/// 关卡资源，用于存储已加载的关卡数据
#[derive(Resource, Debug, Default)]
pub struct LevelAssets {
    levels: Vec<Option<LevelMap>>,
}



/// 从文件内容解析关卡地图
/// il = 钢铁左半（50×100）
/// ir = 钢铁右半（50×100）
/// it = 钢铁上半（100×50）
/// ib = 钢铁下半（100×50）
/// a = 屏障（可破坏，2发子弹）
pub fn parse_level_content(content: &str) -> [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS] {
    let mut result: [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS] = [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

    for (row_idx, line) in content.lines().enumerate() {
        if row_idx >= crate::map::MAP_ROWS {
            break;
        }

        // 按空格分割，支持单字符和双字符的符号
        let tokens: Vec<&str> = line.split_whitespace().collect();
        // 调试第7行
        if row_idx == 6 {
            eprintln!("Row 6: line='{line}', tokens={tokens:?}");
        }
        for (col_idx, token) in tokens.iter().enumerate() {
            if col_idx >= crate::map::MAP_COLS {
                break;
            }

            let terrain = TerrainType::from_str(token);
            result[row_idx][col_idx] = terrain;
            // 调试第7行
            if row_idx == 6 {
                eprintln!("Row 6, Col {col_idx}: token='{token}' -> terrain={terrain:?}");
            }
        }
    }

    result
}

/// 从 `LevelAssets` 资源获取关卡数据
pub fn get_level_from_assets(level_assets: &LevelAssets, level: usize) -> LevelMap {
    let level_idx = level - 1; // 关卡从1开始，数组从0开始
    if level_idx < level_assets.levels.len()
        && let Some(ref map_data) = level_assets.levels[level_idx] {
            return *map_data;
        }
    // 如果关卡未加载，返回空地图
    eprintln!("Warning: Level {level} not loaded, returning empty map");
    [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS]
}

/// 加载所有关卡文件到资源中
/// 关卡文件从当前工作目录的 levels 子目录加载
/// 注意：这是同步加载，在生产环境中可以考虑使用 Bevy 的异步资源加载
pub fn load_level_assets(
    mut level_assets: ResMut<LevelAssets>,
) {
    // 预加载前4个关卡
    for level in 1..=4 {
        if let Ok(content) = std::fs::read_to_string(format!("levels/{level}.txt")) {
            let map_data = parse_level_content(&content);
            if level_assets.levels.len() < level {
                level_assets.levels.resize(level, None);
            }
            level_assets.levels[level - 1] = Some(map_data);

            // 统计地形类型
            let mut brick_count = 0;
            let mut steel_count = 0;
            let mut forest_count = 0;
            let mut sea_count = 0;
            let mut barrier_count = 0;

            for row in &map_data {
                for terrain in row {
                    match terrain {
                        TerrainType::Brick | TerrainType::BrickLeft | TerrainType::BrickRight |
                        TerrainType::BrickTop | TerrainType::BrickBottom => brick_count += 1,
                        TerrainType::Steel | TerrainType::SteelLeft | TerrainType::SteelRight |
                        TerrainType::SteelTop | TerrainType::SteelBottom => steel_count += 1,
                        TerrainType::Forest => forest_count += 1,
                        TerrainType::Sea => sea_count += 1,
                        TerrainType::Barrier => barrier_count += 1,
                        TerrainType::Empty => {}
                    }
                }
            }

            eprintln!("Level {level} loaded successfully: {brick_count} bricks, {steel_count} steels, {forest_count} forests, {sea_count} seas, {barrier_count} barriers");
        } else {
            eprintln!("Warning: Level file levels/{level}.txt not found");
        }
    }
}