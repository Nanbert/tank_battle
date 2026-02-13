//!
//! 天气系统模块
//!
//! 处理雨水、雪花等天气效果

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use rand::Rng;

use crate::constants::*;

// ==================== 天气常量 ====================

/// 降水粒子生成位置（地图顶部上方）
const PARTICLE_SPAWN_Y_OFFSET: f32 = 50.0;

/// 降水粒子销毁位置（地图底部下方）
const PARTICLE_DESPAWN_Y_OFFSET: f32 = 50.0;

// ==================== 天气类型 ====================

/// 天气类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherType {
    /// 无天气
    None,
    /// 下雨
    Rain,
    /// 下雪
    Snow,
}

impl Default for WeatherType {
    fn default() -> Self {
        Self::None
    }
}

impl WeatherType {
    /// 每帧生成数量
    fn particles_per_frame(self) -> usize {
        match self {
            Self::Rain => 5,
            Self::Snow => 3,
            Self::None => 0,
        }
    }

    /// 粒子尺寸
    fn particle_size(self) -> Vec2 {
        match self {
            Self::Rain => Vec2::new(3.0, 15.0),
            Self::Snow => Vec2::new(4.0, 4.0),
            Self::None => Vec2::ZERO,
        }
    }

    /// 粒子颜色
    fn particle_color(self) -> Color {
        match self {
            Self::Rain => Color::srgba(0.6, 0.7, 0.9, 0.5),
            Self::Snow => Color::srgba(1.0, 1.0, 1.0, 0.8),
            Self::None => Color::WHITE,
        }
    }

    /// 下落速度范围（像素/秒）
    fn speed_range(self) -> (f32, f32) {
        match self {
            Self::Rain => (-500.0, -300.0),
            Self::Snow => (-200.0, -100.0),
            Self::None => (0.0, 0.0),
        }
    }

    /// 雪花飘动速度
    fn sway_speed(self) -> f32 {
        match self {
            Self::Snow => 2.0,
            _ => 0.0,
        }
    }

    /// 雪花飘动幅度
    fn sway_amplitude(self) -> f32 {
        match self {
            Self::Snow => 0.5,
            _ => 0.0,
        }
    }
}

// ==================== 资源和组件 ====================

/// 当前天气资源
#[derive(Resource, Default)]
pub struct CurrentWeather {
    pub weather_type: WeatherType,
}

/// 降水粒子组件
#[derive(Component)]
pub struct PrecipitationParticle {
    /// 下落速度
    pub velocity: Vec2,
    /// 飘动偏移（仅雪花使用）
    pub sway_offset: f32,
}

/// 雨天音效播放器标记
#[derive(Component, Default)]
pub struct RainAmbiencePlayer;

// ==================== 天气系统 ====================

/// 进入 Playing 状态时设置随机天气
pub fn on_playing_enter(
    mut weather: ResMut<CurrentWeather>,
    stage_level: Res<crate::resources::StageLevel>,
    mut commands: Commands,
    texture_resources: Res<crate::resources::GameTextureResources>,
    atlas_layouts: Res<crate::resources::GameAtlasLayoutResources>,
) {
    // 第一关强制为雨天（测试雨天对着火效果的影响）
    if stage_level.0 == 1 {
        weather.weather_type = WeatherType::Rain;
        
        // 第一关在地图中央生成气垫船道具（用于测试泡泡效果）
        crate::powerup::spawn_powerup(
            &mut commands,
            &texture_resources,
            &atlas_layouts,
            crate::powerup_strategy::PowerUp::AirCushion,
            bevy::prelude::Vec3::new(0.0, 0.0, crate::constants::Z_FOREST),
        );
    } else {
        let mut rng = rand::rng();
        let weather_options = [WeatherType::None, WeatherType::Rain, WeatherType::Snow];
        weather.weather_type = weather_options[rng.random_range(0..weather_options.len())];
    }

    info!("关卡 {} 天气: {:?}", stage_level.0, weather.weather_type);
}

/// 离开 Playing 状态时清除天气
pub fn on_playing_exit(
    mut weather: ResMut<CurrentWeather>,
    particle_query: Query<Entity, With<PrecipitationParticle>>,
    rain_ambience_players: Query<(Entity, &mut AudioPlayer), With<RainAmbiencePlayer>>,
    mut commands: Commands,
) {
    weather.weather_type = WeatherType::None;

    for entity in particle_query.iter() {
        commands.entity(entity).despawn();
    }

    crate::utils::cleanup_entities(&mut commands, rain_ambience_players.iter().map(|(e, _)| e));
}

/// 降水生成系统
pub fn precipitation_spawn_system(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
) {
    spawn_precipitation(&mut commands, weather.weather_type);
}

/// 降水更新系统
pub fn precipitation_update_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut PrecipitationParticle)>,
    weather: Res<CurrentWeather>,
) {
    for (entity, mut transform, mut particle) in &mut query {
        // 更新位置
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();

        // 雪花有飘动效果
        let sway = weather.weather_type.sway_speed() * particle.sway_offset.sin() * weather.weather_type.sway_amplitude();
        if sway != 0.0 {
            particle.sway_offset += time.delta_secs() * weather.weather_type.sway_speed();
            transform.translation.x += sway;
        }

        // 超出底部销毁
        if transform.translation.y < MAP_BOTTOM_Y - PARTICLE_DESPAWN_Y_OFFSET {
            commands.entity(entity).despawn();
        }
    }
}

// ==================== 辅助函数 ====================

/// 生成降水粒子
fn spawn_precipitation(commands: &mut Commands, weather_type: WeatherType) {
    let count = weather_type.particles_per_frame();
    if count == 0 {
        return;
    }

    let mut rng = rand::rng();
    let size = weather_type.particle_size();
    let color = weather_type.particle_color();
    let speed_range = weather_type.speed_range();

    for _ in 0..count {
        let x = rng.random_range(MAP_LEFT_X..MAP_RIGHT_X);
        let y = MAP_TOP_Y + PARTICLE_SPAWN_Y_OFFSET;
        let velocity = Vec2::new(0.0, rng.random_range(speed_range.0..speed_range.1));
        let sway_offset = rng.random_range(0.0..std::f32::consts::PI * 2.0);

        commands.spawn((
            PrecipitationParticle {
                velocity,
                sway_offset,
            },
            Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, Z_RAIN)),
        ));
    }
}

/// 播放雨天音效系统
pub fn play_rain_ambience(
    mut commands: Commands,
    weather: Res<CurrentWeather>,
    audio_resources: Res<crate::resources::GameAudioResources>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<RainAmbiencePlayer>>,
) {
    if weather.weather_type == WeatherType::Rain && ambience_players.is_empty() {
        let entity = crate::utils::play_looping_sound(
            &mut commands,
            audio_resources.rain.clone(),
            crate::constants::VOLUME_FULL,
        );
        commands.entity(entity).insert(RainAmbiencePlayer::default());
    } else if weather.weather_type != WeatherType::Rain {
        crate::utils::cleanup_entities(&mut commands, ambience_players.iter().map(|(e, _)| e));
    }
}