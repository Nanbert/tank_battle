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
pub fn parse_level_content(
    content: &str,
) -> [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS] {
    let mut result: [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS] =
        [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

    for (row_idx, line) in content.lines().enumerate() {
        if row_idx >= crate::map::MAP_ROWS {
            break;
        }

        // 按空格分割，支持单字符和双字符的符号
        let tokens: Vec<&str> = line.split_whitespace().collect();
        for (col_idx, token) in tokens.iter().enumerate() {
            if col_idx >= crate::map::MAP_COLS {
                break;
            }

            let terrain = TerrainType::from_str(token);
            result[row_idx][col_idx] = terrain;
        }
    }

    result
}

/// 从 `LevelAssets` 资源获取关卡数据
/// 如果关卡未加载，则动态加载
pub fn get_level_from_assets(level_assets: &mut LevelAssets, level: usize) -> LevelMap {
    let level_idx = level - 1; // 关卡从1开始，数组从0开始

    // 检查是否已加载
    if level_idx < level_assets.levels.len() && level_assets.levels[level_idx].is_some() {
        return level_assets.levels[level_idx].unwrap();
    }

    // 动态加载关卡
    if let Some(map_data) = load_level_file(level) {
        // 确保数组足够大
        if level_assets.levels.len() < level {
            level_assets.levels.resize(level, None);
        }
        level_assets.levels[level_idx] = Some(map_data);
        map_data
    } else {
        // 加载失败，返回空地图
        [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS]
    }
}
/// 获取关卡文件目录路径
/// 根据运行环境确定关卡文件路径：
/// - 系统安装位置：/usr/share/tank-battle/levels/
/// - 开发环境/压缩包：当前目录的 levels/
fn get_levels_dir() -> &'static str {
    if std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .as_deref()
        == Some(std::path::Path::new("/usr/bin"))
    {
        "/usr/share/tank-battle/levels"
    } else {
        "levels"
    }
}

/// 加载指定关卡文件
pub fn load_level_file(level: usize) -> Option<LevelMap> {
    let levels_dir = get_levels_dir();
    let level_path = format!("{levels_dir}/{level}.txt");
    match std::fs::read_to_string(&level_path) {
        Ok(content) => Some(parse_level_content(&content)),
        Err(_) => {
            warn!("无法加载关卡文件: {}", level_path);
            None
        }
    }
}

/// 初始化关卡资源（不再预加载）
pub fn load_level_assets() {
    // 不再预加载关卡文件，改为按需加载
}
