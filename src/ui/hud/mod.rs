//! HUD (Head-Up Display) 模块
//!
//! 处理游戏内的 HUD 显示，包括玩家状态、血条、蓝条等

pub mod blink;
pub mod spawn;
pub mod stats;
pub mod update;

// 重新导出公共函数（供 app.rs 使用）
pub use spawn::spawn_hud;
pub use update::{despawn_hud, update_stage_text};
