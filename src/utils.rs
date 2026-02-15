//! 通用工具函数模块
//!
//! 提供游戏中各模块共用的工具函数

use bevy::input::ButtonInput;
use bevy::prelude::*;
use avian2d::prelude::*;
use std::time::Duration;

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
    std::f32::consts::PI.mul_add(3.0, target_angle - current_angle) % (std::f32::consts::PI * 2.0)
        - std::f32::consts::PI
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

/// 播放一次性音效
pub fn play_one_shot_sound(
    commands: &mut Commands,
    audio_source: Handle<AudioSource>,
    volume: f32,
) -> Entity {
    commands
        .spawn((
            AudioPlayer::new(audio_source),
            PlaybackSettings::ONCE.with_volume(bevy::audio::Volume::Linear(volume)),
        ))
        .id()
}

/// 播放循环音效
pub fn play_looping_sound(
    commands: &mut Commands,
    audio_source: Handle<AudioSource>,
    volume: f32,
) -> Entity {
    commands
        .spawn((
            AudioPlayer::new(audio_source),
            PlaybackSettings::LOOP.with_volume(bevy::audio::Volume::Linear(volume)),
        ))
        .id()
}

/// 更新动画帧
///
/// 在指定帧范围内循环播放动画
///
/// # 参数
/// - `timer`: 动画计时器引用
/// - `sprite`: 精灵图引用
/// - `current_frame`: 当前帧引用
/// - `delta`: 时间增量
/// - `loop_start`: 循环起始帧
/// - `loop_end`: 循环结束帧
pub fn advance_next_frame(
    timer: &mut AnimationTimer,
    sprite: &mut Sprite,
    current_frame: &mut CurrentAnimationFrame,
    delta: Duration,
    loop_start: usize,
    loop_end: usize,
) {
    timer.tick(delta);
    if timer.just_finished() {
        let current = current_frame.0;
        let next_index = if current >= loop_end {
            loop_start
        } else {
            current + 1
        };
        current_frame.0 = next_index;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = next_index;
        }
    }
}

/// 生成动画精灵
///
/// # 参数
/// * `commands` - 命令队列
/// * `texture` - 纹理句柄
/// * `texture_atlas` - 纹理图集布局句柄（预加载）
/// * `animation_indices` - 动画索引配置
/// * `frame_time` - 每帧时间（秒）
/// * `transform` - 变换（位置、旋转、缩放）
/// * `display_size` - 显示尺寸
/// * `components` - 额外的组件
///
/// # 返回值
/// 生成的实体 ID
pub fn spawn_animated_sprite(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
    frame_time: f32,
    transform: Transform,
    display_size: Vec2,
    components: impl Bundle,
) -> Entity {
    commands
        .spawn((
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas,
                    index: animation_indices.first,
                }),
                custom_size: Some(display_size),
                ..default()
            },
            transform,
            animation_indices,
            AnimationTimer(Timer::from_seconds(frame_time, TimerMode::Repeating)),
            CurrentAnimationFrame(0),
            components,
        ))
        .id()
}

/// 停止所有坦克的速度
///
/// 用于暂停游戏或游戏结束时停止所有坦克的移动
///
/// # 参数
/// - `player_velocity_query`: 玩家坦克速度查询
/// - `enemy_velocity_query`: 敌方坦克速度查询
pub fn stop_all_tanks_velocity(
    player_velocity_query: &mut Query<&mut LinearVelocity, With<crate::constants::PlayerTank>>,
    enemy_velocity_query: &mut Query<
        &mut LinearVelocity,
        (
            With<crate::constants::EnemyTank>,
            Without<crate::constants::PlayerTank>,
        ),
    >,
) {
    // 停止玩家坦克的移动
    for mut velocity in player_velocity_query.iter_mut() {
        velocity.0 = Vec2::ZERO;
    }

    // 停止敌方坦克的移动
    for mut velocity in enemy_velocity_query.iter_mut() {
        velocity.0 = Vec2::ZERO;
    }
}

/// 从坦克旋转角度计算发射方向
///
/// # 参数
/// - `rotation`: 坦克的旋转四元数
///
/// # 返回值
/// 归一化的方向向量
pub fn calculate_direction_from_rotation(rotation: &Quat) -> Vec2 {
    let euler_angle = rotation.to_euler(EulerRot::XYZ).2;
    let actual_angle = euler_angle + crate::constants::ANGLE_OFFSET_DEGREES.to_radians();
    Vec2::new(actual_angle.cos(), actual_angle.sin())
}

/// 清理玩家关联的回城进度条
///
/// # 参数
/// - `commands`: 命令系统
/// - `progress_bar_query`: 进度条查询
/// - `player_entity`: 玩家实体
pub fn cleanup_progress_bar(
    commands: &mut Commands,
    progress_bar_query: &mut Query<(Entity, &mut Sprite, &crate::constants::RecallProgressBar)>,
    player_entity: Entity,
) {
    for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
        if progress_bar.player_entity == player_entity {
            let () = commands.entity(progress_entity).try_despawn();
        }
    }
}

/// 清理特定类型的子实体
///
/// # 参数
/// - `commands`: 命令系统
/// - `children`: 子实体列表
/// - `target_query`: 目标类型查询
pub fn cleanup_children_by_marker<T: Component>(
    commands: &mut Commands,
    children: Option<&Children>,
    target_query: &Query<(), With<T>>,
) {
    if let Some(children) = children {
        for child in children.iter() {
            if target_query.contains(child) {
                let () = commands.entity(child).try_despawn();
            }
        }
    }
}

/// 批量清理实体列表
///
/// 通用的实体清理函数，减少重复的 despawn 代码
///
/// # 参数
/// - `commands`: 命令系统
/// - `entities`: 要清理的实体迭代器
pub fn cleanup_entities(commands: &mut Commands, entities: impl IntoIterator<Item = Entity>) {
    for entity in entities {
        let () = commands.entity(entity).try_despawn();
    }
}

/// 限制实体在地图边界内
///
/// 通用的边界限制函数，用于玩家坦克、敌方坦克等各种实体
///
/// # 参数
/// - `transform`: 需要限制的实体变换
/// - `half_width`: 实体半宽（用于计算边界偏移）
/// - `half_height`: 实体半高（用于计算边界偏移）
///
/// # 示例
/// ```rust
/// clamp_entity_position(&mut transform, PLAYER_COLLIDER_HALF, PLAYER_COLLIDER_HALF);
/// ```
pub fn clamp_entity_position(transform: &mut Transform, half_width: f32, half_height: f32) {
    transform.translation.x = transform
        .translation
        .x
        .clamp(MAP_LEFT_X + half_width, MAP_RIGHT_X - half_width);
    transform.translation.y = transform
        .translation
        .y
        .clamp(MAP_BOTTOM_Y + half_height, MAP_TOP_Y - half_height);
}

/// 从两个碰撞实体中提取特定类型的实体
///
/// # 参数
/// - `e1`: 第一个碰撞实体
/// - `e2`: 第二个碰撞实体
/// - `query`: 实体查询
///
/// # 返回值
/// 返回 (匹配的实体, 另一个实体)，如果都没有匹配则返回 None
///
/// # 示例
/// ```rust
/// if let Some((bullet_entity, other_entity)) = extract_collision_pair(e1, e2, &bullets) {
///     // 处理子弹碰撞
/// }
/// ```
pub fn extract_collision_pair<D, F>(
    e1: Entity,
    e2: Entity,
    query: &Query<D, F>,
) -> Option<(Entity, Entity)>
where
    D: bevy::ecs::query::QueryData,
    F: bevy::ecs::query::QueryFilter,
{
    if query.get(e1).is_ok() {
        Some((e1, e2))
    } else if query.get(e2).is_ok() {
        Some((e2, e1))
    } else {
        None
    }
}

/// 根据树木颜色获取对应的纹理和图集资源
///
/// # 参数
/// - `tree_color`: 树木颜色（绿色或黄色）
/// - `texture_resources`: 游戏纹理资源
/// - `atlas_layouts`: 纹理图集布局资源
///
/// # 返回值
/// 返回 (纹理句柄, 图集布局句柄) 元组
pub fn get_tree_resources(
    tree_color: crate::resources::TreeColor,
    texture_resources: &crate::resources::GameTextureResources,
    atlas_layouts: &crate::resources::GameAtlasLayoutResources,
) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
    match tree_color {
        crate::resources::TreeColor::Green => {
            (
                texture_resources.forest_fire.clone(),
                atlas_layouts.forest_fire.clone(),
            )
        }
        crate::resources::TreeColor::Yellow => {
            (
                texture_resources.forest_fire_yellow.clone(),
                atlas_layouts.forest_fire_yellow.clone(),
            )
        }
    }
}

/// 根据树木颜色获取树木纹理和图集资源
///
/// # 参数
/// - `tree_color`: 树木颜色（绿色或黄色）
/// - `texture_resources`: 游戏纹理资源
/// - `atlas_layouts`: 纹理图集布局资源
///
/// # 返回值
/// 返回 (树木纹理句柄, 森林图集布局句柄) 元组
pub fn get_forest_resources(
    tree_color: crate::resources::TreeColor,
    texture_resources: &crate::resources::GameTextureResources,
    atlas_layouts: &crate::resources::GameAtlasLayoutResources,
) -> (Handle<Image>, Handle<TextureAtlasLayout>) {
    match tree_color {
        crate::resources::TreeColor::Green => {
            (texture_resources.tree.clone(), atlas_layouts.forest.clone())
        }
        crate::resources::TreeColor::Yellow => (
            texture_resources.tree_yellow.clone(),
            atlas_layouts.forest_yellow.clone(),
        ),
    }
}

/// 根据连击数获取对应的本地化文本
///
/// # 参数
/// - `combo_count`: 连击数（>= 2）
/// - `language`: 语言设置
///
/// # 返回值
/// 返回连击文本字符串
pub fn get_combo_text(combo_count: usize, language: crate::resources::Language) -> String {
    use crate::constants::LocalizedText;

    const COMBO_TEXTS: [&LocalizedText; 4] = [
        &crate::ui::localization::COMBO_FLOATING_2,
        &crate::ui::localization::COMBO_FLOATING_3,
        &crate::ui::localization::COMBO_FLOATING_4,
        &crate::ui::localization::COMBO_FLOATING_5,
    ];

    if combo_count >= 2 && combo_count <= 5 {
        COMBO_TEXTS[combo_count - 2].get(language).to_string()
    } else {
        crate::ui::localization::COMBO_FLOATING_HIGH.format(language, combo_count as u32)
    }
}
