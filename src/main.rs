//! A simplified implementation of the classic game "Battle City 1990"
//!
//!
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
#![allow(clippy::cast_precision_loss)]

mod constants;
mod resources;
mod map;
mod levels;
mod bullet;
mod laser;

use bevy::{
    audio::{AudioPlayer, Volume},
    prelude::*,
    window::{
        PresentMode,
        WindowTheme,
    },
};
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::RecoilForce;

#[allow(clippy::wildcard_imports)]
use constants::*;
#[allow(clippy::wildcard_imports)]
use resources::*;
use crate::bullet::BulletOwner;




fn configure_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "For Communism!".into(),
            name: Some("bevy.app".into()),
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            resolution: (WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).into(),
            present_mode: PresentMode::AutoVsync,
            fit_canvas_to_parent: true,
            prevent_default_event_handling: false,
            window_theme: Some(WindowTheme::Dark),
            enabled_buttons: bevy::window::EnabledButtons {
                maximize: false,
                ..Default::default()
            },
            visible: false,
            ..default()
        }),
        ..default()
    }
}

fn configure_asset_plugin() -> AssetPlugin {
    // 检查是否在系统安装目录运行
    let is_system_install = std::path::Path::new("/usr/share/tank-battle/assets").exists();
    let asset_path = if is_system_install {
        "/usr/share/tank-battle/assets".to_string()
    } else {
        "assets".to_string()
    };

    AssetPlugin {
        file_path: asset_path,
        unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
        ..default()
    }
}

fn configure_game_resources(app: &mut App) {
    app.init_state::<GameState>()
        .add_message::<PlayerStatChanged>()
        .add_message::<crate::bullet::EffectEvent>()
        .init_resource::<BulletTracker>()
        .init_resource::<StartAnimationFrames>()
        .init_resource::<FadingOut>()
        .init_resource::<CurrentMenuSelection>()
        .init_resource::<GameMode>()
        .init_resource::<MenuBlinkTimer>()
        .init_resource::<StageIntroTimer>()
        .init_resource::<EnemySpawnState>()
        .init_resource::<StageLevel>()
        .init_resource::<PlayerInfo>()
        .init_resource::<RecallTimers>()
        .init_resource::<DashTimers>()
        .init_resource::<BlueBarRegenTimer>()
        .init_resource::<CommanderLife>()
        .init_resource::<BulletTracker>()
        .init_resource::<GameEntitiesSpawned>()
        .init_resource::<BarrierDamageTracker>()
        .init_resource::<DashDamageTracker>()
        .insert_resource(PlayerRespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)))
        .insert_resource(ClearColor(BACKGROUND_COLOR));
}

fn register_game_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::StartScreen), (cleanup_playing_entities, spawn_start_screen).chain())
        .add_systems(OnEnter(GameState::FadingOut), setup_fade_out)
        .add_systems(OnEnter(GameState::StageIntro), (reset_for_next_stage, spawn_stage_intro).chain())
        .add_systems(Update, handle_stage_intro_timer.run_if(in_state(GameState::StageIntro)))
        .add_systems(OnExit(GameState::StageIntro), despawn_stage_intro)
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities_if_needed)
        .add_systems(OnEnter(GameState::Paused), spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), ( despawn_pause_ui,))
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), (despawn_game_over_ui, cleanup_playing_entities))
        .add_systems(OnEnter(GameState::About), (cleanup_start_screen_ui, spawn_about_screen).chain())
        .add_systems(OnExit(GameState::About), (despawn_about_screen, spawn_start_screen).chain())
        .add_systems(Update, handle_about_input.run_if(in_state(GameState::About)))
        .add_systems(OnEnter(GameState::Credits), (cleanup_start_screen_ui, spawn_credits_screen).chain())
        .add_systems(OnExit(GameState::Credits), (despawn_credits_screen, spawn_start_screen).chain())
        .add_systems(Update, handle_credits_input.run_if(in_state(GameState::Credits)))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_enemy_tanks).chain().run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy_spawn_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, move_player_tank.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_player_tank_texture.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_enemy_tank_texture.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_player_avatar.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_commander.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_powerup_texture.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_player_info_text.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_explosion.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_laser.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_forest_fire.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_forest.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_sea.run_if(in_state(GameState::Playing)))
        .add_systems(Update, play_sea_ambience.run_if(in_state(GameState::Playing)))
        .add_systems(Update, play_tree_ambience.run_if(in_state(GameState::Playing)))
        .add_systems(Update, play_commander_music.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_commander_music.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_spark.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_enemy_born_animation.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_game_over_delay.run_if(in_state(GameState::Playing)))
        .add_systems(Update, check_game_over.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::enemy_shoot_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::player_shoot_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_bounds_check_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_despawn_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_terrain_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_tank_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_commander_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::handle_effect_events.run_if(in_state(GameState::Playing)))
        .add_systems(Update, laser::player_laser_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_powerup_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_air_cushion_effect.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_stat_changed_for_blink.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_player_info_display.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_blue_bar_regen.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_commander_health_bar.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_enemy_count_display.run_if(in_state(GameState::Playing)))
        .add_systems(Update, check_stage_complete.run_if(in_state(GameState::Playing)))
        // .add_systems(Update, check_bullet_commander_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_start_screen.run_if(not(in_state(GameState::Playing))))
        .add_systems(Update, (
            handle_start_screen_input,
            update_option_colors,
        ).run_if(in_state(GameState::StartScreen)))
        .add_systems(Update, update_menu_blink.run_if(in_state(GameState::FadingOut).or(in_state(GameState::StartScreen))))
        .add_systems(Update, handle_game_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_pause_input.run_if(in_state(GameState::Paused)))
        .add_systems(Update, (handle_game_over_input, update_option_colors)
            .chain().run_if(in_state(GameState::GameOver)))
        .add_systems(Update, (
            handle_recall_input,
            update_recall_timers,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_dash_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_dash_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_dash_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_barrier_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, update_recall_progress_bars.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_recoil_force.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_laser.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_smoke.run_if(in_state(GameState::Playing)))
        .add_systems(Update, laser_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, fade_out_screen.run_if(in_state(GameState::FadingOut)));
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(configure_window_plugin())
            .set(configure_asset_plugin())
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::RenderCreation::Automatic(
                    bevy::render::settings::WgpuSettings {
                        backends: Some(bevy::render::settings::Backends::all()),
                        ..default()
                    },
                ),
                ..default()
            }),
    ))
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

    configure_game_resources(&mut app);
    register_game_systems(&mut app);

    app.run();
}

fn load_start_animation_assets(
    asset_server: &Res<AssetServer>,
    animation_frames: &mut ResMut<StartAnimationFrames>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) {
    // 使用3个较小的精灵图加载背景动画（15帧，每部分5帧）
    // 拆分以支持GPU纹理尺寸限制（最大16384）
    let background_texture1: Handle<Image> = asset_server.load("background/background_sprite_part1.png");
    let background_texture2: Handle<Image> = asset_server.load("background/background_sprite_part2.png");
    let background_texture3: Handle<Image> = asset_server.load("background/background_sprite_part3.png");

    let background_tile_size = UVec2::new(2060, 1300); // 每帧的尺寸（窗口大小）

    // 创建3个纹理图集，每个5帧
    let atlas1 = TextureAtlasLayout::from_grid(background_tile_size, 5, 1, None, None);
    let atlas2 = TextureAtlasLayout::from_grid(background_tile_size, 5, 1, None, None);
    let atlas3 = TextureAtlasLayout::from_grid(background_tile_size, 5, 1, None, None);

    let layout1 = texture_atlas_layouts.add(atlas1);
    let layout2 = texture_atlas_layouts.add(atlas2);
    let layout3 = texture_atlas_layouts.add(atlas3);

    // 存储到资源中
    animation_frames.texture_atlas_layouts = vec![layout1, layout2, layout3];
    animation_frames.textures = vec![background_texture1, background_texture2, background_texture3];
}

fn get_animation_frame(
    frame_index: usize,
    asset_server: &Res<AssetServer>,
    animation_frames: &mut ResMut<StartAnimationFrames>,
) -> Handle<Image> {
    // 如果该帧已加载，直接返回
    if frame_index < animation_frames.frames.len() {
        return animation_frames.frames[frame_index].clone();
    }

    // 否则加载该帧
    let frame_num = format!("{:04}", frame_index + 1);
    let texture = asset_server.load(format!("start_scene/frame_{frame_num}.png"));
    animation_frames.frames.push(texture.clone());
    texture
}

fn spawn_start_screen_background(
    commands: &mut Commands,
    animation_frames: &ResMut<StartAnimationFrames>,
) {
    let animation_indices = AnimationIndices { first: 0, last: 14 };

    // 使用第一个纹理和图集初始化
    let texture_atlas_layout = animation_frames.texture_atlas_layouts[0].clone();
    let texture = animation_frames.textures[0].clone();

    commands.spawn((
        StartScreenUI,
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));
}

fn spawn_start_screen_title(
    commands: &mut Commands,
    font: Handle<Font>,
) {
    commands.spawn((
        StartScreenUI,
        Text2d("For Communism!!".to_string()),
        TextFont {
            font_size: 60.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 400.0, 1.0),
    ));

    // 1 Player 选项
    commands.spawn((
        StartScreenUI,
        Text2d("1 Player".to_string()),
        TextFont {
            font_size: 80.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 初始选中，黄色
        Transform::from_xyz(0.0, 50.0, 1.0),
        MenuOption { index: 0 },
    ));

    // 2 Player 选项
    commands.spawn((
        StartScreenUI,
        Text2d("2 Player".to_string()),
        TextFont {
            font_size: 80.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -50.0, 1.0),
        MenuOption { index: 1 },
    ));

    // About 选项
    commands.spawn((
        StartScreenUI,
        Text2d("About".to_string()),
        TextFont {
            font_size: 80.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -150.0, 1.0),
        MenuOption { index: 2 },
    ));

    // Credits 选项
    commands.spawn((
        StartScreenUI,
        Text2d("Credits".to_string()),
        TextFont {
            font_size: 80.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -250.0, 1.0),
        MenuOption { index: 3 },
    ));

    // EXIT 选项
    commands.spawn((
        StartScreenUI,
        Text2d("EXIT".to_string()),
        TextFont {
            font_size: 80.0,
            font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -350.0, 1.0),
        MenuOption { index: 4 },
    ));
}

fn spawn_start_screen_instructions(commands: &mut Commands, font: &Handle<Font>) {
    // 玩家1操作说明
    commands.spawn((
        StartScreenUI,
        Text2d("Player 1 (Li Yun Long): WASD to move | J to shoot | I to recall | K to dash | L to laser".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(Color::srgb(0.0, 0.5, 1.0)), // 蓝色
        Transform::from_xyz(0.0, -450.0, 1.0),
    ));

    // 玩家2操作说明
    commands.spawn((
        StartScreenUI,
        Text2d("Player 2 (Chu Yun Fei): Arrow Keys to move | 1 to shoot | 4 to recall | 2 to dash | 3 to laser".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)), // 红色
        Transform::from_xyz(0.0, -480.0, 1.0),
    ));

    // 通用操作说明
    commands.spawn((
        StartScreenUI,
        Text2d("W/S to select | SPACE to select/pause | ESC to exit".to_string()),
        TextFont {
            font_size: 20.0,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色
        Transform::from_xyz(0.0, -510.0, 1.0),
    ));
}

fn spawn_start_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_frames: ResMut<StartAnimationFrames>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // 加载所有动画帧
    load_start_animation_assets(&asset_server, &mut animation_frames, &mut texture_atlas_layouts);

    // 添加动画背景
    spawn_start_screen_background(&mut commands, &animation_frames);

    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load(crate::FONT_EN);

    // 添加标题文字
    spawn_start_screen_title(&mut commands, custom_font.clone());

    // 添加操作说明
    spawn_start_screen_instructions(&mut commands, &custom_font);
}

fn spawn_about_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load(crate::FONT_EN);

    // 添加白色背景覆盖
    commands.spawn((
        AboutUI,
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 添加标题
    commands.spawn((
        AboutUI,
        Text2d("About".to_string()),
        TextFont {
            font_size: 70.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 600.0, 1.0),
    ));

    // 添加个人图片占位符（你可以替换为实际图片路径）
    // let profile_image: Handle<Image> = asset_server.load("path/to/your/image.png");
    // commands.spawn((
    //     AboutUI,
    //     Sprite {
    //         image: profile_image,
    //         custom_size: Some(Vec2::new(200.0, 200.0)),
    //         ..default()
    //     },
    //     Transform::from_xyz(0.0, 200.0, 1.0),
    // ));

    // 使用多行文本显示所有信息
    let about_text = "Nanbert\n\n\
        Email: 2726905171@qq.com\n\n\
        Copyright © 2025 Nanbert\n\
        All rights reserved.\n\n\
        This is a tank battle game inspired by Battle City 1990.\n\
        Built with Rust and Bevy game engine.\n\n\
        Special thanks to iFlow for invaluable assistance.\n\n\
        License: MIT License";

    commands.spawn((
        AboutUI,
        Text2d(about_text.to_string()),
        TextFont {
            font_size: 24.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        TextLayout::new_with_justify(bevy::text::Justify::Center),
        Transform::from_xyz(0.0, 350.0, 1.0),
    ));

    // 添加收款码文案
    let support_text = "If you enjoyed the game,\nplease buy me a coffee! ☕️\n(Caffeine is a programmer's fuel)";

    commands.spawn((
        AboutUI,
        Text2d(support_text.to_string()),
        TextFont {
            font_size: 22.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        TextLayout::new_with_justify(bevy::text::Justify::Center),
        Transform::from_xyz(0.0, 50.0, 1.0),
    ));

    // 加载收款码图片
    let alipay_image: Handle<Image> = asset_server.load("alipay.png");
    let wechat_image: Handle<Image> = asset_server.load("wechat.png");

    // 图片大小统一为 400x400 像素
    let qr_size = 400.0;

    // 支付宝收款码
    commands.spawn((
        AboutUI,
        Sprite {
            image: alipay_image,
            custom_size: Some(Vec2::new(qr_size, qr_size)),
            ..default()
        },
        Transform::from_xyz(-250.0, -250.0, 1.0),
    ));

    // 支付宝标签
    commands.spawn((
        AboutUI,
        Text2d("Alipay".to_string()),
        TextFont {
            font_size: 18.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(-250.0, -470.0, 1.0),
    ));

    // 微信收款码
    commands.spawn((
        AboutUI,
        Sprite {
            image: wechat_image,
            custom_size: Some(Vec2::new(qr_size, qr_size)),
            ..default()
        },
        Transform::from_xyz(250.0, -250.0, 1.0),
    ));

    // 微信标签
    commands.spawn((
        AboutUI,
        Text2d("WeChat".to_string()),
        TextFont {
            font_size: 18.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(250.0, -470.0, 1.0),
    ));

    // 添加返回提示
    commands.spawn((
        AboutUI,
        Text2d("Press SPACE to return".to_string()),
        TextFont {
            font_size: 22.0,
            font: custom_font,
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, -550.0, 1.0),
    ));
}

fn despawn_about_screen(
    mut commands: Commands,
    query: Query<Entity, With<AboutUI>>,
) {
    for entity in query.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
}

fn handle_about_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Space 键返回开始界面
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::StartScreen);
    }
}

fn spawn_credits_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load(crate::FONT_EN);

    // 添加白色背景覆盖
    commands.spawn((
        CreditsUI,
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 添加标题
    commands.spawn((
        CreditsUI,
        Text2d("Credits".to_string()),
        TextFont {
            font_size: 80.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 500.0, 1.0),
    ));

    // 使用多行文本显示素材来源
    let credits_text = "Asset Credits\n\n\
        OpenGameArt.org:\n\
        • Bubbles by HorrorPen (CC-BY 3.0)\n\
        • Explosion by Sinestesia (CC0 1.0)\n\
        • Laser by netcake3 (CC-BY-SA 3.0/4.0)\n\
        • Enemy Born by JoesAlotofthings (CC-BY 4.0)\n\
        • Player/Enemy Tanks by irmirx (CC-BY 3.0)\n\
        • Smoke by Skorpio (CC-BY 3.0)\n\
        • Steel Hit by Sinestesia (CC0 1.0)\n\
        • Bullets by Wenrexa (CC0 1.0)\n\n\
        通义千问 (AI Generated):\n\
        • Background, Music Notes (CC0 1.0)\n\
        • Maps (Brick, Steel, Sea, Tree, Barrier) (CC0 1.0)\n\
        • Power-ups (10 types) (CC0 1.0)\n\
        • Avatars & Commander (CC0 1.0)\n\n\
        Fonts:\n\
        • ChelaOne by Latinotype\n\
        • Corben\n\
        • Matemasie\n\
        • LiuHuanKaTongShouShu by 刘欢\n\n\
        See COPYRIGHT file for full details.";

    commands.spawn((
        CreditsUI,
        Text2d(credits_text.to_string()),
        TextFont {
            font_size: 24.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        TextLayout::new_with_justify(bevy::text::Justify::Left),
        Transform::from_xyz(-400.0, 100.0, 1.0),
    ));

    // 添加返回提示
    commands.spawn((
        CreditsUI,
        Text2d("Press SPACE to return".to_string()),
        TextFont {
            font_size: 30.0,
            font: custom_font,
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, -500.0, 1.0),
    ));
}

fn despawn_credits_screen(
    mut commands: Commands,
    query: Query<Entity, With<CreditsUI>>,
) {
    for entity in query.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
}

fn handle_credits_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Space 键返回开始界面
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::StartScreen);
    }
}


fn spawn_walls(commands: &mut Commands) {
    // 左墙（在原游戏区域左边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, MAP_TOP_Y / 100.0),
        Transform{
            translation: Vec3::new(MAP_LEFT_X - 5.0, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(10.0 , MAP_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 右墙（在原游戏区域右边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, MAP_TOP_Y / 100.0),
        Transform{
            translation: Vec3::new(MAP_RIGHT_X + 5.0, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(10.0 , MAP_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 上墙（在原游戏区域上边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(MAP_RIGHT_X / 100.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, MAP_TOP_Y + 5.0, 0.0),
            scale: Vec3::new(MAP_WIDTH, 10.0, 1.0),
            ..default()
        }
    ));

    // 下墙（在原游戏区域下边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(MAP_RIGHT_X / 100.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, MAP_BOTTOM_Y -5.0, 0.0),
            scale: Vec3::new(MAP_WIDTH, 10.0 , 1.0),
            ..default()
        }
    ));
}

fn spawn_map_terrain(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    stage_level: usize,
) {
    use crate::map::{TerrainType, grid_to_world, MAP_ROWS, MAP_COLS};

    let level_map = crate::levels::get_level(stage_level);

    for row in 0..MAP_ROWS {
        for col in 0..MAP_COLS {
            let terrain = level_map[row][col];
            if terrain == TerrainType::Empty {
                continue;
            }

            let pos = grid_to_world(row, col);

            match terrain {
                TerrainType::Forest => {
                    let forest_texture: Handle<Image> = asset_server.load("maps/tree.png");
                    let forest_tile_size = UVec2::new(131, 131);
                    let forest_texture_atlas = TextureAtlasLayout::from_grid(forest_tile_size, 10, 1, None, None);
                    let forest_texture_atlas_layout = texture_atlas_layouts.add(forest_texture_atlas);
                    let forest_animation_indices = AnimationIndices { first: 0, last: 9 };

                    commands.spawn((
                        Forest,
                        PlayingEntity,
                        Sprite::from_atlas_image(
                            forest_texture,
                            TextureAtlas {
                                layout: forest_texture_atlas_layout,
                                index: forest_animation_indices.first,
                            }
                        ),
                        Transform::from_xyz(pos.x, pos.y, 1.0),
                        forest_animation_indices,
                        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                        CurrentAnimationFrame(0),
                        Collider::cuboid(131.0 / 2.0, 131.0 / 2.0),
                        RigidBody::Fixed,
                        Sensor,
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                    ));
                }
                TerrainType::Sea => {
                    let sea_texture: Handle<Image> = asset_server.load(TEXTURE_SEA);
                    let sea_tile_size = UVec2::new(100, 100);
                    let sea_texture_atlas = TextureAtlasLayout::from_grid(sea_tile_size, 3, 1, None, None);
                    let sea_texture_atlas_layout = texture_atlas_layouts.add(sea_texture_atlas);
                    let sea_animation_indices = AnimationIndices { first: 0, last: 2 };

                    commands.spawn((
                        Sea,
                        PlayingEntity,
                        Sprite::from_atlas_image(
                            sea_texture,
                            TextureAtlas {
                                layout: sea_texture_atlas_layout,
                                index: sea_animation_indices.first,
                            }
                        ),
                        Transform::from_xyz(pos.x, pos.y, -0.5),
                        sea_animation_indices,
                        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                        CurrentAnimationFrame(0),
                        RigidBody::Fixed,
                        Collider::cuboid(100.0 / 2.0, 100.0 / 2.0),
                        CollisionGroups::new(SEA_GROUP, Group::all()),
                    ));
                }
                TerrainType::Brick => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    // 生成4块砖块组成100x100的网格
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickLeft => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(-offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickRight => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(offset, offset),
                        Vec2::new(offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickTop => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickBottom => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::Steel => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    // 生成4块钢铁组成100x100的网格
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelLeft => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(-offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelRight => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(offset, offset),
                        Vec2::new(offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelTop => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelBottom => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::Barrier => {
                    let barrier_texture: Handle<Image> = asset_server.load(TEXTURE_BARRIER);
                    commands.spawn((
                        Barrier,
                        PlayingEntity,
                        Sprite {
                            image: barrier_texture,
                            custom_size: Some(Vec2::new(BARRIER_WIDTH, BARRIER_HEIGHT)),
                            ..default()
                        },
                        Transform::from_xyz(pos.x, pos.y, 0.0),
                        RigidBody::Fixed,
                        Collider::cuboid(BARRIER_WIDTH / 2.0, BARRIER_HEIGHT / 2.0),
                        Sensor,
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
                    ));
                }
                TerrainType::Empty => {}
            }
        }
    }
}






fn spawn_commander(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) {
    let commander_texture: Handle<Image> = asset_server.load(TEXTURE_COMMANDER);
    // commander.png 实际尺寸: 1400x1200, 每帧 140x120, 10列 x 10行, 共100帧
    let commander_tile_size = UVec2::new(140, 120);
    let commander_texture_atlas = TextureAtlasLayout::from_grid(commander_tile_size, 10, 10, None, None);
    let commander_texture_atlas_layout = texture_atlas_layouts.add(commander_texture_atlas);
    let commander_animation_indices = AnimationIndices { first: 0, last: 99 };

    let commander_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT / 2.0;
    let commander_x = 0.0;

    // 创建包围司令官的砖块堡垒墙
    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
    let brick_size = 50.0; // 每块砖的大小

    // 司令官边界
    let commander_left = -COMMANDER_WIDTH / 2.0;
    let commander_right = COMMANDER_WIDTH / 2.0;
    let commander_top = commander_y + COMMANDER_HEIGHT / 2.0;
    let commander_bottom = commander_y - COMMANDER_HEIGHT / 2.0;

    // 左墙：3块砖，紧贴司令官左侧
    for i in 0..3 {
        let y = commander_bottom + brick_size / 2.0 + i as f32 * brick_size;
        commands.spawn((
            Brick,
            PlayingEntity,
            Sprite {
                image: brick_texture.clone(),
                custom_size: Some(Vec2::new(brick_size, brick_size)),
                ..default()
            },
            Transform::from_xyz(commander_left - brick_size / 2.0, y, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(brick_size / 2.0, brick_size / 2.0),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::all(),
        ));
    }

    // 右墙：3块砖，紧贴司令官右侧
    for i in 0..3 {
        let y = commander_bottom + brick_size / 2.0 + i as f32 * brick_size;
        commands.spawn((
            Brick,
            PlayingEntity,
            Sprite {
                image: brick_texture.clone(),
                custom_size: Some(Vec2::new(brick_size, brick_size)),
                ..default()
            },
            Transform::from_xyz(commander_right + brick_size / 2.0, y, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(brick_size / 2.0, brick_size / 2.0),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::all(),
        ));
    }

    // 上墙：2块砖封顶，紧贴司令官顶部
    for i in 0..2 {
        let x = -brick_size / 2.0 + i as f32 * brick_size;
        commands.spawn((
            Brick,
            PlayingEntity,
            Sprite {
                image: brick_texture.clone(),
                custom_size: Some(Vec2::new(brick_size, brick_size)),
                ..default()
            },
            Transform::from_xyz(x, commander_top + brick_size / 2.0, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(brick_size / 2.0, brick_size / 2.0),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::all(),
        ));
    }

    commands.spawn((
        Commander,
        PlayingEntity,
        Sprite {
            image: commander_texture,
            texture_atlas: Some(TextureAtlas {
                layout: commander_texture_atlas_layout,
                index: commander_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(COMMANDER_WIDTH, COMMANDER_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(commander_x, commander_y, 0.0),
        commander_animation_indices,
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
        RigidBody::Fixed,
        Collider::cuboid(COMMANDER_WIDTH / 2.0, COMMANDER_HEIGHT / 2.0),
        ActiveEvents::COLLISION_EVENTS,
    ));

    // 创建音乐动画精灵（一直播放）
    let music_texture: Handle<Image> = asset_server.load(TEXTURE_MUSIC_NOTE);
    let music_tile_size = UVec2::new(140, 120);
    let music_texture_atlas = TextureAtlasLayout::from_grid(music_tile_size, 10, 1, None, None);
    let music_texture_atlas_layout = texture_atlas_layouts.add(music_texture_atlas);
    let music_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        CommanderMusicAnimation,
        PlayingEntity,
        Sprite {
            image: music_texture,
            texture_atlas: Some(TextureAtlas {
                layout: music_texture_atlas_layout,
                index: music_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(70.0, 60.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(commander_x, commander_y, 1.0)), // z=1.0 使动画在 Commander 上方
        music_animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // 每0.1秒切换一帧
        CurrentAnimationFrame(0),
    ));
}

fn spawn_player1_tank(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
) -> Entity {
    let player_tank = PlayerTank { tank_type: TankType::Player1 };

    

    commands.spawn_empty()
        .insert(player_tank)
        .insert(PlayingEntity)
        .insert(TankFireConfig::default())
        .insert(RotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))
        .insert(TargetRotation { angle: 0.0_f32.to_radians() })
        .insert(Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            }),
            custom_size: Some(Vec2::new(80.0, 90.0)),
            ..default()
        })
        .insert(Transform::from_xyz(-TANK_WIDTH / 2.0 - COMMANDER_WIDTH/2.0 - 50.0, MAP_BOTTOM_Y+TANK_HEIGHT / 2.0, 0.0))
        .insert(Velocity{ linvel: Vec2::default(), angvel: 0.0 })
        .insert(animation_indices)
        .insert(AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(35.0, 35.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC | ActiveCollisionTypes::KINEMATIC_KINEMATIC)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            filter_groups: None,
            ..default()
        })
        .id()
}

pub fn spawn_enemy_born_animation(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) -> Entity {
    let enemy_born_texture: Handle<Image> = asset_server.load(TEXTURE_ENEMY_BORN);
    let enemy_born_tile_size = UVec2::new(192, 192);
    let enemy_born_texture_atlas = TextureAtlasLayout::from_grid(enemy_born_tile_size, 5, 4, None, None);
    let enemy_born_texture_atlas_layout = texture_atlas_layouts.add(enemy_born_texture_atlas);
    let enemy_born_animation_indices = AnimationIndices { first: 0, last: 12 };

    commands.spawn((
        EnemyBornAnimation,
        PlayingEntity,
        Sprite {
            image: enemy_born_texture,
            texture_atlas: Some(TextureAtlas {
                layout: enemy_born_texture_atlas_layout,
                index: enemy_born_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        Transform::from_translation(position),
        enemy_born_animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
        BornPosition(position), // 记录出生位置
    )).id()
}

// 记录出生位置的组件
#[derive(Component)]
pub struct BornPosition(pub Vec3);

// 回城进度条组件
#[derive(Component)]
pub struct RecallProgressBar {
    pub player_type: TankType,
    pub player_entity: Entity,
}

// 玩家正在回城标记
#[derive(Component)]
pub struct IsRecalling;

fn spawn_player_info(
    commands: &mut Commands,
    font: &Handle<Font>,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &PlayerInfo,
) {
    // 生成玩家1 UI 元素
    for config in PLAYER1_UI_ELEMENTS {
        spawn_ui_element_from_config(commands, font, asset_server, texture_atlas_layouts, config, player_info, TankType::Player1);
    }
    // 生成玩家2 UI 元素
    for config in PLAYER2_UI_ELEMENTS {
        spawn_ui_element_from_config(commands, font, asset_server, texture_atlas_layouts, config, player_info, TankType::Player2);
    }
}

fn spawn_top_text_info(
    commands: &mut Commands,
    font: &Handle<Font>,
    stage_level: usize,
) {
    // 其他游戏信息 UI 元素配置
    let commander_text_x = WINDOW_LEFT_X + 435.0; // 往左平移30像素

    // 关卡信息显示在顶部中心
    commands.spawn((
        PlayingEntity,
        Text2d(format!("Stage {stage_level}")),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色
        Transform::from_xyz(0.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));

    commands.spawn((
        PlayingEntity,
        Text2d("Commander Life:".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(commander_text_x - 42.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
    // Commander 血条（与玩家血条长度相同：160像素），放在文字正右方
    commands.spawn((
        PlayingEntity,
        CommanderHealthBar,
        CommanderHealthBarOriginalPosition(commander_text_x + 172.0), // 文字右侧
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(160.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(commander_text_x + 172.0, WINDOW_TOP_Y - 50.0, 1.0), // 与文字同一Y坐标
    ));
    commands.spawn((
        PlayingEntity,
        EnemyCountText,
        Text2d("Enemy Left: 20/20".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(WINDOW_RIGHT_X - 465.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
}

fn spawn_ui_element_from_config(
    commands: &mut Commands,
    font: &Handle<Font>,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    config: &UIElementConfig,
    player_info: &PlayerInfo,
    tank_type: TankType,
) {
    let player_stats = &player_info.players[&tank_type];
    match config.element_type {
        UIElementType::NormalText(f) => {
            let text = f(player_stats);
            // 检查属性是否达到最大值或On状态，如果是则设置红色
            let text_color = if is_stat_at_max_value(&text, player_stats) {
                Color::srgb(1.0, 0.0, 0.0) // 红色
            } else {
                Color::srgb(1.0, 1.0, 1.0) // 白色
            };

            commands.spawn((
                PlayerUI { player_type: tank_type },
                PlayingEntity,
                Text2d(text),
                TextFont {
                    font_size: config.font_size,
                    font: font.clone(),
                    ..default()
                },
                TextColor(text_color),
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
        UIElementType::PlayerAvatar => {
            let player_avatar_texture: Handle<Image> = asset_server.load(TEXTURE_AVATAR);
            let player_avatar_tile_size = UVec2::new(160, 147);
            let player_avatar_texture_atlas = TextureAtlasLayout::from_grid(player_avatar_tile_size, 13, 3, None, None);
            let player_avatar_texture_atlas_layout = texture_atlas_layouts.add(player_avatar_texture_atlas);
            let player_avatar_animation_indices = AnimationIndices { first: 0, last: 32 };
            commands.spawn((
                PlayerUI { player_type: tank_type },
                PlayerAvatar,
                PlayingEntity,
                Sprite {
                    image: player_avatar_texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: player_avatar_texture_atlas_layout,
                        index: 0,
                    }),
                    custom_size: Some(Vec2::new(160.0, 147.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
                player_avatar_animation_indices,
                AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                CurrentAnimationFrame(0),
            ));
        }
        UIElementType::HealthBar => {
            commands.spawn((
                PlayerUI { player_type: tank_type },
                HealthBar,
                HealthBarOriginalPosition(config.x_pos),
                PlayingEntity,
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(160.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
        UIElementType::BlueBar => {
            commands.spawn((
                PlayerUI { player_type: tank_type },
                BlueBar,
                BlueBarOriginalPosition(config.x_pos),
                PlayingEntity,
                Sprite {
                    color: Color::srgb(0.0, 0.5, 1.0),
                    custom_size: Some(Vec2::new(160.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
    }
}

fn spawn_power_ups(commands: &mut Commands, asset_server: &AssetServer, texture_atlas_layouts: &mut Assets<TextureAtlasLayout>, stage_level: &StageLevel) {
    let powerup_type = if stage_level.0 == 1 {
        // 第一关强制生成 air_cushion 道具
        PowerUp::AirCushion
    } else {
        // 其他关卡随机选择一个道具类型
        let powerup_types = [
            PowerUp::SpeedUp,
            PowerUp::Protection,
            PowerUp::FireSpeed,
            PowerUp::FireShell,
            PowerUp::TrackChain,
            PowerUp::Penetrate,
            PowerUp::Repair,
            PowerUp::Hamburger,
            PowerUp::AirCushion,
            PowerUp::Shell,
        ];

        let mut rng = rand::rng();
        powerup_types[rng.random_range(0..powerup_types.len())]
    };

    // 定义禁止区域
    // 上方：坦克高度区域（MAP_TOP_Y - TANK_HEIGHT 到 MAP_TOP_Y）
    // 下方：commander高度区域（MAP_BOTTOM_Y 到 MAP_BOTTOM_Y + COMMANDER_HEIGHT）
    let top_forbidden_y = MAP_TOP_Y - TANK_HEIGHT;
    let bottom_forbidden_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT;

    // 在随机位置生成道具（在地图范围内），避开禁止区域
    let mut rng = rand::rng();
    let x = rng.random_range(MAP_LEFT_X + 100.0..MAP_RIGHT_X - 100.0);
    let y = rng.random_range(bottom_forbidden_y + 100.0..top_forbidden_y - 100.0);
    let position = Vec3::new(x, y, 0.0);

    spawn_powerup_batch(commands, asset_server, texture_atlas_layouts, powerup_type, powerup_type.texture_path(), &[position]);
}

fn spawn_powerup_batch(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    powerup_type: PowerUp,
    texture_path: &'static str,
    positions: &[Vec3],
) {
    let texture: Handle<Image> = asset_server.load(texture_path);
    let tile_size = UVec2::new(87, 69);
    let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    for pos in positions {
        commands.spawn((
            powerup_type,
            PlayingEntity,
            Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: animation_indices.first,
                }
            ),
            Transform::from_xyz(pos.x, pos.y, 0.8), // z=0.8 使道具高于除了树之外的所有图层
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            CurrentAnimationFrame(0),
            RigidBody::Fixed,
            Collider::cuboid(87.0 / 2.0, 69.0 / 2.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
        ));
    }
}

fn spawn_game_entities_if_needed(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut clear_color: ResMut<ClearColor>,
    _enemy_spawn_state: Res<EnemySpawnState>,
    mut player_info: ResMut<PlayerInfo>,
    stage_level: Res<StageLevel>,
    game_mode: Res<GameMode>,
    mut entities_spawned: ResMut<GameEntitiesSpawned>,
) {
    // 如果游戏实体已经生成，则跳过
    if entities_spawned.0 {
        return;
    }

    // 标记游戏实体已生成
    entities_spawned.0 = true;

    // 设置背景色为黑色
    clear_color.0 = BACKGROUND_COLOR;

    // 生成墙壁
    spawn_walls(&mut commands);

    // 根据地图数组生成地形
    spawn_map_terrain(&mut commands, &asset_server, &mut texture_atlas_layouts, stage_level.0);

    // 生成司令官
    spawn_commander(&mut commands, &asset_server, &mut texture_atlas_layouts);

    // 加载玩家坦克纹理和创建精灵图
    let player1_texture = asset_server.load(TEXTURE_PLAYER_TANK1);
    let player2_texture = asset_server.load(TEXTURE_PLAYER_TANK2);
    let player_tile_size = UVec2::new(293, 328);
    let player_texture_atlas = TextureAtlasLayout::from_grid(player_tile_size, 2, 1, None, None);
    let player_texture_atlas_layout = texture_atlas_layouts.add(player_texture_atlas);
    let player_animation_indices = AnimationIndices { first: 0, last: 1 };

    // 根据游戏模式生成玩家

        match *game_mode {

            GameMode::OnePlayer => {

                // 单人模式：只生成玩家1

                let _player1_tank_entity = spawn_player1_tank(

                    &mut commands,

                    player1_texture,

                    player_texture_atlas_layout,

                    player_animation_indices,

                );

    

                                // 初始化玩家1信息

    

                                player_info.players.insert(TankType::Player1, PlayerStats {

    

                                    name: "Li Yun Long".to_string(),

    

                                    speed: 40,

    

                                    fire_speed: 40,

    

                                    protection: 40,

    

                                    shells: 1,

    

                                    penetrate: false,

    

                                    track_chain: false,

    

                                    air_cushion: false,

    

                                    fire_shell: false,

    

                                    life_red_bar: 3,

    

                                    energy_blue_bar: 3,

                    score: 0,

                });

    

                }

            GameMode::TwoPlayers => {

                // 双人模式：生成玩家1和玩家2

                let _player1_tank_entity = spawn_player1_tank(

                    &mut commands,

                    player1_texture,

                    player_texture_atlas_layout.clone(),

                    player_animation_indices,

                );



                let _player2_tank_entity = commands.spawn_empty()

                    .insert(PlayerTank { tank_type: TankType::Player2 })

                    .insert(PlayingEntity)

                    .insert(TankFireConfig::default())

                    .insert(RotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))

                    .insert(TargetRotation { angle: 0.0_f32.to_radians() })

                    .insert(Sprite {
                        image: player2_texture,
                        texture_atlas: Some(TextureAtlas {
                            layout: player_texture_atlas_layout,
                            index: player_animation_indices.first,
                        }),
                        custom_size: Some(Vec2::new(80.0, 90.0)),
                        ..default()
                    })

                    .insert(Transform::from_xyz(TANK_WIDTH / 2.0 + COMMANDER_WIDTH/2.0 + 50.0, MAP_BOTTOM_Y+TANK_HEIGHT / 2.0, 0.0))

                    .insert(Velocity{ linvel: Vec2::default(), angvel: 0.0 })

                    .insert(player_animation_indices)

                    .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))

                    .insert(RigidBody::KinematicPositionBased)

                                        .insert(Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0))

                                        .insert(ActiveEvents::COLLISION_EVENTS)

                                        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC | ActiveCollisionTypes::KINEMATIC_KINEMATIC)

                                        .insert(LockedAxes::ROTATION_LOCKED)

                                        .insert(KinematicCharacterController {

                                            offset: CharacterLength::Absolute(0.01),

                                            filter_groups: None,

                                            ..default()

                                        })

                                        .id();

    

                // 初始化玩家1信息

                player_info.players.insert(TankType::Player1, PlayerStats {

                    name: "Li Yun Long".to_string(),

                    speed: 40,

                    fire_speed: 40,

                    protection: 40,

                    shells: 1,

                    penetrate: false,

                    track_chain: false,

                    air_cushion: false,

                    fire_shell: false,

                    life_red_bar: 3,

                    energy_blue_bar: 3,

                    score: 0,

                });

    

                // 初始化玩家2信息

                player_info.players.insert(TankType::Player2, PlayerStats {

                    name: "Chu Yun Fei".to_string(),

                    speed: 40,

                    fire_speed: 40,

                    protection: 40,

                    shells: 1,

                    penetrate: false,

                    track_chain: false,

                    air_cushion: false,

                    fire_shell: false,

                    life_red_bar: 3,

                    energy_blue_bar: 3,

                    score: 0,

                });

            }
    }

    // 加载字体
    let font: Handle<Font> = asset_server.load(crate::FONT_EN);

    // 根据游戏模式生成UI
    match *game_mode {
        GameMode::OnePlayer => {
            // 单人模式：只生成玩家1的UI
            for config in PLAYER1_UI_ELEMENTS {
                spawn_ui_element_from_config(&mut commands, &font, &asset_server, &mut texture_atlas_layouts, config, &player_info, TankType::Player1);
            }
        }
        GameMode::TwoPlayers => {
            // 双人模式：生成玩家1和玩家2的UI
            spawn_player_info(&mut commands, &font, &asset_server, &mut texture_atlas_layouts, &player_info);
        }
    }
    
    spawn_top_text_info(&mut commands, &font, stage_level.0);

    // 生成道具
    spawn_power_ups(&mut commands, &asset_server, &mut texture_atlas_layouts, &stage_level);
}

fn handle_start_screen_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu_selection: ResMut<CurrentMenuSelection>,
    mut game_mode: ResMut<GameMode>,
) {
    // Esc 键退出游戏
    if keyboard_input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // W 键向上选择
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        menu_selection.selected_index = if menu_selection.selected_index > 0 {
            menu_selection.selected_index - 1
        } else {
            4
        };
    }
    // S 键向下选择
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        menu_selection.selected_index = (menu_selection.selected_index + 1) % 5;
    }
    // Space 键确认选择
    if keyboard_input.just_pressed(KeyCode::Space) {
        match menu_selection.selected_index {
            0 => {
                *game_mode = GameMode::OnePlayer;
                next_state.set(GameState::FadingOut); // 1 Player
            }
            1 => {
                *game_mode = GameMode::TwoPlayers;
                next_state.set(GameState::FadingOut); // 2 Player
            }
            2 => {
                next_state.set(GameState::About); // About
            }
            3 => {
                next_state.set(GameState::Credits); // Credits
            }
            4 => std::process::exit(0), // EXIT
            _ => {}
        }
    }
}

fn animate_start_screen(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite, &mut CurrentAnimationFrame), With<StartScreenUI>>,
    animation_frames: Res<StartAnimationFrames>,
) {
    for (indices, mut timer, mut sprite, mut current_frame) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            let current = current_frame.0;
            let next_index = if current == indices.last {
                indices.first
            } else {
                current + 1
            };
            current_frame.0 = next_index;

            // 计算使用哪个纹理图集（每个图集5帧）
            let atlas_index = next_index / 5;
            let frame_in_atlas = next_index % 5;

            // 更新纹理和图集
            if atlas_index < animation_frames.textures.len() {
                sprite.image = animation_frames.textures[atlas_index].clone();
                if let Some(texture_atlas) = &mut sprite.texture_atlas {
                    texture_atlas.layout = animation_frames.texture_atlas_layouts[atlas_index].clone();
                    texture_atlas.index = frame_in_atlas;
                }
            }
        }
    }
}

fn setup(
    mut commands: Commands,
) {
    // 创建全局相机
    commands.spawn(Camera2d);
}

fn detect_enemy_tank_collision(
    entity: Entity,
    rapier_context: &RapierContext,
) -> Option<Vec2> {
    for contact_pair in rapier_context.contact_pairs_with(entity) {
        if !contact_pair.has_any_active_contact() {
            continue;
        }

        for manifold in contact_pair.manifolds() {
            if manifold.find_deepest_contact().is_some() {
                // 获取碰撞法线方向
                return Some(if contact_pair.collider1() == Some(entity) {
                    -manifold.normal()
                } else {
                    manifold.normal()
                });
            }
        }
    }
    None
}

fn get_blocked_direction(collision_normal: Vec2) -> Vec2 {
    let abs_x = collision_normal.x.abs();
    let abs_y = collision_normal.y.abs();

    if abs_x > abs_y {
        if collision_normal.x > 0.0 {
            Vec2::new(1.0, 0.0) // 碰撞来自右侧，不能向右
        } else {
            Vec2::new(-1.0, 0.0) // 碰撞来自左侧，不能向左
        }
    } else {
        if collision_normal.y > 0.0 {
            Vec2::new(0.0, 1.0) // 碰撞来自上方，不能向上
        } else {
            Vec2::new(0.0, -1.0) // 碰撞来自下方，不能向下
        }
    }
}

fn choose_available_direction(blocked_direction: Vec2) -> Vec2 {
    let available_directions: Vec<Vec2> = DIRECTIONS
        .iter()
        .filter(|dir| **dir != blocked_direction)
        .copied()
        .collect();

    if available_directions.is_empty() {
        blocked_direction
    } else {
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..available_directions.len());
        available_directions[random_index]
    }
}

fn handle_enemy_tank_collision(
    entity: Entity,
    enemy_tank: &mut EnemyTank,
    direction_timer: &mut DirectionChangeTimer,
    collision_cooldown: &mut CollisionCooldownTimer,
    rapier_context: &RapierContext,
) {
    if let Some(collision_normal) = detect_enemy_tank_collision(entity, rapier_context) {
        let blocked_direction = get_blocked_direction(collision_normal);
        enemy_tank.direction = choose_available_direction(blocked_direction);
        direction_timer.reset();
        collision_cooldown.reset();
    }
}

fn handle_random_direction_change(
    enemy_tank: &mut EnemyTank,
    direction_timer: &mut DirectionChangeTimer,
) {
    let mut rng = rand::rng();
    if rng.random::<f32>() < 0.4 {
        let random_index = rng.random_range(0..DIRECTIONS.len());
        enemy_tank.direction = DIRECTIONS[random_index];
    }
    direction_timer.reset();
}

fn update_enemy_tank_movement(
    enemy_tank: EnemyTank,
    velocity: &mut Velocity,
    target_rotation: &mut TargetRotation,
    rotation_timer: &mut RotationTimer,
) {
    if enemy_tank.direction.length() > 0.0 {
        let angle = enemy_tank.direction.y.atan2(enemy_tank.direction.x);
        let target_angle = angle - 270.0_f32.to_radians();

        // 检查是否需要转向
        let current_euler = target_rotation.angle;
        let angle_diff = std::f32::consts::PI.mul_add(3.0, target_angle - current_euler) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;

        if angle_diff.abs() > 0.01 {
            // 需要转向，设置速度为0实现原地转向
            velocity.linvel = Vec2::ZERO;
            target_rotation.angle = target_angle;
            rotation_timer.reset();
        } else {
            // 不需要转向，正常移动
            velocity.linvel = enemy_tank.direction * TANK_SPEED;
        }
    }
}

#[allow(clippy::type_complexity)]
fn move_enemy_tanks(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Velocity,
        &mut EnemyTank,
        &mut DirectionChangeTimer,
        &mut CollisionCooldownTimer,
        &mut Transform,
        &mut RotationTimer,
        &mut TargetRotation,
    )>,
    rapier_context: ReadRapierContext,
) {    let rapier_context = rapier_context.single().unwrap();

    for (entity, mut velocity, mut enemy_tank, mut direction_timer, mut collision_cooldown, mut transform, mut rotation_timer, mut target_rotation) in &mut query {
        // 更新碰撞冷却计时器
        collision_cooldown.tick(time.delta());

        // 只在冷却时间结束后才检测碰撞
        if collision_cooldown.is_finished() {
            handle_enemy_tank_collision(
                entity,
                &mut enemy_tank,
                &mut direction_timer,
                &mut collision_cooldown,
                &rapier_context,
            );
        }

        // 更新方向计时器
        direction_timer.tick(time.delta());

        // 如果计时器结束，有10%几率随机转向
        if direction_timer.just_finished() {
            handle_random_direction_change(&mut enemy_tank, &mut direction_timer);
        }

        // 更新坦克移动
        update_enemy_tank_movement(*enemy_tank, &mut velocity, &mut target_rotation, &mut rotation_timer);

        // 更新旋转计时器
        rotation_timer.tick(time.delta());

        // 平滑旋转
        let current_rotation = transform.rotation;
        let target_rotation = Quat::from_rotation_z(target_rotation.angle);

        if current_rotation.angle_between(target_rotation) > 0.01 && !rotation_timer.is_finished() {
            // 使用 slerp 进行平滑旋转
            let progress = rotation_timer.elapsed_secs() / rotation_timer.duration().as_secs_f32();
            let eased_progress = progress * progress * 2.0f32.mul_add(-progress, 3.0); // 缓动函数
            transform.rotation = current_rotation.slerp(target_rotation, eased_progress);
        } else if current_rotation.angle_between(target_rotation) > 0.01 {
            // 旋转完成，直接设置为目标角度
            transform.rotation = target_rotation;
        }
    }
}

pub fn spawn_explosion(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载爆炸精灵图（8x8，共64帧，每帧512x512）
    let explosion_texture: Handle<Image> = asset_server.load(TEXTURE_EXPLOSION);
    let explosion_tile_size = UVec2::new(512, 512);
    let explosion_texture_atlas = TextureAtlasLayout::from_grid(explosion_tile_size, 8, 8, None, None);
    let explosion_texture_atlas_layout = texture_atlas_layouts.add(explosion_texture_atlas);
    let explosion_animation_indices = AnimationIndices { first: 0, last: 63 };

    commands.spawn((
        Explosion,
        PlayingEntity,
        Sprite {
            image: explosion_texture,
            texture_atlas: Some(TextureAtlas {
                layout: explosion_texture_atlas_layout,
                index: explosion_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(300.0, 300.0)),
            ..default()
        },
        Transform::from_translation(position),
        explosion_animation_indices,
        AnimationTimer(Timer::from_seconds(0.01, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));

    // 播放爆炸音效
    let explosion_sound: Handle<AudioSource> = asset_server.load(SOUND_EXPLOSION);
    commands.spawn(AudioPlayer::new(explosion_sound));
}

fn spawn_forest_fire(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载树林燃烧精灵图（10帧，每帧131x131，1.5秒播完）
    let forest_fire_texture: Handle<Image> = asset_server.load("maps/tree_fire_sheet.png");
    let forest_fire_tile_size = UVec2::new(131, 131);
    let forest_fire_texture_atlas = TextureAtlasLayout::from_grid(forest_fire_tile_size, 10, 1, None, None);
    let forest_fire_texture_atlas_layout = texture_atlas_layouts.add(forest_fire_texture_atlas);
    let forest_fire_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        ForestFire,
        PlayingEntity,
        Sprite::from_atlas_image(
            forest_fire_texture,
            TextureAtlas {
                layout: forest_fire_texture_atlas_layout,
                index: forest_fire_animation_indices.first,
            }
        ),
        Transform::from_translation(position),
        forest_fire_animation_indices,
        AnimationTimer(Timer::from_seconds(1.5 / 10.0, TimerMode::Repeating)), // 1.5秒播完10帧
        CurrentAnimationFrame(0),
    ));

    // 播放树林燃烧音效
    let burn_tree_sound: Handle<AudioSource> = asset_server.load(SOUND_BURN_TREE);
    commands.spawn(AudioPlayer::new(burn_tree_sound));
}

pub fn spawn_spark(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载打击效果图片（4x4，共16帧，每帧1024x1024）
    let spark_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL_HIT);
    let spark_tile_size = UVec2::new(1024, 1024);
    let spark_texture_atlas = TextureAtlasLayout::from_grid(spark_tile_size, 4, 4, None, None);
    let spark_texture_atlas_layout = texture_atlas_layouts.add(spark_texture_atlas);
    let spark_animation_indices = AnimationIndices { first: 0, last: 15 };

    commands.spawn((
        Spark,
        PlayingEntity,
        Sprite {
            image: spark_texture,
            texture_atlas: Some(TextureAtlas {
                layout: spark_texture_atlas_layout,
                index: spark_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(200.0, 200.0)),
            ..default()
        },
        Transform::from_translation(position),
        spark_animation_indices,
        AnimationTimer(Timer::from_seconds(0.02, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));
}

fn handle_powerup_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    powerups: Query<(Entity, &Transform, &PowerUp)>,
    player_tanks: Query<(&Transform, &PlayerTank, Entity), With<PlayerTank>>,
    mut controllers: Query<&mut KinematicCharacterController>,
    mut player_info: ResMut<PlayerInfo>,
    mut commander_life: ResMut<CommanderLife>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
) {
    for (tank_transform, player_tank, tank_entity) in player_tanks{
        let mut picked_powerup: Option<PowerUp> = None;
        let mut powerup_entity_to_despawn: Option<Entity> = None;

        // 检查道具碰撞
        for (powerup_entity, powerup_transform, powerup_type) in powerups.iter(){
            let distance = (powerup_transform.translation - tank_transform.translation).length();
            if distance < 81.0 {
                picked_powerup = Some(*powerup_type);
                powerup_entity_to_despawn = Some(powerup_entity);
            }
        }

        if let Some(powerup_type) = picked_powerup {
            let powerup_entity = powerup_entity_to_despawn.unwrap();

            // 播放道具音效
            let powerup_sound: Handle<AudioSource> = asset_server.load(SOUND_POWERUP);
            commands.spawn(AudioPlayer::new(powerup_sound));
            let _ = commands.entity(powerup_entity).try_despawn();

            // 根据道具类型应用效果并发送事件
            if let Some(player_stats) = player_info.players.get_mut(&player_tank.tank_type) {
                let stat_type = match powerup_type {
                    PowerUp::SpeedUp => {
                        if player_stats.speed < 100 {
                            player_stats.speed += 20;
                        }
                        Some(StatType::Speed)
                    }
                    PowerUp::Protection => {
                        if player_stats.protection < 100 {
                            player_stats.protection += 20;
                        }
                        Some(StatType::Protection)
                    }
                    PowerUp::FireSpeed => {
                        if player_stats.fire_speed < 100 {
                            player_stats.fire_speed += 20;
                        }
                        Some(StatType::FireSpeed)
                    }
                    PowerUp::FireShell => {
                        player_stats.fire_shell = true;
                        Some(StatType::FireShell)
                    }
                    PowerUp::TrackChain => {
                        player_stats.track_chain = true;
                        Some(StatType::TrackChain)
                    }
                    PowerUp::Penetrate => {
                        player_stats.penetrate = true;
                        Some(StatType::Penetrate)
                    }
                    PowerUp::Repair => {
                        if player_stats.life_red_bar < 3 {
                            player_stats.life_red_bar += 1;
                        }
                        None // 修理道具不需要闪烁文字
                    }
                    PowerUp::Hamburger => {
                        if commander_life.life_red_bar < 3 {
                            commander_life.life_red_bar += 1;
                        }
                        None // 汉堡道具不影响玩家属性，不发送事件
                    }
                    PowerUp::AirCushion => {
                        player_stats.air_cushion = true;
                        // 更新 filter_groups，排除海（GROUP_2）
                        // 玩家坦克不设置 memberships（默认所有组），filters 设置为不包含 GROUP_2
                        if let Ok(mut controller) = controllers.get_mut(tank_entity) {
                            controller.filter_groups = Some(CollisionGroups::new(Group::all(), Group::all() & !SEA_GROUP));
                        }
                        // 添加气泡特效标记
                        commands.entity(tank_entity).insert(crate::constants::BubbleEffect);
                        Some(StatType::AirCushion)
                    }
                    PowerUp::Shell => {
                        // 增加 1 颗子弹，最多 2 颗
                        if player_stats.shells < 2 {
                            player_stats.shells += 1;
                        }
                        Some(StatType::Shell)
                    }
                };

                // 发送属性变更事件
                if let Some(stat_type) = stat_type {
                    stat_changed_events.write(PlayerStatChanged {
                        player_type: player_tank.tank_type,
                        stat_type,
                    });
                }
            }
        }
    }
}

// 获取属性类型对应的前缀
const fn get_stat_prefix(stat_type: StatType) -> &'static str {
    match stat_type {
        StatType::Speed => "Speed:",
        StatType::Protection => "Protection:",
        StatType::FireSpeed => "Fire Speed:",
        StatType::FireShell => "Fire Shell:",
        StatType::TrackChain => "Track Chain:",
        StatType::Penetrate => "Penetrate:",
        StatType::AirCushion => "Air Cushion:",
        StatType::Shell => "Shells:",
        StatType::Score => "Scores",
    }
}

// 处理属性变更事件，触发文字闪烁
fn handle_stat_changed_for_blink(
    mut events: MessageReader<PlayerStatChanged>,
    mut commands: Commands,
    player_info_texts: Query<(Entity, &Text2d, &PlayerUI)>,
) {
    for event in events.read() {
        let prefix = get_stat_prefix(event.stat_type);
        for (entity, text, player_index) in &player_info_texts {
            if player_index.player_type == event.player_type && text.0.starts_with(prefix) {
                commands.entity(entity).insert(PlayerInfoBlinkTimer(
                    Timer::from_seconds(1.2, TimerMode::Once)
                ));
                break;
            }
        }
    }
}

fn animate_player_info_text(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PlayerInfoBlinkTimer, &mut TextColor, &Text2d, &PlayerUI), With<Text2d>>,
    player_info: Res<PlayerInfo>,
) {
    for (entity, mut timer, mut color, text, player_index) in &mut query {
        timer.tick(time.delta());

        // 判断是否达到最大值或On状态
        let is_max = player_info.players.get(&player_index.player_type).is_some_and(|player_stats| is_stat_at_max_value(&text.0, player_stats));

        if is_max {
            // 达到最大值：保持红色，移除闪烁计时器
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = Color::srgb(1.0, 0.0, 0.0);  // 红色
        } else if timer.is_finished() {
            // 闪烁结束，移除计时器组件
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = Color::srgb(1.0, 1.0, 1.0);
        } else {
            // 未达到最大值：闪烁效果
            // 每0.6秒切换颜色（0.3秒亮，0.3秒灭）
            let elapsed = timer.elapsed_secs();
            let cycle = elapsed % 0.6;

            if cycle < 0.3 {
                // 亮状态：绿色
                color.0 = Color::srgb(0.0, 1.0, 0.0);
            } else {
                // 灭状态：透明
                color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
            }
        }
    }
}

// 判断属性是否达到最大值或On状态
fn is_stat_at_max_value(text: &str, player_stats: &PlayerStats) -> bool {
    if text.starts_with("Shells:") {
        player_stats.shells >= 2
    } else if text.starts_with("Speed:") {
        player_stats.speed >= 100
    } else if text.starts_with("Protection:") {
        player_stats.protection >= 100
    } else if text.starts_with("Fire Speed:") {
        player_stats.fire_speed >= 100
    } else if text.starts_with("Fire Shell:") {
        player_stats.fire_shell
    } else if text.starts_with("Air Cushion:") {
        player_stats.air_cushion
    } else if text.starts_with("Track Chain:") {
        player_stats.track_chain
    } else if text.starts_with("Penetrate:") {
        player_stats.penetrate
    } else {
        false  // 分数等其他属性没有最大值
    }
}

fn update_enemy_count_display(
    enemy_spawn_state: Res<EnemySpawnState>,
    enemy_tanks: Query<(), With<EnemyTank>>,
    mut query: Query<&mut Text2d, With<EnemyCountText>>,
) {
    let current_enemy_count = enemy_tanks.iter().count();
    let remaining = enemy_spawn_state.max_count - enemy_spawn_state.has_spawned + current_enemy_count;

    for mut text in &mut query {
        text.0 = format!("Enemy Left: {}/{}", remaining, enemy_spawn_state.max_count);
    }
}

fn animate_powerup_texture(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<PowerUp>>,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

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
}

fn animate_player_tank_texture(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &KinematicCharacterController), With<PlayerTank>>,
) {
    // 玩家坦克：只有移动时才刷新纹理
    for (mut timer, mut sprite, indices, character_controller) in &mut query {
        // 使用 KinematicCharacterController 的 translation 字段判断是否在移动
        let is_moving = character_controller.translation.is_some();
        if sprite.texture_atlas.is_none() {
            continue;
        }
        let atlas = sprite.texture_atlas.as_mut().expect("玩家坦克没有纹理！");
        if !is_moving {
            atlas.index = indices.last;
            timer.reset();
        } else {
            timer.tick(time.delta());
            if !timer.just_finished() {
                continue;
            }
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            }
        }
    }
}

fn animate_enemy_tank_texture(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices), With<EnemyTank>>,
) {
    // 敌方坦克：统一刷新
    for (mut timer, mut sprite, indices) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
    }
}

fn animate_enemy_born_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame, &BornPosition), With<EnemyBornAnimation>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame, born_position) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas {
                let current = current_frame.0;
                let total_frames = indices.last - indices.first + 1;
                let spawn_frame = indices.first + (total_frames / 2); // 1/2 处生成坦克

                if current >= indices.last {
                    // 动画播放完毕，销毁出生动画实体
                    let _ = commands.entity(entity).try_despawn();
                } else {
                    // 继续播放动画
                    let next_index = current + 1;
                    current_frame.0 = next_index;
                    atlas.index = next_index;

                    // 在动画播放到 2/3 时生成敌方坦克
                    if next_index == spawn_frame {
                        // 加载敌方坦克纹理和创建精灵图
                        let enemy_texture = asset_server.load("enemy_tank/enemy_tank1_sprite.png");
                        let enemy_tile_size = UVec2::new(137, 183);
                        let enemy_texture_atlas = TextureAtlasLayout::from_grid(enemy_tile_size, 2, 1, None, None);
                        let enemy_texture_atlas_layout = texture_atlas_layouts.add(enemy_texture_atlas);
                        let enemy_animation_indices = AnimationIndices { first: 0, last: 1 };

                        // 生成敌方坦克
                        let _enemy_entity = commands.spawn_empty()
                            .insert(EnemyTank {
                                direction: Vec2::new(0.0, -1.0),
                            })
                            .insert(PlayingEntity)
                            .insert(TankFireConfig::default())
                            .insert(DirectionChangeTimer(Timer::from_seconds(2.0, TimerMode::Once)))
                            .insert(CollisionCooldownTimer(Timer::from_seconds(0.5, TimerMode::Once)))
                            .insert(RotationTimer(Timer::from_seconds(0.6, TimerMode::Once)))
                            .insert(TargetRotation { angle: 270.0_f32.to_radians() })
                            .insert(AnimationTimer(Timer::from_seconds(0.25, TimerMode::Repeating)))
                            .insert(Sprite {
                                image: enemy_texture,
                                texture_atlas: Some(TextureAtlas {
                                    layout: enemy_texture_atlas_layout,
                                    index: enemy_animation_indices.first,
                                }),
                                custom_size: Some(Vec2::new(80.0, 90.0)),
                                ..default()
                            })
                            .insert(Transform::from_translation(born_position.0))
                            .insert(enemy_animation_indices)
                            .insert(Velocity {
                                linvel: Vec2::new(0.0, -TANK_SPEED),
                                angvel: 0.0,
                            })
                            .insert(RigidBody::Dynamic)
                            .insert(Collider::cuboid(35.0, 35.0))
                            .insert(ActiveEvents::COLLISION_EVENTS|ActiveEvents::CONTACT_FORCE_EVENTS)
                            .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::DYNAMIC_DYNAMIC | ActiveCollisionTypes::DYNAMIC_STATIC)
                            .insert(LockedAxes::ROTATION_LOCKED)
                            .insert(GravityScale(0.0))
                            .insert(Friction::new(0.0))
                            .insert(Restitution::new(0.0))
                            .id();
                    }
                }
            }
    }
}

fn animate_player_avatar(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    commander_life: Res<CommanderLife>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationIndices,
        &mut CurrentAnimationFrame,
        Option<&PlayerDead>,
    ), With<PlayerAvatar>>,
) {
    let commander_dead = commander_life.life_red_bar == 0;

    for (mut timer, mut sprite, indices, mut current_frame, player_dead) in &mut query {
        // 如果玩家已死亡，切换到死亡图片并停止动画
        if player_dead.is_some() {
            let dead_texture: Handle<Image> = asset_server.load(TEXTURE_AVATAR_DEATH); // 暂时使用相同的图片，后续可以添加玩家2的死亡图片
            sprite.image = dead_texture;
            sprite.texture_atlas = None;
            sprite.custom_size = Some(Vec2::new(160.0, 147.0));
            continue;
        }

        // 如果Commander已死亡，切换到commander死亡图片并停止动画
        if commander_dead {
            let dead_texture: Handle<Image> = asset_server.load(TEXTURE_AVATAR_COMMANDER_DEAD); // 暂时使用相同的图片，后续可以添加玩家2的commander死亡图片
            sprite.image = dead_texture;
            sprite.texture_atlas = None;
            sprite.custom_size = Some(Vec2::new(160.0, 147.0));
            continue;
        }

        // 正常动画
        timer.tick(time.delta());
        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas {
                let current = current_frame.0;
                let next_index = if current == indices.last {
                    indices.first
                } else {
                    current + 1
                };
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
    }
}

fn animate_commander(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    commander_life: Res<CommanderLife>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationIndices,
        &mut CurrentAnimationFrame,
    ), With<Commander>>,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        // 如果Commander已死亡，切换到死亡图片并停止动画
        if commander_life.life_red_bar == 0 {
            let dead_texture: Handle<Image> = asset_server.load(TEXTURE_COMMANDER_DEAD);
            sprite.image = dead_texture;
            sprite.texture_atlas = None;
            sprite.custom_size = Some(Vec2::new(COMMANDER_WIDTH, COMMANDER_HEIGHT));
            continue;
        }

        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas {
                let current = current_frame.0;
                let next_index = if current == indices.last {
                    indices.first
                } else {
                    current + 1
                };
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
}
}

fn animate_commander_music(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationIndices,
        &mut CurrentAnimationFrame,
    ), With<CommanderMusicAnimation>>,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas {
                let current = current_frame.0;
                let next_index = if current == indices.last {
                    indices.first
                } else {
                    current + 1
                };

                atlas.index = next_index;
                current_frame.0 = next_index;
            }
    }
}

fn animate_explosion(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁爆炸实体
                let _ = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

fn animate_laser(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Laser>>,
    despawn_entities: Query<(Entity, &Transform), With<DespawnMarker>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁激光实体和所有标记的实体
                for (despawn_entity, transform) in despawn_entities.iter() {
                    // 在被销毁实体的位置播放烟雾效果
                    let smoke_texture: Handle<Image> = asset_server.load(TEXTURE_SMOKE);
                    let smoke_tile_size = UVec2::new(100, 100);
                    let smoke_texture_atlas = TextureAtlasLayout::from_grid(smoke_tile_size, 5, 3, None, None);
                    let smoke_texture_atlas_layout = texture_atlas_layouts.add(smoke_texture_atlas);
                    let smoke_animation_indices = AnimationIndices { first: 0, last: 14 };
                    
                    commands.spawn((
                        PlayingEntity,
                        Smoke,
                        Sprite {
                            image: smoke_texture,
                            texture_atlas: Some(TextureAtlas {
                                layout: smoke_texture_atlas_layout,
                                index: smoke_animation_indices.first,
                            }),
                            custom_size: Some(Vec2::new(100.0, 100.0)),
                            ..default()
                        },
                        Transform::from_xyz(transform.translation.x, transform.translation.y, 1.0),
                        smoke_animation_indices,
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                        CurrentAnimationFrame(0),
                    ));
                    
                    let _ = commands.entity(despawn_entity).try_despawn();
                }
                let _ = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

/// 处理烟雾动画
fn animate_smoke(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Smoke>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁烟雾实体
                let _ = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

/// 处理后坐力效果
fn handle_recoil_force(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut RecoilForce)>,
) {
    for (entity, mut transform, mut recoil) in &mut query {
        recoil.timer.tick(time.delta());
        
        // 使用平滑插值应用后坐力位移
        let progress = recoil.timer.elapsed_secs() / recoil.timer.duration().as_secs_f32();
        let current_offset = recoil.target_offset * (1.0 - progress);
        
        // 从原始位置插值到当前位置
        transform.translation.x = recoil.original_pos.x + current_offset.x;
        transform.translation.y = recoil.original_pos.y + current_offset.y;
        
        // 后坐力时间结束，移除组件
        if recoil.timer.just_finished() {
            commands.entity(entity).remove::<RecoilForce>();
        }
    }
}

/// 激光碰撞检测系统（只收集实体，不立即销毁）
fn laser_collision_system(
    mut commands: Commands,
    mut frame_count: Local<u32>,
    lasers: Query<(Entity, &Transform, &CurrentAnimationFrame, &AnimationIndices), With<Laser>>,
    enemies: Query<(Entity, &Transform), With<EnemyTank>>,
    bullets: Query<(Entity, &Transform), With<BulletOwner>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    forests: Query<(Entity, &Transform), With<Forest>>,
    barriers: Query<(Entity, &Transform), With<Barrier>>,
    seas: Query<(Entity, &Transform), With<Sea>>,
) {
    // 每5帧执行一次碰撞检测
    *frame_count += 1;
    if *frame_count % 5 != 0 {
        return;
    }
    
    for (_laser_entity, laser_transform, _, _) in &lasers {
        // 激光原始尺寸（未旋转）
        let laser_half_width = 35.0; // 70 / 2
        let laser_half_height = 683.0; // 1366 / 2 (1倍)
        
        // 获取激光的旋转角度
        let rotation = laser_transform.rotation;
        
        // 激光的四个角点（未旋转）
        let corners = [
            Vec2::new(-laser_half_width, -laser_half_height),
            Vec2::new(laser_half_width, -laser_half_height),
            Vec2::new(laser_half_width, laser_half_height),
            Vec2::new(-laser_half_width, laser_half_height),
        ];
        
        // 旋转每个角点并加上位置
        let rotated_corners: Vec<Vec2> = corners.iter()
            .map(|corner| {
                let rotated = rotation.mul_vec3(corner.extend(0.0));
                Vec2::new(rotated.x, rotated.y) + Vec2::new(laser_transform.translation.x, laser_transform.translation.y)
            })
            .collect();
        
        // 计算旋转后的边界框
        let laser_left = rotated_corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let laser_right = rotated_corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let laser_bottom = rotated_corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let laser_top = rotated_corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

        // 检测与敌方坦克的碰撞
        for (enemy_entity, enemy_transform) in &enemies {
            let enemy_left = enemy_transform.translation.x - TANK_WIDTH / 2.0;
            let enemy_right = enemy_transform.translation.x + TANK_WIDTH / 2.0;
            let enemy_top = enemy_transform.translation.y + TANK_HEIGHT / 2.0;
            let enemy_bottom = enemy_transform.translation.y - TANK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < enemy_right && laser_right > enemy_left &&
               laser_bottom < enemy_top && laser_top > enemy_bottom {
                // 标记敌方坦克为待销毁
                let _ = commands.entity(enemy_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与子弹的碰撞
        for (bullet_entity, bullet_transform) in &bullets {
            let bullet_left = bullet_transform.translation.x - BULLET_SIZE / 2.0;
            let bullet_right = bullet_transform.translation.x + BULLET_SIZE / 2.0;
            let bullet_top = bullet_transform.translation.y + BULLET_SIZE / 2.0;
            let bullet_bottom = bullet_transform.translation.y - BULLET_SIZE / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < bullet_right && laser_right > bullet_left &&
               laser_bottom < bullet_top && laser_top > bullet_bottom {
                // 标记子弹为待销毁
                let _ = commands.entity(bullet_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与砖块的碰撞
        for (brick_entity, brick_transform) in &bricks {
            let brick_left = brick_transform.translation.x - BRICK_WIDTH / 2.0;
            let brick_right = brick_transform.translation.x + BRICK_WIDTH / 2.0;
            let brick_top = brick_transform.translation.y + BRICK_HEIGHT / 2.0;
            let brick_bottom = brick_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < brick_right && laser_right > brick_left &&
               laser_bottom < brick_top && laser_top > brick_bottom {
                // 标记砖块为待销毁
                let _ = commands.entity(brick_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与钢块的碰撞
        for (steel_entity, steel_transform) in &steels {
            let steel_left = steel_transform.translation.x - BRICK_WIDTH / 2.0;
            let steel_right = steel_transform.translation.x + BRICK_WIDTH / 2.0;
            let steel_top = steel_transform.translation.y + BRICK_HEIGHT / 2.0;
            let steel_bottom = steel_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < steel_right && laser_right > steel_left &&
               laser_bottom < steel_top && laser_top > steel_bottom {
                // 标记钢块为待销毁
                let _ = commands.entity(steel_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与森林的碰撞
        for (forest_entity, forest_transform) in &forests {
            let forest_left = forest_transform.translation.x - BRICK_WIDTH / 2.0;
            let forest_right = forest_transform.translation.x + BRICK_WIDTH / 2.0;
            let forest_top = forest_transform.translation.y + BRICK_HEIGHT / 2.0;
            let forest_bottom = forest_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < forest_right && laser_right > forest_left &&
               laser_bottom < forest_top && laser_top > forest_bottom {
                // 标记森林为待销毁
                let _ = commands.entity(forest_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与障碍的碰撞
        for (barrier_entity, barrier_transform) in &barriers {
            let barrier_left = barrier_transform.translation.x - BRICK_WIDTH / 2.0;
            let barrier_right = barrier_transform.translation.x + BRICK_WIDTH / 2.0;
            let barrier_top = barrier_transform.translation.y + BRICK_HEIGHT / 2.0;
            let barrier_bottom = barrier_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < barrier_right && laser_right > barrier_left &&
               laser_bottom < barrier_top && laser_top > barrier_bottom {
                // 标记障碍为待销毁
                let _ = commands.entity(barrier_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与sea的碰撞
        for (sea_entity, sea_transform) in &seas {
            let sea_left = sea_transform.translation.x - BRICK_WIDTH / 2.0;
            let sea_right = sea_transform.translation.x + BRICK_WIDTH / 2.0;
            let sea_top = sea_transform.translation.y + BRICK_HEIGHT / 2.0;
            let sea_bottom = sea_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < sea_right && laser_right > sea_left &&
               laser_bottom < sea_top && laser_top > sea_bottom {
                // 标记sea为待销毁
                let _ = commands.entity(sea_entity).try_insert(DespawnMarker);
            }
        }
    }
}

fn animate_forest_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<ForestFire>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁森林燃烧实体
                let _ = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

fn animate_forest(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Forest>>,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

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
}

fn play_tree_ambience(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    forests: Query<&Transform, With<Forest>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<TreeAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在树林附近
    let mut is_near_forest = false;
    const DETECTION_RADIUS: f32 = 150.0; // 树林检测半径

    for player_transform in player_tanks.iter() {
        for forest_transform in forests.iter() {
            let distance = player_transform.translation.distance(forest_transform.translation);
            if distance < DETECTION_RADIUS {
                is_near_forest = true;
                break;
            }
        }
        if is_near_forest {
            break;
        }
    }

    if is_near_forest {
        // 如果在树林附近但没有播放音效，则播放
        if ambience_players.is_empty() {
            let tree_ambience_sound: Handle<AudioSource> = asset_server.load(SOUND_TREE_AMBIENCE);
            commands.spawn((
                AudioPlayer::new(tree_ambience_sound),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.8)),
                TreeAmbiencePlayer,
            ));
        }
    } else {
        // 如果不在树林附近，停止播放所有树林音效
        for (entity, _) in ambience_players.iter() {
            let _ = commands.entity(entity).try_despawn();
        }
    }
}

fn animate_spark(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Spark>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁实体
                let _ = commands.entity(entity).try_despawn();
            } else {
                // 继续播放动画
                let next_index = current + 1;
                current_frame.0 = next_index;
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = next_index;
                }
            }
        }
    }
}

fn play_commander_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    commander: Query<&Transform, With<Commander>>,
    ambience_players: Query<Entity, With<CommanderAmbiencePlayer>>,
) {
    // 获取 Commander 位置
    let commander_transform = match commander.single() {
        Ok(t) => t,
        Err(_) => return,
    };

    // 计算最近的玩家坦克距离
    let mut min_distance = f32::MAX;
    for player_transform in player_tanks.iter() {
        let distance = player_transform.translation.distance(commander_transform.translation);
        min_distance = min_distance.min(distance);
    }

    // 根据距离判断是否播放音乐
    const MAX_DISTANCE: f32 = 130.0; // 最大检测距离（再缩小10像素）

    if min_distance < MAX_DISTANCE {
        // 如果在检测范围内但没有播放音效，则播放
        if ambience_players.is_empty() {
            // 随机选择一段音乐
            let music_files = [
                SOUND_COMMANDER_MUSIC_000,
                SOUND_COMMANDER_MUSIC_001,
                SOUND_COMMANDER_MUSIC_002,
                SOUND_COMMANDER_MUSIC_003,
            ];
            let mut rng = rand::rng();
            let selected_music = music_files[rng.random_range(0..music_files.len())];

            let commander_music: Handle<AudioSource> = asset_server.load(selected_music);
            commands.spawn((
                AudioPlayer::new(commander_music),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.6)),
                CommanderAmbiencePlayer,
            ));
        }
    } else {
        // 如果不在检测范围内，停止播放音乐
        for entity in ambience_players.iter() {
            let _ = commands.entity(entity).try_despawn();
        }
    }
}

fn handle_game_over_delay(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer), With<GameOverTimer>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut timer) in &mut query {
        timer.tick(time.delta());
        if timer.is_finished() {
            let _ = commands.entity(entity).try_despawn();
            next_state.set(GameState::GameOver);
        }
    }
}

fn check_game_over(
    mut commands: Commands,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    commander_life: Res<CommanderLife>,
    existing_timers: Query<(), With<GameOverTimer>>,
) {
    // 如果已经存在 GameOverTimer，说明已经触发了 GameOver，不再重复触发
    if !existing_timers.is_empty() {
        return;
    }

    // 检测 Commander 血量是否为 0
    if commander_life.life_red_bar == 0 {
        // 启动 Game Over 延迟计时器（1.2秒），等待爆炸动画完成
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
        ));
        return;
    }

    // 检测所有玩家生命值是否都为 0
    let all_players_dead = if player_info.players.is_empty() {
        false
    } else {
        match *game_mode {
            GameMode::OnePlayer => {
                player_info.players.get(&TankType::Player1).map_or(false, |p| p.life_red_bar == 0)
            }
            GameMode::TwoPlayers => {
                player_info.players.get(&TankType::Player1).map_or(false, |p| p.life_red_bar == 0)
                    && player_info.players.get(&TankType::Player2).map_or(false, |p| p.life_red_bar == 0)
            }
        }
    };

    if all_players_dead {
        // 启动 Game Over 延迟计时器（1.2秒）
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
        ));
    }
}

fn move_player_tank(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    player_info: Res<PlayerInfo>,
    mut query: Query<(&mut Transform, &mut KinematicCharacterController, &mut RotationTimer, &mut TargetRotation, &PlayerTank, Option<&IsDashing>), With<PlayerTank>>,
) {
    for (mut transform, mut character_controller, mut rotation_timer, mut target_rotation, player_tank, is_dashing) in &mut query {
        // 如果正在冲刺，跳过移动处理
        if is_dashing.is_some() {
            continue;
        }
        // 根据玩家索引选择不同的控制键
        let direction = if player_tank.tank_type == TankType::Player1 {
            // 玩家1使用 WASD
            let w_pressed = keyboard_input.pressed(KeyCode::KeyW);
            let s_pressed = keyboard_input.pressed(KeyCode::KeyS);
            let a_pressed = keyboard_input.pressed(KeyCode::KeyA);
            let d_pressed = keyboard_input.pressed(KeyCode::KeyD);
            match (w_pressed, s_pressed, a_pressed, d_pressed) {
                (true, false, false, false) => Vec2::new(0.0, 1.0),  // 上
                (false, true, false, false) => Vec2::new(0.0, -1.0), // 下
                (false, false, true, false) => Vec2::new(-1.0, 0.0), // 左
                (false, false, false, true) => Vec2::new(1.0, 0.0),  // 右
                _ => Vec2::ZERO, // 其他情况（包括多个键同时按下）停止移动
            }
        } else {
            // 玩家2使用方向键
            let up_pressed = keyboard_input.pressed(KeyCode::ArrowUp);
            let down_pressed = keyboard_input.pressed(KeyCode::ArrowDown);
            let left_pressed = keyboard_input.pressed(KeyCode::ArrowLeft);
            let right_pressed = keyboard_input.pressed(KeyCode::ArrowRight);
            match (up_pressed, down_pressed, left_pressed, right_pressed) {
                (true, false, false, false) => Vec2::new(0.0, 1.0),  // 上
                (false, true, false, false) => Vec2::new(0.0, -1.0), // 下
                (false, false, true, false) => Vec2::new(-1.0, 0.0), // 左
                (false, false, false, true) => Vec2::new(1.0, 0.0),  // 右
                _ => Vec2::ZERO, // 其他情况（包括多个键同时按下）停止移动
            }
        };

        // 检查是否需要转向
        let needs_rotation = if direction.length() > 0.0 {
            let angle = direction.y.atan2(direction.x);
            let target_angle = angle - 90.0_f32.to_radians();

            let current_euler = target_rotation.angle;
            let angle_diff = std::f32::consts::PI.mul_add(3.0, target_angle - current_euler) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;

            if angle_diff.abs() > 0.01 {
                target_rotation.angle = target_angle;
                // 只在角度变化较大时才重置计时器，避免频繁重置
                if angle_diff.abs() > 0.1 {
                    rotation_timer.reset();
                }
                true
            } else {
                false
            }
        } else {
            character_controller.translation = None;
            false
        };

        // 使用 KinematicCharacterController 的 translation 字段控制移动
        // 获取玩家的 speed 百分比加成
        let speed_bonus = player_info.players.get(&player_tank.tank_type)
            .map(|p| p.speed as f32 / 100.0)
            .unwrap_or(0.0);
        // 实际速度 = 基础速度 × (1 + speed百分比/100)
        // 转向时保持 50% 速度，减少卡顿感
        let base_speed = PLAYER_TANK_SPEED * (1.0 + speed_bonus);
        let speed = if needs_rotation { base_speed * 0.5 } else { base_speed };
        if direction.length() > 0.0 {
            character_controller.translation = Some(direction * speed * time.delta_secs());
        } else {
            character_controller.translation = None;
        }

        // 只在需要旋转时才更新旋转计时器和计算旋转
        if needs_rotation || !rotation_timer.is_finished() {
            rotation_timer.tick(time.delta());

            // 平滑旋转
            let current_euler = transform.rotation.to_euler(EulerRot::XYZ).2;
            let target_angle = target_rotation.angle;
            let angle_diff = std::f32::consts::PI.mul_add(3.0, target_angle - current_euler) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;

            if angle_diff.abs() > 0.01 && !rotation_timer.is_finished() {
                // 计算旋转进度（0.0 到 1.0）
                let progress = rotation_timer.elapsed_secs() / rotation_timer.duration().as_secs_f32();
                // 使用缓动函数使旋转更平滑
                let eased_progress = progress * progress * 2.0f32.mul_add(-progress, 3.0);
                // 插值计算当前角度
                let new_angle = current_euler + angle_diff * eased_progress;
                transform.rotation = Quat::from_rotation_z(new_angle);
            } else if angle_diff.abs() > 0.01 {
                // 旋转完成，直接设置为目标角度
                transform.rotation = Quat::from_rotation_z(target_angle);
            }
        }
    }
}

fn handle_recall_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    mut recall_timers: ResMut<RecallTimers>,
) {
    for (entity, transform, player_tank) in &query {
        // 检查是否正在回城
        let is_recalling = recall_timers.timers.contains_key(&entity);

        // 根据玩家索引选择不同的回城键
        let is_recall_key_pressed = if player_tank.tank_type == TankType::Player1 {
            // 玩家1使用 I 键回城
            keyboard_input.pressed(KeyCode::KeyI)
        } else {
            // 玩家2使用小键盘4键回城
            keyboard_input.pressed(KeyCode::Numpad4)
        };

        if is_recall_key_pressed && !is_recalling {
            // 计算初始位置
            let initial_position = if player_tank.tank_type == TankType::Player1 {
                Vec3::new(-TANK_WIDTH / 2.0 - COMMANDER_WIDTH/2.0 - 50.0, MAP_BOTTOM_Y+TANK_HEIGHT / 2.0, 0.0)
            } else {
                Vec3::new(TANK_WIDTH / 2.0 + COMMANDER_WIDTH/2.0 + 50.0, MAP_BOTTOM_Y+TANK_HEIGHT / 2.0, 0.0)
            };

            // 开始回城
            let recall_timer = RecallTimer::new(initial_position, RECALL_TIME);
            recall_timers.timers.insert(entity, recall_timer);

            // 添加回城标记
            commands.entity(entity).insert(IsRecalling);

            // 创建回城进度条（在坦克正上方，初始满格）
            commands.spawn((
                PlayingEntity,
                RecallProgressBar { player_type: player_tank.tank_type, player_entity: entity },
                Sprite {
                    color: Color::srgb(0.0, 1.0, 0.0), // 绿色
                    custom_size: Some(Vec2::new(100.0, 8.0)), // 初始宽度100（满格）
                    ..default()
                },
                Transform::from_xyz(transform.translation.x, transform.translation.y + TANK_HEIGHT / 2.0 + 20.0, 2.0), // 在坦克上方
            ));
        }
    }
}

fn update_recall_timers(
    time: Res<Time>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(Entity, &mut Transform, &PlayerTank, Option<&IsRecalling>, Option<&Children>), With<PlayerTank>>,
    mut recall_timers: ResMut<RecallTimers>,
    mut progress_bar_query: Query<(Entity, &mut Sprite, &RecallProgressBar)>,
) {
    for (entity, mut transform, player_tank, is_recalling, children) in &mut player_query {
        if matches!(is_recalling, Some(IsRecalling))
            && let Some(recall_timer) = recall_timers.timers.get_mut(&entity) {
            // 检查是否按住回城键
            let is_recall_key_pressed = if player_tank.tank_type == TankType::Player1 {
                keyboard_input.pressed(KeyCode::KeyI)
            } else {
                keyboard_input.pressed(KeyCode::Numpad4)
            };

            // 检查是否被打断（移动或射击）
            let is_interrupted = if player_tank.tank_type == TankType::Player1 {
                // 玩家1：检查WASD或J键
                keyboard_input.pressed(KeyCode::KeyW) ||
                keyboard_input.pressed(KeyCode::KeyS) ||
                keyboard_input.pressed(KeyCode::KeyA) ||
                keyboard_input.pressed(KeyCode::KeyD) ||
                keyboard_input.pressed(KeyCode::KeyJ)
            } else {
                // 玩家2：检查方向键或小键盘1键
                keyboard_input.pressed(KeyCode::ArrowUp) ||
                keyboard_input.pressed(KeyCode::ArrowDown) ||
                keyboard_input.pressed(KeyCode::ArrowLeft) ||
                keyboard_input.pressed(KeyCode::ArrowRight) ||
                keyboard_input.pressed(KeyCode::Numpad1)
            };

            // 如果没有按住回城键，或者被打断，则取消回城
            if !is_recall_key_pressed || is_interrupted {
                // 打断回城
                commands.entity(entity).remove::<IsRecalling>();
                recall_timers.timers.remove(&entity);

                // 删除进度条
                for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
                    if progress_bar.player_entity == entity {
                        let _ = commands.entity(progress_entity).try_despawn();
                    }
                }
            } else {
                // 更新计时器
                recall_timer.timer.tick(time.delta());

                // 更新进度条（从满格递减）
                let progress = recall_timer.timer.elapsed_secs() / recall_timer.timer.duration().as_secs_f32();
                let bar_width = 100.0 * (1.0 - progress); // 从100递减到0

                for (_, mut sprite, progress_bar) in &mut progress_bar_query {
                    if progress_bar.player_entity == entity {
                        sprite.custom_size = Some(Vec2::new(bar_width, 8.0));
                    }
                }

                // 检查是否完成
                if recall_timer.timer.just_finished() {
                    // 完成回城，传送到初始位置
                    let initial_position = recall_timer.start_position;

                    // 先删除所有子实体（包括气泡），防止Transform传播干扰传送
                    if let Some(children) = children {
                        for child in children.iter() {
                            let _ = commands.entity(child).try_despawn();
                        }
                    }

                    transform.translation = initial_position;

                    // 移除回城标记和计时器
                    commands.entity(entity).remove::<IsRecalling>();
                    recall_timers.timers.remove(&entity);

                    // 删除进度条
                    for (progress_entity, _, progress_bar) in progress_bar_query.iter() {
                        if progress_bar.player_entity == entity {
                            let _ = commands.entity(progress_entity).try_despawn();
                        }
                    }
                }
            }
        }
    }
}

fn update_recall_progress_bars(
    mut param_set: ParamSet<(
        Query<(Entity, &Transform)>,
        Query<(&RecallProgressBar, &mut Transform), Without<PlayerTank>>,
    )>,
) {
    let mut player_transforms: Vec<(Entity, Vec3)> = Vec::new();
    
    // 先收集所有玩家的位置信息
    for (entity, transform) in &param_set.p0() {
        player_transforms.push((entity, transform.translation));
    }
    
    // 然后更新进度条位置
    for (progress_bar, mut progress_transform) in &mut param_set.p1() {
        if let Some((_, player_pos)) = player_transforms.iter().find(|(e, _)| *e == progress_bar.player_entity) {
            // 更新倒计时文本位置（跟随坦克）
            progress_transform.translation.x = player_pos.x;
            progress_transform.translation.y = player_pos.y + TANK_HEIGHT / 2.0 + 20.0;
        }
    }
}

fn handle_dash_input(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    mut dash_timers: ResMut<DashTimers>,
    mut player_info: ResMut<PlayerInfo>,
) {
    for (entity, transform, player_tank) in &query {
        // 检查是否正在冲刺
        let is_dashing = dash_timers.timers.contains_key(&entity);

        // 根据玩家索引选择不同的冲刺键
        let is_dash_key_pressed = if player_tank.tank_type == TankType::Player1 {
            // 玩家1使用 K 键冲刺
            keyboard_input.just_pressed(KeyCode::KeyK)
        } else {
            // 玩家2使用小键盘2键冲刺
            keyboard_input.just_pressed(KeyCode::Numpad2)
        };

        if is_dash_key_pressed && !is_dashing {
            // 检查蓝条是否足够（需要至少1点蓝条）
            if let Some(player_stats) = player_info.players.get_mut(&player_tank.tank_type) {
                let energy_cost = 1; // 1点蓝条（1/3蓝条）
                if player_stats.energy_blue_bar >= energy_cost {
                    // 立即扣除蓝条
                    player_stats.energy_blue_bar -= energy_cost;

                    // 计算坦克当前朝向
                    let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
                    let actual_angle = euler_angle + 90.0_f32.to_radians();
                    let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

                    // 开始冲刺
                    let dash_timer = DashTimer::new(direction, DASH_DURATION);
                    dash_timers.timers.insert(entity, dash_timer);

                    // 添加冲刺标记
                    commands.entity(entity).insert(IsDashing);
                }
            }
        }
    }
}

fn update_dash_movement(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut KinematicCharacterController, Option<&IsDashing>), With<PlayerTank>>,
    mut dash_timers: ResMut<DashTimers>,
    mut dash_damage_tracker: ResMut<DashDamageTracker>,
) {
    for (entity, mut character_controller, is_dashing) in &mut player_query {
        if matches!(is_dashing, Some(IsDashing))
            && let Some(dash_timer) = dash_timers.timers.get_mut(&entity) {
                // 更新计时器
                dash_timer.timer.tick(time.delta());

                // 计算冲刺速度：距离 / 时间
                let dash_speed = DASH_DISTANCE / DASH_DURATION;

                // 设置移动
                let movement = dash_timer.direction * dash_speed * time.delta_secs();
                character_controller.translation = Some(movement);

                // 检查是否完成
                if dash_timer.timer.just_finished() {
                    // 移除冲刺标记和计时器
                    commands.entity(entity).remove::<IsDashing>();
                    dash_timers.timers.remove(&entity);
                    
                    // 清理扣血追踪
                    dash_damage_tracker.has_taken_damage.remove(&entity);
                }
            }
    }
}

fn handle_dash_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut effect_events: MessageWriter<crate::bullet::EffectEvent>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    player_tanks: Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: Query<(Entity, &Transform), With<PlayerTank>>,
    enemy_tanks: Query<(Entity, &Transform), With<EnemyTank>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    mut player_info: ResMut<PlayerInfo>,
    player_avatars: Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
    mut dash_damage_tracker: ResMut<DashDamageTracker>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // 检查是否是玩家坦克与 brick/steel/敌方坦克的碰撞
            let (player_entity, brick_entity, steel_entity, enemy_entity): (Entity, Option<Entity>, Option<Entity>, Option<Entity>) = if let Ok((player_entity, player_tank, is_dashing)) = player_tanks.get(*e1) {
                if is_dashing.is_some() && steels.get(*e2).is_ok() {
                    // 撞到 steel，检查 protection
                    let can_break_steel = if let Some(player_stats) = player_info.players.get(&player_tank.tank_type) {
                        player_stats.protection >= 100
                    } else {
                        false
                    };

                    if can_break_steel {
                        // protection = 100%，可以撞碎铁块
                        (player_entity, None, Some(*e2), None)
                    } else {
                        // protection < 100%，玩家死亡
                        handle_steel_collision(
                            &mut commands,
                            &mut effect_events,
                            &asset_server,
                            &mut texture_atlas_layouts,
                            &player_tanks,
                            &player_tanks_with_transform,
                            &mut player_info,
                            &player_avatars,
                            &mut stat_changed_events,
                            player_entity,
                        );
                        continue;
                    }
                } else if is_dashing.is_some() && bricks.get(*e2).is_ok() {
                    (player_entity, Some(*e2), None, None)
                } else if let Some(enemy) = check_enemy_collision(player_entity, *e1, *e2, &player_tanks, &enemy_tanks) {
                    (player_entity, None, None, Some(enemy))
                } else {
                    continue;
                }
            } else if let Ok((player_entity, player_tank, is_dashing)) = player_tanks.get(*e2) {
                if is_dashing.is_some() && steels.get(*e1).is_ok() {
                    // 撞到 steel，检查 protection
                    let can_break_steel = if let Some(player_stats) = player_info.players.get(&player_tank.tank_type) {
                        player_stats.protection >= 100
                    } else {
                        false
                    };

                    if can_break_steel {
                        // protection = 100%，可以撞碎铁块
                        (player_entity, None, Some(*e1), None)
                    } else {
                        // protection < 100%，玩家死亡
                        handle_steel_collision(
                            &mut commands,
                            &mut effect_events,
                            &asset_server,
                            &mut texture_atlas_layouts,
                            &player_tanks,
                            &player_tanks_with_transform,
                            &mut player_info,
                            &player_avatars,
                            &mut stat_changed_events,
                            player_entity,
                        );
                        continue;
                    }
                } else if is_dashing.is_some() && bricks.get(*e1).is_ok() {
                    (player_entity, Some(*e1), None, None)
                } else if let Some(enemy) = check_enemy_collision(player_entity, *e2, *e1, &player_tanks, &enemy_tanks) {
                    (player_entity, None, None, Some(enemy))
                } else {
                    continue;
                }
            } else if let Some((pe, ee)) = check_enemy_collision_none(*e1, *e2, &player_tanks, &enemy_tanks) {
                (pe, None, None, Some(ee))
            } else {
                continue;
            };

            // 处理 brick 碰撞
            if let Some(b_entity) = brick_entity {
                handle_brick_collision(
                    &mut commands,
                    &mut effect_events,
                    &asset_server,
                    &mut texture_atlas_layouts,
                    &player_tanks,
                    &player_tanks_with_transform,
                    &bricks,
                    &mut player_info,
                    &player_avatars,
                    &mut stat_changed_events,
                    player_entity,
                    b_entity,
                    &mut dash_damage_tracker,
                );
                continue;
            }

            // 处理 steel 碰撞（protection = 100% 时）
            if let Some(s_entity) = steel_entity {
                handle_steel_break(
                    &mut commands,
                    &mut effect_events,
                    &asset_server,
                    &steels,
                    s_entity,
                );
                continue;
            }

            // 处理敌方坦克碰撞
            if let Some(e_entity) = enemy_entity {
                handle_dash_enemy_tank_collision(
                    &mut commands,
                    &mut effect_events,
                    &asset_server,
                    &mut texture_atlas_layouts,
                    &player_tanks,
                    &player_tanks_with_transform,
                    &enemy_tanks,
                    &mut player_info,
                    &player_avatars,
                    &mut stat_changed_events,
                    player_entity,
                    e_entity,
                    &mut dash_damage_tracker,
                );
            }
        }
    }
}

fn check_enemy_collision(
    _player_entity: Entity,
    e1: Entity,
    e2: Entity,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
) -> Option<Entity> {
    if let Ok((_, _, is_dashing)) = player_tanks.get(e1) {
        if is_dashing.is_some() && enemy_tanks.get(e2).is_ok() {
            return Some(e2);
        }
    }
    None
}

fn check_enemy_collision_none(

    e1: Entity,

    e2: Entity,

    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,

    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,

) -> Option<(Entity, Entity)> {

    if let Ok((player_entity, _, is_dashing)) = player_tanks.get(e1) {

        if is_dashing.is_some() && enemy_tanks.get(e2).is_ok() {

            return Some((player_entity, e2));

        }

    } else if let Ok((player_entity, _, is_dashing)) = player_tanks.get(e2) {

        if is_dashing.is_some() && enemy_tanks.get(e1).is_ok() {

            return Some((player_entity, e1));

        }

    }

    None

}

fn handle_brick_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    bricks: &Query<(Entity, &Transform), With<Brick>>,
    player_info: &mut ResMut<PlayerInfo>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    _stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    player_entity: Entity,
    brick_entity: Entity,
    dash_damage_tracker: &mut DashDamageTracker,
) {
    // 获取玩家坦克信息
    let player_tank = player_tanks.iter().find_map(|(e, pt, _)| {
        if e == player_entity { Some(pt) } else { None }
    }).unwrap();

    // 获取 brick 位置用于生成效果
    if let Ok((_, brick_transform)) = bricks.get(brick_entity) {
        // 播放砖块被击中的音效
        let brick_hit_sound: Handle<AudioSource> = asset_server.load(SOUND_BRICK_HIT);
        commands.spawn((
            AudioPlayer::new(brick_hit_sound),
            PlaybackSettings::ONCE,
        ));

        // 发送火花特效事件
        effect_events.write(crate::bullet::EffectEvent::Spark {
            position: brick_transform.translation,
        });

        // 销毁 brick
        let _ = commands.entity(brick_entity).try_despawn();
    }

    // 检查本次 dash 是否已经扣过血
    if dash_damage_tracker.has_taken_damage.contains(&player_entity) {
        return; // 已经扣过血，不再重复扣血
    }

    // 根据 protection 百分比决定扣血量
    if let Some(player_stats) = player_info.players.get_mut(&player_tank.tank_type) {
        let health_cost = if player_stats.protection < 40 {
            2 // 2/3血条
        } else if player_stats.protection < 80 {
            1 // 1/3血条
        } else {
            0 // 不扣血
        };

        player_stats.life_red_bar = player_stats.life_red_bar.saturating_sub(health_cost);

        // 标记本次 dash 已经扣过血
        if health_cost > 0 {
            dash_damage_tracker.has_taken_damage.insert(player_entity);
        }

        // 检查玩家是否死亡
        if player_stats.life_red_bar == 0 {
            // 获取玩家坦克位置用于生成爆炸效果
            if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
                // 生成爆炸效果
                spawn_explosion(commands, asset_server, texture_atlas_layouts, tank_transform.translation);
            }

            // 销毁玩家坦克
            let _ = commands.entity(player_entity).try_despawn();

            // 标记对应玩家的头像为死亡状态
            for (avatar_entity, player_index) in player_avatars.iter() {
                if player_index.player_type == player_tank.tank_type {
                    commands.entity(avatar_entity).insert(PlayerDead);
                }
            }

            // 启动 Game Over 延迟计时器（1.2秒）
            commands.spawn((
                GameOverTimer,
                AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
            ));
        }
    }
}

fn handle_steel_collision(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    _asset_server: &Res<AssetServer>,
    _texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    player_info: &mut ResMut<PlayerInfo>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    _stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    player_entity: Entity,
) {
    // 获取玩家坦克信息
    let player_tank = player_tanks.iter().find_map(|(e, pt, _)| {
        if e == player_entity { Some(pt) } else { None }
    }).unwrap();

    // 检查 protection 是否为 100%
    let can_break_steel = if let Some(player_stats) = player_info.players.get(&player_tank.tank_type) {
        player_stats.protection >= 100
    } else {
        false
    };

    if can_break_steel {
        // protection = 100%，可以撞碎铁块，不扣血
        // 发送火花特效事件
        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
            effect_events.write(crate::bullet::EffectEvent::Spark {
                position: tank_transform.translation,
            });
        }
        // 铁块被撞碎的效果（可以在这里添加更多效果）
    } else {
        // protection < 100%，玩家死亡
        // 发送爆炸特效事件
        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
            effect_events.write(crate::bullet::EffectEvent::Explosion {
                position: tank_transform.translation,
            });
        }

        // 销毁玩家坦克
        let _ = commands.entity(player_entity).try_despawn();

        // 标记对应玩家的头像为死亡状态
        for (avatar_entity, player_index) in player_avatars.iter() {
            if player_index.player_type == player_tank.tank_type {
                commands.entity(avatar_entity).insert(PlayerDead);
            }
        }

        // 启动 Game Over 延迟计时器（1.2秒）
        commands.spawn((
            GameOverTimer,
            AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
        ));
    }
}

fn handle_steel_break(
    commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    _asset_server: &Res<AssetServer>,
    steels: &Query<(Entity, &Transform), With<Steel>>,
    steel_entity: Entity,
) {
    // 获取 steel 位置用于生成效果
    if let Ok((_, steel_transform)) = steels.get(steel_entity) {
        // 发送火花特效事件
        effect_events.write(crate::bullet::EffectEvent::Spark {
            position: steel_transform.translation,
        });

        // 销毁 steel
        let _ = commands.entity(steel_entity).try_despawn();
    }
}

fn handle_dash_enemy_tank_collision(
    mut commands: &mut Commands,
    effect_events: &mut MessageWriter<crate::bullet::EffectEvent>,
    asset_server: &Res<AssetServer>,
    mut texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_tanks: &Query<(Entity, &PlayerTank, Option<&IsDashing>)>,
    player_tanks_with_transform: &Query<(Entity, &Transform), With<PlayerTank>>,
    enemy_tanks: &Query<(Entity, &Transform), With<EnemyTank>>,
    player_info: &mut ResMut<PlayerInfo>,
    player_avatars: &Query<(Entity, &PlayerUI), With<PlayerAvatar>>,
    stat_changed_events: &mut MessageWriter<PlayerStatChanged>,
    player_entity: Entity,
    enemy_entity: Entity,
    dash_damage_tracker: &mut DashDamageTracker,
) {
    // 获取玩家坦克信息
    let (_, player_tank, _) = player_tanks.get(player_entity).unwrap();

    // 获取敌方坦克位置用于生成爆炸效果
    if let Ok((_, enemy_transform)) = enemy_tanks.get(enemy_entity) {
        // 发送爆炸特效事件
        effect_events.write(crate::bullet::EffectEvent::Explosion {
            position: enemy_transform.translation,
        });
    }

    // 销毁敌方坦克
    let _ = commands.entity(enemy_entity).try_despawn();

    // 检查本次 dash 是否已经扣过血
    if dash_damage_tracker.has_taken_damage.contains(&player_entity) {
        return; // 已经扣过血，不再重复扣血
    }

    // 增加分数
    if let Some(player_stats) = player_info.players.get_mut(&player_tank.tank_type) {
        player_stats.score += 100;

        // 发送分数变更事件
        stat_changed_events.write(PlayerStatChanged {
            player_type: player_tank.tank_type,
            stat_type: StatType::Score,
        });

        // 根据 protection 百分比决定扣血量
        let health_cost = if player_stats.protection < 40 {
            2 // 2/3血条
        } else if player_stats.protection < 80 {
            1 // 1/3血条
        } else {
            0 // 不扣血
        };
        player_stats.life_red_bar = player_stats.life_red_bar.saturating_sub(health_cost);

        // 标记本次 dash 已经扣过血
        if health_cost > 0 {
            dash_damage_tracker.has_taken_damage.insert(player_entity);
        }

        // 检查玩家是否死亡
        if player_stats.life_red_bar == 0 {
            // 获取玩家坦克位置用于生成爆炸效果
            if let Ok((_, tank_transform)) = player_tanks_with_transform.get(player_entity) {
                // 生成爆炸效果
                spawn_explosion(&mut commands, &asset_server, &mut texture_atlas_layouts, tank_transform.translation);
            }

            // 销毁玩家坦克
            let _ = commands.entity(player_entity).try_despawn();

            // 标记对应玩家的头像为死亡状态
            for (avatar_entity, player_index) in player_avatars.iter() {
                if player_index.player_type == player_tank.tank_type {
                    commands.entity(avatar_entity).insert(PlayerDead);
                }
            }

            // 启动 Game Over 延迟计时器（1.2秒）
            commands.spawn((
                GameOverTimer,
                AnimationTimer(Timer::from_seconds(1.2, TimerMode::Once)),
            ));
        }
    }
}

fn handle_barrier_collision(
    time: Res<Time>,
    player_tanks: Query<(Entity, &Transform, &PlayerTank), With<PlayerTank>>,
    barriers: Query<(&Transform, Entity), With<Barrier>>,
    mut player_info: ResMut<PlayerInfo>,
    mut barrier_damage_tracker: ResMut<BarrierDamageTracker>,
    mut stat_changed_events: MessageWriter<PlayerStatChanged>,
) {
    // 更新所有冷却计时器
    for (_, timer) in barrier_damage_tracker.cooldowns.iter_mut() {
        timer.tick(time.delta());
    }

    // 检测玩家坦克与 barrier 的距离
    for (player_entity, player_transform, player_tank) in player_tanks.iter() {
        for (barrier_transform, _barrier_entity) in barriers.iter() {
            // 计算距离
            let distance = (player_transform.translation - barrier_transform.translation).length();

            // 如果距离小于阈值，则认为碰撞
            const COLLISION_THRESHOLD: f32 = 70.0;

            if distance < COLLISION_THRESHOLD {
                // 检查冷却是否结束
                let can_take_damage = barrier_damage_tracker.cooldowns
                    .get(&player_entity)
                    .map_or(true, |timer| timer.is_finished());

                if can_take_damage {
                    // 检查玩家是否有 track_chain，如果有则免疫伤害
                    let has_track_chain = if let Some(player_stats) = player_info.players.get(&player_tank.tank_type) {
                        player_stats.track_chain
                    } else {
                        false
                    };

                    if has_track_chain {
                        // 拥有 track_chain，免疫伤害，直接跳过
                        continue;
                    }

                    // 设置 2 秒冷却
                    barrier_damage_tracker.cooldowns.insert(
                        player_entity,
                        Timer::from_seconds(2.0, TimerMode::Once)
                    );

                    // 永久减少 speed 20 和 protection 20（固定值）
                    if let Some(player_stats) = player_info.players.get_mut(&player_tank.tank_type) {
                        player_stats.speed = player_stats.speed.saturating_sub(20);
                        player_stats.protection = player_stats.protection.saturating_sub(20);

                        // 发送 speed 和 protection 变更事件
                        stat_changed_events.write(PlayerStatChanged {
                            player_type: player_tank.tank_type,
                            stat_type: StatType::Speed,
                        });
                        stat_changed_events.write(PlayerStatChanged {
                            player_type: player_tank.tank_type,
                            stat_type: StatType::Protection,
                        });
                    }
                }
            }
        }
    }
}

fn setup_fade_out(
    mut fading_out: ResMut<FadingOut>,
) {
    fading_out.alpha = 1.0;
}

fn update_sprite_alpha(alpha: f32, sprite: &mut Sprite) {
    let linear = sprite.color.to_linear();
    sprite.color = Color::srgba(linear.red, linear.green, linear.blue, alpha);
}

fn update_text_color_alpha(alpha: f32, text_color: &mut TextColor) {
    let linear = text_color.0.to_linear();
    text_color.0 = Color::srgba(linear.red, linear.green, linear.blue, alpha);
}

fn cleanup_start_screen(
    commands: &mut Commands,
    sprite_query: &Query<(Entity, &mut Sprite), With<StartScreenUI>>,
    text_query: &Query<(Entity, &mut TextColor, Option<&MenuOption>), With<StartScreenUI>>,
) {
    for (entity, _) in sprite_query.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
    for (entity, _, _) in text_query.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
}

fn spawn_stage_intro(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut stage_intro_timer: ResMut<StageIntroTimer>,
    mut clear_color: ResMut<ClearColor>,
    stage_level: Res<StageLevel>,
) {
    // 设置背景色为白色
    clear_color.0 = Color::srgb(1.0, 1.0, 1.0);

    // 初始化计时器
    stage_intro_timer.fade_in = Timer::from_seconds(1.0, TimerMode::Once);
    stage_intro_timer.stay = Timer::from_seconds(2.0, TimerMode::Once);
    stage_intro_timer.fade_out = Timer::from_seconds(1.0, TimerMode::Once);

    // 加载字体
    let font_en: Handle<Font> = asset_server.load(crate::FONT_EN);
    let font_cn: Handle<Font> = asset_server.load(crate::FONT_CN);

    // Stage 标题（显示在屏幕中心）
    commands.spawn((
        StageIntroUI,
        Text2d(format!("Stage {}", stage_level.0)),
        TextFont {
            font_size: 80.0,
            font: font_en.clone(),
            ..default()
        },
        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)), // 黑色，初始透明度为0
        Transform::from_xyz(0.0, 100.0, 1.0),
    ));

    // 描述文字（随机选择一条俏皮话）
    let mut rng = rand::rng();
    let quote_index = rng.random_range(0..STAGE_QUOTES_CN.len());
    let quote_text = STAGE_QUOTES_CN[quote_index];
    commands.spawn((
        StageIntroUI,
        Text2d(quote_text.to_string()),
        TextFont {
            font_size: 28.0,
            font: font_cn,
            ..default()
        },
        TextColor(Color::srgba(0.3, 0.3, 0.3, 0.0)), // 暗灰色，初始透明度为0
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, -50.0, 1.0),
    ));
}

fn handle_stage_intro_timer(
    time: Res<Time>,
    mut stage_intro_timer: ResMut<StageIntroTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut text_query: Query<&mut TextColor, With<StageIntroUI>>,
) {
    // 淡入阶段
    if !stage_intro_timer.fade_in.is_finished() {
        stage_intro_timer.fade_in.tick(time.delta());
        let progress = stage_intro_timer.fade_in.elapsed_secs() / stage_intro_timer.fade_in.duration().as_secs_f32();
        let alpha = progress.min(1.0);
        for mut text_color in &mut text_query {
            // 获取当前颜色（不包含透明度）
            let color = text_color.0;
            // 只更新透明度，保持原始颜色
            text_color.0 = color.with_alpha(alpha);
        }
    }
    // 停留阶段
    else if !stage_intro_timer.stay.is_finished() {
        stage_intro_timer.stay.tick(time.delta());
    }
    // 淡出阶段
    else if !stage_intro_timer.fade_out.is_finished() {
        stage_intro_timer.fade_out.tick(time.delta());
        let progress = stage_intro_timer.fade_out.elapsed_secs() / stage_intro_timer.fade_out.duration().as_secs_f32();
        let alpha = 1.0 - progress.min(1.0);
        for mut text_color in &mut text_query {
            // 获取当前颜色（不包含透明度）
            let color = text_color.0;
            // 只更新透明度，保持原始颜色
            text_color.0 = color.with_alpha(alpha);
        }
    }
    // 所有阶段完成，切换到 Playing 状态
    else {
        next_state.set(GameState::Playing);
    }
}

fn despawn_stage_intro(
    mut commands: Commands,
    stage_intro_query: Query<Entity, With<StageIntroUI>>,
) {
    for entity in stage_intro_query.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
}

fn fade_out_screen(
    mut commands: Commands,
    time: Res<Time>,
    mut fading_out: ResMut<FadingOut>,
    mut next_state: ResMut<NextState<GameState>>,
    menu_selection: Res<CurrentMenuSelection>,
    mut sprite_query: Query<(Entity, &mut Sprite), With<StartScreenUI>>,
    mut text_query: Query<(Entity, &mut TextColor, Option<&MenuOption>), With<StartScreenUI>>,
) {
    // 减少透明度
    fading_out.alpha -= time.delta_secs() * (1.0 / 1.5); // 淡出速度，1.5秒完成

    // 更新所有 Sprite 元素的透明度
    for (_, mut sprite) in &mut sprite_query {
        update_sprite_alpha(fading_out.alpha, &mut sprite);
    }

    // 更新所有 Text 元素的颜色（跳过当前选中的选项，因为它的闪烁由 update_menu_blink 处理）
    let selected_index = menu_selection.selected_index;

    for (_, mut text_color, menu_option) in &mut text_query {
        // 如果是当前选中的选项，跳过透明度更新
        if menu_option.is_some_and(|opt| opt.index == selected_index) {
            continue;
        }
        update_text_color_alpha(fading_out.alpha, &mut text_color);
    }

    // 淡出完成，切换到 StageIntro 状态并清理所有 StartScreenUI 元素
    if fading_out.alpha <= 0.0 {
        next_state.set(GameState::StageIntro);
        cleanup_start_screen(&mut commands, &sprite_query, &text_query);
    }
}

fn update_option_colors(
    menu_selection: Res<CurrentMenuSelection>,
    mut text_query: Query<(&MenuOption, &mut TextColor)>,
) {
    for (option, mut text_color) in &mut text_query {
        if option.index == menu_selection.selected_index {
            // 选中的选项使用黄色
            text_color.0 = Color::srgb(1.0, 1.0, 0.0);
        } else {
            text_color.0 = Color::srgb(1.0, 1.0, 1.0); // 白色
        }
    }
}

// 文本更新函数类型
type TextUpdateFn = fn(&PlayerStats, TankType) -> Option<String>;

// 获取文本更新函数
fn get_text_update_fn(prefix: &str) -> TextUpdateFn {
    match prefix {
        s if s.starts_with("Scores") => |stats, _| {
            Some(format!("Scores: {}", stats.score))
        },
        s if s.starts_with("Speed") => |stats, _| {
            Some(if stats.speed < 100 {
                format!("Speed: {}%", stats.speed)
            } else {
                "Speed: Max".to_string()
            })
        },
        s if s.starts_with("Shells") => |stats, _| {
            Some(format!("Shells: {}", stats.shells))
        },
        s if s.starts_with("Protection") => |stats, _| {
            Some(if stats.protection < 100 {
                format!("Protection: {}%", stats.protection)
            } else {
                "Protection: Max".to_string()
            })
        },
        s if s.starts_with("Fire Speed") => |stats, _| {
            Some(if stats.fire_speed < 100 {
                format!("Fire Speed: {}%", stats.fire_speed)
            } else {
                "Fire Speed: Max".to_string()
            })
        },
        s if s.starts_with("Fire Shell") => |stats, _| {
            if stats.fire_shell {
                Some("Fire Shell: On".to_string())
            } else {
                Some("Fire Shell: Off".to_string())
            }
        },
        s if s.starts_with("Air Cushion") => |stats, _| {
            if stats.air_cushion {
                Some("Air Cushion: On".to_string())
            } else {
                Some("Air Cushion: Off".to_string())
            }
        },
        s if s.starts_with("Track Chain") => |stats, _| {
            if stats.track_chain {
                Some("Track Chain: On".to_string())
            } else {
                Some("Track Chain: Off".to_string())
            }
        },
        s if s.starts_with("Penetrate") => |stats, _| {
            if stats.penetrate {
                Some("Penetrate: On".to_string())
            } else {
                Some("Penetrate: Off".to_string())
            }
        },
        _ => |_, _| None,
    }
}

fn update_player_info_display(
    changed_player_info: Res<PlayerInfo>,
    mut text2ds: Query<(&PlayerUI, &mut Text2d), With<Text2d>>,
    mut bar_queries: ParamSet<(
        Query<(&mut Sprite, &HealthBarOriginalPosition, &mut Transform, &PlayerUI), With<HealthBar>>,
        Query<(&mut Sprite, &BlueBarOriginalPosition, &mut Transform, &PlayerUI), With<BlueBar>>,
    )>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
) {
    for player_tank in player_tanks {
        if let Some(player_stats) = changed_player_info.players.get(&player_tank.tank_type) {
            // 更新文本信息
            for (player_index, mut text) in &mut text2ds {
                if player_tank.tank_type != player_index.player_type {
                    continue;
                }
                let update_fn = get_text_update_fn(&text.0);
                if let Some(new_text) = update_fn(player_stats, player_index.player_type) {
                    text.0 = new_text;
                }
            }

            // 更新血条
            for (mut sprite, original_pos, mut transform, player_index) in &mut bar_queries.p0() {
                if player_tank.tank_type != player_index.player_type {
                    continue;
                }
                // 血条总宽度 160，生命值 3，每条代表 1/3
                let health_width = (player_stats.life_red_bar as f32 / 3.0) * 160.0;
                sprite.custom_size = Some(Vec2::new(health_width, 10.0));

                // 左对齐：将血条向左移动，使其从左边界开始
                // 原始位置是中心点，需要向左偏移 (160 - health_width) / 2
                let offset = (160.0 - health_width) / 2.0;
                transform.translation.x = original_pos.0 - offset;
            }

            // 更新蓝条
            for (mut sprite, original_pos, mut transform, player_index) in &mut bar_queries.p1() {
                if player_tank.tank_type != player_index.player_type {
                    continue;
                }
                // 蓝条总宽度 160，能量值 100
                let blue_width = (player_stats.energy_blue_bar as f32 / 3.0) * 160.0;
                sprite.custom_size = Some(Vec2::new(blue_width, 10.0));

                // 左对齐：将蓝条向左移动，使其从左边界开始
                // 原始位置是中心点，需要向左偏移 (160 - blue_width) / 2
                let offset = (160.0 - blue_width) / 2.0;
                transform.translation.x = original_pos.0 - offset;
            }
        }
    }
}

fn update_commander_health_bar(
    changed_commander_life: Res<CommanderLife>,
    mut health_bars: Query<(&mut Sprite, &CommanderHealthBarOriginalPosition, &mut Transform), With<CommanderHealthBar>>,
) {
    for (mut sprite, original_pos, mut transform) in &mut health_bars {
        let health_width = (changed_commander_life.life_red_bar as f32 / 3.0) * 160.0;
        sprite.custom_size = Some(Vec2::new(health_width, 10.0));
        transform.translation.x = original_pos.0 - (160.0 - health_width) / 2.0;
    }
}

fn update_blue_bar_regen(
    time: Res<Time>,
    mut regen_timer: ResMut<BlueBarRegenTimer>,
    mut player_info: ResMut<PlayerInfo>,
) {
    // 检查是否有玩家蓝条不满
    let any_player_needs_regen = player_info.players.values().any(|p| p.energy_blue_bar < 3);

    // 只有当有玩家蓝条不满时才更新计时器
    if any_player_needs_regen {
        regen_timer.timer.tick(time.delta());

        // 当计时器触发时，恢复1点蓝条
        if regen_timer.timer.just_finished() {
            for player_stats in player_info.players.values_mut() {
                if player_stats.energy_blue_bar < 3 {
                    player_stats.energy_blue_bar = (player_stats.energy_blue_bar + 1).min(3);
                }
            }
        }
    } else {
        // 所有玩家蓝条都满时，重置计时器
        regen_timer.timer.reset();
    }
}

fn update_menu_blink(
    time: Res<Time>,
    fading_out: Res<FadingOut>,
    menu_selection: Res<CurrentMenuSelection>,
    mut blink_timer: ResMut<MenuBlinkTimer>,
    mut text_query: Query<(&MenuOption, &mut TextColor), Without<MenuArrow>>,
    game_state: Res<State<GameState>>,
) {
    // 在 FadingOut 状态下闪烁 + 淡出
    if *game_state.get() == GameState::FadingOut {
        blink_timer.0.tick(time.delta());

        // 初始化计时器（0.2秒闪烁）
        if blink_timer.0.duration().is_zero() {
            blink_timer.0 = Timer::from_seconds(0.2, TimerMode::Repeating);
        }

        if blink_timer.0.just_finished() {
            for (option, mut text_color) in &mut text_query {
                if option.index == menu_selection.selected_index {
                    // 当前选中的选项闪烁
                    // 出现时使用当前淡出透明度，消失时完全透明
                    let linear = text_color.0.to_linear();
                    let alpha = if linear.alpha < 0.5 {
                        // 当前不可见，切换到可见（使用当前淡出透明度）
                        fading_out.alpha
                    } else {
                        // 当前可见，切换到不可见（完全透明）
                        0.0
                    };
                    text_color.0 = Color::srgb(1.0, 1.0, 0.0).with_alpha(alpha);
                }
            }
        }
    } else {
        // 在 StartScreen 状态下，选中的选项保持黄色常亮
        for (option, mut text_color) in &mut text_query {
            if option.index == menu_selection.selected_index {
                text_color.0 = Color::srgb(1.0, 1.0, 0.0);
            }
        }
    }
}

fn spawn_pause_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_velocity_query: Query<&mut Velocity, With<PlayerTank>>,
    mut enemy_velocity_query: Query<&mut Velocity, (With<EnemyTank>, Without<PlayerTank>)>,
) {
    let font: Handle<Font> = asset_server.load(crate::FONT_EN);

    // 停止玩家坦克的移动
    for mut velocity in &mut player_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    // 停止敌方坦克的移动
    for mut velocity in &mut enemy_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    commands.spawn((
        PauseUI,
        Text2d("PAUSED".to_string()),
        TextFont {
            font_size: 100.0,
            font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));

    commands.spawn((
        PauseUI,
        Text2d("Press SPACE to resume | B to menu | ESC to exit".to_string()),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -100.0, 10.0),
    ));
}

fn despawn_pause_ui(mut commands: Commands, query: Query<Entity, With<PauseUI>>) {
    for entity in query.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
}

fn handle_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Space 键暂停
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Paused);
    }
    // Esc 键退出
    if keyboard_input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Space 键恢复游戏
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
    // B 键返回菜单
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        next_state.set(GameState::StartScreen);
    }
    // Esc 键退出
    if keyboard_input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn spawn_game_over_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_velocity_query: Query<&mut Velocity, With<PlayerTank>>,
    mut enemy_velocity_query: Query<&mut Velocity, (With<EnemyTank>, Without<PlayerTank>)>,
) {
    let font: Handle<Font> = asset_server.load(crate::FONT_EN);

    // 停止玩家坦克的移动
    for mut velocity in &mut player_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    // 停止敌方坦克的移动
    for mut velocity in &mut enemy_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    commands.spawn((
        GameOverUI,
        Text2d("GAME OVER".to_string()),
        TextFont {
            font_size: 100.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 100.0, 10.0),
    ));

    // Restart 选项
    commands.spawn((
        GameOverUI,
        Text2d("RESTART".to_string()),
        TextFont {
            font_size: 50.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, 0.0, 10.0),
        MenuOption { index: 0 },
    ));

    // Back to Menu 选项
    commands.spawn((
        GameOverUI,
        Text2d("BACK TO MENU".to_string()),
        TextFont {
            font_size: 50.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -60.0, 10.0),
        MenuOption { index: 1 },
    ));

    // Exit 选项
    commands.spawn((
        GameOverUI,
        Text2d("EXIT".to_string()),
        TextFont {
            font_size: 50.0,
            font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -120.0, 10.0),
        MenuOption { index: 2 },
    ));

    // 操作说明
    commands.spawn((
        GameOverUI,
        Text2d("W/S to select | SPACE to confirm".to_string()),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -200.0, 10.0),
    ));
}

fn handle_game_over_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu_selection: ResMut<CurrentMenuSelection>,
) {
    // W 键向上选择
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        menu_selection.selected_index = if menu_selection.selected_index == 0 {
            2
        } else {
            menu_selection.selected_index - 1
        };
    }
    // S 键向下选择
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        menu_selection.selected_index = (menu_selection.selected_index + 1) % 3;
    }
    // Space 键确认选择
    if keyboard_input.just_pressed(KeyCode::Space) {
        match menu_selection.selected_index {
            0 => {
                // Restart: 重置游戏状态并重新开始
                next_state.set(GameState::Playing);
            }
            1 => {
                // Back to Menu: 返回开始界面
                next_state.set(GameState::StartScreen);
            }
            2 => {
                // Exit: 退出游戏
                std::process::exit(0);
            }
            _ => {}
        }
    }
}

fn despawn_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
}

// 清理开始界面的UI元素
fn cleanup_start_screen_ui(
    mut commands: Commands,
    start_screen_ui: Query<Entity, With<StartScreenUI>>,
) {
    for entity in start_screen_ui.iter() {
        let _ = commands.entity(entity).try_despawn();
    }
}

// 清理游戏过程中的所有entity
fn cleanup_playing_entities(
    mut commands: Commands,
    playing_entities: Query<Entity, With<PlayingEntity>>,
    mut player_info: ResMut<PlayerInfo>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    mut stage_level: ResMut<StageLevel>,
    mut commander_life: ResMut<CommanderLife>,
    mut entities_spawned: ResMut<GameEntitiesSpawned>,
) {
    // 清理所有游戏实体
    for entity in playing_entities.iter() {
        commands.entity(entity).try_despawn();
    }

    // 重置玩家信息
    player_info.players.clear();

    // 重置敌方坦克计数
    enemy_spawn_state.has_spawned = 0;
    enemy_spawn_state.spawn_cooldown.reset();

    // 重置关卡数
    stage_level.0 = 1;

    // 重置 Commander 生命值
    commander_life.life_red_bar = 3;

    // 重置游戏实体生成标志
    entities_spawned.0 = false;
}

fn check_stage_complete(
    enemy_spawn_state: Res<EnemySpawnState>,
    enemies: Query<(), With<EnemyTank>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stage_level: ResMut<StageLevel>,
) {
    // 检查是否完成关卡：已生成所有敌方坦克且当前没有存活的敌方坦克
    let current_enemy_count = enemies.iter().count();
    if enemy_spawn_state.has_spawned >= enemy_spawn_state.max_count && current_enemy_count == 0 {
        // 进入下一关
        stage_level.0 += 1;
        next_state.set(GameState::StageIntro);
    }
}

fn reset_for_next_stage(
    mut commands: Commands,
    playing_entities: Query<Entity, With<PlayingEntity>>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    mut entities_spawned: ResMut<GameEntitiesSpawned>,
) {
    // 清理所有游戏实体
    for entity in playing_entities.iter() {
        commands.entity(entity).try_despawn();
    }

    // 重置敌方坦克计数
    enemy_spawn_state.has_spawned = 0;
    enemy_spawn_state.spawn_cooldown.reset();

    // 重置游戏实体生成标志
    entities_spawned.0 = false;
}

fn animate_sea(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Sea>>,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

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
}

fn play_sea_ambience(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    player_tanks: Query<&Transform, With<PlayerTank>>,
    seas: Query<&Transform, With<Sea>>,
    ambience_players: Query<(Entity, &mut AudioPlayer), With<SeaAmbiencePlayer>>,
) {
    // 检查是否有玩家坦克在海附近
    let mut is_near_sea = false;
    const DETECTION_RADIUS: f32 = 150.0; // 海检测半径

    for player_transform in player_tanks.iter() {
        for sea_transform in seas.iter() {
            let distance = player_transform.translation.distance(sea_transform.translation);
            if distance < DETECTION_RADIUS {
                is_near_sea = true;
                break;
            }
        }
        if is_near_sea {
            break;
        }
    }

    if is_near_sea {
        // 如果在海附近但没有播放音效，则播放
        if ambience_players.is_empty() {
            let sea_ambience_sound: Handle<AudioSource> = asset_server.load(SOUND_SEA_AMBIENCE);
            commands.spawn((
                AudioPlayer::new(sea_ambience_sound),
                PlaybackSettings::LOOP.with_volume(Volume::Linear(0.3)),
                SeaAmbiencePlayer,
            ));
        }
    } else {
        // 如果不在海附近但有播放音效，则停止
        for (entity, _) in ambience_players.iter() {
            let _ = commands.entity(entity).try_despawn();
        }
    }
}

fn enemy_spawn_system(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    enemy_tanks: Query<(), With<EnemyTank>>,
) {
    // 更新生成冷却时间
    enemy_spawn_state.spawn_cooldown.tick(time.delta());

    // 动态获取当前场上敌方坦克数量
    let current_enemy_count = enemy_tanks.iter().count();

    // 检查是否需要生成新敌人
    // 条件：未达到总数上限 + 场上敌人数量少于4个 + 冷却时间已结束
    if enemy_spawn_state.has_spawned < enemy_spawn_state.max_count
        && current_enemy_count < 4
        && enemy_spawn_state.spawn_cooldown.is_finished()
    {
        // 生成敌方坦克出生动画
        let mut rng = rand::rng();
        let random_index = rng.random_range(0..ENEMY_BORN_PLACES.len());
        let position = ENEMY_BORN_PLACES[random_index];
        spawn_enemy_born_animation(&mut commands, &asset_server, &mut texture_atlas_layouts, position);

        // 更新计数
        enemy_spawn_state.has_spawned += 1;

        // 重置冷却时间
        enemy_spawn_state.spawn_cooldown.reset();
    }
}    
    fn update_air_cushion_effect(
    
        mut commands: Commands,
    
        asset_server: Res<AssetServer>,
    
        player_tanks: Query<(Entity, Option<&Children>, Has<crate::constants::BubbleEffect>), With<PlayerTank>>,
    
        bubble_effects: Query<&crate::constants::BubbleEffect>,
    
    ) {
    
        for (entity, children, has_bubble_effect) in player_tanks.iter() {
    
            if has_bubble_effect {
    
                // 检查是否已经有气泡特效子实体
    
                let has_bubble_sprite = if let Some(children) = children {
    
                    children.iter().any(|child| bubble_effects.contains(child))
    
                } else {
    
                    false
    
                };
    
        
    
                if !has_bubble_sprite {
    
                    // 加载气泡纹理并缩放到 100x100
    
                                        let bubble_texture: Handle<Image> = asset_server.load(TEXTURE_BUBBLE);
    
                                        
    
                                        
    
                                        // 创建气泡特效实体
    
                                        
    
                                        
    
                                                                                commands.entity(entity).with_children(|parent| {
    
                                        
    
                                        
    
                                                                                    parent.spawn((
    
                                        
    
                                        
    
                                                                                        Sprite {
    
                                        
    
                                        
    
                                                                                            image: bubble_texture,
    
                                        
    
                                        
    
                                                                                            custom_size: Some(Vec2::new(100.0, 100.0)),
    
                                        
    
                                        
    
                                                                                            ..default()
    
                                        
    
                                        
    
                                                                                        },
    
                            Transform::from_xyz(0.0, 0.0, 1.0), // 在坦克中心
    
                            crate::constants::BubbleEffect,
    
                        ));
    
                    });
    
                }
    
            } else {
    
                // 移除所有气泡特效子实体
    
                if let Some(children) = children {

                    for child in children.iter() {

                        if bubble_effects.contains(child) {

                            let _ = commands.entity(child).try_despawn();

                        }

                    }

                }

            }

        }

    }
