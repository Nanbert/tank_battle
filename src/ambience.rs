//!
//! 环境氛围特效模块
//!
//! 处理森林落叶、雨滴溅射等环境特效

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use rand::Rng;

use crate::constants::*;

// ==================== 雨滴溅射常量 ====================

/// 每帧生成溅射概率（下雨时）
const RAIN_SPLASH_CHANCE: f32 = 0.5; // 50%的概率

/// 溅射粒子数量
const SPLASH_PARTICLE_COUNT: usize = 6;

/// 溅射粒子初始尺寸
const SPLASH_SIZE_INITIAL: f32 = 8.0;

/// 溅射粒子扩散速度
const SPLASH_SPREAD_SPEED: f32 = 60.0;

/// 溅射粒子生命周期（秒）
const SPLASH_LIFETIME: f32 = 0.3;

// ==================== 落叶常量 ====================

/// 落叶旋转速度范围（弧度/秒）
const LEAVES_ROTATION_SPEED_MIN: f32 = 1.0;
const LEAVES_ROTATION_SPEED_MAX: f32 = 3.0;

/// 落叶尺寸范围（放大5倍）
const LEAVES_SIZE_MIN: f32 = 40.0;   // 8 * 5
const LEAVES_SIZE_MAX: f32 = 60.0;   // 12 * 5

/// 每帧生成落叶概率
const LEAVES_SPAWN_CHANCE: f32 = 0.045; // 约4.5%的概率（放大3倍）

// ==================== 组件定义 ====================

/// 雨滴溅射粒子组件
#[derive(Component)]
pub struct RainSplashParticle {
    /// 扩散方向（角度）
    pub angle: f32,
    /// 生命周期计时器
    pub lifetime: Timer,
}

/// 落叶粒子组件
#[derive(Component)]
pub struct LeafParticle {
    /// 飘动方向（角度）
    pub drift_angle: f32,
    /// 飘动速度
    pub drift_speed: f32,
    /// 旋转速度
    pub rotation_speed: f32,
    /// 生命周期计时器
    pub lifetime: Timer,
}

// ==================== 雨滴溅射系统 ====================

/// 雨滴溅射生成系统
/// 在地图内随机位置生成溅射效果
pub fn rain_splash_spawn_system(
    mut commands: Commands,
    weather: Res<crate::weather::CurrentWeather>,
) {
    // 只在下雨时生成溅射
    if weather.weather_type != crate::weather::WeatherType::Rain {
        return;
    }

    let mut rng = rand::thread_rng();

    // 按概率决定是否生成
    if rand::random::<f32>() > RAIN_SPLASH_CHANCE {
        return;
    }

    // 在地图内随机位置生成溅射点
    let splash_x = rng.gen_range(MAP_LEFT_X..MAP_RIGHT_X);
    let splash_y = rng.gen_range(MAP_BOTTOM_Y..MAP_TOP_Y);
    let splash_position = Vec3::new(splash_x, splash_y, Z_DEFAULT);

    // 创建多个溅射粒子，向四周扩散
    for i in 0..SPLASH_PARTICLE_COUNT {
        let angle = (i as f32 / SPLASH_PARTICLE_COUNT as f32) * std::f32::consts::PI * 2.0;

        commands.spawn((
            RainSplashParticle {
                angle,
                lifetime: Timer::from_seconds(SPLASH_LIFETIME, TimerMode::Once),
            },
            Sprite {
                color: Color::srgba(0.8, 0.9, 1.0, 0.7), // 浅蓝色
                custom_size: Some(Vec2::new(SPLASH_SIZE_INITIAL, SPLASH_SIZE_INITIAL)),
                ..default()
            },
            Transform::from_translation(splash_position),
        ));
    }
}

/// 雨滴溅射粒子更新系统
/// 粒子向四周扩散并逐渐消失
pub fn rain_splash_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut RainSplashParticle)>,
) {
    for (entity, mut transform, mut sprite, mut splash) in &mut query {
        let delta = time.delta_secs();

        // 更新生命周期
        splash.lifetime.tick(time.delta());
        if splash.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // 向四周扩散
        let spread_amount = SPLASH_SPREAD_SPEED * delta;
        transform.translation.x += splash.angle.cos() * spread_amount;
        transform.translation.y += splash.angle.sin() * spread_amount;

        // 随着生命周期减少粒子大小和透明度
        let life_fraction = splash.lifetime.fraction();
        let scale = 1.0 - life_fraction; // 从1缩小到0

        if let Some(size) = sprite.custom_size.as_mut() {
            *size = Vec2::new(
                SPLASH_SIZE_INITIAL * scale,
                SPLASH_SIZE_INITIAL * scale,
            );
        }

        // 透明度逐渐降低
        sprite.color.set_alpha(0.7 * scale);
    }
}

// ==================== 落叶系统 ====================

/// 落叶生成系统
/// 当玩家在森林附近时，在地面随机位置生成，四处飘动
pub fn leaves_spawn_system(
    mut commands: Commands,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    forests: Query<&Transform, With<Forest>>,
    texture_resources: Res<crate::resources::GameTextureResources>,
    tree_color: Res<crate::resources::TreeColor>,
) {
    // 检查是否有玩家在森林附近
    let player_near_forest = player_tanks.iter().any(|player_transform| {
        forests.iter().any(|forest_transform| {
            player_transform
                .translation
                .distance(forest_transform.translation)
                < DETECTION_RADIUS
        })
    });

    if !player_near_forest {
        return;
    }

    let mut rng = rand::thread_rng();

    // 按概率决定是否生成
    if rand::random::<f32>() > LEAVES_SPAWN_CHANCE {
        return;
    }

    // 在地面随机位置生成（在森林附近范围内）
    let spawn_x = rng.gen_range(MAP_LEFT_X..MAP_RIGHT_X);
    let spawn_y = rng.gen_range(MAP_BOTTOM_Y..MAP_TOP_Y);

    // 随机落叶属性
    let size = rng.gen_range(LEAVES_SIZE_MIN..LEAVES_SIZE_MAX);
    let drift_angle = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
    let drift_speed = rng.gen_range(20.0..40.0); // 飘动速度
    let rotation_speed = rng.gen_range(LEAVES_ROTATION_SPEED_MIN..LEAVES_ROTATION_SPEED_MAX);

    // 随机初始旋转
    let initial_rotation = rng.gen_range(0.0..std::f32::consts::PI * 2.0);

    // 根据当前树木颜色选择落叶纹理
    let leaves_texture = {
        let leaves_index = rng.gen_range(0..5);
        match *tree_color {
            crate::resources::TreeColor::Green => texture_resources.leaves[leaves_index].clone(),
            crate::resources::TreeColor::Yellow => texture_resources.leaves_yellow[leaves_index].clone(),
        }
    };

    commands.spawn((
                LeafParticle {
                    drift_angle,
                    drift_speed,
                    rotation_speed,
                    lifetime: Timer::from_seconds(8.0, TimerMode::Once),
                },        Sprite {
            image: leaves_texture,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        Transform::from_translation(Vec3::new(spawn_x, spawn_y, Z_FOREST - 0.1))
            .with_rotation(Quat::from_rotation_z(initial_rotation)),
    ));
}

/// 落叶更新系统
/// 处理落叶的飘动、旋转、缩小和生命周期
pub fn leaves_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut LeafParticle)>,
) {
    for (entity, mut transform, mut sprite, mut leaf) in &mut query {
        let delta = time.delta_secs();

        // 更新生命周期
        leaf.lifetime.tick(time.delta());
        if leaf.lifetime.is_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // 向指定方向飘动
        transform.translation.x += leaf.drift_angle.cos() * leaf.drift_speed * delta;
        transform.translation.y += leaf.drift_angle.sin() * leaf.drift_speed * delta;

        // 旋转
        let current_rotation = transform.rotation.to_euler(EulerRot::XYZ).2;
        transform.rotation = Quat::from_rotation_z(current_rotation + leaf.rotation_speed * delta);

        // 随着生命周期逐渐缩小
        let life_fraction = leaf.lifetime.fraction();
        let scale = 1.0 - life_fraction; // 从1缩小到0

        if let Some(size) = sprite.custom_size.as_mut() {
            *size = Vec2::new(
                size.x * scale.powf(delta * 2.0), // 平滑缩小
                size.y * scale.powf(delta * 2.0),
            );
        }

        // 透明度逐渐降低
        sprite.color.set_alpha(1.0 * scale);

        // 超出地图范围销毁
        if transform.translation.y < MAP_BOTTOM_Y - 100.0
            || transform.translation.x < MAP_LEFT_X - 100.0
            || transform.translation.x > MAP_RIGHT_X + 100.0
        {
            commands.entity(entity).despawn();
        }
    }
}

/// 清理所有落叶粒子
pub fn cleanup_leaves(
    mut commands: Commands,
    leaves: Query<Entity, With<LeafParticle>>,
) {
    for entity in leaves.iter() {
        commands.entity(entity).despawn();
    }
}