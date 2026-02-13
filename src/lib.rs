//! Steel Command - Web Library Entry Point
//!
//! This library provides the game logic for both desktop and web platforms.

// Include all the game modules
pub mod app;
pub mod ambience;
pub mod atlas;
pub mod bullet;
pub mod commander;
pub mod constants;
pub mod dash;
pub mod effects;
pub mod enemy;
pub mod game_state;
pub mod global_rng;
pub mod laser;
pub mod levels;
pub mod map;
pub mod physics_config;
pub mod player;
pub mod powerup;
pub mod powerup_strategy;
pub mod resources;
pub mod ui;
pub mod utils;
pub mod weather;

// Re-export necessary items
pub use app::configure_game_resources;
pub use app::register_game_systems;
pub use global_rng::GlobalRngPlugin;

#[cfg(not(target_arch = "wasm32"))]
pub use app::configure_plugins_desktop;

#[cfg(target_arch = "wasm32")]
pub use app::configure_plugins_web;

#[cfg(target_arch = "wasm32")]
pub use levels::LevelAssets;

#[cfg(target_arch = "wasm32")]
pub use get_preloaded_level_assets;

// Shared initialization function
pub fn init_game() {
    use bevy::prelude::*;

    #[cfg(target_arch = "wasm32")]
    {
        // 设置 panic hook 以在 Web 端显示错误信息
        console_error_panic_hook::set_once();
    }

    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    {
        app.add_plugins(configure_plugins_web());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(configure_plugins_desktop());
    }

    app.add_plugins(avian2d::PhysicsPlugins::default().with_length_unit(100.0))
        .insert_resource(avian2d::dynamics::integrator::Gravity::ZERO)
        .add_plugins(global_rng::GlobalRngPlugin { seed: None });

    configure_game_resources(&mut app);
    register_game_systems(&mut app);
    app.run();
}

// WebAssembly exports for browser
#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use crate::{init_game, levels::load_all_levels_async};

    // 全局关卡资源存储
    use bevy::prelude::*;
    use crate::levels::LevelAssets;

    static mut LEVEL_ASSETS: Option<LevelAssets> = None;
    static mut LEVELS_READY: bool = false;

    /// 获取预加载的关卡数据
    #[wasm_bindgen]
    pub fn get_loaded_level_assets() -> *const LevelAssets {
        unsafe {
            LEVEL_ASSETS.as_ref().map(|la| la as *const LevelAssets).unwrap_or(std::ptr::null())
        }
    }

    /// 检查关卡是否已加载
    #[wasm_bindgen]
    pub fn is_levels_ready() -> bool {
        unsafe { LEVELS_READY }
    }

    /// 获取预加载的关卡数据（返回克隆）
    #[wasm_bindgen]
    pub fn get_level_assets_clone() -> JsValue {
        unsafe {
            if let Some(ref assets) = LEVEL_ASSETS {
                // 这里无法直接序列化 LevelAssets，需要使用其他方式
                // 暂时返回 null
                JsValue::NULL
            } else {
                JsValue::NULL
            }
        }
    }

    #[wasm_bindgen(start)]
    pub async fn start() {
        // 预加载所有关卡
        info!("Web 端开始加载关卡...");
        let level_assets = load_all_levels_async().await;

        // 将关卡数据存储到静态变量中
        unsafe {
            LEVEL_ASSETS = Some(level_assets);
            LEVELS_READY = true;
        }

        info!("关卡加载完成，启动游戏");
        init_game();
    }
}

/// Web 端：从预加载的关卡资源中获取数据
#[cfg(target_arch = "wasm32")]
pub fn get_preloaded_level_assets() -> Option<LevelAssets> {
    unsafe {
        wasm::LEVEL_ASSETS.as_ref().map(|la| {
            // 简单的克隆实现
            // 注意：这需要 LevelAssets 实现 Clone
            // 如果 LevelAssets 包含大量数据，应该使用更高效的方式
            let mut new_assets = LevelAssets::default();
            for (idx, level_opt) in la.levels.iter().enumerate() {
                if let Some(ref level) = level_opt {
                    new_assets.levels.push(Some(*level));
                } else {
                    new_assets.levels.push(None);
                }
            }
            new_assets
        })
    }
}