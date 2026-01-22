//! Level definitions for the game

use crate::map::TerrainType;
use std::fs;
use std::path::Path;

/// 关卡地图数据（使用 TerrainType 枚举）
pub type LevelMap = [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

/// 从文件加载关卡数据
/// 地形符号对照：
/// . = 空地
/// t = 树林（坦克可穿过，提供掩护）
/// s = 海（子弹可穿过，坦克不可）
/// b = 砖块（可破坏，1发子弹，完整 100×100）
/// bl = 砖块左半（50×100）
/// br = 砖块右半（50×100）
/// bt = 砖块上半（100×50）
/// bb = 砖块下半（100×50）
/// i = 钢铁（不可破坏，完整 100×100）
/// il = 钢铁左半（50×100）
/// ir = 钢铁右半（50×100）
/// it = 钢铁上半（100×50）
/// ib = 钢铁下半（100×50）
/// a = 屏障（可破坏，2发子弹）
fn load_level_from_file(level_num: usize) -> Result<LevelMap, String> {
    // 检查是否在系统安装目录运行
    let levels_dir = if Path::new("/usr/share/tank-battle/levels").exists() {
        "/usr/share/tank-battle/levels"
    } else {
        "levels"
    };
    let file_path = format!("{}/{}.txt", levels_dir, level_num);
    let path = Path::new(&file_path);

    if !path.exists() {
        return Err(format!("Level file not found: {}", file_path));
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read level file: {}", e))?;

    let mut result: LevelMap = [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

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

    Ok(result)
}

/// 获取指定关卡的地图数据
/// 如果关卡文件不存在，返回第1关作为默认值
pub fn get_level(level: usize) -> LevelMap {
    match load_level_from_file(level) {
        Ok(map) => map,
        Err(_) => {
            // 如果加载失败，尝试加载第1关
            if level != 1 {
                eprintln!("Warning: Failed to load level {}, falling back to level 1", level);
                load_level_from_file(1).unwrap_or_else(|_| {
                    // 如果第1关也加载失败，返回空地图
                    [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS]
                })
            } else {
                eprintln!("Error: Failed to load level 1, using empty map");
                [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS]
            }
        }
    }
}
