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

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    StartScreen,
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
struct MoveTimer(Timer);

#[derive(Component)]
struct EnemyTank;

#[derive(Component, Deref, DerefMut)]
struct PauseTimer(Timer);

#[derive(Component)]
struct Accelerating {
    target_speed: f32,
    current_speed: f32,
}

#[derive(Component)]
struct PlayerTank;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct Fortress;

#[derive(Resource, Default)]
struct CanFire(HashSet<Entity>);

#[derive(Component, Deref, DerefMut)]
struct PlayerShootTimer(Timer);

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const ARENA_WIDTH: f32 = 1600.0;
const ARENA_HEIGHT: f32 = 1200.0;
const TANK_WIDTH: f32 = 87.0;
const TANK_HEIGHT: f32 = 103.0;
const TANK_SPEED: f32 = 200.0;
const BULLET_SPEED: f32 = 400.0;
const BULLET_SIZE: f32 = 10.0;

const ENEMY_BORN_PLACES: [Vec3; 3] = [
    Vec3::new(-ARENA_WIDTH / 2.0 + TANK_WIDTH / 2.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(0.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(ARENA_WIDTH/2.0 - TANK_WIDTH / 2.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
];

const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);

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
        .insert_resource(Score(0))
        .insert_resource(Life(2))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::StartScreen), spawn_start_screen)
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities)
        .add_systems(Update, (
            animate_sprite,
            (collect_contact_info, collect_collision, move_enemy_tanks).chain(),
            move_player_tank,
            shoot_bullets,
            player_shoot_bullet,
            move_bullets,
            check_bullet_bounds,
            check_bullet_destruction,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_start_screen_input.run_if(in_state(GameState::StartScreen)))
        .run();
}

fn spawn_start_screen(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 苏联硬核风格背景 - 深红色渐变效果
    commands.spawn((
        StartScreenUI,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.05, 0.05)),
    ));

    // 大五角星背景装饰
    commands.spawn((
        StartScreenUI,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.8, 0.1, 0.1)),
    ));

    commands.spawn((
        StartScreenUI,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(50.0),
            right: Val::Px(50.0),
            width: Val::Px(80.0),
            height: Val::Px(80.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.6, 0.08, 0.08)),
    ));

    // 装饰性线条 - 镰刀锤头风格
    commands.spawn((
        StartScreenUI,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(ARENA_HEIGHT / 2.0 - 5.0),
            left: Val::Px(0.0),
            width: Val::Px(20.0),
            height: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.9, 0.2, 0.0)),
    ));

    commands.spawn((
        StartScreenUI,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(ARENA_HEIGHT / 2.0 - 5.0),
            right: Val::Px(0.0),
            width: Val::Px(20.0),
            height: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.9, 0.2, 0.0)),
    ));

    // 主标题 "开始"
    commands.spawn((
        StartScreenUI,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(ARENA_HEIGHT / 2.0 - 80.0),
            left: Val::Px(ARENA_WIDTH / 2.0 - 100.0),
            width: Val::Px(200.0),
            height: Val::Px(120.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        Text::new("Fire for Communism!!"),
        TextFont {
            font: asset_server.load("/home/nanbert/.fonts/msyh.ttc"),
            font_size: 120.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.9, 0.0)),
        TextLayout::new_with_justify(Justify::Center),
    ));

    // 底部操作提示
    commands.spawn((
        StartScreenUI,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(100.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Auto,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new("WASD: Direction  Space: shoot/confirm"),
        TextFont {
            font: asset_server.load("/home/nanbert/.fonts/msyh.ttc"),
            font_size: 32.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextLayout::new_with_justify(Justify::Center),
    ));
}


fn spawn_game_entities(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut can_fire: ResMut<CanFire>,
) {
    let texture = asset_server.load("texture/tank_player.png");

    // 精灵图是 1x3 的布局
    let tile_size = UVec2::new(87, 103); // 每个精灵的实际尺寸
    let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    // 玩家坦克 - 初始位置在堡垒左侧
    let fortress_y = -ARENA_HEIGHT / 2.0 + FORTRESS_SIZE / 2.0 + 20.0;
    let player_tank_entity = commands.spawn((
        PlayerTank,
        Sprite::from_atlas_image(
            texture.clone(),
            TextureAtlas{
                layout: texture_atlas_layout.clone(),
                index: animation_indices.first,
            }
        ),
        Transform::from_xyz(-FORTRESS_SIZE - WALL_THICKNESS * 2.0 - TANK_WIDTH / 2.0 - 20.0, fortress_y, 0.0),
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        PlayerShootTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
        Velocity{
            linvel: Vec2::new(0.0, 0.0),
            angvel: 0.0,
        },
        RigidBody::KinematicVelocityBased,
        Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0),
        ActiveEvents::COLLISION_EVENTS,
        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
        LockedAxes::ROTATION_LOCKED,
    )).id();

    // 敌方坦克
    let mut enemy_tank_entities = Vec::new();
    for pos in ENEMY_BORN_PLACES {
        let entity = commands.spawn((
            EnemyTank,
            Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas{
                    layout: texture_atlas_layout.clone(),
                    index: animation_indices.first,
                }
            ),
            Transform::from_translation(pos),
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Velocity{
                linvel: Vec2::new(0.0, -TANK_SPEED),
                angvel: 0.0,
            },
            MoveTimer(Timer::from_seconds(6.0, TimerMode::Repeating)),
            RigidBody::KinematicVelocityBased,
            Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0),
            ActiveEvents::COLLISION_EVENTS|ActiveEvents::CONTACT_FORCE_EVENTS,
            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC | ActiveCollisionTypes::KINEMATIC_STATIC,
            LockedAxes::ROTATION_LOCKED,
        )).id();
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
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    query: Query<Entity, With<StartScreenUI>>
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
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

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite)>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished()
            && let Some(atlas) = &mut sprite.texture_atlas
        {
            atlas.index = if atlas.index == indices.last {
                indices.first
            } else {
                atlas.index + 1
            };
        }
    }
}
fn setup(
    mut commands: Commands,
) {
    // Camera
    commands.spawn(Camera2d);

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
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &mut MoveTimer, Option<&mut PauseTimer>, Option<&mut Accelerating>), With<EnemyTank>>,
    time: Res<Time>,
    collision_set: ResMut<ColliderEventSet>,
    contact_info: ResMut<TankContactMap>,
) {
    for (entity, mut transform, mut velocity, mut timer, mut pause_timer, mut accelerating) in &mut query {
        timer.tick(time.delta());

        // 如果在停顿状态，处理停顿计时器
        if let Some(ref mut pause) = pause_timer {
            pause.tick(time.delta());
            if pause.just_finished() {
                // 停顿结束，移除停顿组件，进入加速状态
                commands.entity(entity).remove::<PauseTimer>();

                // 获取当前方向
                let direction = if velocity.linvel.length() > 0.0 {
                    velocity.linvel.normalize()
                } else {
                    Vec2::new(0.0, -1.0)
                };

                // 添加加速组件
                commands.entity(entity).insert(Accelerating {
                    target_speed: TANK_SPEED,
                    current_speed: 0.0,
                });

                velocity.linvel = direction * 0.0;
            } else {
                // 停顿期间，速度为0
                velocity.linvel = Vec2::ZERO;
                continue;
            }
        }

        // 如果在加速状态，处理加速逻辑
        if let Some(ref mut acc) = accelerating {
            let acceleration_rate = TANK_SPEED * 2.0; // 0.5秒内加速到目标速度
            acc.current_speed += acceleration_rate * time.delta_secs();

            if acc.current_speed >= acc.target_speed {
                // 加速完成
                acc.current_speed = acc.target_speed;
                commands.entity(entity).remove::<Accelerating>();
            }

            let direction = if velocity.linvel.length() > 0.0 {
                velocity.linvel.normalize()
            } else {
                Vec2::new(0.0, -1.0)
            };

            velocity.linvel = direction * acc.current_speed;
        }

        // 碰撞检测和转向逻辑
        if let Some(_max_depth_normal) = contact_info.max_depth_normals.get(&entity) {
            // 遇到障碍物，停顿并准备转向
            if pause_timer.is_none() && accelerating.is_none() {
                commands.entity(entity).insert(PauseTimer(Timer::from_seconds(0.5, TimerMode::Once)));
                velocity.linvel = Vec2::ZERO;
            }
        } else if timer.just_finished() || collision_set.entities.contains(&entity) {
            // 定时转向或碰撞后转向
            if pause_timer.is_none() && accelerating.is_none() {
                let rand_num = rand::random::<f32>();
                let current_vel = velocity.linvel;

                let new_direction = if rand_num < 0.25 {
                    // 逆时针90: (x, y) -> (-y, x)
                    Vec2::new(-current_vel.y, current_vel.x)
                } else if rand_num < 0.5 {
                    // 顺时针90: (x, y) -> (y, -x)
                    Vec2::new(current_vel.y, -current_vel.x)
                } else {
                    // 180度: (x, y) -> (-x, -y)
                    Vec2::new(-current_vel.x, -current_vel.y)
                };

                let normalized_dir = if new_direction.length() > 0.0 {
                    new_direction.normalize()
                } else {
                    Vec2::new(0.0, -1.0)
                };

                velocity.linvel = normalized_dir * TANK_SPEED;
            }
        }

        // 更新旋转
        if velocity.linvel.length() > 0.0 {
            let angle = velocity.linvel.y.atan2(velocity.linvel.x);
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
        if max_depth < 0.4{
            continue;
        }
        contact_info.max_depth_normals.insert(entity_tank, max_depth_normal);
    }

}

fn shoot_bullets(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &Velocity), With<EnemyTank>>,
    mut can_fire: ResMut<CanFire>,
) {
    for (entity, transform, velocity) in &mut query {
        // 检查是否可以射击（当前没有子弹）
        if can_fire.0.contains(&entity) {
            // 随机射击，每帧有 0.5% 的概率射击
            if rand::random::<f32>() < 0.005 {
                // 计算子弹发射方向（基于坦克当前移动方向）
                let direction = if velocity.linvel.length() > 0.0 {
                    velocity.linvel.normalize()
                } else {
                    Vec2::new(0.0, -1.0) // 默认向下
                };

                // 计算子弹初始位置（坦克前方）
                let bullet_pos = transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

                // 生成子弹
                commands.spawn((
                    Bullet,
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
                ));

                // 标记该坦克暂时不能射击
                can_fire.0.remove(&entity);
            }
        }
    }
}

fn move_bullets(
    _query: Query<&Transform, With<Bullet>>,
) {
    // 子弹移动由物理引擎处理，这里不需要额外操作
}

fn check_bullet_bounds(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in &mut query {
        let x = transform.translation.x;
        let y = transform.translation.y;

        // 检查子弹是否超出游戏窗口边界
        if x < -ARENA_WIDTH / 2.0 || x > ARENA_WIDTH / 2.0 ||
           y < -ARENA_HEIGHT / 2.0 || y > ARENA_HEIGHT / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn check_bullet_destruction(
    removed_bullets: RemovedComponents<Bullet>,
    mut can_fire: ResMut<CanFire>,
    enemy_tanks: Query<Entity, With<EnemyTank>>,
    player_tanks: Query<Entity, With<PlayerTank>>,
) {
    // 当子弹被销毁时，允许所有敌方坦克和玩家坦克可以再次射击
    if !removed_bullets.is_empty() {
        for tank_entity in enemy_tanks.iter() {
            can_fire.0.insert(tank_entity);
        }
        for tank_entity in player_tanks.iter() {
            can_fire.0.insert(tank_entity);
        }
    }
}

fn move_player_tank(
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

        // 应用速度
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
    mut query: Query<(Entity, &Transform, &Velocity, &mut PlayerShootTimer), With<PlayerTank>>,
    mut can_fire: ResMut<CanFire>,
    _player_tanks: Query<Entity, With<PlayerTank>>,
) {
    for (entity, transform, velocity, mut timer) in &mut query {
        timer.tick(time.delta());

        // 空格键射击
        if keyboard_input.pressed(KeyCode::Space) && timer.just_finished() && can_fire.0.contains(&entity) {
            // 计算子弹发射方向（基于坦克当前移动方向，如果没有移动则向上）
            let direction = if velocity.linvel.length() > 0.0 {
                velocity.linvel.normalize()
            } else {
                Vec2::new(0.0, 1.0) // 默认向上
            };

            // 计算子弹初始位置（坦克前方）
            let bullet_pos = transform.translation + direction.extend(0.0) * (TANK_HEIGHT / 2.0 + BULLET_SIZE);

            // 生成玩家子弹
            commands.spawn((
                Bullet,
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
            ));

            // 标记玩家坦克暂时不能射击
            can_fire.0.remove(&entity);
        }
    }
}
