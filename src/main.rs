//! A simplified implementation of the classic game "Battle City 1990"
//!
//!
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

#[derive(Component)]
struct Wall;
// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
const ARENA_WIDTH: f32 = 1600.0;
const ARENA_HEIGHT: f32 = 1200.0;
const TANK_WIDTH: f32 = 87.0;
const TANK_HEIGHT: f32 = 103.0;
const TANK_SPEED: f32 = 120.0;

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
        .init_resource::<TankContactMap>()
        .insert_resource(Score(0))
        .insert_resource(Life(2))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_sprite, (collect_contact_info, collect_collision, move_enemy_tanks).chain()))
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
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    // Camera
    commands.spawn(Camera2d);


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
                linvel: Vec2::new(0.0, -TANK_SPEED),
                angvel: 0.0,
            },
            MoveTimer(Timer::from_seconds(6.0, TimerMode::Repeating)),
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
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &mut MoveTimer), With<EnemyTank>>,
    time: Res<Time>,
    collision_set: ResMut<ColliderEventSet>,
    contact_info: ResMut<TankContactMap>,
) {
    for (entity, mut transform, mut velocity, mut timer) in query.iter_mut() {
        timer.tick(time.delta());
        // 移动
        //transform.translation += velocity.linvel.extend(0.0) * time.delta_secs();
        if let Some(max_depth_normal) = contact_info.max_depth_normals.get(&entity){
            velocity.linvel = max_depth_normal*TANK_SPEED;
        }
        else if timer.just_finished() || collision_set.entities.contains(&entity){
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
        let angle = velocity.linvel.y.atan2(velocity.linvel.x); // 速度向量的角度
        transform.rotation = Quat::from_rotation_z(angle - 270.0_f32.to_radians());
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
