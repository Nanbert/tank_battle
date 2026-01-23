//! 应用配置模块
//!
//! 处理窗口配置、资源配置、游戏初始化等

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use bevy::audio::Volume;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::*;
use crate::enemy;
use crate::player;
use crate::ui;
use crate::effects;
use crate::terrain;
use crate::game_state;
use crate::bullet;
use crate::laser;
use crate::map;
use crate::levels;

pub fn configure_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "For Communism!!".to_string(),
            name: Some("tank_battle".to_string()),
            resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
            present_mode: PresentMode::AutoVsync,
            resizable: false,
            mode: bevy::window::WindowMode::Windowed,
            ..default()
        }),
        ..default()
    }
}

pub fn configure_asset_plugin() -> AssetPlugin {
    AssetPlugin {
        file_path: "assets".to_string(),
        ..default()
    }
}

pub fn configure_game_resources(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(GameMode::OnePlayer)
        .insert_resource(StageLevel(1))
        .insert_resource(CommanderLife {
            life_red_bar: COMMANDER_LIFE_MAX,
        })
        .insert_resource(PlayerInfo::default())
        .insert_resource(EnemySpawnState::default())
        .insert_resource(RecallTimers::default())
        .insert_resource(DashTimers::default())
        .insert_resource(DashDamageTracker::default())
        .insert_resource(BarrierDamageTracker::default())
        .insert_resource(StartAnimationFrames::default())
        .insert_resource(FadingOut { alpha: 1.0 })
        .insert_resource(CurrentMenuSelection { selected_index: 0 })
        .insert_resource(AnimationIndices { first: 0, last: 14 })
        .insert_resource(CurrentAnimationFrame(0))
        .init_resource::<GameEntitiesSpawned>()
        .init_resource::<StageIntroTimer>();
}

pub fn register_game_systems(app: &mut App) {
    app.init_state::<GameState>()
        .add_message::<PlayerStatChanged>()
        .add_message::<crate::bullet::EffectEvent>()
        .init_resource::<BulletTracker>()
        .add_systems(Startup, setup)
        .add_systems(OnEnter(GameState::StartScreen), (game_state::cleanup_playing_entities, ui::spawn_start_screen).chain())
        .add_systems(OnEnter(GameState::About), (ui::cleanup_start_screen_ui, ui::spawn_about_screen).chain())
        .add_systems(OnExit(GameState::About), (ui::despawn_about_screen, ui::spawn_start_screen).chain())
        .add_systems(Update, ui::handle_about_input.run_if(in_state(GameState::About)))
        .add_systems(OnEnter(GameState::Credits), (ui::cleanup_start_screen_ui, ui::spawn_credits_screen).chain())
        .add_systems(OnExit(GameState::Credits), (ui::despawn_credits_screen, ui::spawn_start_screen).chain())
        .add_systems(Update, ui::handle_credits_input.run_if(in_state(GameState::Credits)))
        .add_systems(OnEnter(GameState::StageIntro), (game_state::reset_for_next_stage, ui::spawn_stage_intro).chain())
        .add_systems(Update, ui::handle_stage_intro_timer.run_if(in_state(GameState::StageIntro)))
        .add_systems(OnExit(GameState::StageIntro), ui::despawn_stage_intro)
        // .add_systems(OnEnter(GameState::Playing), terrain::spawn_game_entities_if_needed)
        .add_systems(OnEnter(GameState::Paused), ui::spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), ( ui::despawn_pause_ui,))
        .add_systems(OnEnter(GameState::GameOver), ui::spawn_game_over_ui)
        .add_systems(OnExit(GameState::GameOver), (ui::despawn_game_over_ui, game_state::cleanup_playing_entities))
        .add_systems(Update, enemy::move_enemy_tanks.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy::enemy_spawn_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy::animate_enemy_born_animation.run_if(in_state(GameState::Playing)))
        .add_systems(Update, enemy::animate_enemy_tank_texture.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::move_player_tank.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::handle_recall_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::update_recall_timers.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::handle_dash_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::update_dash_movement.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::handle_dash_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::handle_barrier_collision.run_if(in_state(GameState::Playing)))
        .add_systems(Update, player::update_recall_progress_bars.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::enemy_shoot_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::player_shoot_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_bounds_check_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_despawn_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_terrain_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_tank_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::bullet_commander_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, bullet::handle_effect_events.run_if(in_state(GameState::Playing)))
        .add_systems(Update, laser::player_laser_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, effects::animate_explosion.run_if(in_state(GameState::Playing)))
        .add_systems(Update, effects::animate_laser.run_if(in_state(GameState::Playing)))
        .add_systems(Update, effects::animate_forest_fire.run_if(in_state(GameState::Playing)))
        .add_systems(Update, effects::animate_forest.run_if(in_state(GameState::Playing)))
        .add_systems(Update, effects::animate_spark.run_if(in_state(GameState::Playing)))
        .add_systems(Update, game_state::handle_game_over_delay.run_if(in_state(GameState::Playing)))
        .add_systems(Update, game_state::check_game_over.run_if(in_state(GameState::Playing)))
        .add_systems(Update, game_state::check_stage_complete.run_if(in_state(GameState::Playing)))
        .add_systems(Update, ui::animate_start_screen.run_if(not(in_state(GameState::Playing))))
        .add_systems(Update, (
            ui::handle_start_screen_input,
            ui::update_option_colors,
        ).run_if(in_state(GameState::StartScreen)))
        .add_systems(Update, ui::handle_game_input.run_if(in_state(GameState::Playing)))
        .add_systems(Update, ui::handle_pause_input.run_if(in_state(GameState::Paused)))
        .add_systems(Update, (ui::handle_game_over_input, ui::update_option_colors)
            .chain().run_if(in_state(GameState::GameOver)))
        .add_systems(Update, effects::handle_recoil_force.run_if(in_state(GameState::Playing)))
        .add_systems(Update, effects::animate_smoke.run_if(in_state(GameState::Playing)))
        .add_systems(Update, effects::laser_collision_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, ui::fade_out_screen.run_if(in_state(GameState::FadingOut)));
}

pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // 创建全局相机
    commands.spawn(Camera2d);

    // 加载玩家1坦克纹理
    let player1_texture: Handle<Image> = asset_server.load("texture/player_tank1_sprite.png");
    let player_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(87, 103), 4, 4, None, None);
    let player_texture_atlas_layout = texture_atlas_layouts.add(player_texture_atlas_layout);
    let player_animation_indices = AnimationIndices { first: 0, last: 15 };
    commands.insert_resource(Player1Texture {
        texture: player1_texture,
        texture_atlas_layout: player_texture_atlas_layout,
        animation_indices: player_animation_indices,
    });

    // 加载玩家2坦克纹理
    let player2_texture: Handle<Image> = asset_server.load("texture/player_tank2_sprite.png");
    let player2_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(87, 103), 4, 4, None, None);
    let player2_texture_atlas_layout = texture_atlas_layouts.add(player2_texture_atlas_layout);
    let player2_animation_indices = AnimationIndices { first: 0, last: 15 };
    commands.insert_resource(Player2Texture {
        texture: player2_texture,
        texture_atlas_layout: player2_texture_atlas_layout,
        animation_indices: player2_animation_indices,
    });

    // 加载敌方坦克纹理
    let enemy_tank1_texture: Handle<Image> = asset_server.load("enemy_tank/enemy_tank1_sprite.png");
    let enemy_tank1_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(87, 103), 4, 4, None, None);
    let enemy_tank1_texture_atlas_layout = texture_atlas_layouts.add(enemy_tank1_texture_atlas_layout);
    let enemy_tank1_animation_indices = AnimationIndices { first: 0, last: 15 };
    commands.insert_resource(EnemyTank1Texture {
        texture: enemy_tank1_texture,
        texture_atlas_layout: enemy_tank1_texture_atlas_layout,
        animation_indices: enemy_tank1_animation_indices,
    });

    // 加载子弹纹理
    let bullet_texture: Handle<Image> = asset_server.load("texture/bullets/bullet_player1.png");
    commands.insert_resource(BulletTexture {
        texture: bullet_texture,
    });

    // 加载激光纹理
    let laser_texture: Handle<Image> = asset_server.load("effect/texture_laser_red.png");
    let laser_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(70, 1366), 1, 2, None, None);
    let laser_texture_atlas_layout = texture_atlas_layouts.add(laser_texture_atlas_layout);
    commands.insert_resource(LaserTexture {
        texture: laser_texture,
        texture_atlas_layout: laser_texture_atlas_layout,
    });

    // 加载爆炸纹理
    let explosion_texture: Handle<Image> = asset_server.load("effect/explosion.png");
    let explosion_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 5, 3, None, None);
    let explosion_texture_atlas_layout = texture_atlas_layouts.add(explosion_texture_atlas_layout);
    let explosion_animation_indices = AnimationIndices { first: 0, last: 14 };
    commands.insert_resource(ExplosionTexture {
        texture: explosion_texture,
        texture_atlas_layout: explosion_texture_atlas_layout,
        animation_indices: explosion_animation_indices,
    });

    // 加载烟雾纹理
    let smoke_texture: Handle<Image> = asset_server.load("effect/smoke_sprite.png");
    let smoke_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 5, 3, None, None);
    let smoke_texture_atlas_layout = texture_atlas_layouts.add(smoke_texture_atlas_layout);
    let smoke_animation_indices = AnimationIndices { first: 0, last: 14 };
    commands.insert_resource(SmokeTexture {
        texture: smoke_texture,
        texture_atlas_layout: smoke_texture_atlas_layout,
        animation_indices: smoke_animation_indices,
    });

    // 加载火花纹理
    let spark_texture: Handle<Image> = asset_server.load("effect/BubbleBlue.png");
    let spark_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 5, 3, None, None);
    let spark_texture_atlas_layout = texture_atlas_layouts.add(spark_texture_atlas_layout);
    let spark_animation_indices = AnimationIndices { first: 0, last: 14 };
    commands.insert_resource(SparkTexture {
        texture: spark_texture,
        texture_atlas_layout: spark_texture_atlas_layout,
        animation_indices: spark_animation_indices,
    });

    // 加载森林火焰纹理
    let forest_fire_texture: Handle<Image> = asset_server.load("maps/tree_fire_sheet.png");
    let forest_fire_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(131, 131), 10, 1, None, None);
    let forest_fire_texture_atlas_layout = texture_atlas_layouts.add(forest_fire_texture_atlas_layout);
    let forest_fire_animation_indices = AnimationIndices { first: 0, last: 14 };
    commands.insert_resource(ForestFireTexture {
        texture: forest_fire_texture,
        texture_atlas_layout: forest_fire_texture_atlas_layout,
        animation_indices: forest_fire_animation_indices,
    });

    // 加载敌方坦克出生动画纹理
    let enemy_born_texture: Handle<Image> = asset_server.load("effect/enemy_born.png");
    let enemy_born_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(100, 100), 5, 3, None, None);
    let enemy_born_texture_atlas_layout = texture_atlas_layouts.add(enemy_born_texture_atlas_layout);
    let enemy_born_animation_indices = AnimationIndices { first: 0, last: 14 };
    commands.insert_resource(EnemyBornTexture {
        texture: enemy_born_texture,
        texture_atlas_layout: enemy_born_texture_atlas_layout,
        animation_indices: enemy_born_animation_indices,
    });

    // 加载指挥官纹理
    let commander_texture: Handle<Image> = asset_server.load("texture/commander.png");
    let commander_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(160, 147), 4, 4, None, None);
    let commander_texture_atlas_layout = texture_atlas_layouts.add(commander_texture_atlas_layout);
    let commander_animation_indices = AnimationIndices { first: 0, last: 15 };
    commands.insert_resource(CommanderTexture {
        texture: commander_texture,
        texture_atlas_layout: commander_texture_atlas_layout,
        animation_indices: commander_animation_indices,
    });

    // 加载指挥官死亡纹理
    let commander_dead_texture: Handle<Image> = asset_server.load("texture/commander_dead.png");
    commands.insert_resource(CommanderDeadTexture {
        texture: commander_dead_texture,
    });

    // 加载玩家头像纹理
    let avatar_texture: Handle<Image> = asset_server.load("texture/avatar.png");
    let avatar_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(160, 147), 4, 4, None, None);
    let avatar_texture_atlas_layout = texture_atlas_layouts.add(avatar_texture_atlas_layout);
    let avatar_animation_indices = AnimationIndices { first: 0, last: 15 };
    commands.insert_resource(AvatarTexture {
        texture: avatar_texture,
        texture_atlas_layout: avatar_texture_atlas_layout,
        animation_indices: avatar_animation_indices,
    });

    // 加载玩家死亡纹理
    let avatar_dead_texture: Handle<Image> = asset_server.load("texture/avatar_death.png");
    commands.insert_resource(AvatarDeadTexture {
        texture: avatar_dead_texture,
    });

    // 加载道具纹理
    let powerup_texture: Handle<Image> = asset_server.load("power_up/hamburger.png");
    let powerup_texture_atlas_layout = TextureAtlasLayout::from_grid(UVec2::new(50, 50), 1, 1, None, None);
    let powerup_texture_atlas_layout = texture_atlas_layouts.add(powerup_texture_atlas_layout);
    let powerup_animation_indices = AnimationIndices { first: 0, last: 0 };
    commands.insert_resource(PowerUpTexture {
        texture: powerup_texture,
        texture_atlas_layout: powerup_texture_atlas_layout,
        animation_indices: powerup_animation_indices,
    });

    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load(FONT_EN);
    commands.insert_resource(CustomFont { font: custom_font });
}
