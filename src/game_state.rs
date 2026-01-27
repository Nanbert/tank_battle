//! 游戏状态管理模块
//!
//! 处理游戏关卡、游戏结束、游戏重置等状态管理

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::*;
pub fn handle_game_over_delay(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer), With<GameOverTimer>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, mut timer) in &mut query {
        timer.tick(time.delta());
        if timer.is_finished() {
            let () = commands.entity(entity).try_despawn();
            next_state.set(GameState::GameOver);
        }
    }
}

pub fn check_game_over(
    mut commands: Commands,
    player_info: Res<PlayerInfo>,
    game_mode: Res<GameMode>,
    commander_life: Res<CommanderLife>,
    existing_timers: Query<(), With<GameOverTimer>>,
) {
    // 卫语句：已有计时器则跳过
    if !existing_timers.is_empty() {
        return;
    }

    // 卫语句：司令官阵亡
    if commander_life.life_points == 0 {
        spawn_game_over_timer(&mut commands);
        return;
    }

    // 卫语句：无玩家则跳过（单人模式下 player1 必定存在）
    if *game_mode == GameMode::OnePlayer && player_info.player1.life_points == 0 {
        return;
    }

    let all_players_dead = check_all_players_dead(&player_info, &game_mode);

    if all_players_dead {
        spawn_game_over_timer(&mut commands);
    }
}

/// 检查所有玩家是否阵亡
fn check_all_players_dead(player_info: &PlayerInfo, game_mode: &GameMode) -> bool {
    match *game_mode {
        GameMode::OnePlayer => player_info.player1.life_points == 0,
        GameMode::TwoPlayers => {
            player_info.player1.life_points == 0
                && player_info
                    .player2
                    .as_ref()
                    .is_some_and(|p| p.life_points == 0)
        }
    }
}

/// 生成 Game Over 计时器
fn spawn_game_over_timer(commands: &mut Commands) {
    commands.spawn((
        GameOverTimer,
        AnimationTimer(Timer::from_seconds(GAME_OVER_DELAY, TimerMode::Once)),
    ));
}

/// 重置 `FadingOut` 资源的 alpha 值为 1.0
pub fn reset_fading_out(mut fading_out: ResMut<FadingOut>) {
    fading_out.alpha = 1.0;
}

pub fn update_menu_blink(
    time: Res<Time>,
    fading_out: Res<FadingOut>,
    menu_selection: Res<CurrentMenuSelection>,
    mut blink_timer: ResMut<MenuBlinkTimer>,
    mut text_query: Query<(&MenuOption, &mut TextColor), Without<MenuArrow>>,
    game_state: Res<State<GameState>>,
) {
    match game_state.get() {
        GameState::FadingOut => {
            update_fading_out_blink(
                &time,
                &fading_out,
                &menu_selection,
                &mut blink_timer,
                &mut text_query,
            );
        }
        GameState::StartScreen => {
            update_start_screen_blink(
                &time,
                &menu_selection,
                &mut blink_timer,
                &mut text_query,
            );
        }
        _ => {}
    }
}

/// 更新淡出状态下的闪烁
fn update_fading_out_blink(
    time: &Res<Time>,
    fading_out: &Res<FadingOut>,
    menu_selection: &Res<CurrentMenuSelection>,
    blink_timer: &mut ResMut<MenuBlinkTimer>,
    text_query: &mut Query<(&MenuOption, &mut TextColor), Without<MenuArrow>>,
) {
    const FADE_OUT_BLINK_PERIOD: f32 = MENU_BLINK_PERIOD;

    // 初始化计时器
    if blink_timer.0.duration().is_zero() {
        blink_timer.0 = Timer::from_seconds(FADE_OUT_BLINK_PERIOD, TimerMode::Repeating);
    }

    blink_timer.0.tick(time.delta());

    for (option, mut text_color) in text_query.iter_mut() {
        if option.index != menu_selection.selected_index {
            continue;
        }

        let elapsed = blink_timer.0.elapsed_secs();
        let cycle = elapsed % FADE_OUT_BLINK_PERIOD;
        let half_period = FADE_OUT_BLINK_PERIOD / 2.0;
        let blink_alpha = (cycle / half_period * std::f32::consts::PI).sin().max(0.0);
        let final_alpha = blink_alpha * fading_out.alpha;

        text_color.0 = Color::srgb(1.0, 1.0, 0.0).with_alpha(final_alpha);
    }
}

/// 更新开始屏幕状态下的闪烁
fn update_start_screen_blink(
    time: &Res<Time>,
    menu_selection: &Res<CurrentMenuSelection>,
    blink_timer: &mut ResMut<MenuBlinkTimer>,
    text_query: &mut Query<(&MenuOption, &mut TextColor), Without<MenuArrow>>,
) {
    // 初始化计时器
    if blink_timer.0.duration().is_zero() {
        blink_timer.0 = Timer::from_seconds(MENU_BLINK_PERIOD, TimerMode::Repeating);
    }

    blink_timer.0.tick(time.delta());

    if !blink_timer.0.just_finished() {
        return;
    }

    for (option, mut text_color) in text_query.iter_mut() {
        if option.index != menu_selection.selected_index {
            continue;
        }

        let linear = text_color.0.to_linear();
        let is_yellow = linear.red > 0.9 && linear.green > 0.9 && linear.blue < 0.1;
        text_color.0 = if is_yellow {
            Color::srgb(1.0, 1.0, 1.0)
        } else {
            Color::srgb(1.0, 1.0, 0.0)
        };
    }
}

pub fn cleanup_playing_entities(
    mut commands: Commands,
    playing_entities: Query<Entity, With<PlayingEntity>>,
    mut player_info: ResMut<PlayerInfo>,
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    mut stage_level: ResMut<StageLevel>,
    mut commander_life: ResMut<CommanderLife>,
) {
    // 清理所有游戏实体
    for entity in playing_entities.iter() {
        commands.entity(entity).try_despawn();
    }

    // 重置玩家信息
    player_info.player1 = PlayerStats {
        name: "Player 1".to_string(),
        speed: 1,
        fire_speed: 1,
        protection: 0,
        shells: 1,
        penetrate: false,
        track_chain: false,
        air_cushion: false,
        fire_shell: false,
        life_points: 3,
        energy_points: 3,
        score: 0,
    };
    player_info.player2 = None;

    // 重置敌方坦克计数
    enemy_spawn_state.has_spawned = 0;
    enemy_spawn_state.spawn_cooldown.reset();

    // 重置关卡数
    stage_level.0 = 1;

    // 重置 Commander 生命值
    commander_life.life_points = 3;
}

pub fn check_stage_complete(
    enemy_spawn_state: Res<EnemySpawnState>,
    enemies: Query<(), With<EnemyTank>>,
    player_info: Res<PlayerInfo>,
    commander_life: Res<CommanderLife>,
    game_mode: Res<GameMode>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stage_level: ResMut<StageLevel>,
) {
    // 卫语句：检查是否完成关卡
    let current_enemy_count = enemies.iter().count();
    if enemy_spawn_state.has_spawned < enemy_spawn_state.max_count || current_enemy_count > 0 {
        return;
    }

    // 卫语句：玩家或 Commander 已阵亡
    let all_players_dead = check_all_players_dead(&player_info, &game_mode);
    if all_players_dead || commander_life.life_points == 0 {
        return;
    }

    // 进入下一关
    stage_level.0 += 1;
    next_state.set(GameState::StageIntro);
}

/// 重置玩家坦克位置到出生点
pub fn reset_player_positions(
    mut player_tanks: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut KinematicCharacterController,
            &PlayerTank,
        ),
        With<PlayerTank>,
    >,
) {
    for (mut transform, mut velocity, mut character_controller, player_tank) in &mut player_tanks {
        // 重置物理引擎速度和位移累积
        velocity.linvel = Vec2::ZERO;
        velocity.angvel = 0.0;
        character_controller.translation = None;

        match player_tank.tank_type {
            TankType::Player1 => {
                // 玩家1出生位置：左侧
                transform.translation.x =
                    -TANK_WIDTH / 2.0 - COMMANDER_WIDTH / 2.0 - PLAYER_SPAWN_OFFSET;
                transform.translation.y = MAP_BOTTOM_Y + TANK_HEIGHT / 2.0;
            }
            TankType::Player2 => {
                // 玩家2出生位置：右侧
                transform.translation.x =
                    TANK_WIDTH / 2.0 + COMMANDER_WIDTH / 2.0 + PLAYER_SPAWN_OFFSET;
                transform.translation.y = MAP_BOTTOM_Y + TANK_HEIGHT / 2.0;
            }
            TankType::Enemy => {}
        }
        transform.rotation = Quat::IDENTITY;
    }
}

pub fn reset_for_next_stage(mut enemy_spawn_state: ResMut<EnemySpawnState>) {
    // 重置敌方坦克计数
    enemy_spawn_state.has_spawned = 0;
    enemy_spawn_state.spawn_cooldown.reset();
}

// 文本更新函数类型
