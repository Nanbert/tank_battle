//! 通用工具函数模块
//!
//! 提供游戏中各模块共用的工具函数

use bevy::input::ButtonInput;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
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
    
    std::f32::consts::PI.mul_add(3.0, target_angle - current_angle)
        % (std::f32::consts::PI * 2.0)
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

/// 处理精灵图动画播放
///
/// 更新动画计时器，并在计时器完成时切换到下一帧
///
/// # 参数
/// - `timer`: 动画计时器引用
/// - `sprite`: 精灵图引用
/// - `indices`: 动画索引配置
/// - `current_frame`: 当前帧引用
/// - `delta`: 时间增量
pub fn animate_sprite(
    timer: &mut AnimationTimer,
    sprite: &mut Sprite,
    indices: &AnimationIndices,
    current_frame: &mut CurrentAnimationFrame,
    delta: Duration,
) {
    timer.tick(delta);
    if timer.just_finished() {
        let current = current_frame.0;
        let next_index = if current == indices.last {
            indices.first
        } else {
            current + 1
        };
        current_frame.0 = next_index;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = next_index;
        }
    }
}

/// 添加纹理图集到资源管理器
///
/// 创建纹理图集布局并添加到 Assets 中，返回句柄
///
/// # 参数
/// - `texture_atlas_layouts`: 纹理图集布局资源管理器
/// - `tile_size`: 单个图块大小
/// - `columns`: 列数
/// - `rows`: 行数
///
/// # 返回值
/// 纹理图集布局句柄
pub fn add_texture_atlas(
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    tile_size: UVec2,
    columns: u32,
    rows: u32,
) -> Handle<TextureAtlasLayout> {
    let layout = create_texture_atlas(tile_size, columns, rows);
    texture_atlas_layouts.add(layout)
}

/// 生成带动画的精灵
///
/// 统一处理纹理图集加载、动画索引和计时器的创建
///
/// # 参数
/// - `commands`: 命令系统
/// - `texture_atlas_layouts`: 纹理图集布局资源管理器
/// - `texture`: 纹理句柄
/// - `tile_size`: 单个图块大小
/// - `columns`: 列数
/// - `rows`: 行数
/// - `animation_indices`: 动画索引配置
/// - `frame_time`: 每帧时间（秒）
/// - `position`: 位置
/// - `size`: 显示大小（如果为 None 则使用 tile_size）
/// - `components`: 额外的组件
///
/// # 返回值
/// 生成的实体 ID
pub fn spawn_animated_sprite(
    commands: &mut Commands,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    texture: Handle<Image>,
    tile_size: UVec2,
    columns: u32,
    rows: u32,
    animation_indices: AnimationIndices,
    frame_time: f32,
    position: Vec3,
    size: Option<Vec2>,
    components: impl Bundle,
) -> Entity {
    let texture_atlas = add_texture_atlas(texture_atlas_layouts, tile_size, columns, rows);
    let custom_size = size.unwrap_or_else(|| Vec2::new(tile_size.x as f32, tile_size.y as f32));

    commands
        .spawn((
            Sprite {
                image: texture,
                texture_atlas: Some(TextureAtlas {
                    layout: texture_atlas,
                    index: animation_indices.first,
                }),
                custom_size: Some(custom_size),
                ..default()
            },
            Transform::from_translation(position),
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
    player_velocity_query: &mut Query<&mut Velocity, With<crate::constants::PlayerTank>>,
    enemy_velocity_query: &mut Query<&mut Velocity, (With<crate::constants::EnemyTank>, Without<crate::constants::PlayerTank>)>,
) {
    // 停止玩家坦克的移动
    for mut velocity in player_velocity_query.iter_mut() {
        velocity.linvel = Vec2::ZERO;
    }

    // 停止敌方坦克的移动
    for mut velocity in enemy_velocity_query.iter_mut() {
        velocity.linvel = Vec2::ZERO;
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