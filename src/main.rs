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

use bevy::{
    prelude::*,
    window::{
        PresentMode,
        WindowTheme,
    },
};
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[allow(clippy::wildcard_imports)]
use constants::*;
#[allow(clippy::wildcard_imports)]
use resources::*;




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
    AssetPlugin {
        processed_file_path: "assets".to_string(),
        unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
        ..default()
    }
}

fn configure_game_resources(app: &mut App) {
    app.init_state::<GameState>()
        .init_resource::<CanFire>()
        .init_resource::<BulletOwners>()
        .init_resource::<StartAnimationFrames>()
        .init_resource::<FadingOut>()
        .init_resource::<CurrentMenuSelection>()
        .init_resource::<GameStarted>()
        .init_resource::<MenuBlinkTimer>()
        .insert_resource(PlayerRespawnTimer(Timer::from_seconds(3.0, TimerMode::Once)))
        .insert_resource(ClearColor(BACKGROUND_COLOR));
}

fn register_game_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::StartScreen), (cleanup_playing_entities, spawn_start_screen).chain())
        .add_systems(OnEnter(GameState::FadingOut), setup_fade_out)
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities)
        .add_systems(OnEnter(GameState::Paused), spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), ( despawn_pause_ui,))
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), (despawn_game_over_ui, cleanup_playing_entities))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            (move_enemy_tanks).chain(),
            move_player_tank,
            animate_player_tank_texture,
            animate_enemy_tank_texture,
            animate_player_avatar,
            animate_powerup_border,
            animate_powerup_texture,
            animate_player_info_text,
            animate_explosion,
            animate_spark,
            handle_game_over_delay,
            shoot_bullets,
            player_shoot_bullet,
            check_bullet_bounds,
            check_bullet_destruction,
            handle_bullet_collisions,
            handle_powerup_collision,
            update_player_info_display,
            update_health_bar,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_start_screen.run_if(not(in_state(GameState::Playing))))
        .add_systems(Update, (
            handle_start_screen_input,
            update_menu_highlight,
            update_option_colors,
        ).run_if(in_state(GameState::StartScreen)))
        .add_systems(Update, update_menu_blink.run_if(in_state(GameState::FadingOut)))
        .add_systems(Update, handle_game_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_pause_input.run_if(in_state(GameState::Paused)))
        .add_systems(Update, (handle_game_over_input, update_menu_highlight, update_option_colors)
            .chain().run_if(in_state(GameState::GameOver)))
        .add_systems(Update, update_menu_highlight.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, update_option_colors.run_if(in_state(GameState::GameOver)))
        .add_systems(Update, fade_out_screen.run_if(in_state(GameState::FadingOut)));
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(configure_window_plugin())
            .set(configure_asset_plugin()),
    ))
    .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0));

    configure_game_resources(&mut app);
    register_game_systems(&mut app);

    app.run();
}

fn load_animation_frames(
    asset_server: &Res<AssetServer>,
    animation_frames: &mut ResMut<StartAnimationFrames>,
) {
    // 预加载所有 70 帧动画，避免播放时按需加载导致闪烁
    for i in 0..70 {
        let frame_num = format!("{:04}", i + 1);
        let texture = asset_server.load(format!("start_scene/frame_{frame_num}.png"));
        animation_frames.frames.push(texture);
    }
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
    let animation_indices = AnimationIndices { first: 0, last: 69 };

    commands.spawn((
        StartScreenUI,
        Sprite::from_image(animation_frames.frames[0].clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.083, TimerMode::Repeating)),
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

    // 菜单箭头（初始指向 PLAY）
    commands.spawn((
        StartScreenUI,
        MenuArrow,
        Text2d("->".to_string()),
        TextFont {
            font_size: 80.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色箭头
        Transform::from_xyz(-150.0, 0.0, 1.0),
    ));

    // PLAY 选项
    commands.spawn((
        StartScreenUI,
        Text2d("PLAY".to_string()),
        TextFont {
            font_size: 80.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, 0.0, 1.0),
        MenuOption { index: 0 },
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
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -100.0, 1.0),
        MenuOption { index: 1 },
    ));
}

fn spawn_start_screen_instructions(commands: &mut Commands) {
    commands.spawn((
        StartScreenUI,
        Text2d("W/S to select | SPACE to select/pause | J to shoot | ESC to exit".to_string()),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -450.0, 1.0),
    ));
}

fn spawn_start_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_frames: ResMut<StartAnimationFrames>,
    mut clear_color: ResMut<ClearColor>,
) {
    // 设置背景色为蓝色
    clear_color.0 = START_SCREEN_BACKGROUND_COLOR;

    // 加载所有动画帧
    load_animation_frames(&asset_server, &mut animation_frames);

    // 添加动画背景
    spawn_start_screen_background(&mut commands, &animation_frames);

    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load("/home/nanbert/.fonts/SHOWG.TTF");

    // 添加标题文字
    spawn_start_screen_title(&mut commands, custom_font);

    // 添加操作说明
    spawn_start_screen_instructions(&mut commands);
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

fn spawn_fortress(commands: &mut Commands) {
    let fortress_y = MAP_BOTTOM_Y + FORTRESS_SIZE / 2.0 + 20.0 ;
    let fortress_x = 0.0;

    // 堡垒主体（红色五角星）
    commands.spawn((
        Fortress,
        PlayingEntity,
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(FORTRESS_SIZE, FORTRESS_SIZE)),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(FORTRESS_SIZE / 200.0, FORTRESS_SIZE / 200.0),
        Transform{
            translation: Vec3::new(fortress_x, fortress_y, 0.0),
            ..default()
        }
    ));

    // 堡垒保护墙 - 左墙
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.6, 0.4, 0.2), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(WALL_THICKNESS / 200.0, FORTRESS_SIZE / 200.0),
        Transform{
            translation: Vec3::new(fortress_x - FORTRESS_SIZE / 2.0 - WALL_THICKNESS / 2.0, fortress_y, 0.0),
            scale: Vec3::new(WALL_THICKNESS, FORTRESS_SIZE, 1.0),
            ..default()
        }
    ));

    // 堡垒保护墙 - 右墙
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.6, 0.4, 0.2), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(WALL_THICKNESS / 200.0, FORTRESS_SIZE / 200.0),
        Transform{
            translation: Vec3::new(fortress_x + FORTRESS_SIZE / 2.0 + WALL_THICKNESS / 2.0, fortress_y, 0.0),
            scale: Vec3::new(WALL_THICKNESS, FORTRESS_SIZE, 1.0),
            ..default()
        }
    ));

    // 堡垒保护墙 - 上墙
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.6, 0.4, 0.2), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(FORTRESS_SIZE / 200.0, WALL_THICKNESS / 200.0),
        Transform{
            translation: Vec3::new(fortress_x, fortress_y + FORTRESS_SIZE / 2.0 + WALL_THICKNESS / 2.0, 0.0),
            scale: Vec3::new(FORTRESS_SIZE, WALL_THICKNESS, 1.0),
            ..default()
        }
    ));
}

fn spawn_player1_tank(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
) -> (Entity, PlayerTank) {
    let player_tank = PlayerTank{
        score:0,
        index: 0,
        name: "player1".to_string(),
        speed: 40,
        fire_speed: 40,
        protection: 40,
        shells: 1,
        penetrate: false,
        track_chain: false,
        air_cushion: false,
        fire_shell:false,
        life_red_bar: 3,
        energy_blue_bar: 100,
    };

    let entity = commands.spawn_empty()
        .insert(player_tank.clone())
        .insert(PlayingEntity)
        .insert(RotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))
        .insert(TargetRotation { angle: 180.0_f32.to_radians() })
        .insert(Sprite::from_atlas_image(
            texture,
            TextureAtlas{
                layout: texture_atlas_layout,
                index: animation_indices.first,
            }
        ))
        .insert(Transform::from_xyz(WALL_THICKNESS.mul_add(-2.0, -FORTRESS_SIZE) - TANK_WIDTH / 2.0 - 20.0, MAP_BOTTOM_Y+TANK_HEIGHT / 2.0, 0.0))
        .insert(Velocity{ linvel: Vec2::default(), angvel: 0.0 })
        .insert(animation_indices)
        .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC | ActiveCollisionTypes::KINEMATIC_KINEMATIC)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            ..default()
        })
        .id();

    (entity, player_tank)
}

fn spawn_enemy_tank(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
    position: Vec3,
) -> Entity {
    commands.spawn_empty()
        .insert(EnemyTank {
            direction: Vec2::new(0.0, -1.0),
        })
        .insert(PlayingEntity)
        .insert(DirectionChangeTimer(Timer::from_seconds(2.0, TimerMode::Once)))
        .insert(CollisionCooldownTimer(Timer::from_seconds(0.5, TimerMode::Once)))
        .insert(RotationTimer(Timer::from_seconds(0.6, TimerMode::Once)))
        .insert(TargetRotation { angle: 270.0_f32.to_radians() })
        .insert(AnimationTimer(Timer::from_seconds(0.25, TimerMode::Repeating)))
        .insert(Sprite::from_atlas_image(
            texture,
            TextureAtlas{
                layout: texture_atlas_layout,
                index: animation_indices.first,
            }
        ))
        .insert(Transform::from_translation(position))
        .insert(animation_indices)
        .insert(Velocity{
            linvel: Vec2::new(0.0, -TANK_SPEED),
            angvel: 0.0,
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0))
        .insert(ActiveEvents::COLLISION_EVENTS|ActiveEvents::CONTACT_FORCE_EVENTS)
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::DYNAMIC_DYNAMIC | ActiveCollisionTypes::DYNAMIC_STATIC)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(GravityScale(0.0))
        .insert(Friction::new(0.0))
        .insert(Restitution::new(0.0))
        .id()
}

fn spawn_player1_info(
    commands: &mut Commands,
    player_tank: &PlayerTank,
    font: &Handle<Font>,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) {
    // 生成玩家1 UI 元素
    for config in PLAYER1_UI_ELEMENTS {
        spawn_ui_element_from_config(commands, font, asset_server, texture_atlas_layouts,config,player_tank);
    }
}

fn spawn_top_text_info(
    commands: &mut Commands,
    font: &Handle<Font>,
) {
    // 其他游戏信息 UI 元素配置
    commands.spawn((
        PlayingEntity,
        Text2d("Commander Life:".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(WINDOW_LEFT_X + 465.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
    commands.spawn((
        PlayingEntity,
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
    player_tank: &PlayerTank,
) {
    match config.element_type {
        UIElementType::NormalText(f) => {
            let text = f(player_tank);
            commands.spawn((
                PlayerIndex(player_tank.index),
                PlayingEntity,
                Text2d(text),
                TextFont {
                    font_size: config.font_size,
                    font: font.clone(),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
        UIElementType::PlayerAvatar => {
            let player_avatar_texture: Handle<Image> = asset_server.load("player.png");
            let player_avatar_tile_size = UVec2::new(120, 112);
            let player_avatar_texture_atlas = TextureAtlasLayout::from_grid(player_avatar_tile_size, 13, 4, None, None);
            let player_avatar_texture_atlas_layout = texture_atlas_layouts.add(player_avatar_texture_atlas);
            let player_avatar_animation_indices = AnimationIndices { first: 0, last: 51 };
            commands.spawn((
                PlayerIndex(player_tank.index),
                PlayerAvatar,
                PlayingEntity,
                Sprite {
                    image: player_avatar_texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: player_avatar_texture_atlas_layout,
                        index: 0,
                    }),
                    custom_size: Some(Vec2::new(120.0, 112.0)),
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
                PlayerIndex(player_tank.index),
                HealthBar,
                HealthBarOriginalPosition(config.x_pos),
                PlayingEntity,
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(100.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
        UIElementType::BlueBar => {
            commands.spawn((
                PlayerIndex(player_tank.index),
                BlueBar,
                PlayingEntity,
                Sprite {
                    color: Color::srgb(0.0, 0.5, 1.0),
                    custom_size: Some(Vec2::new(100.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
    }
}

fn spawn_power_ups(commands: &mut Commands, asset_server: &AssetServer, texture_atlas_layouts: &mut Assets<TextureAtlasLayout>) {
    // 生成道具（轮胎精灵图）
    let power_up_texture: Handle<Image> = asset_server.load("tire_sprite_sheet.png");
    let power_up_tile_size = UVec2::new(87, 69);
    let power_up_texture_atlas = TextureAtlasLayout::from_grid(power_up_tile_size, 3, 1, None, None);
    let power_up_texture_atlas_layout = texture_atlas_layouts.add(power_up_texture_atlas);
    let power_up_animation_indices = AnimationIndices { first: 0, last: 2 };

    // 道具位置：墙内任意位置（包括中间的道具）
    let power_up_positions = [
        Vec3::new(0.0, 0.0, 0.0),        // 中间
        Vec3::new(-400.0, 200.0, 0.0),   // 左上
        Vec3::new(400.0, -100.0, 0.0),   // 右下
    ];

    for pos in power_up_positions {
        // 白色边框
        commands.spawn((
            PowerUpBorder,
            PlayingEntity,
            Sprite {
                color: Color::WHITE,
                custom_size: Some(Vec2::new(87.0 + 6.0, 69.0 + 6.0)),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y, pos.z - 0.1),
            AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        ));

        commands.spawn((
            PowerUp,
            PlayingEntity,
            Sprite::from_atlas_image(
                power_up_texture.clone(),
                TextureAtlas {
                    layout: power_up_texture_atlas_layout.clone(),
                    index: power_up_animation_indices.first,
                }
            ),
            Transform::from_translation(pos),
            power_up_animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            CurrentAnimationFrame(0),
            RigidBody::Fixed,
            Collider::cuboid(87.0 / 2.0, 69.0 / 2.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
        ));
    }
}

fn spawn_game_entities(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut can_fire: ResMut<CanFire>,
    mut clear_color: ResMut<ClearColor>,
    mut game_started: ResMut<GameStarted>,
) {
    // 如果游戏已经启动过，就不再生成实体
    if game_started.0 {
        return;
    }
    game_started.0 = true;
    // 设置背景色为黑色
    clear_color.0 = BACKGROUND_COLOR;

    // 生成墙壁
    spawn_walls(&mut commands);

    // 生成堡垒
    spawn_fortress(&mut commands);

    // 加载玩家坦克纹理和创建精灵图
    let player_texture = asset_server.load("texture/player_tank_sprite.png");
    let player_tile_size = UVec2::new(87, 87);
    let player_texture_atlas = TextureAtlasLayout::from_grid(player_tile_size, 3, 1, None, None);
    let player_texture_atlas_layout = texture_atlas_layouts.add(player_texture_atlas);
    let player_animation_indices = AnimationIndices { first: 0, last: 2 };

    // 生成玩家坦克（暂时只生成玩家1）
    let (player1_tank_entity, player_tank) = spawn_player1_tank(
        &mut commands,
        player_texture,
        player_texture_atlas_layout,
        player_animation_indices,
    );

    // 加载字体
    let font: Handle<Font> = asset_server.load("/home/nanbert/.fonts/SHOWG.TTF");
    spawn_player1_info(&mut commands, &player_tank, &font, &asset_server, &mut texture_atlas_layouts);
    spawn_top_text_info(&mut commands, &font);
    // 加载敌方坦克纹理和创建精灵图
    let enemy_texture = asset_server.load("texture/tank_player.png");
    let enemy_tile_size = UVec2::new(87, 103);
    let enemy_texture_atlas = TextureAtlasLayout::from_grid(enemy_tile_size, 3, 1, None, None);
    let enemy_texture_atlas_layout = texture_atlas_layouts.add(enemy_texture_atlas);
    let enemy_animation_indices = AnimationIndices { first: 0, last: 2 };

    // 生成敌方坦克
    let enemy_tank_entities: Vec<Entity> = ENEMY_BORN_PLACES
        .iter()
        .map(|&pos| spawn_enemy_tank(
            &mut commands,
            enemy_texture.clone(),
            enemy_texture_atlas_layout.clone(),
            enemy_animation_indices,
            pos,
        ))
        .collect();

    // 初始化所有坦克都可以射击
    for entity in enemy_tank_entities {
        can_fire.0.insert(entity);
    }
    can_fire.0.insert(player1_tank_entity);
    
    // 生成道具
    spawn_power_ups(&mut commands, &asset_server, &mut texture_atlas_layouts);
}

fn handle_start_screen_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu_selection: ResMut<CurrentMenuSelection>,
) {
    // Esc 键退出游戏
    if keyboard_input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }

    // W 键向上选择
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        menu_selection.selected_index = 0;
    }
    // S 键向下选择
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        menu_selection.selected_index = 1;
    }
    // Space 键确认选择
    if keyboard_input.just_pressed(KeyCode::Space) {
        match menu_selection.selected_index {
            0 => next_state.set(GameState::FadingOut), // PLAY
            1 => std::process::exit(0), // EXIT
            _ => {}
        }
    }
}

fn animate_start_screen(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite, &mut CurrentAnimationFrame), With<StartScreenUI>>,
    asset_server: Res<AssetServer>,
    mut animation_frames: ResMut<StartAnimationFrames>,
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
            *sprite = Sprite::from_image(get_animation_frame(next_index, &asset_server, &mut animation_frames));
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

fn shoot_bullets(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Velocity), With<EnemyTank>>,
    mut can_fire: ResMut<CanFire>,
    mut bullet_owners: ResMut<BulletOwners>,
) {
    for (entity, transform, velocity) in &mut query {
        // 检查是否可以射击（当前没有子弹）
        if can_fire.0.contains(&entity) {
            // 随机射击，每帧有 0.5% 的概率射击
            let mut rng = rand::rng();
            if rng.random::<f32>() < 0.005 {
                // 计算子弹发射方向（基于坦克当前移动方向）
                let direction = if velocity.linvel.length() > 0.0 {
                    velocity.linvel.normalize()
                } else {
                    Vec2::new(0.0, -1.0) // 默认向下
                };

                // 计算子弹初始位置（坦克前方）
                let bullet_pos = transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

                // 生成子弹
                let bullet_entity = commands.spawn((
                    Bullet,
                    PlayingEntity,
                    BulletOwner { owner: entity },
                    Sprite {
                        color: Color::srgb(1.0, 1.0, 1.0),
                        custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(bullet_pos),
                    Velocity {
                        linvel: direction * BULLET_SPEED,
                        angvel: 0.0,
                    },
                    RigidBody::KinematicVelocityBased,
                    Collider::ball(BULLET_SIZE / 200.0),
                    LockedAxes::ROTATION_LOCKED,
                    Sensor, // 设置为 Sensor，不施加物理推力
                    ActiveEvents::COLLISION_EVENTS,
                )).id();

                // 记录子弹所有者
                bullet_owners.owners.insert(bullet_entity, entity);

                // 标记该坦克暂时不能射击
                can_fire.0.remove(&entity);
            }
        }
    }
}

fn check_bullet_bounds(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in &mut query {
        let x = transform.translation.x;
        let y = transform.translation.y;

        // 检查子弹是否超出游戏窗口边界
        if !(MAP_LEFT_X..=MAP_RIGHT_X).contains(&x) ||
           !(MAP_BOTTOM_Y..=MAP_TOP_Y).contains(&y) {
            commands.entity(entity).despawn();
        }
    }
}

fn check_bullet_destruction(
    mut removed_bullets: RemovedComponents<Bullet>,
    mut can_fire: ResMut<CanFire>,
    mut bullet_owners: ResMut<BulletOwners>,
) {
    // 当子弹被销毁时，只允许该子弹所属的坦克可以再次射击
    for bullet_entity in removed_bullets.read() {
        if let Some(tank_entity) = bullet_owners.owners.remove(&bullet_entity) {
            can_fire.0.insert(tank_entity);
        }
    }
}

fn find_bullet_and_tank_in_collision(
    e1: Entity,
    e2: Entity,
    bullets: &Query<(Entity, &BulletOwner), With<Bullet>>,
    enemy_tanks: &Query<(), With<EnemyTank>>,
    player_tanks: &Query<&mut PlayerTank, With<PlayerTank>>,
) -> Option<(Entity, Entity)> {
    if bullets.get(e1).is_ok() && (enemy_tanks.get(e2).is_ok() || player_tanks.get(e2).is_ok()) {
        return Some((e1, e2));
    } else if bullets.get(e2).is_ok()
        && (enemy_tanks.get(e1).is_ok() || player_tanks.get(e1).is_ok()) {
        return Some((e2, e1));
    }
    None
}

fn should_bullet_destroy(
    bullet_owner: Entity,
    tank_entity: Entity,
    enemy_tanks: &Query<(), With<EnemyTank>>,
    player_tanks: &Query<&mut PlayerTank, With<PlayerTank>>,
) -> bool {
    let is_player_bullet = player_tanks.get(bullet_owner).is_ok();
    let is_enemy_bullet = enemy_tanks.get(bullet_owner).is_ok();
    let is_player_tank = player_tanks.get(tank_entity).is_ok();
    let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();

    // 规则：
    // 1. 玩家子弹打到敌方坦克 -> 子弹消失
    // 2. 敌方子弹打到玩家坦克 -> 子弹消失
    // 3. 敌方子弹打到敌方坦克 -> 子弹穿过（不消失）
    // 4. 玩家子弹打到玩家坦克 -> 子弹穿过（不消失）
    (is_player_bullet && is_enemy_tank) || (is_enemy_bullet && is_player_tank)
}

fn spawn_explosion(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载爆炸精灵图（40帧，每帧150x119）
    let explosion_texture: Handle<Image> = asset_server.load("explosion.png");
    let explosion_tile_size = UVec2::new(150, 119);
    let explosion_texture_atlas = TextureAtlasLayout::from_grid(explosion_tile_size, 40, 1, None, None);
    let explosion_texture_atlas_layout = texture_atlas_layouts.add(explosion_texture_atlas);
    let explosion_animation_indices = AnimationIndices { first: 0, last: 39 };

    commands.spawn((
        Explosion,
        PlayingEntity,
        Sprite::from_atlas_image(
            explosion_texture,
            TextureAtlas {
                layout: explosion_texture_atlas_layout,
                index: explosion_animation_indices.first,
            }
        ),
        Transform::from_translation(position),
        explosion_animation_indices,
        AnimationTimer(Timer::from_seconds(0.03, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));

    // 播放爆炸音效
    let explosion_sound: Handle<AudioSource> = asset_server.load("explosion_sound.ogg");
    commands.spawn(AudioPlayer::new(explosion_sound));
}

fn spawn_spark(
    commands: &mut Commands,
    asset_server: &AssetServer,
    position: Vec3,
) {
    // 加载火花图片（已裁剪到 87x83.6 像素，保持 spark2.png 的宽高比）
    let spark_texture: Handle<Image> = asset_server.load("spark.png");

    commands.spawn((
        Spark,
        PlayingEntity,
        Sprite {
            image: spark_texture,
            custom_size: Some(Vec2::new(87.0, 83.6)),
            ..default()
        },
        Transform::from_translation(position),
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Once)),
    ));
}

fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    bullets: Query<(Entity, &BulletOwner), With<Bullet>>,
    enemy_tanks: Query<(), With<EnemyTank>>,
    enemy_tanks_with_transform: Query<(Entity, &Transform), With<EnemyTank>>,
    mut player_tanks: Query<&mut PlayerTank, With<PlayerTank>>,
    player_tanks_with_transform: Query<(Entity, &Transform), With<PlayerTank>>,
    player_avatars: Query<(Entity, &PlayerIndex)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // 检查是否是子弹与坦克的碰撞
            if let Some((bullet_entity, tank_entity)) = find_bullet_and_tank_in_collision(
                *e1,
                *e2,
                &bullets,
                &enemy_tanks,
                &player_tanks,
            ) {
                let bullet_owner = bullets.get(bullet_entity).unwrap().1.owner;

                if should_bullet_destroy(bullet_owner, tank_entity, &enemy_tanks, &player_tanks) {
                    // 检查是否是玩家子弹击中敌方坦克
                    let is_player_bullet = player_tanks.get(bullet_owner).is_ok();
                    let is_enemy_tank = enemy_tanks.get(tank_entity).is_ok();
                    let is_player_tank = player_tanks.get(tank_entity).is_ok();

                    if is_player_bullet && is_enemy_tank {
                        // 获取敌方坦克的位置
                        if let Ok((_, tank_transform)) = enemy_tanks_with_transform.get(tank_entity) {
                            // 生成爆炸效果
                            spawn_explosion(&mut commands, &asset_server, &mut texture_atlas_layouts, tank_transform.translation);
                        }

                        // 销毁敌方坦克
                        commands.entity(tank_entity).despawn();

                        // 增加分数
                        let mut player_tank = player_tanks.get_mut(bullet_owner).expect("无法获取玩家坦克!");
                        player_tank.score += 100;
                    } else if !is_player_bullet && is_player_tank {
                        let mut player_tank = player_tanks.get_mut(tank_entity).expect("无法获取玩家坦克!");
                        // 敌方子弹击中玩家坦克
                        // 播放中弹音效
                        let hit_sound: Handle<AudioSource> = asset_server.load("hit_sound.ogg");
                        commands.spawn(AudioPlayer::new(hit_sound));

                        // 生成火花效果
                        if let Ok((_, tank_transform)) = player_tanks_with_transform.get(tank_entity) {
                            spawn_spark(&mut commands, &asset_server, tank_transform.translation);
                        }

                        // 扣除对应玩家的生命值
                        if player_tank.life_red_bar > 0 {
                            player_tank.life_red_bar -= 1;
                        }
                        if player_tank.life_red_bar == 0{
                            // 获取玩家坦克的位置
                            if let Ok((_, tank_transform)) = player_tanks_with_transform.get(tank_entity) {
                                // 生成爆炸效果
                                spawn_explosion(&mut commands, &asset_server, &mut texture_atlas_layouts, tank_transform.translation);
                            }

                            // 销毁玩家坦克
                            commands.entity(tank_entity).despawn();

                            // 标记对应玩家的头像为死亡状态
                            for (avatar_entity, player_index) in player_avatars.iter() {
                                if player_index.0 == player_tank.index {
                                    commands.entity(avatar_entity).insert(PlayerDead);
                                }
                            }
                            // 启动 Game Over 延迟计时器（2秒）
                            commands.spawn((
                                GameOverTimer,
                                AnimationTimer(Timer::from_seconds(2.0, TimerMode::Once)),
                            ));
                        }
                    }
                    // 销毁子弹
                    commands.entity(bullet_entity).despawn();
                }
            }
        }
    }
}

fn handle_powerup_collision(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    powerups: Query<(Entity, &Transform), With<PowerUp>>,
    powerup_borders: Query<(Entity, &Transform), With<PowerUpBorder>>,
    player_tanks: Query<(&Transform, &mut PlayerTank), With<PlayerTank>>,
    player_info_texts: Query<(Entity, &Text2d), With<Text2d>>,
) {
    for (tank_transform, mut player_tank) in player_tanks{
        let mut is_power_up:bool = false;
        // 销毁texture
        for (powerup_entity, powerup_transform) in powerups{
            let distance = (powerup_transform.translation - tank_transform.translation).length();
            if distance < 81.0 {
                // 播放道具音效
                let powerup_sound: Handle<AudioSource> = asset_server.load("powerup_sound.ogg");
                commands.spawn(AudioPlayer::new(powerup_sound));
                commands.entity(powerup_entity).despawn();
                is_power_up = true;
            }
            // 销毁道具
        }
        // 销毁对应位置的白色边框
        for (border_entity, border_transform) in powerup_borders.iter() {
            // 检查边框是否在道具附近（位置相近）
            let distance = (tank_transform.translation - border_transform.translation).length();
            if distance < 81.0 {
                commands.entity(border_entity).despawn();
                is_power_up = true;
            }
        }
        if !is_power_up{
            continue;
        }
        // 增加速度
        if player_tank.speed < 100 {
            player_tank.speed += 20;
        }

        // 闪烁文字
        for (entity, text) in player_info_texts.iter() {
            if text.0.starts_with("Speed:") {
                commands.entity(entity).insert(PlayerInfoBlinkTimer(Timer::from_seconds(1.8, TimerMode::Once)));
                break;
            }
        }
    }
}

fn animate_powerup_border(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite), With<PowerUpBorder>>,
) {
    for (mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            // 每0.2秒切换显示/隐藏状态
            if sprite.color == Color::WHITE {
                sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.0);
            } else {
                sprite.color = Color::WHITE;
            }
        }
    }
}

fn animate_player_info_text(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PlayerInfoBlinkTimer, &mut TextColor), With<Text2d>>,
) {
    for (entity, mut timer, mut color) in &mut query {
        timer.tick(time.delta());

        if timer.is_finished() {
            // 闪烁结束，移除计时器组件
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = Color::srgb(1.0, 1.0, 1.0);
        } else {
            // 每0.6秒切换颜色（0.3秒亮，0.3秒灭）
            let elapsed = timer.elapsed_secs();
            let cycle = elapsed % 0.6;
            color.0 = if cycle < 0.3 {
                Color::srgb(1.0, 1.0, 1.0)
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.0)
            };
        }
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
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &Velocity), With<PlayerTank>>,
) {
    // 玩家坦克：只有移动时才刷新纹理
    for (mut timer, mut sprite, indices, velocity) in &mut query {
        let speed = velocity.linvel.length();
        if sprite.texture_atlas.is_none() {
            continue;
        }
        let atlas = sprite.texture_atlas.as_mut().expect("玩家坦克没有纹理！");
        if speed <= 0.0 {
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

fn animate_player_avatar(
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut Sprite,
        &AnimationIndices,
        &mut CurrentAnimationFrame,
        Option<&PlayerDead>,
    ), With<PlayerAvatar>>,
) {
    for (mut timer, mut sprite, indices, mut current_frame, player_dead) in &mut query {
        // 如果玩家已死亡，切换到死亡图片并停止动画
        if player_dead.is_some() {
            let dead_texture: Handle<Image> = asset_server.load("player_dead.png");
            sprite.image = dead_texture;
            sprite.texture_atlas = None;
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
                commands.entity(entity).despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

fn animate_spark(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer), With<Spark>>,
) {
    for (entity, mut timer) in &mut query {
        timer.tick(time.delta());
        if timer.is_finished() {
            // 0.5秒后销毁火花实体
            commands.entity(entity).despawn();
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
            // 2秒后切换到 GameOver 状态
            commands.entity(entity).despawn();
            next_state.set(GameState::GameOver);
        }
    }
}

fn move_player_tank(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut KinematicCharacterController, &mut RotationTimer, &mut TargetRotation), With<PlayerTank>>,
) {
    for (mut transform, mut character_controller, mut rotation_timer, mut target_rotation) in &mut query {
        let w_pressed = keyboard_input.pressed(KeyCode::KeyW);
        let s_pressed = keyboard_input.pressed(KeyCode::KeyS);
        let a_pressed = keyboard_input.pressed(KeyCode::KeyA);
        let d_pressed = keyboard_input.pressed(KeyCode::KeyD);

        // 只允许单一方向移动，多个方向键同时按下时停止
        let direction = match (w_pressed, s_pressed, a_pressed, d_pressed) {
            (true, false, false, false) => Vec2::new(0.0, 1.0),  // 上
            (false, true, false, false) => Vec2::new(0.0, -1.0), // 下
            (false, false, true, false) => Vec2::new(-1.0, 0.0), // 左
            (false, false, false, true) => Vec2::new(1.0, 0.0),  // 右
            _ => Vec2::ZERO, // 其他情况（包括多个键同时按下）停止移动
        };

        // 检查是否需要转向
        let needs_rotation = if direction.length() > 0.0 {
            let angle = direction.y.atan2(direction.x);
            let target_angle = angle - 270.0_f32.to_radians();

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
        // 转向时保持 50% 速度，减少卡顿感
        let speed = if needs_rotation { TANK_SPEED * 0.5 } else { TANK_SPEED };
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

fn player_shoot_bullet(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &Transform), With<PlayerTank>>,
    mut can_fire: ResMut<CanFire>,
    _player_tanks: Query<Entity, With<PlayerTank>>,
    mut bullet_owners: ResMut<BulletOwners>,
) {
    for (entity, transform) in &query {
        // J 键射击
        if keyboard_input.just_pressed(KeyCode::KeyJ) && can_fire.0.contains(&entity) {
            // 计算子弹发射方向（基于坦克当前的旋转角度）
            // 坦克旋转时使用：angle - 270.0_f32.to_radians()
            // 因此需要补偿：actual_angle = euler_angle + 270.0_f32.to_radians()
            let euler_angle = transform.rotation.to_euler(EulerRot::XYZ).2;
            let actual_angle = euler_angle + 270.0_f32.to_radians();
            let direction = Vec2::new(actual_angle.cos(), actual_angle.sin());

            // 计算子弹初始位置（坦克前方）
            let bullet_pos = transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

            // 生成玩家子弹
            let bullet_entity = commands.spawn((
                Bullet,
                PlayingEntity,
                BulletOwner { owner: entity },
                Sprite {
                    color: Color::srgb(1.0, 1.0, 1.0),
                    custom_size: Some(Vec2::new(BULLET_SIZE, BULLET_SIZE)),
                    ..default()
                },
                Transform::from_translation(bullet_pos),
                Velocity {
                    linvel: direction * BULLET_SPEED,
                    angvel: 0.0,
                },
                RigidBody::KinematicVelocityBased,
                Collider::ball(BULLET_SIZE / 200.0),
                LockedAxes::ROTATION_LOCKED,
                Sensor, // 设置为 Sensor，不施加物理推力
                ActiveEvents::COLLISION_EVENTS,
            )).id();

            // 记录子弹所有者
            bullet_owners.owners.insert(bullet_entity, entity);

            // 标记玩家坦克暂时不能射击
            can_fire.0.remove(&entity);
        }
    }
}

fn setup_fade_out(
    mut fading_out: ResMut<FadingOut>,
    mut game_started: ResMut<GameStarted>,
) {
    fading_out.alpha = 1.0;
    game_started.0 = false; // 重置游戏启动标志，以便重新游戏时重新生成实体
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
        commands.entity(entity).despawn();
    }
    for (entity, _, _) in text_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn fade_out_screen(
    mut commands: Commands,
    time: Res<Time>,
    mut fading_out: ResMut<FadingOut>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sprite_query: Query<(Entity, &mut Sprite), With<StartScreenUI>>,
    mut text_query: Query<(Entity, &mut TextColor, Option<&MenuOption>), With<StartScreenUI>>,
) {
    // 减少透明度
    fading_out.alpha -= time.delta_secs() * (1.0 / 3.0); // 淡出速度，约 3 秒完成

    // 更新所有 Sprite 元素的透明度
    for (_entity, mut sprite) in &mut sprite_query {
        update_sprite_alpha(fading_out.alpha, &mut sprite);
    }

    // 更新所有 Text 元素的颜色（跳过 PLAY 选项，因为它的闪烁由 update_menu_blink 处理）
    for (_entity, mut text_color, menu_option) in &mut text_query {
        // 如果是 PLAY 选项（index=0），跳过透明度更新
        if menu_option.is_some_and(|opt| opt.index == 0) {
            continue;
        }
        update_text_color_alpha(fading_out.alpha, &mut text_color);
    }

    // 淡出完成，切换到 Playing 状态并清理所有 StartScreenUI 元素
    if fading_out.alpha <= 0.0 {
        next_state.set(GameState::Playing);
        cleanup_start_screen(&mut commands, &sprite_query, &text_query);
    }
}

fn update_menu_highlight(
    menu_selection: Res<CurrentMenuSelection>,
    mut arrow_query: Query<&mut Transform, With<MenuArrow>>,
) {
    // 获取箭头位置
    if let Ok(mut arrow_transform) = arrow_query.single_mut() {
        // 根据当前选择的索引更新箭头位置
        let y_position = if menu_selection.selected_index == 1 {
            -100.0 // EXIT 的 Y 位置
        } else {
            0.0 // PLAY 的 Y 位置
        };
        arrow_transform.translation.y = y_position;
    }
}

fn update_option_colors(
    menu_selection: Res<CurrentMenuSelection>,
    mut text_query: Query<(&MenuOption, &mut TextColor), Without<MenuArrow>>,
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

fn update_player_info_display(
    mut text2ds: Query<(&PlayerIndex, &mut Text2d), With<Text2d>>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
) {
    for player_tank in player_tanks {
        for (player_index, mut text) in &mut text2ds {
            if player_tank.index != player_index.0{
                continue;
            }
            //更新文字
            if text.0.starts_with("Scores"){
                let key_words = format!("Scores{}:", (player_index.0+1).to_string());
                text.0 = key_words;
            } else if text.0.starts_with("Speed"){
                let key_words = if player_tank.speed<100{
                    format!("Speed: {}%", player_tank.speed)
                } else{
                    format!("Speed: Max")
                };
                text.0 = key_words;
            }
        }
    }
}

fn update_health_bar(
    mut health_bars: Query<(&mut Sprite, &HealthBarOriginalPosition, &mut Transform), With<HealthBar>>,
    player_tanks: Query<&PlayerTank, With<PlayerTank>>,
) {
    for (mut sprite, original_pos, mut transform) in &mut health_bars {
        for player_tank in player_tanks{
            // 血条总宽度 100，生命值 3，每条代表 1/3
            let health_width = (player_tank.life_red_bar as f32 / 3.0) * 100.0;
            sprite.custom_size = Some(Vec2::new(health_width, 10.0));

            // 左对齐：将血条向左移动，使其从左边界开始
            // 原始位置是中心点，需要向左偏移 (100 - health_width) / 2
            let offset = (100.0 - health_width) / 2.0;
            transform.translation.x = original_pos.0 - offset;
        }
    }
}


fn update_menu_blink(
    time: Res<Time>,
    fading_out: Res<FadingOut>,
    mut blink_timer: ResMut<MenuBlinkTimer>,
    mut text_query: Query<(&MenuOption, &mut TextColor), Without<MenuArrow>>,
) {
    blink_timer.0.tick(time.delta());

    // 初始化计时器（0.2秒闪烁）
    if blink_timer.0.duration().is_zero() {
        blink_timer.0 = Timer::from_seconds(0.2, TimerMode::Repeating);
    }

    if blink_timer.0.just_finished() {
        for (option, mut text_color) in &mut text_query {
            if option.index == 0 {
                // PLAY 选项（index=0）闪烁
                // 出现时使用当前淡出透明度，消失时完全透明
                let linear = text_color.0.to_linear();
                if linear.alpha < 0.5 {
                    // 当前不可见，切换到可见（使用当前淡出透明度）
                    text_color.0 = Color::srgb(1.0, 1.0, 0.0).with_alpha(fading_out.alpha);
                } else {
                    // 当前可见，切换到不可见（完全透明）
                    text_color.0 = Color::srgb(1.0, 1.0, 0.0).with_alpha(0.0);
                }
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
    let font: Handle<Font> = asset_server.load("/home/nanbert/.fonts/SHOWG.TTF");

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
        commands.entity(entity).despawn();
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
    let font: Handle<Font> = asset_server.load("/home/nanbert/.fonts/SHOWG.TTF");

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
    mut game_started: ResMut<GameStarted>,
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
                game_started.0 = false;
                next_state.set(GameState::Playing);
            }
            1 => {
                // Back to Menu: 返回开始界面
                game_started.0 = false;
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
        commands.entity(entity).despawn();
    }
}
// 清理游戏过程中的所有entity
fn cleanup_playing_entities(
    mut commands: Commands,
    playing_entities: Query<Entity, With<PlayingEntity>>,
) {
    // 清理所有游戏实体
    for entity in playing_entities.iter() {
        commands.entity(entity).despawn();
    }
}
