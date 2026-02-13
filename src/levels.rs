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

impl LevelAssets {
    /// 获取关卡数据（如果未加载则返回空地图）
    pub fn get(&self, level: usize) -> Option<LevelMap> {
        let level_idx = level - 1;
        if level_idx < self.levels.len() {
            self.levels[level_idx]
        } else {
            None
        }
    }

    /// 设置关卡数据
    pub fn set(&mut self, level: usize, map_data: LevelMap) {
        let level_idx = level - 1;
        if self.levels.len() < level {
            self.levels.resize(level, None);
        }
        self.levels[level_idx] = Some(map_data);
    }
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
#[deprecated(note = "Use LevelAssets::get() instead")]
pub fn get_level_from_assets(level_assets: &mut LevelAssets, level: usize) -> LevelMap {
    level_assets.get(level).unwrap_or_else(|| {
        [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS]
    })
}
/// 获取关卡文件目录路径
/// 根据运行环境确定关卡文件路径：
/// - Web 端：levels/
/// - 系统安装位置：/usr/share/tank-battle/levels/
/// - 开发环境/压缩包：当前目录的 levels/
#[cfg(not(target_arch = "wasm32"))]
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

/// 加载指定关卡文件（桌面端）
#[cfg(not(target_arch = "wasm32"))]
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

/// Web 端：初始化关卡资源（使用预定义数据）
#[cfg(target_arch = "wasm32")]
pub fn load_level_assets() {
    info!("Web 端使用预定义关卡数据 - 注意：关卡需要在游戏启动时初始化到 LevelAssets 资源中");
}

/// 桌面端：初始化关卡资源
#[cfg(not(target_arch = "wasm32"))]
pub fn load_level_assets() {
    info!("桌面端关卡资源初始化");
}

/// Web 端：初始化关卡资源到 LevelAssets
#[cfg(target_arch = "wasm32")]
pub fn init_level_assets(mut level_assets: ResMut<LevelAssets>) {
    info!("初始化 Web 端关卡数据");
    for level in 1..=4 {
        if let Some(map_data) = load_level_file(level) {
            level_assets.set(level, map_data);
            info!("关卡 {} 已加载", level);
        }
    }
}

/// 桌面端：初始化关卡资源到 LevelAssets
#[cfg(not(target_arch = "wasm32"))]
pub fn init_level_assets(mut level_assets: ResMut<LevelAssets>) {
    info!("初始化桌面端关卡数据");
    for level in 1..=4 {
        if let Some(map_data) = load_level_file(level) {
            level_assets.set(level, map_data);
            info!("关卡 {} 已加载", level);
        }
    }
}

/// Web 端：使用内嵌的关卡数据
#[cfg(target_arch = "wasm32")]
pub fn load_level_file(level: usize) -> Option<LevelMap> {
    // Web 端使用预定义的关卡数据
    // 这里可以内嵌关卡数据，或者返回 None 让系统使用默认地图
    match level {
        1 => Some(get_level_1()),
        2 => Some(get_level_2()),
        3 => Some(get_level_3()),
        4 => Some(get_level_4()),
        _ => None,
    }
}

/// 预定义关卡 1 数据
#[cfg(target_arch = "wasm32")]
fn get_level_1() -> LevelMap {
    parse_level_content(
        ". . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
b . . b . b . . b . . . b . . .
b b . b . b . b b b . b b b . .
b . b b . b b . . . b . . . b .
b . . b b b b . . . b . . . b .
i . i b . . . . . . s . . b i .
t b . b . b b b b . s . b t b t
t b . b . b . . b . s . b t b t
t b . b i b . . b . . i b t b t
t b . b . . . . . . . . . b . b
t t t t . . . . . . . . . t t .
",
    )
}

/// 预定义关卡 2 数据
#[cfg(target_arch = "wasm32")]
fn get_level_2() -> LevelMap {
    parse_level_content(
        ". . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
b b b b . . . . . . b b b b .
b . . b . . . . . . . b . . b .
b . . b . . . . . . . b . . b .
b . . b . . . . . . . b . . b .
. . . . . s . . s . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
",
    )
}

/// 预定义关卡 3 数据
#[cfg(target_arch = "wasm32")]
fn get_level_3() -> LevelMap {
    parse_level_content(
        ". . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . i i i . i i i . . . .
. . . i . i . i . i . . . .
. . . i . i . i . i . . . .
. . . . . . . . . . . . . .
. . . . . s . s . . . . . .
. . . . . . . . . . . . . .
. . . . . . . . . . . . . .
. . . . . . . . . . . . . .
. . . . . . . . . . . . . .
. . . . . . . . . . . . . .
",
    )
}

/// 预定义关卡 4 数据
#[cfg(target_arch = "wasm32")]
fn get_level_4() -> LevelMap {
    parse_level_content(
        ". . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . . .
. . . . . . . . . . . . . .
",
    )
}


