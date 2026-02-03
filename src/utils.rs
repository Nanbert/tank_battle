//! 通用工具函数模块
//!
//! 提供游戏中各模块共用的工具函数

use bevy::input::ButtonInput;
use bevy::prelude::*;

use crate::constants::*;

/// 计算两个角度之间的最小角度差（范围：-π 到 π）
///
/// # 参数
/// - `target_angle`: 目标角度（弧度）
/// - `current_angle`: 当前角度（弧度）
///
/// # 返回值
/// 最小角度差，范围在 -π 到 π 之间
pub fn calculate_angle_difference(target_angle: f32, current_angle: f32) -> f32 {
    let diff = std::f32::consts::PI.mul_add(3.0, target_angle - current_angle)
        % (std::f32::consts::PI * 2.0)
        - std::f32::consts::PI;
    diff
}

/// 检查玩家是否正在移动或射击（用于打断持续状态）
///
/// # 参数
/// - `keyboard`: 键盘输入状态
/// - `tank_type`: 坦克类型
///
/// # 返回值
/// 如果玩家正在移动或射击返回 true，否则返回 false
pub fn is_movement_interrupted(keyboard: &ButtonInput<KeyCode>, tank_type: TankType) -> bool {
    let key_bindings = tank_type.get_key_bindings();
    key_bindings.is_moving(keyboard) || key_bindings.is_shooting(keyboard)
}

/// 创建纹理图集布局的辅助函数
pub fn create_texture_atlas(
    tile_size: UVec2,
    columns: u32,
    rows: u32,
) -> TextureAtlasLayout {
    TextureAtlasLayout::from_grid(tile_size, columns, rows, None, None)
}

/// 播放一次性音效
pub fn play_one_shot_sound(commands: &mut Commands, audio_source: Handle<AudioSource>, volume: f32) -> Entity {
    commands.spawn((
        AudioPlayer::new(audio_source),
        PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(volume)),
    )).id()
}

/// 播放循环音效
pub fn play_looping_sound(commands: &mut Commands, audio_source: Handle<AudioSource>, volume: f32) -> Entity {
    commands.spawn((
        AudioPlayer::new(audio_source),
        PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(volume)),
    )).id()
}