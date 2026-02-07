//! UI 通用组件和工具函数
//!
//! 提供跨多个 UI 模块共享的通用功能

use bevy::prelude::*;
use crate::resources::*;

/// 获取指定语言的字体句柄
pub fn get_font(resources: &GameTextureResources, language: Language) -> Handle<Font> {
    resources.get_font(language)
}

/// HUD 坐标计算辅助函数
///
/// 统一处理 HUD 元素的 Y 坐标计算，避免重复的 WINDOW_TOP_Y - offset 计算
pub fn hud_y_position(y_position: crate::ui::HudYPosition) -> f32 {
    crate::constants::WINDOW_TOP_Y - y_position.offset()
}

/// 计算 HUD 元素的完整位置（包含 X 坐标）
pub fn hud_position(x_pos: f32, y_position: crate::ui::HudYPosition, z_index: f32) -> Vec3 {
    Vec3::new(x_pos, hud_y_position(y_position), z_index)
}

// ==================== 文本生成辅助函数 ====================

/// 创建统一的 TextFont 配置
///
/// 减少重复的 TextFont 构建代码
pub fn create_text_font(font: &Handle<Font>, font_size: f32) -> TextFont {
    TextFont {
        font_size,
        font: font.clone(),
        ..default()
    }
}

/// 生成带标记的 2D 文本实体
#[inline]
pub fn spawn_simple_text_with_marker<M: Component>(
    commands: &mut Commands,
    text: impl Into<String>,
    font: &Handle<Font>,
    font_size: f32,
    position: Vec3,
    color: Color,
    marker: M,
    z_index: f32,
) -> Entity {
    let mut transform = Transform::from_translation(position);
    transform.translation.z = z_index;

    commands
        .spawn((
            marker,
            Text2d(text.into()),
            create_text_font(font, font_size),
            TextColor(color),
            transform,
        ))
        .id()
}

/// 生成带标记和对齐方式的 2D 文本实体
#[inline]
pub fn spawn_text_with_justify_and_marker<M: Component>(
    commands: &mut Commands,
    text: impl Into<String>,
    font: &Handle<Font>,
    font_size: f32,
    position: Vec3,
    color: Color,
    marker: M,
    justify: Justify,
    z_index: f32,
) -> Entity {
    let mut transform = Transform::from_translation(position);
    transform.translation.z = z_index;

    commands
        .spawn((
            marker,
            Text2d(text.into()),
            create_text_font(font, font_size),
            TextColor(color),
            TextLayout::new_with_justify(justify),
            transform,
        ))
        .id()
}

// ==================== 淡入淡出辅助函数 ====================

/// 通用透明度更新 trait
/// 
/// 用于统一处理不同类型的透明度更新（Sprite、TextColor 等）
pub trait AlphaMut {
    /// 设置透明度
    fn set_alpha(&mut self, alpha: f32);
}

impl AlphaMut for Sprite {
    fn set_alpha(&mut self, alpha: f32) {
        let linear = self.color.to_linear();
        self.color = Color::srgba(linear.red, linear.green, linear.blue, alpha);
    }
}

impl AlphaMut for TextColor {
    fn set_alpha(&mut self, alpha: f32) {
        let color = self.0;
        self.0 = color.with_alpha(alpha);
    }
}

/// 更新任意支持 AlphaMut 类型的透明度
pub fn update_alpha<T: AlphaMut>(alpha: f32, target: &mut T) {
    target.set_alpha(alpha);
}

/// 根据计时器进度计算透明度并更新文本颜色
/// 用于淡入淡出动画
pub fn update_text_alpha_from_timer<F: bevy::ecs::query::QueryFilter>(
    timer: &Timer,
    is_fade_in: bool,
    text_query: &mut Query<&mut TextColor, F>,
) {
    let progress = timer.elapsed_secs() / timer.duration().as_secs_f32();
    let alpha = if is_fade_in {
        progress.min(1.0) // 淡入：透明度从 0 增加到 1
    } else {
        1.0 - progress.min(1.0) // 淡出：透明度从 1 减少到 0
    };

    for mut text_color in text_query.iter_mut() {
        let color = text_color.0;
        text_color.0 = color.with_alpha(alpha);
    }
}

// ==================== 菜单导航辅助函数 ====================

/// 菜单导航行为
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavigationWrap {
    /// 循环导航（最后一个选项后回到第一个）
    WrapAround,
    /// 边界停止（第一个选项不能往上，最后一个不能往下）
    Clamped,
}

/// 处理菜单导航（W/S 键）
pub fn handle_menu_navigation(
    keyboard_input: &ButtonInput<KeyCode>,
    selection: &mut usize,
    max_index: usize,
    wrap_behavior: NavigationWrap,
) -> bool {
    let mut changed = false;
    
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        if *selection > 0 {
            *selection -= 1;
            changed = true;
        } else if matches!(wrap_behavior, NavigationWrap::WrapAround) {
            *selection = max_index;
            changed = true;
        }
    }
    
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        if *selection < max_index {
            *selection += 1;
            changed = true;
        } else if matches!(wrap_behavior, NavigationWrap::WrapAround) {
            *selection = 0;
            changed = true;
        }
    }
    
    changed
}

/// 清除指定标记的所有实体
pub fn despawn_by_marker<T: Component>(commands: &mut Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}

/// 生成等间距的 Y 坐标数组（用于菜单选项）
pub fn generate_menu_y_positions(start_y: f32, step: f32, count: usize) -> Vec<f32> {
    (0..count).map(|i| start_y - (i as f32) * step).collect()
}

// ==================== 覆盖层通用函数 ====================

/// 生成覆盖层背景（全屏纯色背景）
/// 
/// # 参数
/// - `commands`: Bevy 命令队列
/// - `color`: 背景颜色
/// - `z_index`: Z 轴索引
/// - `size`: 背景尺寸
/// - `marker`: 标记组件
pub fn spawn_overlay_background<M: Component>(
    commands: &mut Commands,
    color: Color,
    z_index: f32,
    size: Vec2,
    marker: M,
) -> Entity {
    commands.spawn((
        marker,
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, z_index),
    )).id()
}

/// 检查资源是否已加载完成
/// 
/// # 参数
/// - `asset_server`: 资源服务器
/// - `fonts`: 需要检查的字体句柄列表
/// - `textures`: 需要检查的纹理句柄列表
/// 
/// # 返回
/// - `true`: 所有资源都已加载
/// - `false`: 至少有一个资源未加载
pub fn ensure_assets_loaded<'a>(
    asset_server: &AssetServer,
    fonts: &[&'a Handle<Font>],
    textures: &[&'a Handle<Image>],
) -> bool {
    fonts.iter().all(|f| asset_server.is_loaded(*f))
        && textures.iter().all(|t| asset_server.is_loaded(*t))
}

// ==================== 进度条生成 ====================

/// 生成进度条（可选使用纹理或纯色）
/// 
/// # 参数
/// - `commands`: Bevy 命令队列
/// - `texture`: 可选的纹理句柄，None 则使用纯色
/// - `color`: 纯色（texture 为 None 时使用）
/// - `position`: 位置
/// - `size`: 尺寸
/// - `z_index`: Z 轴索引
/// - `marker`: 标记组件
pub fn spawn_bar<T: Component + Clone>(
    commands: &mut Commands,
    texture: Option<Handle<Image>>,
    color: Color,
    position: Vec3,
    size: Vec2,
    z_index: f32,
    marker: T,
) -> Entity {
    let mut transform = Transform::from_translation(position);
    transform.translation.z = z_index;
    
    let sprite = if let Some(tex) = texture {
        Sprite {
            image: tex,
            custom_size: Some(size),
            ..default()
        }
    } else {
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        }
    };
    
    commands.spawn((marker, sprite, transform)).id()
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



// ==================== 通用闪烁动画系统 ====================

/// 闪烁动画配置
#[derive(Component, Clone)]
pub struct BlinkAnimation {
    /// 计时器
    pub timer: Timer,
    /// 显示颜色
    pub on_color: Color,
    /// 隐藏颜色（通常为透明）
    pub off_color: Color,
    /// 完成后的颜色
    pub final_color: Color,
    /// 是否在完成后移除组件
    pub remove_on_complete: bool,
    /// 是否在完成后销毁实体
    pub despawn_on_complete: bool,
}

impl BlinkAnimation {
    /// 创建新的闪烁动画（带销毁选项）
    /// 
    /// # 参数
    /// - `duration`: 总持续时间（秒）
    /// - `on_color`: 显示时的颜色
    /// - `off_color`: 隐藏时的颜色
    /// - `final_color`: 完成后的颜色
    /// - `despawn_on_complete`: 完成后是否销毁实体
    pub fn new_with_despawn(
        duration: f32,
        on_color: Color,
        off_color: Color,
        final_color: Color,
        despawn_on_complete: bool,
    ) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            on_color,
            off_color,
            final_color,
            remove_on_complete: false,
            despawn_on_complete,
        }
    }

    /// 创建默认的闪烁动画（金色闪烁，销毁实体）
    pub fn gold_blink_despawn(duration: f32) -> Self {
        Self::new_with_despawn(
            duration,
            crate::ui::COLOR_GOLD,
            crate::ui::COLOR_TRANSPARENT_BLACK,
            crate::ui::COLOR_WHITE,
            true,
        )
    }
    
    /// 更新闪烁动画
    /// 
    /// # 返回
    /// - `Some(颜色)`: 当前应该显示的颜色
    /// - `None`: 动画已完成
    pub fn update(&mut self, delta: std::time::Duration) -> Option<Color> {
        self.timer.tick(delta);
        
        if self.timer.is_finished() {
            None
        } else {
            let elapsed = self.timer.elapsed_secs();
            let blink_period = self.timer.duration().as_secs_f32() / 2.0;
            let cycle_progress = (elapsed % blink_period) / blink_period;
            
            Some(if cycle_progress < 0.5 {
                self.on_color
            } else {
                self.off_color
            })
        }
    }
}

/// 通用闪烁动画更新系统
///
/// 此系统更新所有带有 `BlinkAnimation` 和 `TextColor` 组件的实体。
///
/// # 闪烁逻辑
/// - 在动画持续时间内，实体会在 `on_color` 和 `off_color` 之间闪烁
/// - 闪烁周期为总持续时间的一半
/// - 周期内前 50% 显示 `on_color`，后 50% 显示 `off_color`
/// - 动画完成后，文本颜色变为 `final_color`
///
/// # 参数
/// - `time`: 时间资源
/// - `commands`: 命令队列
/// - `query`: 包含实体、BlinkAnimation 和 TextColor 的查询
///
/// # 行为
/// - 如果 `remove_on_complete` 为 true，动画完成后会移除 `BlinkAnimation` 组件
/// - 如果 `despawn_on_complete` 为 true，动画完成后会销毁整个实体
/// - 否则，`BlinkAnimation` 组件会保留，但不再更新颜色
pub fn update_blink_animations(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BlinkAnimation, &mut TextColor)>,
) {
    for (entity, mut blink, mut text_color) in query.iter_mut() {
        if let Some(color) = blink.update(time.delta()) {
            text_color.0 = color;
        } else {
            // 动画完成
            text_color.0 = blink.final_color;
            if blink.despawn_on_complete {
                commands.entity(entity).despawn();
            } else if blink.remove_on_complete {
                commands.entity(entity).remove::<BlinkAnimation>();
            }
        }
    }
}
