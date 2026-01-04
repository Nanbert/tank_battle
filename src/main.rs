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
}

#[derive(Component)]
struct StartScreenUI;

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

#[derive(Resource, Default)]
struct TankContactMap{
    max_depth_normals:HashMap<Entity, Vec2>,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct CurrentAnimationFrame(usize);

#[derive(Component, Deref, DerefMut)]
struct DirectionChangeTimer(Timer);

#[derive(Component, Deref, DerefMut)]
struct CollisionCooldownTimer(Timer);

#[derive(Component, Copy, Clone)]
struct EnemyTank {
    direction: Vec2,
}

#[derive(Component)]
struct PlayerTank;

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

#[derive(Resource, Default)]
struct CanFire(HashSet<Entity>);

#[derive(Resource, Default)]
struct BulletOwners {
    owners: HashMap<Entity, Entity>, // 子弹实体 -> 坦克实体
}

#[derive(Resource, Default, Deref, DerefMut)]
struct StartAnimationFrames(Vec<Handle<Image>>);

#[derive(Resource)]
struct FadingOut {
    alpha: f32,
}

impl Default for FadingOut {
    fn default() -> Self {
        Self { alpha: 1.0 }
    }
}

#[derive(Component, Deref, DerefMut)]
struct PlayerShootTimer(Timer);

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const ARENA_WIDTH: f32 = 1600.0;
const ARENA_HEIGHT: f32 = 1200.0;
const TANK_WIDTH: f32 = 87.0;
const TANK_HEIGHT: f32 = 103.0;
const TANK_SPEED: f32 = 300.0;
const BULLET_SPEED: f32 = 900.0;
const BULLET_SIZE: f32 = 10.0;

const ENEMY_BORN_PLACES: [Vec3; 3] = [
    Vec3::new(-ARENA_WIDTH / 2.0 + TANK_WIDTH / 2.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(0.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(ARENA_WIDTH/2.0 - TANK_WIDTH / 2.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
];

const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const START_SCREEN_BACKGROUND_COLOR: Color = Color::srgb(17.0/255.0, 81.0/255.0, 170.0/255.0);

const FORTRESS_SIZE: f32 = 60.0;
const WALL_THICKNESS: f32 = 15.0;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "For Communism!".into(),
                    name: Some("bevy.app".into()),
                    resolution: (ARENA_WIDTH as u32, ARENA_HEIGHT as u32).into(),
                    present_mode: PresentMode::AutoVsync,
                    // Tells Wasm to resize the window according to the available canvas
                    fit_canvas_to_parent: true,
                    // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                    prevent_default_event_handling: false,
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..Default::default()
                    },
                    // This will spawn an invisible window
                    // The window will be made visible in the make_visible() system after 3 frames.
                    // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
                    visible: false,
                    ..default()
                }),
                ..default()
            }).set(AssetPlugin {
                processed_file_path: "assets".to_string(),
                unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
                ..default()
            }),
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .init_state::<GameState>()
        .init_resource::<ColliderEventSet>()
        .init_resource::<TankContactMap>()
        .init_resource::<CanFire>()
        .init_resource::<BulletOwners>()
        .init_resource::<StartAnimationFrames>()
        .init_resource::<FadingOut>()
        .insert_resource(Score(0))
        .insert_resource(Life(2))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(OnEnter(GameState::StartScreen), spawn_start_screen)
        .add_systems(OnEnter(GameState::FadingOut), setup_fade_out)
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            (collect_contact_info, collect_collision, move_enemy_tanks).chain(),
            move_player_tank,
            animate_tank_texture,
            shoot_bullets,
            player_shoot_bullet,
            check_bullet_bounds,
            check_bullet_destruction,
            handle_bullet_collisions,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_start_screen.run_if(not(in_state(GameState::Playing))))
        .add_systems(Update, handle_start_screen_input.run_if(in_state(GameState::StartScreen)))
        .add_systems(Update, fade_out_screen.run_if(in_state(GameState::FadingOut)))
        .run();
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
    for i in 1..=70 {
        let frame_num = format!("{i:04}");
        let texture = asset_server.load(format!("start_scene/frame_{frame_num}.png"));
        animation_frames.push(texture);
    }

    let animation_indices = AnimationIndices { first: 0, last: 69 };

    // 添加动画背景
    commands.spawn((
        StartScreenUI,
        Sprite::from_image(animation_frames[0].clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.083, TimerMode::Repeating)), // 约12fps
        CurrentAnimationFrame(0),
    ));

    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load("/home/nanbert/.fonts/SHOWG.TTF");

    // 添加上方红色标题
    commands.spawn((
        StartScreenUI,
        Text2d("For Communism!!".to_string()),
        TextFont {
            font_size: 60.0,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 400.0, 1.0),
    ));

    // 添加中间 PLAY 文字
    commands.spawn((
        StartScreenUI,
        Text2d("PLAY".to_string()),
        TextFont {
            font_size: 80.0,
            font: custom_font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    // 添加下方操作说明
    commands.spawn((
        StartScreenUI,
        Text2d("Press SPACE to start | WASD to move | SPACE to shoot".to_string()),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -450.0, 1.0),
    ));
}


fn spawn_game_entities(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut can_fire: ResMut<CanFire>,
    mut clear_color: ResMut<ClearColor>,
) {
    // 设置背景色为黑色
    clear_color.0 = BACKGROUND_COLOR;

    // 左墙
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, ARENA_HEIGHT / 200.0),
        Transform{
            translation: Vec3::new(-ARENA_WIDTH / 2.0 - 5.0, 0.0, 0.0),
            scale: Vec3::new(10.0 , ARENA_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 右墙
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, ARENA_HEIGHT / 200.0),
        Transform{
            translation: Vec3::new(ARENA_WIDTH / 2.0 + 5.0, 0.0, 0.0),
            scale: Vec3::new(10.0 , ARENA_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 上墙
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(ARENA_WIDTH / 200.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, ARENA_HEIGHT / 2.0 + 5.0, 0.0),
            scale: Vec3::new(ARENA_WIDTH, 10.0, 1.0),
            ..default()
        }
    ));

    // 下墙
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(ARENA_WIDTH / 200.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, -ARENA_HEIGHT / 2.0 -5.0, 0.0),
            scale: Vec3::new(ARENA_WIDTH, 10.0 , 1.0),
            ..default()
        }
    ));

    // 堡垒（底部中央）
    let fortress_y = -ARENA_HEIGHT / 2.0 + FORTRESS_SIZE / 2.0 + 20.0;
    let fortress_x = 0.0;

    // 堡垒主体（红色五角星）
    let _star_points = create_star_points(FORTRESS_SIZE / 2.0);
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
    let texture = asset_server.load("texture/tank_player.png");

    // 精灵图是 1x3 的布局
    let tile_size = UVec2::new(87, 103); // 每个精灵的实际尺寸
    let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    // 玩家坦克 - 初始位置在堡垒左侧
    let fortress_y = -ARENA_HEIGHT / 2.0 + FORTRESS_SIZE / 2.0 + 20.0;
    let player_tank_entity = commands.spawn_empty()
        .insert(PlayerTank)
        .insert(Sprite::from_atlas_image(
            texture.clone(),
            TextureAtlas{
                layout: texture_atlas_layout.clone(),
                index: animation_indices.first,
            }
        ))
        .insert(Transform::from_xyz(WALL_THICKNESS.mul_add(-2.0, -FORTRESS_SIZE) - TANK_WIDTH / 2.0 - 20.0, fortress_y, 0.0))
        .insert(animation_indices)
        .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
        .insert(PlayerShootTimer(Timer::from_seconds(0.3, TimerMode::Repeating)))
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
        .id();

    // 敌方坦克
    let mut enemy_tank_entities = Vec::new();
    for pos in ENEMY_BORN_PLACES {

        let entity = commands.spawn_empty()
            .insert(EnemyTank {
                direction: Vec2::new(0.0, -1.0),
            })
            .insert(DirectionChangeTimer(Timer::from_seconds(4.0, TimerMode::Once)))
            .insert(CollisionCooldownTimer(Timer::from_seconds(0.5, TimerMode::Once)))
            .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))
            .insert(Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas{
                    layout: texture_atlas_layout.clone(),
                    index: animation_indices.first,
                }
            ))
            .insert(Transform::from_translation(pos))
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
            .id();
        enemy_tank_entities.push(entity);
    }

    // 初始化所有敌方坦克都可以射击
    for entity in enemy_tank_entities {
        can_fire.0.insert(entity);
    }
    // 初始化玩家坦克可以射击
    can_fire.0.insert(player_tank_entity);
}

fn handle_start_screen_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::FadingOut);
    }
}

fn create_star_points(radius: f32) -> Vec<Vec2> {
    let mut points = Vec::with_capacity(10);
    let inner_radius = radius * 0.4;

    for i in 0..10 {
        let angle = std::f32::consts::PI * 2.0 * i as f32 / 10.0 - std::f32::consts::PI / 2.0;
        let r = if i % 2 == 0 { radius } else { inner_radius };
        points.push(Vec2::new(r * angle.cos(), r * angle.sin()));
    }
    points
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
            *sprite = Sprite::from_image(animation_frames[next_index].clone());
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

fn move_enemy_tanks(
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut Velocity,
        &mut EnemyTank,
        &mut DirectionChangeTimer,
        &mut CollisionCooldownTimer,
        &mut Transform,
    )>,
    rapier_context: ReadRapierContext,
) {
    let rapier_context = rapier_context.single().unwrap();

    for (entity, mut velocity, mut enemy_tank, mut direction_timer, mut collision_cooldown, mut transform) in &mut query {
        // 更新碰撞冷却计时器
        collision_cooldown.tick(time.delta());

        // 只在冷却时间结束后才检测碰撞
        if collision_cooldown.is_finished() {
            // 检查碰撞
            let mut collision_detected = false;
            let mut collision_normal = Vec2::ZERO;

            for contact_pair in rapier_context.contact_pairs_with(entity) {
                if !contact_pair.has_any_active_contact() {
                    continue;
                }

                for manifold in contact_pair.manifolds() {
                    if let Some(_contact) = manifold.find_deepest_contact() {
                        // 检测到碰撞
                        collision_detected = true;

                        // 获取碰撞法线方向
                        collision_normal = if contact_pair.collider1() == Some(entity) {
                            -manifold.normal()
                        } else {
                            manifold.normal()
                        };
                    }
                }
            }

            // 如果检测到碰撞，根据碰撞法线选择可移动方向
            if collision_detected {
                // 将法线规范化到四个主要方向
                let abs_x = collision_normal.x.abs();
                let abs_y = collision_normal.y.abs();

                let blocked_direction = if abs_x > abs_y {
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
                };

                // 选择一个不与碰撞方向相反的可移动方向
                let possible_directions = [
                    Vec2::new(0.0, 1.0),   // 上
                    Vec2::new(0.0, -1.0),  // 下
                    Vec2::new(-1.0, 0.0),  // 左
                    Vec2::new(1.0, 0.0),   // 右
                ];

                let available_directions: Vec<Vec2> = possible_directions
                    .iter()
                    .filter(|dir| **dir != blocked_direction)
                    .copied()
                    .collect();

                if !available_directions.is_empty() {
                    let mut rng = rand::rng();
                    let random_index = rng.random_range(0..available_directions.len());
                    enemy_tank.direction = available_directions[random_index];
                }

                // 重置计时器
                direction_timer.reset();
                // 重置碰撞冷却时间
                collision_cooldown.reset();
            }
        }

        // 更新方向计时器
        direction_timer.tick(time.delta());

        // 如果计时器结束，有10%几率随机转向
        if direction_timer.just_finished() {
            let mut rng = rand::rng();
            if rng.random::<f32>() < 0.1 {
                let possible_directions = [
                    Vec2::new(0.0, 1.0),   // 上
                    Vec2::new(0.0, -1.0),  // 下
                    Vec2::new(-1.0, 0.0),  // 左
                    Vec2::new(1.0, 0.0),   // 右
                ];

                let random_index = rng.random_range(0..possible_directions.len());
                enemy_tank.direction = possible_directions[random_index];
            }
            // 重置计时器为4秒
            direction_timer.reset();
        }

        // 直接设置速度以保持匀速运动
        velocity.linvel = enemy_tank.direction * TANK_SPEED;
        // 更新旋转以面向移动方向
        if enemy_tank.direction.length() > 0.0 {
            let angle = enemy_tank.direction.y.atan2(enemy_tank.direction.x);
            transform.rotation = Quat::from_rotation_z(angle - 270.0_f32.to_radians());
        }
    }
}

fn collect_contact_info(
    rapier_context: ReadRapierContext,
    tanks: Query<Entity, With<EnemyTank>>,
    mut contact_info: ResMut<TankContactMap>,
){
    contact_info.max_depth_normals.clear();
    let rapier_context = rapier_context.single().unwrap();
    for entity_tank in tanks{
        let mut max_depth:f32 = 0.0;
        let mut max_depth_normal = Vec2::new(0.0, 0.0);
        for contact_pair in rapier_context.contact_pairs_with(entity_tank){
            if !contact_pair.has_any_active_contact(){
                continue;
            }
            for manifold in contact_pair.manifolds(){
                let dist = -manifold.find_deepest_contact().unwrap().dist();
                if dist < max_depth{
                    continue;
                }
                max_depth = dist;
                max_depth_normal = if contact_pair.collider1() == Some(entity_tank) {
                    -manifold.normal()
                } else{
                    manifold.normal()
                }
            }
        }
        let abs_x =max_depth_normal.x.abs();
        let abs_y =max_depth_normal.y.abs();

        max_depth_normal = if abs_x > abs_y {
            Vec2::new(max_depth_normal.x.signum(), 0.0)
        } else {
            Vec2::new(0.0, max_depth_normal.y.signum())
        };
        // 降低阈值，更早检测到碰撞
        if max_depth < 0.2{
            continue;
        }
        contact_info.max_depth_normals.insert(entity_tank, max_depth_normal);
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

fn handle_bullet_collisions(
    mut commands: Commands,
    mut collision_events: MessageReader<CollisionEvent>,
    bullets: Query<(Entity, &BulletOwner), With<Bullet>>,
    enemy_tanks: Query<Entity, With<EnemyTank>>,
    player_tanks: Query<Entity, With<PlayerTank>>,
) {
    let enemy_tank_set: HashSet<Entity> = enemy_tanks.iter().collect();
    let player_tank_set: HashSet<Entity> = player_tanks.iter().collect();

    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            // 检查是否是子弹与坦克的碰撞
            let (bullet_entity, tank_entity) = if bullets.get(*e1).is_ok() {
                if enemy_tank_set.contains(e2) || player_tank_set.contains(e2) {
                    (*e1, *e2)
                } else {
                    continue;
                }
            } else if bullets.get(*e2).is_ok() {
                if enemy_tank_set.contains(e1) || player_tank_set.contains(e1) {
                    (*e2, *e1)
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let bullet_owner = bullets.get(bullet_entity).unwrap().1.owner;

            // 判断碰撞类型
            let is_player_bullet = player_tank_set.contains(&bullet_owner);
            let is_enemy_bullet = enemy_tank_set.contains(&bullet_owner);
            let is_player_tank = player_tank_set.contains(&tank_entity);
            let is_enemy_tank = enemy_tank_set.contains(&tank_entity);

            // 规则：
            // 1. 玩家子弹打到敌方坦克 -> 子弹消失
            // 2. 敌方子弹打到玩家坦克 -> 子弹消失
            // 3. 敌方子弹打到敌方坦克 -> 子弹穿过（不消失）
            // 4. 玩家子弹打到玩家坦克 -> 子弹穿过（不消失）

            if (is_player_bullet && is_enemy_tank) || (is_enemy_bullet && is_player_tank) {
                // 子弹应该消失
                commands.entity(bullet_entity).despawn();
            }
            // 其他情况子弹穿过，不做处理
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
            if timer.just_finished() {
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = if atlas.index == indices.last {
                        indices.first
                    } else {
                        atlas.index + 1
                    };
                }
            }
        } else {
            // 坦克静止时重置到第一帧
            timer.reset();
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = indices.first;
            }
        }
    }
}

fn move_player_tank(
    _time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<PlayerTank>>,
) {
    for (mut transform, mut velocity) in &mut query {
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

        // 应用速度（直接设置速度，配合高阻尼实现类似 Kinematic 的控制）
        if direction.length() > 0.0 {
            velocity.linvel = direction * TANK_SPEED;

            // 更新旋转以面向移动方向
            let angle = direction.y.atan2(direction.x);
            transform.rotation = Quat::from_rotation_z(angle - 270.0_f32.to_radians());
        } else {
            velocity.linvel = Vec2::ZERO;
        }
    }
}

fn player_shoot_bullet(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(Entity, &Transform, &mut PlayerShootTimer), With<PlayerTank>>,
    mut can_fire: ResMut<CanFire>,
    _player_tanks: Query<Entity, With<PlayerTank>>,
    mut bullet_owners: ResMut<BulletOwners>,
) {
    for (entity, transform, mut timer) in &mut query {
        timer.tick(time.delta());

        // 空格键射击
        if keyboard_input.pressed(KeyCode::Space) && timer.just_finished() && can_fire.0.contains(&entity) {
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

fn setup_fade_out(mut fading_out: ResMut<FadingOut>) {
    fading_out.alpha = 1.0;
}

fn fade_out_screen(
    mut commands: Commands,
    time: Res<Time>,
    mut fading_out: ResMut<FadingOut>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sprite_query: Query<(Entity, &mut Sprite), With<StartScreenUI>>,
    mut text_query: Query<(Entity, &mut TextColor), With<StartScreenUI>>,
) {
    // 减少透明度
    fading_out.alpha -= time.delta_secs() * (1.0 / 3.0); // 淡出速度，约 3 秒完成

    // 更新所有 Sprite 元素的透明度
    for (_entity, mut sprite) in &mut sprite_query {
        let linear = sprite.color.to_linear();
        sprite.color = Color::srgba(linear.red, linear.green, linear.blue, fading_out.alpha);
    }

    // 更新所有 Text 元素的颜色
    for (_entity, mut text_color) in &mut text_query {
        let linear = text_color.0.to_linear();
        text_color.0 = Color::srgba(linear.red, linear.green, linear.blue, fading_out.alpha);
    }

    // 淡出完成，切换到 Playing 状态并清理所有 StartScreenUI 元素
    if fading_out.alpha <= 0.0 {
        next_state.set(GameState::Playing);
        for (entity, _) in sprite_query.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _) in text_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}
