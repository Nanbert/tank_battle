//! A simplified implementation of the classic game "Battle City 1990"
//!
//!
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::needless_pass_by_value)]
use std::collections::HashSet;
use std::collections::HashMap;
use bevy::{
    prelude::*,
    window::{
        PresentMode,
        WindowTheme,
    },
};
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    StartScreen,
    FadingOut,
    Playing,
    Paused,
}

#[derive(Component)]
struct StartScreenUI;

#[derive(Component)]
struct MenuOption {
    index: usize,
}

#[derive(Component)]
struct MenuArrow;

#[derive(Component)]
struct PauseUI;

#[derive(Resource, Deref, DerefMut)]
struct Score(usize);

#[derive(Resource, Deref, DerefMut)]
struct Life(usize);

#[derive(Component, Copy, Clone)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Resource, Default)]
struct ColliderEventSet{
    entities:HashSet<Entity>,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct CurrentAnimationFrame(usize);

#[derive(Component, Deref, DerefMut)]
struct DirectionChangeTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct CollisionCooldownTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct RotationTimer(Timer);

#[derive(Component)]
struct TargetRotation {
    angle: f32,
}

#[derive(Component, Copy, Clone)]
struct EnemyTank {
    direction: Vec2,
}

#[derive(Component)]
struct PlayerTank;

#[derive(Component)]
struct PlayerAvatar;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct BulletOwner {
    owner: Entity,
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Fortress;

#[derive(Component)]
struct PowerUp;

#[derive(Component)]
struct PowerUpBorder;

#[derive(Component)]
struct PlayerInfoText;

#[derive(Component, Deref, DerefMut)]
struct PlayerInfoBlinkTimer(Timer);

#[derive(Resource, Default)]
struct PlayerSpeed(usize);

#[derive(Resource, Default)]
struct CanFire(HashSet<Entity>);

#[derive(Resource, Default)]
struct BulletOwners {
    owners: HashMap<Entity, Entity>, // 子弹实体 -> 坦克实体
}

#[derive(Resource, Default)]
struct StartAnimationFrames {
    frames: Vec<Handle<Image>>,
}

#[derive(Resource)]
struct FadingOut {
    alpha: f32,
}

impl Default for FadingOut {
    fn default() -> Self {
        Self { alpha: 1.0 }
    }
}

#[derive(Resource, Default)]
struct CurrentMenuSelection {
    selected_index: usize, // 0 = PLAY, 1 = EXIT
}

#[derive(Resource, Default)]
struct GameStarted(bool);

#[derive(Resource, Default)]
struct MenuBlinkTimer(Timer);

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const ORIGINAL_WIDTH: f32 = 1600.0; // 原游戏区域宽度
const ORIGINAL_HEIGHT: f32 = 1200.0; // 原游戏区域高度
const LEFT_PADDING: f32 = 230.0; // 左侧留白
const RIGHT_PADDING: f32 = 230.0; // 右侧留白
const TOP_PADDING: f32 = 100.0; // 上方留白
const BOTTOM_PADDING: f32 = 0.0; // 下方留白
const VERTICAL_OFFSET: f32 = -50.0; // 垂直偏移，向下平移50像素
const ARENA_WIDTH: f32 = ORIGINAL_WIDTH + LEFT_PADDING + RIGHT_PADDING; // 总宽度
const ARENA_HEIGHT: f32 = ORIGINAL_HEIGHT + TOP_PADDING + BOTTOM_PADDING; // 总高度
const TANK_WIDTH: f32 = 87.0;
const TANK_HEIGHT: f32 = 103.0;
const TANK_SPEED: f32 = 300.0;
const BULLET_SPEED: f32 = 900.0;
const BULLET_SIZE: f32 = 10.0;

const ENEMY_BORN_PLACES: [Vec3; 3] = [
    Vec3::new(-ORIGINAL_WIDTH / 2.0 + TANK_WIDTH / 2.0, ORIGINAL_HEIGHT/2.0 - TANK_HEIGHT / 2.0 + VERTICAL_OFFSET, 0.0),
    Vec3::new(0.0, ORIGINAL_HEIGHT/2.0 - TANK_HEIGHT / 2.0 + VERTICAL_OFFSET, 0.0),
    Vec3::new(ORIGINAL_WIDTH/2.0 - TANK_WIDTH / 2.0, ORIGINAL_HEIGHT/2.0 - TANK_HEIGHT / 2.0 + VERTICAL_OFFSET, 0.0),
];

const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.5, 0.5); // 蓝绿色
const START_SCREEN_BACKGROUND_COLOR: Color = Color::srgb(17.0/255.0, 81.0/255.0, 170.0/255.0);

const FORTRESS_SIZE: f32 = 60.0;
const WALL_THICKNESS: f32 = 15.0;

const DIRECTIONS: [Vec2; 4] = [
    Vec2::new(0.0, 1.0),   // 上
    Vec2::new(0.0, -1.0),  // 下
    Vec2::new(-1.0, 0.0),  // 左
    Vec2::new(1.0, 0.0),   // 右
];

fn configure_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "For Communism!".into(),
            name: Some("bevy.app".into()),
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            resolution: (ARENA_WIDTH as u32, ARENA_HEIGHT as u32).into(),
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
        .init_resource::<ColliderEventSet>()
        .init_resource::<CanFire>()
        .init_resource::<BulletOwners>()
        .init_resource::<StartAnimationFrames>()
        .init_resource::<FadingOut>()
        .init_resource::<CurrentMenuSelection>()
        .init_resource::<GameStarted>()
        .init_resource::<MenuBlinkTimer>()
        .insert_resource(Score(0))
        .insert_resource(Life(2))
        .insert_resource(PlayerSpeed(40))
        .insert_resource(ClearColor(BACKGROUND_COLOR));
}

fn register_game_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::StartScreen), spawn_start_screen)
        .add_systems(OnEnter(GameState::FadingOut), setup_fade_out)
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities)
        .add_systems(OnEnter(GameState::Paused), spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), despawn_pause_ui)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            (collect_collision, move_enemy_tanks).chain(),
            move_player_tank,
            animate_tank_texture,
            animate_player_avatar,
            animate_powerup_border,
            animate_powerup_texture,
            animate_player_info_text,
            shoot_bullets,
            player_shoot_bullet,
            check_bullet_bounds,
            check_bullet_destruction,
            handle_bullet_collisions,
            handle_powerup_collision,
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
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, ORIGINAL_HEIGHT / 200.0),
        Transform{
            translation: Vec3::new(-ORIGINAL_WIDTH / 2.0 - 5.0, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(10.0 , ORIGINAL_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 右墙（在原游戏区域右边界，向下平移40像素）
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, ORIGINAL_HEIGHT / 200.0),
        Transform{
            translation: Vec3::new(ORIGINAL_WIDTH / 2.0 + 5.0, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(10.0 , ORIGINAL_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 上墙（在原游戏区域上边界，向下平移40像素）
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(ORIGINAL_WIDTH / 200.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, ORIGINAL_HEIGHT / 2.0 + 5.0 + VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(ORIGINAL_WIDTH, 10.0, 1.0),
            ..default()
        }
    ));

    // 下墙（在原游戏区域下边界，向下平移40像素）
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(ORIGINAL_WIDTH / 200.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, -ORIGINAL_HEIGHT / 2.0 -5.0 + VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(ORIGINAL_WIDTH, 10.0 , 1.0),
            ..default()
        }
    ));
}

fn spawn_fortress(commands: &mut Commands) {
    let fortress_y = -ORIGINAL_HEIGHT / 2.0 + FORTRESS_SIZE / 2.0 + 20.0 + VERTICAL_OFFSET;
    let fortress_x = 0.0;

    // 堡垒主体（红色五角星）
    commands.spawn((
        Fortress,
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

fn spawn_player_tank(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
) -> Entity {
    let fortress_y = -ORIGINAL_HEIGHT / 2.0 + FORTRESS_SIZE / 2.0 + 20.0 + VERTICAL_OFFSET;

    commands.spawn_empty()
        .insert(PlayerTank)
        .insert(RotationTimer(Timer::from_seconds(0.2, TimerMode::Once)))
        .insert(TargetRotation { angle: 270.0_f32.to_radians() })
        .insert(Sprite::from_atlas_image(
            texture,
            TextureAtlas{
                layout: texture_atlas_layout,
                index: animation_indices.first,
            }
        ))
        .insert(Transform::from_xyz(WALL_THICKNESS.mul_add(-2.0, -FORTRESS_SIZE) - TANK_WIDTH / 2.0 - 20.0, fortress_y, 0.0))
        .insert(animation_indices)
        .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert(Velocity{
            linvel: Vec2::new(0.0, 0.0),
            angvel: 0.0,
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::DYNAMIC_STATIC)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(GravityScale(0.0))
        .insert(Friction::new(0.0))
        .insert(Restitution::new(0.0))
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 0.5,
        })
        .id()
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
        .insert(DirectionChangeTimer(Timer::from_seconds(4.0, TimerMode::Once)))
        .insert(CollisionCooldownTimer(Timer::from_seconds(0.5, TimerMode::Once)))
        .insert(RotationTimer(Timer::from_seconds(0.6, TimerMode::Once)))
        .insert(TargetRotation { angle: 270.0_f32.to_radians() })
        .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
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

    // 加载纹理和创建精灵图
    let texture = asset_server.load("texture/tank_player.png");
    let tile_size = UVec2::new(87, 103);
    let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    // 生成玩家坦克
    let player_tank_entity = spawn_player_tank(
        &mut commands,
        texture.clone(),
        texture_atlas_layout.clone(),
        animation_indices,
    );

    // 生成敌方坦克
    let enemy_tank_entities: Vec<Entity> = ENEMY_BORN_PLACES
        .iter()
        .map(|&pos| spawn_enemy_tank(
            &mut commands,
            texture.clone(),
            texture_atlas_layout.clone(),
            animation_indices,
            pos,
        ))
        .collect();

    // 初始化所有坦克都可以射击
    for entity in enemy_tank_entities {
        can_fire.0.insert(entity);
    }
    can_fire.0.insert(player_tank_entity);

    // 生成玩家信息文字（在左侧留白处）
    let font: Handle<Font> = asset_server.load("/home/nanbert/.fonts/SHOWG.TTF");

// player1
    commands.spawn((
        Text2d("player1".to_string()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET - 80.0, 1.0),
    ));

    // Speed:40%
    commands.spawn((
        PlayerInfoText,
        Text2d("Speed:40%".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET - 130.0, 1.0),
    ));

    // Fire Speed:40%
    commands.spawn((
        Text2d("Fire Speed:40%".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET - 180.0, 1.0),
    ));

    // Protection:40%
    commands.spawn((
        Text2d("Protection:40%".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET - 230.0, 1.0),
    ));

    // Shells: 1
    commands.spawn((
        Text2d("Shells: 1".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET - 280.0, 1.0),
    ));

    // Pnetrate: No
    commands.spawn((
        Text2d("Pnetrate: No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET + 100.0, 1.0),
    ));

    // Track Chain:No
    commands.spawn((
        Text2d("Track Chain:No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET + 150.0, 1.0),
    ));

    // Air Cushion:No
    commands.spawn((
        Text2d("Air Cushion:No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET + 200.0, 1.0),
    ));

    // Fire Shell:No
    commands.spawn((
        Text2d("Fire Shell:No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET + 250.0, 1.0),
    ));

    // Effects
    commands.spawn((
        Text2d("Effects".to_string()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, VERTICAL_OFFSET + 300.0, 1.0),
    ));

    // Scores1:0（左上角）
    commands.spawn((
        Text2d("Scores1:0".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, ARENA_HEIGHT / 2.0 - 50.0, 1.0),
    ));

    // player头像（Scores1下方）
    let player_avatar_texture: Handle<Image> = asset_server.load("player.png");
    let player_avatar_tile_size = UVec2::new(120, 112);
    let player_avatar_texture_atlas = TextureAtlasLayout::from_grid(player_avatar_tile_size, 13, 4, None, None);
    let player_avatar_texture_atlas_layout = texture_atlas_layouts.add(player_avatar_texture_atlas);
    // 13列4行，共52帧，索引从0到51
    let player_avatar_animation_indices = AnimationIndices { first: 0, last: 51 };
    commands.spawn((
        PlayerAvatar,
        Sprite {
            image: player_avatar_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: player_avatar_texture_atlas_layout,
                index: 0,
            }),
            custom_size: Some(Vec2::new(120.0, 112.0)),
            ..default()
        },
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, ARENA_HEIGHT / 2.0 - 150.0, 1.0),
        player_avatar_animation_indices,
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));

    // 血条（红色）
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(100.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, ARENA_HEIGHT / 2.0 - 215.0, 1.0),
    ));

    // 蓝条（蓝色）
    commands.spawn((
        Sprite {
            color: Color::srgb(0.0, 0.5, 1.0),
            custom_size: Some(Vec2::new(100.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0, ARENA_HEIGHT / 2.0 - 230.0, 1.0),
    ));

    // Commander Life:（Scores1右侧200像素）
    commands.spawn((
        Text2d("Commander Life:".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(-ORIGINAL_WIDTH / 2.0 - LEFT_PADDING / 2.0 + 350.0, ARENA_HEIGHT / 2.0 - 50.0, 1.0),
    ));

    // Scores2:0（右上角）
    commands.spawn((
        Text2d("Scores2:0".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, ARENA_HEIGHT / 2.0 - 50.0, 1.0),
    ));

    // Enemy Left:20/20（Scores2左侧300像素）
    commands.spawn((
        Text2d("Enemy Left:20/20".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0 - 300.0, ARENA_HEIGHT / 2.0 - 50.0, 1.0),
    ));

    // player2 信息（在右侧留白处）
    // player2
    commands.spawn((
        Text2d("player2".to_string()),
        TextFont {
            font_size: 32.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET - 80.0, 1.0),
    ));

    // Speed:40%
    commands.spawn((
        Text2d("Speed:40%".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET - 130.0, 1.0),
    ));

    // Fire Speed:40%
    commands.spawn((
        Text2d("Fire Speed:40%".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET - 180.0, 1.0),
    ));

    // Protection:40%
    commands.spawn((
        Text2d("Protection:40%".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET - 230.0, 1.0),
    ));

    // Shells: 1
    commands.spawn((
        Text2d("Shells: 1".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET - 280.0, 1.0),
    ));

    // Pnetrate: No
    commands.spawn((
        Text2d("Pnetrate: No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET + 100.0, 1.0),
    ));

    // Track Chain:No
    commands.spawn((
        Text2d("Track Chain:No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET + 150.0, 1.0),
    ));

    // Air Cushion:No
    commands.spawn((
        Text2d("Air Cushion:No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET + 200.0, 1.0),
    ));

    // Fire Shell:No
    commands.spawn((
        Text2d("Fire Shell:No".to_string()),
        TextFont {
            font_size: 24.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET + 250.0, 1.0),
    ));

    // Effects
    commands.spawn((
        Text2d("Effects".to_string()),
        TextFont {
            font_size: 32.0,
            font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(ORIGINAL_WIDTH / 2.0 + RIGHT_PADDING / 2.0, VERTICAL_OFFSET + 300.0, 1.0),
    ));

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
fn collect_collision(
    mut collision_events: MessageReader<CollisionEvent>,
    mut collision_set: ResMut<ColliderEventSet>,
)
{
    collision_set.entities.clear();
    for event in collision_events.read(){
        if let CollisionEvent::Started(e1, e2, _) = event{
            collision_set.entities.insert(*e1);
            collision_set.entities.insert(*e2);
        }
    }
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
    if rng.random::<f32>() < 0.1 {
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
        let angle_diff = (target_angle - current_euler + std::f32::consts::PI * 3.0) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
        
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
) {
    let rapier_context = rapier_context.single().unwrap();

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
        let angle_diff = (target_angle - current_euler + std::f32::consts::PI * 3.0) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
        
        if angle_diff.abs() > 0.01 && !rotation_timer.is_finished() {
            // 计算旋转进度（0.0 到 1.0）
            let progress = rotation_timer.elapsed_secs() / rotation_timer.duration().as_secs_f32();
            // 使用缓动函数使旋转更平滑
            let eased_progress = progress * progress * (3.0 - 2.0 * progress);
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
        if !(-ARENA_WIDTH / 2.0..=ARENA_WIDTH / 2.0).contains(&x) ||
           !(-ARENA_HEIGHT / 2.0..=ARENA_HEIGHT / 2.0).contains(&y) {
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
    player_tanks: &Query<(), With<PlayerTank>>,
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
    player_tanks: &Query<(), With<PlayerTank>>,
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

fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    bullets: Query<(Entity, &BulletOwner), With<Bullet>>,
    enemy_tanks: Query<(), With<EnemyTank>>,
    player_tanks: Query<(), With<PlayerTank>>,
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
                    commands.entity(bullet_entity).despawn();
                }
            }
        }
    }
}

fn handle_powerup_collision(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    powerups: Query<(Entity, &Transform), With<PowerUp>>,
    powerup_borders: Query<(Entity, &Transform), With<PowerUpBorder>>,
    player_tanks: Query<(), With<PlayerTank>>,
    player_info_texts: Query<(Entity, &Text2d), With<PlayerInfoText>>,
    mut player_speed: ResMut<PlayerSpeed>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // 检查是否是玩家坦克与道具的碰撞
            let (powerup_entity, powerup_transform) = if powerups.get(*e1).is_ok() && player_tanks.get(*e2).is_ok() {
                (*e1, powerups.get(*e1).unwrap().1)
            } else if powerups.get(*e2).is_ok() && player_tanks.get(*e1).is_ok() {
                (*e2, powerups.get(*e2).unwrap().1)
            } else {
                continue;
            };

            // 销毁道具
            commands.entity(powerup_entity).despawn();

            // 只销毁对应位置的白色边框
            for (border_entity, border_transform) in powerup_borders.iter() {
                // 检查边框是否在道具附近（位置相近）
                let distance = (powerup_transform.translation - border_transform.translation).length();
                if distance < 1.0 {
                    commands.entity(border_entity).despawn();
                    break;
                }
            }

            // 增加速度
            if player_speed.0 < 100 {
                player_speed.0 += 20;
            }

            // 更新 Speed 文字显示
            for (entity, text) in player_info_texts.iter() {
                if text.0.starts_with("Speed:") {
                    let speed_text = if player_speed.0 >= 100 {
                        "Speed:MAX".to_string()
                    } else {
                        format!("Speed:{}%", player_speed.0)
                    };
                    commands.entity(entity).insert(Text2d(speed_text));
                    commands.entity(entity).insert(PlayerInfoBlinkTimer(Timer::from_seconds(1.8, TimerMode::Once)));
                    break;
                }
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
    mut query: Query<(Entity, &mut PlayerInfoBlinkTimer, &mut TextColor), With<PlayerInfoText>>,
) {
    for (entity, mut timer, mut color) in &mut query {
        timer.tick(time.delta());

        if timer.is_finished() {
            // 闪烁结束，移除计时器组件
            commands.entity(entity).remove::<PlayerInfoBlinkTimer>();
            color.0 = Color::srgb(1.0, 1.0, 1.0);
        } else {
            // 每0.3秒切换颜色
            let elapsed = timer.elapsed_secs();
            if (elapsed / 0.3) as u32 % 2 == 0 {
                color.0 = Color::srgb(1.0, 1.0, 1.0);
            } else {
                color.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
            }
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

fn animate_tank_texture(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &Velocity)>,
) {
    for (mut timer, mut sprite, indices, velocity) in &mut query {
        // 只有坦克在运动时才刷新纹理
        if velocity.linvel.length() > 0.0 {
            timer.tick(time.delta());
            if timer.just_finished()
                && let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = if atlas.index == indices.last {
                        indices.first
                    } else {
                        atlas.index + 1
                    };
                }
        } else {
            // 坦克静止时，只在纹理索引不是第一帧时才重置
            if let Some(atlas) = &mut sprite.texture_atlas
                && atlas.index != indices.first {
                    atlas.index = indices.first;
                    timer.reset();
                }
        }
    }
}

fn animate_player_avatar(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<PlayerAvatar>>,
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
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
    }
}

fn move_player_tank(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut RotationTimer, &mut TargetRotation), With<PlayerTank>>,
) {
    for (mut transform, mut velocity, mut rotation_timer, mut target_rotation) in &mut query {
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
            let angle_diff = (target_angle - current_euler + std::f32::consts::PI * 3.0) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
            
            if angle_diff.abs() > 0.01 {
                target_rotation.angle = target_angle;
                rotation_timer.reset();
                true
            } else {
                false
            }
        } else {
            velocity.linvel = Vec2::ZERO;
            false
        };

        // 应用速度（原地转向时速度为0）
        if needs_rotation {
            velocity.linvel = Vec2::ZERO;
        } else if direction.length() > 0.0 {
            velocity.linvel = direction * TANK_SPEED;
        }

        // 更新旋转计时器
        rotation_timer.tick(time.delta());

        // 平滑旋转
        let current_euler = transform.rotation.to_euler(EulerRot::XYZ).2;
        let target_angle = target_rotation.angle;
        let angle_diff = (target_angle - current_euler + std::f32::consts::PI * 3.0) % (std::f32::consts::PI * 2.0) - std::f32::consts::PI;
        
        if angle_diff.abs() > 0.01 && !rotation_timer.is_finished() {
            // 计算旋转进度（0.0 到 1.0）
            let progress = rotation_timer.elapsed_secs() / rotation_timer.duration().as_secs_f32();
            // 使用缓动函数使旋转更平滑
            let eased_progress = progress * progress * (3.0 - 2.0 * progress);
            // 插值计算当前角度
            let new_angle = current_euler + angle_diff * eased_progress;
            transform.rotation = Quat::from_rotation_z(new_angle);
        } else if angle_diff.abs() > 0.01 {
            // 旋转完成，直接设置为目标角度
            transform.rotation = Quat::from_rotation_z(target_angle);
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
        Text2d("Press SPACE to resume | ESC to exit".to_string()),
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
    // Esc 键退出
    if keyboard_input.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}
