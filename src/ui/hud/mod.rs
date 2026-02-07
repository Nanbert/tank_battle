//! HUD (Head-Up Display) 模块
//!
//! 处理游戏内的 HUD 显示，包括玩家状态、血条、蓝条等

pub mod stats;
pub mod spawn;
pub mod update;
pub mod blink;

// 重新导出公共函数（供 app.rs 使用）
pub use spawn::spawn_hud;
pub use update::{update_stage_text, despawn_hud};