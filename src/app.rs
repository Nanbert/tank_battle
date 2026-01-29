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
use crate::commander;
use crate::dash;
use crate::effects;
use crate::enemy;
use crate::game_state;
use crate::hud_ui;
use crate::laser;
use crate::menus_ui;
use crate::map;
use crate::overlay_ui;
use crate::player;
use crate::powerup;

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
    app.insert_resource(ClearColor(COLOR_BLACK))
        .insert_resource(FontResources {
            cn: Handle::default(),
            en: Handle::default(),
        })
        .insert_resource(PlayerTankResources {
            player1: Handle::default(),
            player2: Handle::default(),
        })
        .insert_resource(CommanderResources {
            texture: Handle::default(),
            dead_texture: Handle::default(),
            avatar: Handle::default(),
            avatar_death: Handle::default(),
            avatar_commander_dead: Handle::default(),
        })
.insert_resource(BulletResources {
            bullet_player1: Handle::default(),
            bullet_player2: Handle::default(),
            bullet_enemy: Handle::default(),
            brick_hit_sound: Handle::default(),
            hit_sound: Handle::default(),
            metal_crash_sound: Handle::default(),
        })
        .insert_resource(EffectResources {
            explosion: Handle::default(),
            spark: Handle::default(),
            smoke: Handle::default(),
            bubble: Handle::default(),
            steel_hit: Handle::default(),
        })
        .insert_resource(GameMode::OnePlayer)
        .insert_resource(StageLevel(1))
        .insert_resource(CommanderLife {
            life_points: COMMANDER_LIFE_MAX,
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
        .init_resource::<StageIntroTimer>()
        .init_resource::<crate::levels::LevelAssets>();
}

pub fn register_game_systems(app: &mut App) {
    app.init_state::<GameState>()
        .add_message::<PlayerStatChanged>()
        .add_message::<crate::bullet::EffectEvent>()
        .init_resource::<BulletTracker>()
        .add_systems(
            Startup,
            (
                setup,
                crate::levels::load_level_assets,
                init_game_resources,
            )
        )
        .add_systems(
            OnEnter(GameState::StartScreen),
            (
                commander::despawn_commander,
                player::despawn_players,
                enemy::despawn_enemy_tank,
                map::despawn_map,
                powerup::despawn_powerups,
                |mut stage_level: ResMut<crate::resources::StageLevel>| {
                    stage_level.0 = 1;
                },
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
                player::init_players.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1,
                ),
                map::spawn_map,
                powerup::despawn_powerups,
                powerup::spawn_power_ups_air_cushion.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1,
                ),
                powerup::spawn_power_ups_random.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 > 1,
                ),
                hud_ui::spawn_hud.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1,
                ),
                hud_ui::update_stage_text,
                player::reset_player_positions,
                enemy::reset_enemy_spawn_state,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(GameState::StageIntro),
            (
                commander::spawn_commander.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1,
                ),
                overlay_ui::spawn_stage_intro,
            )
                .chain(),
        )
        .add_systems(
            Update,
            overlay_ui::handle_stage_intro_timer.run_if(in_state(GameState::StageIntro)),
        )
        .add_systems(OnExit(GameState::StageIntro), overlay_ui::despawn_stage_intro)
        .add_systems(OnEnter(GameState::Paused), overlay_ui::spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), (overlay_ui::despawn_pause_ui,))
        .add_systems(OnEnter(GameState::GameOver), overlay_ui::spawn_game_over_ui)
        .add_systems(
            OnExit(GameState::GameOver),
            (
                overlay_ui::despawn_game_over_ui,
                enemy::despawn_enemy_tank,
                |mut stage_level: ResMut<crate::resources::StageLevel>| {
                    stage_level.0 = 1;
                },
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
            hud_ui::animate_player_avatar.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::handle_player_avatar_death.run_if(in_state(GameState::Playing)),
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
            dash::handle_dash_input.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            dash::update_dash_movement.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            dash::handle_dash_collision.run_if(in_state(GameState::Playing)),
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
            player::recover_energy.run_if(in_state(GameState::Playing)),
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
            commander::animate_commander.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::animate_looping_sprite::<CommanderMusicAnimation>.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::play_sea_ambience.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            effects::play_commander_ambience.run_if(in_state(GameState::Playing)),
        ) // 测试司令官音乐
        .add_systems(
            Update,
            effects::play_tree_ambience.run_if(in_state(GameState::Playing)),
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
            hud_ui::update_player_hud.run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            hud_ui::handle_commander_death.run_if(in_state(GameState::Playing)),
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
            effects::update_air_cushion_effect.run_if(in_state(GameState::Playing)),
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

pub fn setup(mut commands: Commands) {
    // 创建全局相机
    commands.spawn(Camera2d);
}

/// 初始化游戏资源
/// 预加载常用的字体、纹理和音效，避免运行时重复加载
pub fn init_game_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // 地形纹理图集布局
    commands.insert_resource(TerrainAtlasLayouts {
        sea: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(100, 100),
            3,
            1,
            None,
            None,
        )),
        forest: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(131, 131),
            10,
            1,
            None,
            None,
        )),
        forest_fire: texture_atlas_layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(FOREST_FIRE_TILE_SIZE as u32, FOREST_FIRE_TILE_SIZE as u32),
            10,
            1,
            None,
            None,
        )),
    });

    // 字体资源
    commands.insert_resource(FontResources {
        cn: asset_server.load(FONT_CN),
        en: asset_server.load(FONT_EN),
    });

    // 玩家坦克资源
    commands.insert_resource(PlayerTankResources {
        player1: asset_server.load(TEXTURE_PLAYER_TANK1),
        player2: asset_server.load(TEXTURE_PLAYER_TANK2),
    });

    // 司令官资源
    commands.insert_resource(CommanderResources {
        texture: asset_server.load(TEXTURE_COMMANDER),
        dead_texture: asset_server.load(TEXTURE_COMMANDER_DEAD),
        avatar: asset_server.load(TEXTURE_AVATAR),
        avatar_death: asset_server.load(TEXTURE_AVATAR_DEATH),
        avatar_commander_dead: asset_server.load(TEXTURE_AVATAR_COMMANDER_DEAD),
    });

    // 音效资源
    commands.insert_resource(SoundResources {
        explosion: asset_server.load(SOUND_EXPLOSION),
        brick_hit: asset_server.load(SOUND_BRICK_HIT),
        hit: asset_server.load(SOUND_HIT),
        metal_crash: asset_server.load(SOUND_METAL_CRASH),
        laser_charge: asset_server.load(SOUND_LASER_CHARGE),
        laser: asset_server.load(SOUND_LASER),
        commander_get_shot: asset_server.load(SOUND_COMMANDER_GET_SHOT),
        commander_death: asset_server.load(SOUND_COMMANDER_DEATH),
    });

    // 特效纹理资源
    commands.insert_resource(EffectResources {
        explosion: asset_server.load(TEXTURE_EXPLOSION),
        spark: asset_server.load(TEXTURE_STEEL_HIT),
        smoke: asset_server.load(TEXTURE_SMOKE),
        bubble: asset_server.load(TEXTURE_BUBBLE),
        steel_hit: asset_server.load(TEXTURE_STEEL_HIT),
    });

    // 子弹资源
    commands.insert_resource(BulletResources {
        bullet_player1: asset_server.load("texture/bullets/bullet_player1.png"),
        bullet_player2: asset_server.load("texture/bullets/bullet_player2.png"),
        bullet_enemy: asset_server.load("texture/bullets/bullet_enemy.png"),
        brick_hit_sound: asset_server.load(SOUND_BRICK_HIT),
        hit_sound: asset_server.load(SOUND_HIT),
        metal_crash_sound: asset_server.load(SOUND_METAL_CRASH),
    });
}
