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
mod enemy;
mod player;
mod ui;

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
    app.add_systems(OnEnter(GameState::StartScreen), (cleanup_playing_entities, ui::spawn_start_screen).chain())
        .add_systems(OnEnter(GameState::FadingOut), setup_fade_out)
        .add_systems(OnEnter(GameState::StageIntro), (reset_for_next_stage, ui::spawn_stage_intro).chain())
        .add_systems(Update, ui::handle_stage_intro_timer.run_if(in_state(GameState::StageIntro)))
        .add_systems(OnExit(GameState::StageIntro), ui::despawn_stage_intro)
        .add_systems(OnEnter(GameState::Playing), spawn_game_entities_if_needed)
        .add_systems(OnEnter(GameState::Paused), ui::spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), ( ui::despawn_pause_ui,))
        .add_systems(OnEnter(GameState::GameOver), ui::spawn_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), (ui::despawn_game_over_ui, cleanup_playing_entities))
        .add_systems(OnEnter(GameState::About), (ui::cleanup_start_screen_ui, ui::spawn_about_screen).chain())
        .add_systems(OnExit(GameState::About), (ui::despawn_about_screen, ui::spawn_start_screen).chain())
        .add_systems(Update, ui::handle_about_input.run_if(in_state(GameState::About)))
        .add_systems(OnEnter(GameState::Credits), (ui::cleanup_start_screen_ui, ui::spawn_credits_screen).chain())
        .add_systems(OnExit(GameState::Credits), (ui::despawn_credits_screen, ui::spawn_start_screen).chain())
        .add_systems(Update, ui::handle_credits_input.run_if(in_state(GameState::Credits)))
        .add_systems(Startup, setup)
        .add_systems(Update, (enemy::move_enemy_tanks).chain().run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy::enemy_spawn_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::move_player_tank.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_player_tank_texture.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy::animate_enemy_tank_texture.run_if(in_state(GameState::Playing)))
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
        .add_systems(Update, enemy::animate_enemy_born_animation.run_if(in_state(GameState::Playing)))
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
        .add_systems(Update, ui::animate_start_screen.run_if(not(in_state(GameState::Playing))))
        .add_systems(Update, (
            ui::handle_start_screen_input,
            ui::update_option_colors,
        ).run_if(in_state(GameState::StartScreen)))
        .add_systems(Update, update_menu_blink.run_if(in_state(GameState::FadingOut).or(in_state(GameState::StartScreen))))
        .add_systems(Update, ui::handle_game_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, ui::handle_pause_input.run_if(in_state(GameState::Paused)))
        .add_systems(Update, (ui::handle_game_over_input, ui::update_option_colors)
            .chain().run_if(in_state(GameState::GameOver)))
        .add_systems(Update, (
            player::handle_recall_input,
            player::update_recall_timers,
        ).run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::handle_dash_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::update_dash_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::handle_dash_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::handle_barrier_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::update_recall_progress_bars.run_if(in_state(GameState::Playing)))
        .add_systems(Update, handle_recoil_force.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_laser.run_if(in_state(GameState::Playing)))
        .add_systems(Update, animate_smoke.run_if(in_state(GameState::Playing)))
        .add_systems(Update, laser_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, ui::fade_out_screen.run_if(in_state(GameState::FadingOut)));
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
            autostep: Some(bevy_rapier2d::prelude::CharacterAutostep {
                max_height: CharacterLength::Absolute(5.0),
                min_width: CharacterLength::Absolute(0.5),
                include_dynamic_bodies: false,
            }),
            ..default()
        })
        .id()
}

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

                                            autostep: Some(bevy_rapier2d::prelude::CharacterAutostep {

                                                max_height: CharacterLength::Absolute(5.0),

                                                min_width: CharacterLength::Absolute(0.5),

                                                include_dynamic_bodies: false,

                                            }),

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

fn setup(
    mut commands: Commands,
) {
    // 创建全局相机
    commands.spawn(Camera2d);
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
    commands.spawn((
        AudioPlayer::new(explosion_sound),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(0.5)),
    ));
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

fn setup_fade_out(
    mut fading_out: ResMut<FadingOut>,
) {
    fading_out.alpha = 1.0;
}

// 文本更新函数类型
type TextUpdateFn = fn(&PlayerStats, TankType) -> Option<String>;

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
    player_info: Res<PlayerInfo>,
    commander_life: Res<CommanderLife>,
    game_mode: Res<GameMode>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stage_level: ResMut<StageLevel>,
) {
    // 检查是否完成关卡：已生成所有敌方坦克且当前没有存活的敌方坦克
    let current_enemy_count = enemies.iter().count();
    if enemy_spawn_state.has_spawned >= enemy_spawn_state.max_count && current_enemy_count == 0 {
        // 检查玩家是否阵亡
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

        // 如果玩家或 Commander 已阵亡，不能进入下一关
        if all_players_dead || commander_life.life_red_bar == 0 {
            return;
        }

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
