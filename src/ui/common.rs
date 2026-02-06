//! UI 通用组件和工具函数
//!
//! 提供跨多个 UI 模块共享的通用功能

use bevy::prelude::*;
use crate::resources::*;

/// 获取指定语言的字体句柄
pub fn get_font(resources: &GameTextureResources, language: Language) -> Handle<Font> {
    resources.get_font(language)
}

/// 根据语言选择字体（辅助函数，用于需要同时传入中英文字体的场景）
pub fn get_font_by_language(
    cn_font: &Handle<Font>,
    en_font: &Handle<Font>,
    language: Language,
) -> Handle<Font> {
    match language {
        Language::Chinese => cn_font.clone(),
        Language::English => en_font.clone(),
    }
}

/// 生成 2D 文本实体
pub fn spawn_text_2d(
    commands: &mut Commands,
    text: impl Into<String>,
    font: &Handle<Font>,
    font_size: f32,
    position: Vec3,
    color: Color,
) -> Entity {
    commands.spawn((
        Text2d(text.into()),
        TextFont {
            font_size,
            font: font.clone(),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
    )).id()
}

/// 生成 2D 文本实体（带额外标记组件）
pub fn spawn_text_2d_with_markers<M: Bundle>(
    commands: &mut Commands,
    text: impl Into<String>,
    font: &Handle<Font>,
    font_size: f32,
    position: Vec3,
    color: Color,
    markers: M,
) -> Entity {
    commands.spawn((
        Text2d(text.into()),
        TextFont {
            font_size,
            font: font.clone(),
            ..default()
        },
        TextColor(color),
        Transform::from_translation(position),
        markers,
    )).id()
}

/// 生成进度条（使用纹理）
pub fn spawn_bar<T: Component + Clone>(
    commands: &mut Commands,
    texture: Handle<Image>,
    position: Vec3,
    size: Vec2,
    z_index: f32,
    marker: T,
) -> Entity {
    let mut transform = Transform::from_translation(position);
    transform.translation.z = z_index;
    
    commands.spawn((
        marker,
        Sprite {
            image: texture,
            custom_size: Some(size),
            ..default()
        },
        transform,
    )).id()
}

/// 生成进度条（使用纯色）
pub fn spawn_bar_colored<T: Component + Clone>(
    commands: &mut Commands,
    color: Color,
    position: Vec3,
    size: Vec2,
    z_index: f32,
    marker: T,
) -> Entity {
    let mut transform = Transform::from_translation(position);
    transform.translation.z = z_index;
    
    commands.spawn((
        marker,
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        transform,
    )).id()
}

/// 更新进度条（血条、蓝条等）
pub fn update_bar(
    sprite: &mut Sprite,
    transform: &mut Transform,
    value: f32,
    max_value: f32,
    base_x: f32,
    bar_width: f32,
    bar_height: f32,
) {
    let width = bar_width * (value / max_value);
    sprite.custom_size = Some(Vec2::new(width, bar_height));
    transform.translation.x = base_x - bar_width / 2.0 + width / 2.0;
}