//! 应用配置模块
//!
//! 处理窗口配置、资源配置、游戏初始化等

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

use crate::constants::*;
use crate::resources::*;
use crate::utils;

/// 游戏系统调度集
/// 将系统按功能分组，减少 run_if 检查的开销
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameSystemSet {
    /// 敌方坦克系统集
    EnemySystems,
    /// 玩家坦克系统集
    PlayerSystems,
    /// 子弹系统集
    BulletSystems,
    /// 激光系统集
    LaserSystems,
    /// 特效和动画系统集
    EffectsAndAnimationSystems,
    /// 司令官系统集
    CommanderSystems,
    /// HUD UI 系统集
    HudSystems,
    /// 道具系统集
    PowerUpSystems,
    /// 游戏状态管理系统集
    GameStateSystems,
    /// 环境音效系统集
    AmbienceSystems,
}

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
            title: "Steel Command".to_string(),
            name: Some("steel_command".to_string()),
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
    // 检查是否从系统安装位置运行
    let asset_path = if std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .as_deref()
        == Some(std::path::Path::new("/usr/bin"))
    {
        // 系统安装位置：资源在 /usr/share/tank-battle/assets
        "/usr/share/tank-battle/assets".to_string()
    } else {
        // 开发环境或从压缩包运行：资源在当前目录的 assets
        "assets".to_string()
    };

    AssetPlugin {
        file_path: asset_path,
        ..default()
    }
}

pub fn configure_game_resources(app: &mut App) {
    app.insert_resource(ClearColor(crate::constants::COLOR_GRAY))
        .insert_resource(FontResources {
            cn: Handle::default(),
            en: Handle::default(),
        })
        .insert_resource(PlayerTankResources {
            player1: Handle::default(),
            player2: Handle::default(),
            single_barrel: Handle::default(),
            double_barrel: Handle::default(),
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
            bullet_fire_effect: Handle::default(),
            bullet_penetrate_effect: Handle::default(),
        })
        .insert_resource(EffectAtlasLayouts {
            fire_effect: Handle::default(),
            penetrate_effect: Handle::default(),
        })
        .insert_resource(EffectResources {
            explosion: Handle::default(),
            spark: Handle::default(),
            smoke: Handle::default(),
            bubble: Handle::default(),
            energy_blue_ball: Handle::default(),
            energy_red_ball: Handle::default(),
            forest_fire: Handle::default(),
        })
        .insert_resource(MapResources {
            brick: Handle::default(),
            steel: Handle::default(),
            tree: Handle::default(),
            sea: Handle::default(),
            barrier: Handle::default(),
        })
        .insert_resource(EnemyResources {
            enemy_born: Handle::default(),
            enemy_tank: Handle::default(),
        })
        .insert_resource(PowerUpResources {
            speed_up: Handle::default(),
            protection: Handle::default(),
            fire_speed: Handle::default(),
            fire_shell: Handle::default(),
            track_chain: Handle::default(),
            track_train: Handle::default(),
            penetrate: Handle::default(),
            repair: Handle::default(),
            hamburger: Handle::default(),
            air_cushion: Handle::default(),
            shell: Handle::default(),
        })
        .insert_resource(LaserResources {
            laser_blue: Handle::default(),
            laser_red: Handle::default(),
        })
        .insert_resource(MenuResources {
            background: Handle::default(),
        })
        .insert_resource(CommanderMusicResources {
            music_note: Handle::default(),
        })
        .insert_resource(Language::default())
        .insert_resource(AmbienceResources {
            burn_tree: Handle::default(),
            sea_ambience: Handle::default(),
            commander_music_000: Handle::default(),
            commander_music_001: Handle::default(),
            commander_music_002: Handle::default(),
            commander_music_003: Handle::default(),
            tree_ambience: Handle::default(),
        })
        .insert_resource(GameMode::OnePlayer)
        .insert_resource(StageLevel(1))
        .insert_resource(CommanderLife {
            life_points: COMMANDER_LIFE_MAX,
        })
        .insert_resource(PlayerInfo::default())
        .insert_resource(EnemySpawnState::default())
        .init_resource::<EnemyCollisionCache>()
        .insert_resource(RecallTimers::default())
        .insert_resource(DashTimers::default())
        .insert_resource(DashDamageTracker::default())
        .insert_resource(BarrierDamageTracker::default())
        .insert_resource(InsufficientEnergyTracker::default())
        .init_resource::<BlueBarRegenTimer>()
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
        // ==================== StartScreen 状态系统 ====================
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
                |mut clear_color: ResMut<ClearColor>| {
                    clear_color.0 = crate::constants::COLOR_GRAY;
                },
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                // 清理游戏实体
                |mut commands: Commands, entities: Query<Entity, With<crate::constants::PlayingEntity>>| {
                    for entity in entities.iter() {
                        let () = commands.entity(entity).try_despawn();
                    }
                },
                // 生成开始菜单
                menus_ui::spawn_start_screen.run_if(
                    |query: Query<(), With<crate::constants::StartScreenUI>>| {
                        query.is_empty()
                    }
                ),
                // 语言变化时重新生成菜单
                (
                    menus_ui::cleanup_start_screen_ui,
                    menus_ui::spawn_start_screen,
                )
                    .chain()
                    .run_if(|language: Res<crate::resources::Language>| language.is_changed()),
                // 处理菜单输入
                menus_ui::handle_start_screen_input,
                menus_ui::update_option_colors,
                menus_ui::animate_start_screen_background,
                game_state::update_menu_blink,
            )
                .run_if(in_state(GameState::StartScreen)),
        )
        // ==================== About 和 Credits 状态系统 ====================
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
        // ==================== StageIntro 状态系统 ====================
        .add_systems(
            OnEnter(GameState::StageIntro),
            (
                player::init_players.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1,
                ),
                map::spawn_map,
                powerup::despawn_powerups,
                powerup::spawn_test_powerup_stage1.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1,
                ),
                powerup::spawn_power_ups_random.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 > 1,
                ),
                hud_ui::spawn_hud.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1,
                ),
                hud_ui::update_stage_text.run_if(
                    |stage_level: Res<crate::resources::StageLevel>| stage_level.0 > 1,
                ),
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
            (
                game_state::cleanup_effects_and_bullets,
                game_state::cleanup_trackers_and_timers,
                overlay_ui::handle_stage_intro_timer,
            )
                .run_if(in_state(GameState::StageIntro)),
        )
        .add_systems(OnExit(GameState::StageIntro), overlay_ui::despawn_stage_intro)
        // ==================== Paused 状态系统 ====================
        .add_systems(OnEnter(GameState::Paused), overlay_ui::spawn_pause_ui)
        .add_systems(OnExit(GameState::Paused), 
            (overlay_ui::despawn_pause_ui, overlay_ui::despawn_insufficient_energy_warnings).chain())
        .add_systems(Update, overlay_ui::handle_pause_input.run_if(in_state(GameState::Paused)))
        // ==================== GameOver 状态系统 ====================
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
            (overlay_ui::handle_game_over_input, menus_ui::update_option_colors)
                .chain()
                .run_if(in_state(GameState::GameOver)),
        )
        // ==================== FadingOut 状态系统 ====================
        .add_systems(
            Update,
            (
                game_state::update_menu_blink,
                overlay_ui::fade_out_screen,
            )
                .run_if(in_state(GameState::FadingOut)),
        )
        // ==================== Playing 状态系统 ====================
        // 使用 SystemSet 组织系统，只在 Playing 状态时执行
        // 通过配置调度集来减少 run_if 检查
        .configure_sets(
            Update,
            (
                GameSystemSet::EnemySystems,
                GameSystemSet::PlayerSystems,
                GameSystemSet::BulletSystems,
                GameSystemSet::LaserSystems,
                GameSystemSet::EffectsAndAnimationSystems,
                GameSystemSet::CommanderSystems,
                GameSystemSet::HudSystems,
                GameSystemSet::PowerUpSystems,
                GameSystemSet::GameStateSystems,
                GameSystemSet::AmbienceSystems,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            // 敌方坦克系统集
            (
                enemy::collect_enemy_collisions,
                enemy::collect_contact_forces,
                enemy::move_enemy_tanks,
            )
                .chain()
                .in_set(GameSystemSet::EnemySystems),
        )
        .add_systems(
            Update,
            (
                enemy::enemy_spawn_system,
                enemy::animate_enemy_born_animation,
            )
                .in_set(GameSystemSet::EnemySystems),
        )
        .add_systems(
            Update,
            (
                // 玩家坦克系统集
                player::move_player_tank,
                player::handle_recall_input,
                player::update_recall_timers,
                dash::handle_dash_input,
                dash::update_dash_movement,
                dash::handle_dash_collision,
                player::handle_barrier_collision,
                player::update_recall_progress_bars,
                player::recover_energy,
                player::update_barrel_system,
                player::handle_barrel_recoil_force,
            )
                .in_set(GameSystemSet::PlayerSystems),
        )
        .add_systems(
            Update,
            (
                // 子弹系统集
                bullet::enemy_shoot_system,
                bullet::player_shoot_system,
                bullet::bullet_bounds_check_system,
                bullet::bullet_terrain_collision_system,
                bullet::bullet_tank_collision_system,
                bullet::bullet_commander_collision_system,
                bullet::handle_effect_events,
            )
                .in_set(GameSystemSet::BulletSystems),
        )
        .add_systems(
            Update,
            (
                // 激光系统集
                laser::player_laser_system,
                laser::animate_laser,
                laser::animate_energy_ball,
                laser::handle_recoil_force,
            )
                .in_set(GameSystemSet::LaserSystems),
        )
        .add_systems(
            Update,
            (
                // 特效和动画系统集
                effects::animate_looping_sprite,
                effects::animate_one_shot_animations,
                effects::update_air_cushion_effect,
            )
                .in_set(GameSystemSet::EffectsAndAnimationSystems),
        )
        .add_systems(
            Update,
            // 司令官系统集
            commander::animate_commander.in_set(GameSystemSet::CommanderSystems),
        )
        .add_systems(
            Update,
            (
                // HUD UI 系统集
                hud_ui::animate_player_avatar,
                hud_ui::handle_player_avatar_death,
                hud_ui::handle_hud_stat_changed,
                hud_ui::animate_hud_text,
                hud_ui::update_enemy_count_display
                    .run_if(resource_changed::<crate::resources::EnemySpawnState>),
                hud_ui::update_commander_health_bar
                    .run_if(resource_changed::<CommanderLife>),
                hud_ui::update_player_hud
                    .run_if(resource_changed::<PlayerInfo>),
                hud_ui::handle_commander_death,
            )
                .in_set(GameSystemSet::HudSystems),
        )
        .add_systems(
            Update,
            (
                // 道具系统集
                powerup::animate_powerup,
                powerup::handle_powerup_collision,
                powerup::update_track_chain_effect,
                powerup::animate_track_chain,
            )
                .in_set(GameSystemSet::PowerUpSystems),
        )
        .add_systems(
            Update,
            (
                // 游戏状态管理系统集
                game_state::handle_game_over_delay,
                game_state::check_game_over,
                game_state::check_stage_complete,
            )
                .in_set(GameSystemSet::GameStateSystems),
        )
        .add_systems(
            Update,
            (
                // 环境音效系统集
                effects::play_sea_ambience,
                effects::play_commander_ambience,
                effects::play_tree_ambience,
            )
                .in_set(GameSystemSet::AmbienceSystems),
        )
        .add_systems(
            Update,
            // 其他系统（不属于特定集的 Playing 状态系统）
            (
                overlay_ui::handle_game_input,
                overlay_ui::update_insufficient_energy_warnings,
            )
                .run_if(in_state(GameState::Playing)),
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
    let sea_tile_size = UVec2::new(100, 100);
    let forest_tile_size = UVec2::new(131, 131);
    let forest_fire_tile_size = UVec2::new(FOREST_FIRE_TILE_SIZE as u32, FOREST_FIRE_TILE_SIZE as u32);

    commands.insert_resource(TerrainAtlasLayouts {
        sea: utils::add_texture_atlas(&mut texture_atlas_layouts, sea_tile_size, 3, 1),
        forest: utils::add_texture_atlas(&mut texture_atlas_layouts, forest_tile_size, 10, 1),
        forest_fire: utils::add_texture_atlas(&mut texture_atlas_layouts, forest_fire_tile_size, 10, 1),
    });

    // 背景纹理图集布局
    let background_atlas = utils::add_texture_atlas(
        &mut texture_atlas_layouts,
        UVec2::new(BACKGROUND_TILE_WIDTH as u32, BACKGROUND_TILE_HEIGHT as u32),
        BACKGROUND_COLUMNS as u32,
        BACKGROUND_ROWS as u32,
    );

    commands.insert_resource(BackgroundAtlasLayout {
        layout: background_atlas,
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
        single_barrel: asset_server.load(TEXTURE_SINGLE_BARREL),
        double_barrel: asset_server.load(TEXTURE_DOUBLE_BARREL),
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
        player_shot: asset_server.load(SOUND_PLAYER_SHOT),
    });

    // 特效纹理资源
    commands.insert_resource(EffectResources {
        explosion: asset_server.load(TEXTURE_EXPLOSION),
        spark: asset_server.load(TEXTURE_STEEL_HIT),
        smoke: asset_server.load(TEXTURE_SMOKE),
        bubble: asset_server.load(TEXTURE_BUBBLE),
        energy_blue_ball: asset_server.load(TEXTURE_ENERGY_BLUE_BALL),
        energy_red_ball: asset_server.load(TEXTURE_ENERGY_RED_BALL),
        forest_fire: asset_server.load(TEXTURE_TREE_FIRE_SHEET),
    });

    // 子弹资源
    commands.insert_resource(BulletResources {
        bullet_player1: asset_server.load(TEXTURE_BULLET_PLAYER1),
        bullet_player2: asset_server.load(TEXTURE_BULLET_PLAYER2),
        bullet_enemy: asset_server.load(TEXTURE_BULLET_ENEMY),
        bullet_fire_effect: asset_server.load(TEXTURE_BULLET_FIRE_EFFECT),
        bullet_penetrate_effect: asset_server.load("texture/bullets/penetrate_effect.png"),
    });

    // 预加载子弹特效纹理图集布局（避免每次发射子弹时重复创建）
    let fire_effect_tile_size = UVec2::new(FIRE_EFFECT_TILE_WIDTH as u32, FIRE_EFFECT_TILE_HEIGHT as u32);
    let fire_effect_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, fire_effect_tile_size, FIRE_EFFECT_COLUMNS as u32, FIRE_EFFECT_ROWS as u32);

    let penetrate_effect_tile_size = UVec2::new(PENETRATE_EFFECT_TILE_WIDTH as u32, PENETRATE_EFFECT_TILE_HEIGHT as u32);
    let penetrate_effect_atlas = utils::add_texture_atlas(&mut texture_atlas_layouts, penetrate_effect_tile_size, PENETRATE_EFFECT_COLUMNS as u32, PENETRATE_EFFECT_ROWS as u32);

    commands.insert_resource(EffectAtlasLayouts {
        fire_effect: fire_effect_atlas,
        penetrate_effect: penetrate_effect_atlas,
    });

    // 地图纹理资源
    commands.insert_resource(MapResources {
        brick: asset_server.load(TEXTURE_BRICK),
        steel: asset_server.load(TEXTURE_STEEL),
        tree: asset_server.load(TEXTURE_TREE),
        sea: asset_server.load(TEXTURE_SEA),
        barrier: asset_server.load(TEXTURE_BARRIER),
    });

    // 敌方坦克资源
    commands.insert_resource(EnemyResources {
        enemy_born: asset_server.load(TEXTURE_ENEMY_BORN),
        enemy_tank: asset_server.load(TEXTURE_ENEMY_TANK1),
    });

    // 道具纹理资源
    commands.insert_resource(PowerUpResources {
        speed_up: asset_server.load("power_up/speed_up.png"),
        protection: asset_server.load("power_up/protection.png"),
        fire_speed: asset_server.load("power_up/fire_speed.png"),
        fire_shell: asset_server.load("power_up/fire_shell.png"),
        track_chain: asset_server.load("power_up/track_chain.png"),
        track_train: asset_server.load("texture/track_train.png"),
        penetrate: asset_server.load("power_up/penetrate.png"),
        repair: asset_server.load("power_up/repair.png"),
        hamburger: asset_server.load("power_up/hamburger.png"),
        air_cushion: asset_server.load("power_up/air_cushion.png"),
        shell: asset_server.load("power_up/shell.png"),
    });

    // 激光纹理资源
    commands.insert_resource(LaserResources {
        laser_blue: asset_server.load(TEXTURE_LASER_BLUE),
        laser_red: asset_server.load(TEXTURE_LASER_RED),
    });

    // 菜单背景纹理资源
    commands.insert_resource(MenuResources {
        background: asset_server.load(TEXTURE_BACKGROUND),
    });

    // 司令官音乐纹理资源
    commands.insert_resource(CommanderMusicResources {
        music_note: asset_server.load(TEXTURE_MUSIC_NOTE),
    });

    // 环境音效资源
    commands.insert_resource(AmbienceResources {
        burn_tree: asset_server.load(SOUND_BURN_TREE),
        sea_ambience: asset_server.load(SOUND_SEA_AMBIENCE),
        commander_music_000: asset_server.load(SOUND_COMMANDER_MUSIC_000),
        commander_music_001: asset_server.load(SOUND_COMMANDER_MUSIC_001),
        commander_music_002: asset_server.load(SOUND_COMMANDER_MUSIC_002),
        commander_music_003: asset_server.load(SOUND_COMMANDER_MUSIC_003),
        tree_ambience: asset_server.load(SOUND_TREE_AMBIENCE),
    });
}
