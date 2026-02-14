//! Level definitions for the game

use crate::map::TerrainType;
use bevy::prelude::*;

/// 关卡地图数据（使用 `TerrainType` 枚举）
pub type LevelMap = [[TerrainType; crate::map::MAP_COLS]; crate::map::MAP_ROWS];

/// 关卡资源，用于存储已加载的关卡数据
#[derive(Resource, Debug, Default, Clone)]
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

/// Web 端：初始化关卡资源到 LevelAssets（异步）
#[cfg(target_arch = "wasm32")]
pub fn init_level_assets() {
    info!("初始化 Web 端关卡数据（异步加载）");
    // 注意：实际加载将在 wasm 启动时完成，这里只设置状态
    // 关卡数据会在 load_levels_async 中异步加载并注入到 World 中
}

/// 桌面端：获取实际存在的关卡文件数量
#[cfg(not(target_arch = "wasm32"))]
pub fn get_available_level_count() -> usize {
    let levels_dir = get_levels_dir();
    let mut max_level = 0;
    
    // 遍历关卡目录，找出最大的关卡编号
    if let Ok(entries) = std::fs::read_dir(levels_dir) {
        for entry in entries.flatten() {
            if let Some(file_name) = entry.file_name().to_str() {
                // 匹配 "数字.txt" 格式的文件
                if let Some(level_str) = file_name.strip_suffix(".txt") {
                    if let Ok(level_num) = level_str.parse::<usize>() {
                        max_level = max_level.max(level_num);
                    }
                }
            }
        }
    }
    
    // 返回实际关卡数和代码设置的最小值之间的较大者
    max_level.max(crate::constants::MIN_LEVELS)
}

/// Web 端：返回预定义的关卡数（Web 端需要预加载）
#[cfg(target_arch = "wasm32")]
pub fn get_available_level_count() -> usize {
    crate::constants::MIN_LEVELS
}

/// 桌面端：初始化关卡资源到 LevelAssets
#[cfg(not(target_arch = "wasm32"))]
pub fn init_level_assets(mut level_assets: ResMut<LevelAssets>) {
    info!("初始化桌面端关卡数据");
    let max_level = get_available_level_count();
    for level in 1..=max_level {
        if let Some(map_data) = load_level_file(level) {
            level_assets.set(level, map_data);
            info!("关卡 {} 已加载", level);
        } else {
            warn!("关卡 {} 加载失败，将使用空地图", level);
            level_assets.set(level, [[TerrainType::Empty; crate::map::MAP_COLS]; crate::map::MAP_ROWS]);
        }
    }
}

/// Web 端：使用 HTTP 请求动态读取关卡文件
#[cfg(target_arch = "wasm32")]
pub async fn load_level_file(level: usize) -> Option<LevelMap> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let path = format!("levels/{}.txt", level);
    let window = web_sys::window().unwrap();

    // 使用 fetch API 请求关卡文件
    let response = match JsFuture::from(window.fetch_with_str(&path)).await {
        Ok(res) => res,
        Err(e) => {
            warn!("无法获取关卡文件 {}: {:?}", path, e);
            return None;
        }
    };

    let response = response.dyn_into::<web_sys::Response>().unwrap();

    // 检查响应状态
    if !response.ok() {
        warn!("关卡文件 {} 返回错误状态: {}", path, response.status());
        return None;
    }

    // 读取响应文本
    let text = match JsFuture::from(response.text().unwrap()).await {
        Ok(text) => text,
        Err(e) => {
            warn!("无法读取关卡文件 {} 内容: {:?}", path, e);
            return None;
        }
    };

    let content = text.as_string()?;
    let map_data = parse_level_content(&content);
    info!("Web 端关卡 {} 加载成功", level);
    Some(map_data)
}

/// Web 端：异步加载所有关卡（在 wasm 启动时调用）
#[cfg(target_arch = "wasm32")]
pub async fn load_all_levels_async() -> LevelAssets {
    let mut level_assets = LevelAssets::default();

    for level in 1..=crate::constants::MIN_LEVELS {
        if let Some(map_data) = load_level_file(level).await {
            level_assets.set(level, map_data);
            info!("Web 端关卡 {} 加载完成", level);
        } else {
            warn!("Web 端关卡 {} 加载失败", level);
        }
    }

    level_assets
}
