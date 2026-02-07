//! UI 模块
//!
//! 提供游戏界面的各种 UI 组件和系统

pub mod common;
pub mod constants;
pub mod hud;
pub mod localization;
pub mod menus;
pub mod overlay;

// 重新导出 UI 常量，方便外部模块使用
pub use constants::*;