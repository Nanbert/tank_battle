//! A simplified implementation of the classic game "Battle City 1990"
//!
//!
use std::collections::HashSet;
use bevy::{
    diagnostic::{FrameCount, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    window::{
        CursorGrabMode, CursorIcon, CursorOptions, PresentMode, SystemCursorIcon, WindowLevel,
        WindowTheme,
    },
};
use bevy_rapier2d::prelude::*;

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
struct MoveTimer(Timer);

#[derive(Component)]
struct EnemyTank;

#[derive(Component)]
struct Wall;
// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const PLAYER_SPEED: f32 = 500.0;
const HOME_BASE_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const HOME_BASE_WIDTH: f32 = 20.0;
const ARENA_WIDTH: f32 = 1600.0;
const ARENA_HEIGHT: f32 = 1200.0;
const TANK_WIDTH: f32 = 87.0;
const TANK_HEIGHT: f32 = 103.0;

const ENEMY_BORN_PLACES: [Vec3; 3] = [
    Vec3::new(-ARENA_WIDTH / 2.0 + TANK_WIDTH / 2.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(0.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
    Vec3::new(ARENA_WIDTH/2.0 - TANK_WIDTH / 2.0, ARENA_HEIGHT/2.0 - TANK_HEIGHT / 2.0, 0.0),
];

const BACKGROUND_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);

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
            }),
        ))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .init_resource::<ColliderEventSet>()
        .insert_resource(Score(0))
        .insert_resource(Life(2))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_sprite, (display_contact_info, collect_collision, collect_contact, move_enemy_tanks).chain()))
        .run();
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2d);

    // 玩家坦克
    let player_tank_pos_x = -HOME_BASE_WIDTH / 2.0;
    let player_tank_pos_y = -HOME_BASE_WIDTH / 2.0;

    let texture = asset_server.load("texture/tank_player.png");
    
    // 精灵图是 1x3 的布局
    let tile_size = UVec2::new(87, 103); // 每个精灵的实际尺寸
    let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    commands.spawn((
        Sprite::from_atlas_image(
            texture.clone(),
            TextureAtlas{
                layout: texture_atlas_layout.clone(),
                index: animation_indices.first,
            }
        ),
        Transform::from_xyz(-30.0, -300.0, 0.0),
        animation_indices.clone(),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0),
        LockedAxes::ROTATION_LOCKED,
    ));
    // 敌方坦克
    for pos in ENEMY_BORN_PLACES {
        commands.spawn((
            EnemyTank,
            Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas{
                    layout: texture_atlas_layout.clone(),
                    index: animation_indices.first,
                }
            ),
            Transform::from_translation(pos),
            animation_indices.clone(),
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            Velocity{
                linvel: Vec2::new(0.0, -100.0),
                angvel: 0.0,
            },
            MoveTimer(Timer::from_seconds(4.0, TimerMode::Repeating)),
            RigidBody::KinematicVelocityBased,
            Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0),
            ActiveEvents::COLLISION_EVENTS|ActiveEvents::CONTACT_FORCE_EVENTS,
            ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC | ActiveCollisionTypes::KINEMATIC_STATIC,
            LockedAxes::ROTATION_LOCKED,
        ));
    }
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
}
fn collect_collision(
    mut collision_events: EventReader<CollisionEvent>,
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
fn collect_contact(
    mut contact_events: EventReader<ContactForceEvent>,
)
{
    for event in contact_events.read(){
            println!("contact happen");
        
    }
}

fn move_enemy_tanks(
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &mut MoveTimer), With<EnemyTank>>,
    time: Res<Time>,
    collision_set: ResMut<ColliderEventSet>,
) {
    for (entity, mut transform, mut velocity, mut timer) in query.iter_mut() {
        //timer.tick(time.delta());
        // 移动
        //transform.translation += velocity.linvel.extend(0.0) * time.delta_secs();
        // 每次都根据速度方向更新朝向（关键！）
        let angle = velocity.linvel.y.atan2(velocity.linvel.x); // 速度向量的角度
        transform.rotation = Quat::from_rotation_z((angle - 270.0_f32.to_radians()));
        if timer.just_finished() || collision_set.entities.contains(&entity){
            let rand_num = rand::random::<f32>();
            let current_vel = velocity.linvel;

            velocity.linvel = if rand_num < 0.25 {
                // 逆时针90: (x, y) -> (-y, x)
                Vec2::new(-current_vel.y, current_vel.x)
            } else if rand_num < 0.5 {
                // 顺时针90: (x, y) -> (y, -x)
                Vec2::new(current_vel.y, -current_vel.x)
            } else  {
                // 180度: (x, y) -> (-x, -y)
                Vec2::new(-current_vel.x, -current_vel.y)
                    
            };
        }
    }
}

fn display_contact_info(
    rapier_context: ReadRapierContext, 
    tanks: Query<(Entity), With<EnemyTank>>,
    walls: Query<(Entity), With<Wall>>,
    score: Res<Score>
){
    let rapier_context = rapier_context.single().unwrap();
    for entity_tank in tanks{
        for entity_wall in walls{
            if let Some(contact_pair) = rapier_context.contact_pair(entity_tank, entity_wall){
                if contact_pair.has_any_active_contact(){
                    let mut i:u16 = 0;
                    for manifold in contact_pair.manifolds(){
                        println!("World-space contact normal:{}, {}", i, manifold.normal());
                        i+=1;
                    }
                }
            }
        }
    }

    // /* Find the contact pair, if it exists, between two colliders. */
    // if let Some(contact_pair) = rapier_context.contact_pair(entity1, entity2) {
    //     // The contact pair exists meaning that the broad-phase identified a potential contact.
    //     if contact_pair.has_any_active_contact() {
    //         // The contact pair has active contacts, meaning that it
    //         // contains contacts for which contact forces were computed.
    //     }

    //     // We may also read the contact manifolds to access the contact geometry.
    //     for manifold in contact_pair.manifolds() {
    //         println!("Local-space contact normal: {}", manifold.local_n1());
    //         println!("Local-space contact normal: {}", manifold.local_n2());
    //         println!("World-space contact normal: {}", manifold.normal());

    //         // Read the geometric contacts.
    //         for contact_point in manifold.points() {
    //             // Keep in mind that all the geometric contact data are expressed in the local-space of the colliders.
    //             println!(
    //                 "Found local contact point 1: {:?}",
    //                 contact_point.local_p1()
    //             );
    //             println!("Found contact distance: {:?}", contact_point.dist()); // Negative if there is a penetration.
    //             println!("Found contact impulse: {}", contact_point.raw.data.impulse);
    //             println!(
    //                 "Found friction impulse: {}",
    //                 contact_point.raw.data.tangent_impulse
    //             );
    //         }

    //         // Read the solver contacts.
    //         for solver_contact in &manifold.raw.data.solver_contacts {
    //             // Keep in mind that all the solver contact data are expressed in world-space.
    //             println!("Found solver contact point: {:?}", solver_contact.point);
    //             // The solver contact distance is negative if there is a penetration.
    //             println!("Found solver contact distance: {:?}", solver_contact.dist);
    //         }
    //     }
    // }
}
