//! 应用配置模块
//!
//! 处理窗口配置、资源配置、游戏初始化等

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

use crate::constants::*;
use crate::resources::*;

// 导入模块以便使用其函数
use crate::bullet;
use crate::effects;
use crate::enemy;
use crate::game_state;
use crate::hud_ui;
use crate::laser;
use crate::menus_ui;
use crate::overlay_ui;
use crate::player;
use crate::powerup;
use crate::terrain;

pub fn configure_window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: "For Communism!!".to_string(),
            name: Some("tank_battle".to_string()),
            resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            present_mode: PresentMode::AutoVsync,
            resizable: false,
            mode: bevy::window::WindowMode::Windowed,
            focused: true,
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
        .init_resource::<BlueBarRegenTimer>()
        .insert_resource(StartAnimationFrames::default())
        .insert_resource(FadingOut { alpha: 1.0 })
        .insert_resource(CurrentMenuSelection { selected_index: 0 })
        .insert_resource(AnimationIndices { first: 0, last: 14 })
        .insert_resource(CurrentAnimationFrame(0))
        .insert_resource(MenuBlinkTimer(Timer::default()))
        .init_resource::<GameEntitiesSpawned>()
        .init_resource::<StageIntroTimer>()
        .init_resource::<crate::levels::LevelAssets>();
}

pub fn register_game_systems(app: &mut App) {
    app.init_state::<GameState>()
        .add_message::<PlayerStatChanged>()
        .add_message::<crate::bullet::EffectEvent>()
        .init_resource::<BulletTracker>()
        .add_systems(Startup, (setup, crate::levels::load_level_assets))
        .add_systems(
            OnEnter(GameState::StartScreen),
            (
                game_state::cleanup_playing_entities,
                hud_ui::despawn_hud,
                game_state::reset_fading_out,
                menus_ui::spawn_start_screen,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(GameState::About),
            (menus_ui::cleanup_start_screen_ui, menus_ui::spawn_about_screen).chain(),
        )
        .add_systems(
            OnExit(GameState::About),
            (menus_ui::despawn_about_screen, menus_ui::spawn_start_screen).chain(),
        )
        .add_systems(
            Update,
            menus_ui::handle_about_input.run_if(in_state(GameState::About)),
        )
        .add_systems(
            OnEnter(GameState::Credits),
            (menus_ui::cleanup_start_screen_ui, menus_ui::spawn_credits_screen).chain(),
        )
        .add_systems(
            OnExit(GameState::Credits),
            (menus_ui::despawn_credits_screen, menus_ui::spawn_start_screen).chain(),
        )
        .add_systems(
            Update,
            menus_ui::handle_credits_input.run_if(in_state(GameState::Credits)),
        )
        .add_systems(
            OnEnter(GameState::StageIntro),
            (
                terrain::respawn_terrain_for_next_stage,
                powerup::despawn_powerups,
                powerup::spawn_power_ups,
                hud_ui::spawn_hud,
                hud_ui::update_stage_text,
                game_state::reset_player_positions,
                game_state::reset_for_next_stage,
                overlay_ui::spawn_stage_intro,
            )
                .chain(),
        )
        .add_systems(
            Update,
            overlay_ui::handle_stage_intro_timer.run_if(in_state(GameState::StageIntro)),
        )
        .add_systems(OnExit(GameState::StageIntro), overlay_ui::despawn_stage_intro)
        .add_systems(
            OnEnter(GameState::Playing),
            terrain::spawn_game_entities_if_needed,
        )
        .add_systems(OnEnter(GameState::Paused), overlay_ui::spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), (overlay_ui::despawn_pause_ui,))
        .add_systems(OnEnter(GameState::GameOver), overlay_ui::spawn_game_over_ui)
        .add_systems(
            OnExit(GameState::GameOver),
            (
                overlay_ui::despawn_game_over_ui,
                game_state::cleanup_playing_entities,
                hud_ui::despawn_hud,
            ),
        )
        .add_systems(
            Update,
            enemy::move_enemy_tanks.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            enemy::enemy_spawn_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            enemy::animate_enemy_born_animation.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::animate_looping_sprite::<EnemyTank>.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::move_player_tank.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::handle_player_avatar_death.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::handle_recall_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::update_recall_timers.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::handle_dash_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::update_dash_movement.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::handle_dash_collision.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::handle_barrier_collision.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            player::update_recall_progress_bars.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::update_blue_bar_regen.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            bullet::enemy_shoot_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            bullet::player_shoot_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            bullet::bullet_bounds_check_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            bullet::bullet_terrain_collision_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            bullet::bullet_tank_collision_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            bullet::bullet_commander_collision_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            bullet::handle_effect_events.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            laser::player_laser_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::animate_explosion.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            laser::animate_laser.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::animate_forest_fire.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::animate_looping_sprite::<Forest>.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::animate_looping_sprite::<Sea>.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::animate_commander_music.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::play_sea_ambience.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::play_commander_music.run_if(in_state(GameState::Playing)),
        ) // 测试司令官音乐
        .add_systems(
            Update,
            game_state::play_tree_ambience.run_if(in_state(GameState::Playing)),
        ) // 测试森林环绕声
        .add_systems(
            Update,
            effects::animate_spark.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::handle_game_over_delay.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::check_game_over.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::check_stage_complete.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::handle_hud_stat_changed.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::animate_hud_text.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::update_enemy_count_display.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::update_commander_health_bar.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::update_player1_hud.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::update_player2_hud.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::handle_commander_death.run_if(in_state(GameState::Playing)),
        ) // 测试司令官阵亡处理
        .add_systems(
            Update,
            menus_ui::animate_start_screen.run_if(not(in_state(GameState::Playing))),
        )
        .add_systems(
            Update,
            (menus_ui::handle_start_screen_input, menus_ui::update_option_colors)
                .run_if(in_state(GameState::StartScreen)),
        )
        .add_systems(
            Update,
            overlay_ui::handle_game_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            overlay_ui::handle_pause_input.run_if(in_state(GameState::Paused)),
        )
        .add_systems(
            Update,
            (overlay_ui::handle_game_over_input, menus_ui::update_option_colors)
                .chain()
                .run_if(in_state(GameState::GameOver)),
        )
        .add_systems(
            Update,
            laser::handle_recoil_force.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::animate_smoke.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            laser::laser_collision_system.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            powerup::animate_powerup.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            powerup::handle_powerup_collision.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::update_air_cushion_effect.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            game_state::update_menu_blink.run_if(in_state(GameState::StartScreen)),
        )
        .add_systems(
            Update,
            game_state::update_menu_blink.run_if(in_state(GameState::FadingOut)),
        )
        .add_systems(
            Update,
            overlay_ui::fade_out_screen.run_if(in_state(GameState::FadingOut)),
        );
}

pub fn setup(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // 创建全局相机
    commands.spawn(Camera2d);

    // 初始化地形纹理图集布局
    let sea_texture_atlas_layout =
        TextureAtlasLayout::from_grid(UVec2::new(100, 100), 3, 1, None, None);
    let sea_texture_atlas_layout = texture_atlas_layouts.add(sea_texture_atlas_layout);
    let forest_texture_atlas_layout =
        TextureAtlasLayout::from_grid(UVec2::new(131, 131), 10, 1, None, None);
    let forest_texture_atlas_layout = texture_atlas_layouts.add(forest_texture_atlas_layout);
    commands.insert_resource(TerrainAtlasLayouts {
        sea: sea_texture_atlas_layout,
        forest: forest_texture_atlas_layout,
    });
}
