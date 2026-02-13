//! 应用配置模块
//!
//! 处理窗口配置、资源配置、游戏初始化等

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};

use crate::constants::*;
use crate::resources::*;
use crate::weather;

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

/// 注册 StartScreen 状态的系统
fn register_start_screen_systems(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::StartScreen),
        (
            commander::despawn_commander,
            player::despawn_players,
            enemy::despawn_enemy_tank,
            map::despawn_map,
            ui::overlay::despawn_powerups,
            |mut stage_level: ResMut<crate::resources::StageLevel>| {
                stage_level.0 = 1;
            },
            ui::hud::despawn_hud,
            game_state::reset_fading_out,
            |mut clear_color: ResMut<ClearColor>| {
                clear_color.0 = crate::ui::COLOR_GRAY;
            },
        )
            .chain(),
    )
    .add_systems(
        Update,
        (
            // 清理游戏实体
            |mut commands: Commands, entities: Query<Entity, With<crate::ui::PlayingEntity>>| {
                crate::utils::cleanup_entities(&mut commands, entities.iter());
            },
            // 生成开始菜单
            ui::menus::spawn_start_screen
                .run_if(|query: Query<(), With<crate::ui::StartScreenUI>>| query.is_empty()),
            // 语言变化时重新生成菜单
            (
                ui::overlay::despawn_start_screen_ui,
                ui::menus::spawn_start_screen,
            )
                .chain()
                .run_if(|language: Res<crate::resources::Language>| language.is_changed()),
            // 处理菜单输入
            ui::menus::handle_start_screen_input,
            ui::menus::update_option_colors,
            game_state::update_menu_blink,
            effects::animate_effects,
        )
            .run_if(in_state(GameState::StartScreen)),
    );
}

/// 注册 About 和 Credits 状态的系统
fn register_about_credits_systems(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::About),
        (
            ui::overlay::despawn_start_screen_ui,
            ui::menus::spawn_about_screen,
        )
            .chain(),
    )
    .add_systems(
        OnExit(GameState::About),
        (
            ui::overlay::despawn_about_screen,
            ui::menus::spawn_start_screen,
        )
            .chain(),
    )
    .add_systems(
        Update,
        ui::menus::handle_about_input.run_if(in_state(GameState::About)),
    )
    .add_systems(
        OnEnter(GameState::Credits),
        (
            ui::overlay::despawn_start_screen_ui,
            ui::menus::spawn_credits_screen,
        )
            .chain(),
    )
    .add_systems(
        OnExit(GameState::Credits),
        (
            ui::overlay::despawn_credits_screen,
            ui::menus::spawn_start_screen,
        )
            .chain(),
    )
    .add_systems(
        Update,
        ui::menus::handle_credits_input.run_if(in_state(GameState::Credits)),
    );
}

/// 注册 StageIntro 状态的系统
fn register_stage_intro_systems(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::StageIntro),
        (
            // 先设置天气（在生成地图之前）
            weather::on_playing_enter,
            player::spawn_players
                .run_if(|stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1),
            map::spawn_map,
            ui::overlay::despawn_powerups,
            ui::hud::spawn_hud
                .run_if(|stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1),
            ui::hud::update_stage_text
                .run_if(|stage_level: Res<crate::resources::StageLevel>| stage_level.0 > 1),
            player::reset_player_positions,
            enemy::reset_enemy_spawn_state,
        )
            .chain(),
    )
    .add_systems(
        OnEnter(GameState::StageIntro),
        (
            commander::spawn_commander
                .run_if(|stage_level: Res<crate::resources::StageLevel>| stage_level.0 == 1),
            ui::overlay::spawn_stage_intro,
        )
            .chain(),
    )
    .add_systems(
                Update,
                (
                    game_state::cleanup_all_game_state,
                    ui::overlay::handle_stage_intro_timer,
                )
                    .run_if(in_state(GameState::StageIntro)),
            )    .add_systems(
        OnExit(GameState::StageIntro),
        ui::overlay::despawn_stage_intro,
    );
}

/// 注册 Paused 状态的系统
fn register_paused_systems(app: &mut App) {
    app.add_systems(OnEnter(GameState::Paused), ui::overlay::spawn_pause_ui)
        .add_systems(
            OnExit(GameState::Paused),
            (
                ui::overlay::despawn_pause_ui,
                ui::overlay::despawn_insufficient_energy_warnings,
            )
                .chain(),
        )
        .add_systems(
            Update,
            ui::overlay::handle_pause_input.run_if(in_state(GameState::Paused)),
        );
}

/// 注册 GameOver 状态的系统
fn register_game_over_systems(app: &mut App) {
    app.add_systems(
        OnEnter(GameState::GameOver),
        ui::overlay::spawn_game_over_ui,
    )
    .add_systems(
        OnExit(GameState::GameOver),
        (
            ui::overlay::despawn_game_over_ui,
            enemy::despawn_enemy_tank,
            |mut stage_level: ResMut<crate::resources::StageLevel>| {
                stage_level.0 = 1;
            },
            ui::hud::despawn_hud,
        ),
    )
    .add_systems(
        Update,
        (
            ui::overlay::handle_game_over_input,
            ui::menus::update_option_colors,
        )
            .chain()
            .run_if(in_state(GameState::GameOver)),
    );
}

/// 注册 FadingOut 状态的系统
fn register_fading_out_systems(app: &mut App) {
    app.add_systems(
        Update,
        (game_state::update_menu_blink, ui::overlay::fade_out_screen)
            .run_if(in_state(GameState::FadingOut)),
    );
}

/// 注册 Playing 状态的系统
fn register_playing_systems(app: &mut App) {
    // 配置系统调度集
    app.configure_sets(
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
    );

    // 敌方坦克系统集
    app.add_systems(
        Update,
        (
            enemy::collect_enemy_collisions,
            enemy::collect_contact_forces,
            enemy::enemy_fire_spread_system,
            enemy::move_enemy_tanks,
            enemy::update_enemy_life_dots,
            enemy::enemy_burning_effect_system,
        )
            .chain()
            .in_set(GameSystemSet::EnemySystems),
    )
    .add_systems(
        Update,
        (enemy::enemy_spawn_system, enemy::handle_spawn_enemy_event)
            .in_set(GameSystemSet::EnemySystems),
    );

    // 玩家坦克系统集
app.add_systems(
        Update,
        (
            player::move_player_tank.in_set(GameSystemSet::PlayerSystems),
            player::handle_recall_input.in_set(GameSystemSet::PlayerSystems),
            player::update_recall_timers.in_set(GameSystemSet::PlayerSystems),
            dash::handle_dash_input.in_set(GameSystemSet::PlayerSystems),
            dash::update_dash_movement.in_set(GameSystemSet::PlayerSystems),
            dash::handle_dash_collision.in_set(GameSystemSet::PlayerSystems),
            player::handle_barrier_collision.in_set(GameSystemSet::PlayerSystems),
            player::update_recall_progress_bars.in_set(GameSystemSet::PlayerSystems),
            player::recover_energy.in_set(GameSystemSet::PlayerSystems),
            player::update_barrel_system.in_set(GameSystemSet::PlayerSystems),
            player::handle_barrel_recoil_force.in_set(GameSystemSet::PlayerSystems),
            powerup::update_low_health_smoke_effects.in_set(GameSystemSet::PlayerSystems),
        ),
    );

    // 子弹系统集
    app.add_systems(
        Update,
        (
            bullet::enemy_shoot_system.in_set(GameSystemSet::BulletSystems),
            bullet::player_shoot_system.in_set(GameSystemSet::BulletSystems),
            bullet::bullet_bounds_check_system.in_set(GameSystemSet::BulletSystems),
            bullet::bullet_terrain_collision_system.in_set(GameSystemSet::BulletSystems),
            bullet::bullet_tank_collision_system.in_set(GameSystemSet::BulletSystems),
            bullet::bullet_commander_collision_system.in_set(GameSystemSet::BulletSystems),
            bullet::handle_effect_events.in_set(GameSystemSet::BulletSystems),
        ),
    )
    .add_message::<bullet::ComboEvent>();

    // 连击系统
    app.add_systems(
        Update,
        (
            bullet::handle_combo_events,
            bullet::update_combo_system,
        ),
    );

    // 激光系统集
    app.add_systems(
        Update,
        (
            laser::player_laser_system,
            laser::handle_laser_end_events,
            laser::handle_camera_shake,
        )
            .in_set(GameSystemSet::LaserSystems),
    );

    // 特效和动画系统集
    app.add_systems(
        Update,
        (
            effects::animate_effects,
            effects::update_air_cushion_effect,
            weather::precipitation_spawn_system,
            weather::precipitation_update_system,
            ui::common::update_blink_animations, // 通用闪烁动画系统
        )
            .in_set(GameSystemSet::EffectsAndAnimationSystems),
    );

    // 天气系统：离开 Playing 状态时清理
    app.add_systems(OnExit(GameState::Playing), (
            weather::on_playing_exit,
            crate::ambience::cleanup_leaves,
        ).chain());

    // 司令官系统集
    app.add_systems(
        Update,
        commander::animate_commander.in_set(GameSystemSet::CommanderSystems),
    );

    // HUD UI 系统集
    app.add_systems(
        Update,
        (
            ui::hud::update::update_bar_animations,
            ui::hud::update::handle_player_avatar_death,
            ui::hud::blink::handle_hud_stat_changed,
            ui::hud::blink::handle_hud_stat_max_value,
            ui::hud::update::update_enemy_count_text
                .run_if(resource_changed::<crate::resources::EnemySpawnState>),
            ui::hud::update::update_commander_health_bar.run_if(resource_changed::<CommanderLife>),
            ui::hud::update::update_player_hud.run_if(resource_changed::<PlayerInfo>),
            ui::hud::update::handle_commander_death,
        )
            .in_set(GameSystemSet::HudSystems),
    );

    // 道具系统集
    app.add_systems(
        Update,
        (
            powerup::handle_powerup_collision,
            powerup::update_track_chain_effect,
            ui::overlay::update_floating_texts,
        )
            .in_set(GameSystemSet::PowerUpSystems),
    );

    // 游戏状态管理系统集
    app.add_systems(
        Update,
        (
            game_state::handle_game_over_delay,
            game_state::check_game_over,
            game_state::check_stage_complete,
        )
            .in_set(GameSystemSet::GameStateSystems),
    );

    // 环境音效系统集
    app.add_systems(
        Update,
        (
            effects::play_sea_ambience,
            effects::play_commander_ambience,
            effects::play_tree_ambience,
            effects::play_bubble_ambience,
            effects::spawn_sea_bubbles,
            effects::animate_sea_bubbles,
            crate::ambience::rain_splash_spawn_system,
            crate::ambience::rain_splash_update_system,
            crate::ambience::leaves_spawn_system,
            crate::ambience::leaves_update_system,
            crate::weather::play_rain_ambience,
        )
            .in_set(GameSystemSet::AmbienceSystems),
    );

    // 其他系统（不属于特定集的 Playing 状态系统）
    app.add_systems(
        Update,
        ui::overlay::handle_game_input.run_if(in_state(GameState::Playing)),
    );
}

// 导入模块以便使用其函数
use crate::bullet;
use crate::commander;
use crate::dash;
use crate::effects;
use crate::enemy;
use crate::game_state;
use crate::laser;
use crate::map;
use crate::player;
use crate::powerup;
use crate::ui;

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
    app.insert_resource(ClearColor(crate::ui::COLOR_GRAY))
        .insert_resource(crate::resources::GameTextureResources {
            cn: Handle::default(),
            en: Handle::default(),
            player1: Handle::default(),
            player2: Handle::default(),
            single_barrel: Handle::default(),
            double_barrel: Handle::default(),
            commander: Handle::default(),
            commander_dead: Handle::default(),
            avatar: Handle::default(),
            avatar_death: Handle::default(),
            avatar_commander_dead: Handle::default(),
            bullet_player1: Handle::default(),
            bullet_player2: Handle::default(),
            bullet_enemy: Handle::default(),
            bullet_fire_effect: Handle::default(),
            bullet_penetrate_effect: Handle::default(),
            explosion: Handle::default(),
            spark: Handle::default(),
            smoke: Handle::default(),
            bubble: Handle::default(),
            energy_blue_ball: Handle::default(),
            energy_red_ball: Handle::default(),
            forest_fire: Handle::default(),
            forest_fire_yellow: Handle::default(),
            laser_blue: Handle::default(),
            laser_red: Handle::default(),
            dash_dust_effect: Handle::default(),
            brick: Handle::default(),
            steel: Handle::default(),
            tree: Handle::default(),
            tree_yellow: Handle::default(),
            sea: Handle::default(),
            barrier: Handle::default(),
            enemy_born: Handle::default(),
            enemy_tank_normal: Handle::default(),
            enemy_tank_fire: Handle::default(),
            enemy_tank_heavy: Handle::default(),
            enemy_tank_light: Handle::default(),
            enemy_tank_burning: Handle::default(),
            speed_up_icon: Handle::default(),
            protection_icon: Handle::default(),
            fire_speed_icon: Handle::default(),
            fire_shell_icon: Handle::default(),
            track_chain_icon: Handle::default(),
            track_chain_effect: Handle::default(),
            tank_smoke_effect: Handle::default(),
            penetrate_icon: Handle::default(),
            repair_icon: Handle::default(),
            hamburger_icon: Handle::default(),
            air_cushion_icon: Handle::default(),
            shell_icon: Handle::default(),
            background: Handle::default(),
            music_note: Handle::default(),
            sea_bubble_texture: Handle::default(),
            leaves: Default::default(),
            leaves_yellow: Default::default(),
        })
        .insert_resource(crate::resources::GameAudioResources {
            explosion: Handle::default(),
            brick_hit: Handle::default(),
            hit: Handle::default(),
            metal_crash: Handle::default(),
            laser_charge: Handle::default(),
            laser: Handle::default(),
            powerup_sound: Handle::default(),
            commander_get_shot: Handle::default(),
            commander_death: Handle::default(),
            player_shot: Handle::default(),
            dash: Handle::default(),
            burn_tree: Handle::default(),
            sea_ambience: Handle::default(),
            bubble_ambience: Handle::default(),
            rain: Handle::default(),
            music_note_000: Handle::default(),
            music_note_001: Handle::default(),
            music_note_002: Handle::default(),
            music_note_003: Handle::default(),
            tree_ambience: Handle::default(),
        })
        .insert_resource(crate::resources::GameAtlasLayoutResources {
            sea: Handle::default(),
            forest: Handle::default(),
            forest_fire: Handle::default(),
            forest_yellow: Handle::default(),
            forest_fire_yellow: Handle::default(),
            background: Handle::default(),
            fire_effect: Handle::default(),
            penetrate_effect: Handle::default(),
            smoke_atlas: Handle::default(),
            explosion: Handle::default(),
            spark: Handle::default(),
            commander: Handle::default(),
            music_note: Handle::default(),
            sea_bubble: Handle::default(),
            player_avatar: Handle::default(),
            enemy_born: Handle::default(),
            enemy_tank_normal: Handle::default(),
            enemy_tank_fire: Handle::default(),
            enemy_tank_heavy: Handle::default(),
            enemy_tank_light: Handle::default(),
            enemy_tank_burning: Handle::default(),
            laser_blue: Handle::default(),
            laser_red: Handle::default(),
            energy_blue_ball: Handle::default(),
            energy_red_ball: Handle::default(),
            speed_up_icon: Handle::default(),
            protection_icon: Handle::default(),
            fire_speed_icon: Handle::default(),
            fire_shell_icon: Handle::default(),
            track_chain_icon: Handle::default(),
            track_chain_effect: Handle::default(),
            tank_smoke_effect: Handle::default(),
            dash_dust_effect: Handle::default(),
            penetrate_icon: Handle::default(),
            repair_icon: Handle::default(),
            hamburger_icon: Handle::default(),
            air_cushion_icon: Handle::default(),
            shell_icon: Handle::default(),
        })
        .insert_resource(Language::default())
        .insert_resource(GameMode::OnePlayer)
        .insert_resource(StageLevel(1))
        .insert_resource(CommanderLife {
            life_points: COMMANDER_LIFE_MAX,
        })
        .insert_resource(PlayerInfo::default())
        .insert_resource(EnemySpawnState::default())
        .insert_resource(FadingOut { alpha: 1.0 })
        .insert_resource(CurrentMenuSelection { selected_index: 0 })
        .insert_resource(AnimationIndices { first: 0, last: 14 })
        .insert_resource(CurrentAnimationFrame(0))
        .init_resource::<EnemyCollisionCache>()
        .init_resource::<GameTrackers>()
        .init_resource::<GameTimers>()
        .init_resource::<crate::levels::LevelAssets>()
        .init_resource::<weather::CurrentWeather>()
        .insert_resource(crate::resources::TreeColor::Green);
}

/// 注册所有游戏系统
///
/// 将系统按游戏状态分组注册，使用辅助函数提高可维护性
pub fn register_game_systems(app: &mut App) {
    app.init_state::<GameState>()
        .add_message::<crate::constants::LaserEndEvent>()
        .add_message::<PlayerStatChanged>()
        .add_message::<crate::bullet::EffectEvent>()
        .add_message::<crate::enemy::SpawnEnemyEvent>()
        .init_resource::<BulletTracker>()
        .init_resource::<ComboTracker>()
        .add_systems(
            Startup,
            (setup, crate::levels::load_level_assets, init_game_resources),
        );

    // 注册各游戏状态系统
    register_start_screen_systems(app);
    register_about_credits_systems(app);
    register_stage_intro_systems(app);
    register_paused_systems(app);
    register_game_over_systems(app);
    register_fading_out_systems(app);
    register_playing_systems(app);
}

pub fn setup(mut commands: Commands) {
    // 创建全局相机
    commands.spawn(Camera2d);
}

/// 初始化纹理资源
fn init_textures(asset_server: &AssetServer) -> crate::resources::GameTextureResources {
    crate::resources::GameTextureResources {
        // 字体
        cn: asset_server.load(FONT_CN),
        en: asset_server.load(FONT_EN),
        // 玩家坦克
        player1: crate::atlas::PLAYER_TANK1_ATLAS.load_texture(asset_server),
        player2: crate::atlas::PLAYER_TANK2_ATLAS.load_texture(asset_server),
        single_barrel: asset_server.load(TEXTURE_SINGLE_BARREL),
        double_barrel: asset_server.load(TEXTURE_DOUBLE_BARREL),
        // 司令官
        commander: crate::atlas::COMMANDER_ATLAS.load_texture(asset_server),
        commander_dead: asset_server.load(TEXTURE_COMMANDER_DEAD),
        avatar: crate::atlas::PLAYER_AVATAR_ATLAS.load_texture(asset_server),
        avatar_death: asset_server.load(TEXTURE_AVATAR_DEATH),
        avatar_commander_dead: asset_server.load(TEXTURE_AVATAR_COMMANDER_DEAD),
        // 子弹
        bullet_player1: asset_server.load(TEXTURE_BULLET_PLAYER1),
        bullet_player2: asset_server.load(TEXTURE_BULLET_PLAYER2),
        bullet_enemy: asset_server.load(TEXTURE_BULLET_ENEMY),
        bullet_fire_effect: crate::atlas::FIRE_EFFECT_ATLAS.load_texture(asset_server),
        bullet_penetrate_effect: crate::atlas::PENETRATE_EFFECT_ATLAS.load_texture(asset_server),
        // 特效
        explosion: crate::atlas::EXPLOSION_ATLAS.load_texture(asset_server),
        spark: crate::atlas::SPARK_ATLAS.load_texture(asset_server),
        smoke: crate::atlas::SMOKE_ATLAS.load_texture(asset_server),
        bubble: asset_server.load(TEXTURE_BUBBLE),
        energy_blue_ball: crate::atlas::ENERGY_BALL_BLUE_ATLAS.load_texture(asset_server),
        energy_red_ball: crate::atlas::ENERGY_BALL_RED_ATLAS.load_texture(asset_server),
        forest_fire: crate::atlas::FOREST_FIRE_ATLAS.load_texture(asset_server),
        forest_fire_yellow: crate::atlas::FOREST_FIRE_YELLOW_ATLAS.load_texture(asset_server),
        laser_blue: crate::atlas::LASER_BLUE_ATLAS.load_texture(asset_server),
        laser_red: crate::atlas::LASER_RED_ATLAS.load_texture(asset_server),
        // 地图
        brick: asset_server.load(TEXTURE_BRICK),
        steel: asset_server.load(TEXTURE_STEEL),
        tree: crate::atlas::FOREST_ATLAS.load_texture(asset_server),
        tree_yellow: crate::atlas::FOREST_YELLOW_ATLAS.load_texture(asset_server),
        sea: crate::atlas::SEA_ATLAS.load_texture(asset_server),
        barrier: asset_server.load(TEXTURE_BARRIER),
        // 敌方坦克
        enemy_born: crate::atlas::ENEMY_BORN_ATLAS.load_texture(asset_server),
        enemy_tank_normal: crate::atlas::ENEMY_TANK_NORMAL_ATLAS.load_texture(asset_server),
        enemy_tank_fire: crate::atlas::ENEMY_TANK_FIRE_ATLAS.load_texture(asset_server),
        enemy_tank_heavy: crate::atlas::ENEMY_TANK_HEAVY_ATLAS.load_texture(asset_server),
        enemy_tank_light: crate::atlas::ENEMY_TANK_LIGHT_ATLAS.load_texture(asset_server),
        enemy_tank_burning: crate::atlas::ENEMY_TANK_BURNING_ATLAS.load_texture(asset_server),
        // 道具
        speed_up_icon: crate::atlas::POWER_UP_SPEED_UP_ATLAS.load_texture(asset_server),
        protection_icon: crate::atlas::POWER_UP_PROTECTION_ATLAS.load_texture(asset_server),
        fire_speed_icon: crate::atlas::POWER_UP_FIRE_SPEED_ATLAS.load_texture(asset_server),
        fire_shell_icon: crate::atlas::POWER_UP_FIRE_SHELL_ATLAS.load_texture(asset_server),
        track_chain_icon: crate::atlas::POWER_UP_TRACK_CHAIN_ATLAS.load_texture(asset_server),
        track_chain_effect: crate::atlas::TRACK_CHAIN_ATLAS.load_texture(asset_server),
        tank_smoke_effect: crate::atlas::TANK_SMOKE_ATLAS.load_texture(asset_server),
        dash_dust_effect: crate::atlas::DASH_DUST_ATLAS.load_texture(asset_server),
        penetrate_icon: crate::atlas::POWER_UP_PENETRATE_ATLAS.load_texture(asset_server),
        repair_icon: crate::atlas::POWER_UP_REPAIR_ATLAS.load_texture(asset_server),
        hamburger_icon: crate::atlas::POWER_UP_HAMBURGER_ATLAS.load_texture(asset_server),
        air_cushion_icon: crate::atlas::POWER_UP_AIR_CUSHION_ATLAS.load_texture(asset_server),
        shell_icon: crate::atlas::POWER_UP_SHELL_ATLAS.load_texture(asset_server),
        // 菜单
        background: crate::atlas::BACKGROUND_ATLAS.load_texture(asset_server),
        music_note: crate::atlas::MUSIC_NOTE_ATLAS.load_texture(asset_server),
        sea_bubble_texture: asset_server.load(TEXTURE_SEA_BUBBLE),
        leaves: [
            asset_server.load(TEXTURE_LEAVES_1),
            asset_server.load(TEXTURE_LEAVES_2),
            asset_server.load(TEXTURE_LEAVES_3),
            asset_server.load(TEXTURE_LEAVES_4),
            asset_server.load(TEXTURE_LEAVES_5),
        ],
        leaves_yellow: [
            asset_server.load(TEXTURE_LEAVES_1_YELLOW),
            asset_server.load(TEXTURE_LEAVES_2_YELLOW),
            asset_server.load(TEXTURE_LEAVES_3_YELLOW),
            asset_server.load(TEXTURE_LEAVES_4_YELLOW),
            asset_server.load(TEXTURE_LEAVES_5_YELLOW),
        ],
    }
}

/// 初始化音频资源
fn init_audio(asset_server: &AssetServer) -> crate::resources::GameAudioResources {
    crate::resources::GameAudioResources {
        // 音效
        explosion: asset_server.load(SOUND_EXPLOSION),
        brick_hit: asset_server.load(SOUND_BRICK_HIT),
        hit: asset_server.load(SOUND_HIT),
        metal_crash: asset_server.load(SOUND_METAL_CRASH),
        laser_charge: asset_server.load(SOUND_LASER_CHARGE),
        laser: asset_server.load(SOUND_LASER),
        powerup_sound: asset_server.load(SOUND_POWERUP),
        commander_get_shot: asset_server.load(SOUND_COMMANDER_GET_SHOT),
        commander_death: asset_server.load(SOUND_COMMANDER_DEATH),
        player_shot: asset_server.load(SOUND_PLAYER_SHOT),
        dash: asset_server.load(SOUND_DASH),
        // 环境音效
        burn_tree: asset_server.load(SOUND_BURN_TREE),
        sea_ambience: asset_server.load(SOUND_SEA_AMBIENCE),
        bubble_ambience: asset_server.load(SOUND_BUBBLE_AMBIENCE),
        rain: asset_server.load(SOUND_RAIN),
        music_note_000: asset_server.load(SOUND_MUSIC_NOTE_000),
        music_note_001: asset_server.load(SOUND_MUSIC_NOTE_001),
        music_note_002: asset_server.load(SOUND_MUSIC_NOTE_002),
        music_note_003: asset_server.load(SOUND_MUSIC_NOTE_003),
        tree_ambience: asset_server.load(SOUND_TREE_AMBIENCE),
    }
}

/// 初始化图集布局资源
fn init_atlas_layouts(
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> crate::resources::GameAtlasLayoutResources {
    let background_atlas = crate::atlas::BACKGROUND_ATLAS.add_to_assets(texture_atlas_layouts);

    crate::resources::GameAtlasLayoutResources {
        // 地形
        sea: crate::atlas::SEA_ATLAS.add_to_assets(texture_atlas_layouts),
        forest: crate::atlas::FOREST_ATLAS.add_to_assets(texture_atlas_layouts),
        forest_fire: crate::atlas::FOREST_FIRE_ATLAS.add_to_assets(texture_atlas_layouts),
        forest_yellow: crate::atlas::FOREST_YELLOW_ATLAS.add_to_assets(texture_atlas_layouts),
        forest_fire_yellow: crate::atlas::FOREST_FIRE_YELLOW_ATLAS.add_to_assets(texture_atlas_layouts),
        // 背景
        background: background_atlas,
        // 子弹特效
        fire_effect: crate::atlas::FIRE_EFFECT_ATLAS.add_to_assets(texture_atlas_layouts),
        penetrate_effect: crate::atlas::PENETRATE_EFFECT_ATLAS.add_to_assets(texture_atlas_layouts),
        // 烟雾特效
        smoke_atlas: crate::atlas::SMOKE_ATLAS.add_to_assets(texture_atlas_layouts),
        // 爆炸特效
        explosion: crate::atlas::EXPLOSION_ATLAS.add_to_assets(texture_atlas_layouts),
        spark: crate::atlas::SPARK_ATLAS.add_to_assets(texture_atlas_layouts),
        // 指挥官
        commander: crate::atlas::COMMANDER_ATLAS.add_to_assets(texture_atlas_layouts),
        music_note: crate::atlas::MUSIC_NOTE_ATLAS.add_to_assets(texture_atlas_layouts),
        sea_bubble: crate::atlas::SEA_BUBBLE_ATLAS.add_to_assets(texture_atlas_layouts),
        player_avatar: crate::atlas::PLAYER_AVATAR_ATLAS.add_to_assets(texture_atlas_layouts),
        // 敌方出生
        enemy_born: crate::atlas::ENEMY_BORN_ATLAS.add_to_assets(texture_atlas_layouts),
        enemy_tank_normal: crate::atlas::ENEMY_TANK_NORMAL_ATLAS.add_to_assets(texture_atlas_layouts),
        enemy_tank_fire: crate::atlas::ENEMY_TANK_FIRE_ATLAS.add_to_assets(texture_atlas_layouts),
        enemy_tank_heavy: crate::atlas::ENEMY_TANK_HEAVY_ATLAS.add_to_assets(texture_atlas_layouts),
        enemy_tank_light: crate::atlas::ENEMY_TANK_LIGHT_ATLAS.add_to_assets(texture_atlas_layouts),
        enemy_tank_burning: crate::atlas::ENEMY_TANK_BURNING_ATLAS.add_to_assets(texture_atlas_layouts),
        laser_blue: crate::atlas::LASER_BLUE_ATLAS.add_to_assets(texture_atlas_layouts),
        laser_red: crate::atlas::LASER_RED_ATLAS.add_to_assets(texture_atlas_layouts),
        // 能量球
        energy_blue_ball: crate::atlas::ENERGY_BALL_BLUE_ATLAS.add_to_assets(texture_atlas_layouts),
        energy_red_ball: crate::atlas::ENERGY_BALL_RED_ATLAS.add_to_assets(texture_atlas_layouts),
        // 道具
        speed_up_icon: crate::atlas::POWER_UP_SPEED_UP_ATLAS.add_to_assets(texture_atlas_layouts),
        protection_icon: crate::atlas::POWER_UP_PROTECTION_ATLAS.add_to_assets(texture_atlas_layouts),
        fire_speed_icon: crate::atlas::POWER_UP_FIRE_SPEED_ATLAS.add_to_assets(texture_atlas_layouts),
        fire_shell_icon: crate::atlas::POWER_UP_FIRE_SHELL_ATLAS.add_to_assets(texture_atlas_layouts),
        track_chain_icon: crate::atlas::POWER_UP_TRACK_CHAIN_ATLAS.add_to_assets(texture_atlas_layouts),
        track_chain_effect: crate::atlas::TRACK_CHAIN_ATLAS.add_to_assets(texture_atlas_layouts),
        tank_smoke_effect: crate::atlas::TANK_SMOKE_ATLAS.add_to_assets(texture_atlas_layouts),
        dash_dust_effect: crate::atlas::DASH_DUST_ATLAS.add_to_assets(texture_atlas_layouts),
        penetrate_icon: crate::atlas::POWER_UP_PENETRATE_ATLAS.add_to_assets(texture_atlas_layouts),
        repair_icon: crate::atlas::POWER_UP_REPAIR_ATLAS.add_to_assets(texture_atlas_layouts),
        hamburger_icon: crate::atlas::POWER_UP_HAMBURGER_ATLAS.add_to_assets(texture_atlas_layouts),
        air_cushion_icon: crate::atlas::POWER_UP_AIR_CUSHION_ATLAS.add_to_assets(texture_atlas_layouts),
        shell_icon: crate::atlas::POWER_UP_SHELL_ATLAS.add_to_assets(texture_atlas_layouts),
    }
}

/// 初始化游戏资源
/// 预加载常用的字体、纹理和音效，避免运行时重复加载
pub fn init_game_resources(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let textures = init_textures(&asset_server);
    let audio = init_audio(&asset_server);
    let atlases = init_atlas_layouts(&mut texture_atlas_layouts);

    commands.insert_resource(textures);
    commands.insert_resource(audio);
    commands.insert_resource(atlases);
}
