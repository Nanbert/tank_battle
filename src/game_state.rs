//! 游戏状态管理模块
//!
//! 处理游戏关卡、游戏结束、游戏重置等状态管理

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;

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
    game_timers: ResMut<GameTimers>,
    mut text_query: Query<(&MenuOption, &mut TextColor)>,
    game_state: Res<State<GameState>>,
) {
    match game_state.get() {
        GameState::FadingOut => {
            update_fading_out_blink(
                &time,
                &fading_out,
                &menu_selection,
                game_timers,
                &mut text_query,
            );
        }
        GameState::StartScreen => {
            update_start_screen_blink(&time, &menu_selection, game_timers, &mut text_query);
        }
        _ => {}
    }
}

/// 更新淡出状态下的闪烁
fn update_fading_out_blink(
    time: &Res<Time>,
    fading_out: &Res<FadingOut>,
    menu_selection: &Res<CurrentMenuSelection>,
    mut game_timers: ResMut<GameTimers>,
    text_query: &mut Query<(&MenuOption, &mut TextColor)>,
) {
    const FADE_OUT_BLINK_PERIOD: f32 = MENU_BLINK_PERIOD;

    // 初始化计时器
    if game_timers.menu_blink.0.duration().is_zero() {
        game_timers.menu_blink.0 = Timer::from_seconds(FADE_OUT_BLINK_PERIOD, TimerMode::Repeating);
    }

    game_timers.menu_blink.0.tick(time.delta());

    for (option, mut text_color) in text_query.iter_mut() {
        if option.index != menu_selection.selected_index {
            continue;
        }

        let elapsed = game_timers.menu_blink.0.elapsed_secs();
        let cycle = elapsed % FADE_OUT_BLINK_PERIOD;
        let half_period = FADE_OUT_BLINK_PERIOD / 2.0;
        let blink_alpha = (cycle / half_period * std::f32::consts::PI).sin().max(0.0);
        let final_alpha = blink_alpha * fading_out.alpha;

        text_color.0 = COLOR_YELLOW.with_alpha(final_alpha);
    }
}

/// 更新开始屏幕状态下的闪烁
fn update_start_screen_blink(
    time: &Res<Time>,
    menu_selection: &Res<CurrentMenuSelection>,
    mut game_timers: ResMut<GameTimers>,
    text_query: &mut Query<(&MenuOption, &mut TextColor)>,
) {
    // 初始化计时器
    if game_timers.menu_blink.0.duration().is_zero() {
        game_timers.menu_blink.0 = Timer::from_seconds(MENU_BLINK_PERIOD, TimerMode::Repeating);
    }

    game_timers.menu_blink.0.tick(time.delta());

    if !game_timers.menu_blink.0.just_finished() {
        return;
    }

    for (option, mut text_color) in text_query.iter_mut() {
        if option.index != menu_selection.selected_index {
            continue;
        }

        let linear = text_color.0.to_linear();
        let is_yellow = linear.red > 0.9 && linear.green > 0.9 && linear.blue < 0.1;
        text_color.0 = if is_yellow { COLOR_WHITE } else { COLOR_YELLOW };
    }
}

pub fn check_stage_complete(
    mut enemy_spawn_state: ResMut<EnemySpawnState>,
    enemies: Query<(), Or<(With<EnemyTank>, With<EnemyBornAnimation>)>>,
    player_info: Res<PlayerInfo>,
    commander_life: Res<CommanderLife>,
    game_mode: Res<GameMode>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stage_level: ResMut<StageLevel>,
    time: Res<Time>,
) {
    // 卫语句：检查是否完成关卡
    // 统计当前场上敌方坦克数量（包括已生成的和正在出生动画中的）
    let current_enemy_count = enemies.iter().count();
    if enemy_spawn_state.has_spawned < enemy_spawn_state.max_count || current_enemy_count > 0 {
        // 如果还有敌人，重置延迟计时器
        enemy_spawn_state.stage_complete_delay.reset();
        return;
    }

    // 卫语句：玩家或 Commander 已阵亡
    let all_players_dead = check_all_players_dead(&player_info, &game_mode);
    if all_players_dead || commander_life.life_points == 0 {
        return;
    }

    // 更新延迟计时器
    enemy_spawn_state.stage_complete_delay.tick(time.delta());

    // 检查延迟是否完成
    if enemy_spawn_state.stage_complete_delay.just_finished() {
        // 进入下一关
        stage_level.0 += 1;
        next_state.set(GameState::StageIntro);
    }
}

/// 清理所有 effect 和 bullet 实体
/// 在进入 StageIntro 状态时调用，确保场景中没有残留的特效和子弹
pub fn cleanup_effects_and_bullets(
    mut commands: Commands,
    bullets: Query<Entity, With<crate::bullet::Bullet>>,
    explosions: Query<Entity, With<crate::constants::Explosion>>,
    sparks: Query<Entity, With<crate::constants::Spark>>,
    smokes: Query<Entity, With<crate::constants::Smoke>>,
    forest_fires: Query<Entity, With<crate::constants::ForestFire>>,
    energy_balls: Query<Entity, With<crate::constants::EnergyBall>>,
    lasers: Query<Entity, With<crate::constants::Laser>>,
    // 只清理环境音效播放器（循环音效），一次性音效会自动回收
    ambience_players: Query<
        Entity,
        Or<(
            With<crate::constants::SeaAmbiencePlayer>,
            With<crate::constants::TreeAmbiencePlayer>,
            With<crate::constants::CommanderAmbiencePlayer>,
        )>,
    >,
    mut bullet_tracker: ResMut<crate::resources::BulletTracker>,
) {
    // 清理所有子弹（需要先从 tracker 中移除）
    for bullet in bullets.iter() {
        bullet_tracker.remove_bullet(bullet);
    }
    crate::utils::cleanup_entities(&mut commands, bullets.iter());

    // 清理其他实体
    crate::utils::cleanup_entities(&mut commands, explosions.iter());
    crate::utils::cleanup_entities(&mut commands, sparks.iter());
    crate::utils::cleanup_entities(&mut commands, smokes.iter());
    crate::utils::cleanup_entities(&mut commands, forest_fires.iter());
    crate::utils::cleanup_entities(&mut commands, energy_balls.iter());
    crate::utils::cleanup_entities(&mut commands, lasers.iter());
    // 只清理环境音效播放器（循环音效）
    crate::utils::cleanup_entities(&mut commands, ambience_players.iter());

    // 强制重置 BulletTracker，防止状态不同步
    bullet_tracker.clear();
}

/// 清理所有追踪器和计时器
/// 在进入 StageIntro 状态时调用，确保没有残留的追踪数据
pub fn cleanup_trackers_and_timers(
    mut bullet_tracker: ResMut<crate::resources::BulletTracker>,
    mut recall_timers: ResMut<crate::resources::RecallTimers>,
    mut dash_timers: ResMut<crate::resources::DashTimers>,
    mut dash_damage_tracker: ResMut<crate::resources::DashDamageTracker>,
    mut barrier_damage_tracker: ResMut<crate::resources::BarrierDamageTracker>,
    mut insufficient_energy_tracker: ResMut<crate::resources::InsufficientEnergyTracker>,
    mut collision_cache: ResMut<crate::constants::EnemyCollisionCache>,
    mut enemy_spawn_state: ResMut<crate::resources::EnemySpawnState>,
) {
    // 清理 BulletTracker
    bullet_tracker.clear();

    // 清理 RecallTimers
    recall_timers.timers.clear();

    // 清理 DashTimers
    dash_timers.timers.clear();

    // 清理 DashDamageTracker
    dash_damage_tracker.has_taken_damage.clear();

    // 清理 BarrierDamageTracker
    barrier_damage_tracker.cooldowns.clear();

    // 清理 InsufficientEnergyTracker
    insufficient_energy_tracker.p1_cooldown = None;
    insufficient_energy_tracker.p2_cooldown = None;

    // 清理 EnemyCollisionCache（事件驱动缓存）
    collision_cache.clear();

    // 重置关卡完成延迟计时器
    enemy_spawn_state.stage_complete_delay.reset();
}

// 文本更新函数类型
