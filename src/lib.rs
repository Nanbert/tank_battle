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
pub use levels::LevelAssets;
pub use app::configure_game_resources;
pub use app::register_game_systems;
pub use global_rng::GlobalRngPlugin;

#[cfg(not(target_arch = "wasm32"))]
pub use app::configure_plugins_desktop;

#[cfg(target_arch = "wasm32")]
pub use app::configure_plugins_web;

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
    use crate::{init_game, levels::load_all_levels_async};

    // 全局关卡资源存储
    use bevy::prelude::*;
    use crate::levels::LevelAssets;
    use std::sync::OnceLock;

    pub(crate) static LEVEL_ASSETS: OnceLock<LevelAssets> = OnceLock::new();
    static LEVELS_READY: OnceLock<bool> = OnceLock::new();

    /// 获取预加载的关卡数据
    #[wasm_bindgen]
    pub fn get_loaded_level_assets() -> *const LevelAssets {
        LEVEL_ASSETS.get().map(|la| la as *const LevelAssets).unwrap_or(std::ptr::null())
    }

    /// 检查关卡是否已加载
    #[wasm_bindgen]
    pub fn is_levels_ready() -> bool {
        *LEVELS_READY.get().unwrap_or(&false)
    }

    /// 获取预加载的关卡数据（返回克隆）
    #[wasm_bindgen]
    pub fn get_level_assets_clone() -> JsValue {
        // 这里无法直接序列化 LevelAssets，需要使用其他方式
        // 暂时返回 null
        JsValue::NULL
    }

    #[wasm_bindgen(start)]
    pub async fn start() {
        // 预加载所有关卡
        info!("Web 端开始加载关卡...");
        let level_assets = load_all_levels_async().await;

        // 将关卡数据存储到静态变量中
        LEVEL_ASSETS.set(level_assets).unwrap();
        LEVELS_READY.set(true).unwrap();

        info!("关卡加载完成，启动游戏");
        init_game();
    }
}

/// Web 端：从预加载的关卡资源中获取数据
#[cfg(target_arch = "wasm32")]
pub(crate) fn get_preloaded_level_assets() -> Option<LevelAssets> {
    wasm::LEVEL_ASSETS.get().map(|la| la.clone())
}