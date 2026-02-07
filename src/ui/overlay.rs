//! 游戏覆盖界面模块
//!
//! 处理暂停界面、游戏结束界面、关卡介绍界面等覆盖在游戏上的界面

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;
use super::common;
// 从 localization 模块导入本地化常量
use super::localization::*;

/// 淡出屏幕效果
pub fn fade_out_screen(
    mut commands: Commands,
    time: Res<Time>,
    mut fading_out: ResMut<FadingOut>,
    mut next_state: ResMut<NextState<GameState>>,
    menu_selection: Res<CurrentMenuSelection>,
    mut ui_query: Query<(Entity, Option<&mut Sprite>, Option<&mut TextColor>, Option<&MenuOption>), With<StartScreenUI>>,
) {
    // 减少透明度
    fading_out.alpha -= time.delta_secs() * (1.0 / FADE_OUT_SPEED); // 淡出速度，1.5秒完成

    // 更新所有 Sprite 元素的透明度
    let selected_index = menu_selection.selected_index;

    for (_entity, sprite_opt, text_color_opt, menu_option_opt) in &mut ui_query {
        // 更新 Sprite 透明度
        if let Some(mut sprite) = sprite_opt {
            common::update_alpha(fading_out.alpha, sprite.as_mut());
        }

        // 更新 Text 元素的颜色（选中的选项由 update_menu_blink 处理闪烁，但需要跟随淡出）
        if let Some(mut text_color) = text_color_opt {
            // 如果是当前选中的选项，跳过透明度更新（闪烁由 update_menu_blink 处理）
            if menu_option_opt.is_some_and(|opt| opt.index == selected_index) {
                continue;
            }
            common::update_alpha(fading_out.alpha, text_color.as_mut());
        }
    }

    // 淡出完成，切换到 StageIntro 状态并清理所有 StartScreenUI 元素
    if fading_out.alpha <= 0.0 {
        next_state.set(GameState::StageIntro);
        for (entity, _, _, _) in &mut ui_query {
            commands.entity(entity).despawn();
        }
    }
}

/// 生成关卡介绍界面
pub fn spawn_stage_intro(
    mut commands: Commands,
    mut game_timers: ResMut<GameTimers>,
    mut clear_color: ResMut<ClearColor>,
    stage_level: Res<StageLevel>,
    font_resources: Res<GameTextureResources>,
    language: Res<Language>,
) {
    // 设置背景色为白色
    clear_color.0 = COLOR_WHITE;

    // 初始化计时器
    game_timers.stage_intro.fade_in = Timer::from_seconds(STAGE_FADE_IN_DURATION, TimerMode::Once);
    game_timers.stage_intro.stay = Timer::from_seconds(STAGE_FADE_HOLD_DURATION, TimerMode::Once);
    game_timers.stage_intro.fade_out = Timer::from_seconds(STAGE_FADE_OUT_DURATION, TimerMode::Once);

    // 创建全屏白色背景方块，遮挡所有游戏元素
    commands.spawn((
        StageIntroUI,
        Sprite {
            color: COLOR_WHITE, // 白色
            custom_size: Some(WINDOW_SIZE),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_STAGE_INTRO_BG), // z=100.0 确保在所有游戏元素之上
    ));

    // 使用预加载的字体
    let en_font = font_resources.en.clone();
    let zh_font = font_resources.cn.clone();

    // 根据语言选择字体和文本
    let (stage_text, quote_text, quote_font, stage_font) = match *language {
        Language::Chinese => {
            let mut rng = rand::rng();
            let quote_index = rng.random_range(0..STAGE_QUOTES_CN.len());
            (
                format!("第 {} 关", stage_level.0),
                STAGE_QUOTES_CN[quote_index].to_string(),
                zh_font.clone(),
                zh_font,
            )
        }
        Language::English => {
            let mut rng = rand::rng();
            let quote_index = rng.random_range(0..STAGE_QUOTES_EN.len());
            (
                format!("Stage {}", stage_level.0),
                STAGE_QUOTES_EN[quote_index].to_string(),
                en_font.clone(),
                en_font,
            )
        }
    };

    // Stage 标题（显示在屏幕中心）
    common::spawn_simple_text_with_marker(
        &mut commands,
        stage_text,
        &stage_font,
        FONT_SIZE_MENU,
        Vec3::new(0.0, 100.0, Z_STAGE_INTRO_TEXT),
        COLOR_TRANSPARENT_BLACK, // 黑色，初始透明度为0
        StageIntroUI,
        Z_STAGE_INTRO_TEXT,
    );

    // 描述文字（俏皮话）
    common::spawn_text_with_justify_and_marker(
        &mut commands,
        quote_text,
        &quote_font,
        FONT_SIZE_SCORE,
        Vec3::new(0.0, -50.0, Z_STAGE_INTRO_TEXT),
        COLOR_DARK_GRAY.with_alpha(0.0), // 暗灰色，初始透明度为0
        StageIntroUI,
        Justify::Center,
        Z_STAGE_INTRO_TEXT,
    );
}

/// 处理关卡介绍界面的计时器
pub fn handle_stage_intro_timer(
    time: Res<Time>,
    mut game_timers: ResMut<GameTimers>,
    mut next_state: ResMut<NextState<GameState>>,
    mut text_query: Query<&mut TextColor, With<StageIntroUI>>,
) {
    // 淡入阶段
    if !game_timers.stage_intro.fade_in.is_finished() {
        game_timers.stage_intro.fade_in.tick(time.delta());
        common::update_text_alpha_from_timer(&game_timers.stage_intro.fade_in, true, &mut text_query);
    }
    // 停留阶段
    else if !game_timers.stage_intro.stay.is_finished() {
        game_timers.stage_intro.stay.tick(time.delta());
    }
    // 淡出阶段
    else if !game_timers.stage_intro.fade_out.is_finished() {
        game_timers.stage_intro.fade_out.tick(time.delta());
        common::update_text_alpha_from_timer(&game_timers.stage_intro.fade_out, false, &mut text_query);
    }
    // 所有阶段完成，切换到 Playing 状态
    else {
        next_state.set(GameState::Playing);
    }
}

/// 销毁关卡介绍界面
pub fn despawn_stage_intro(
    mut commands: Commands,
    mut clear_color: ResMut<ClearColor>,
    stage_intro_query: Query<Entity, With<StageIntroUI>>,
) {
    // 重置背景色为游戏背景色
    clear_color.0 = COLOR_BACKGROUND;

    crate::utils::cleanup_entities(&mut commands, stage_intro_query.iter());
}

/// 生成暂停界面
pub fn spawn_pause_ui(
    mut commands: Commands,
    font_resources: Res<GameTextureResources>,
    language: Res<Language>,
    mut player_velocity_query: Query<&mut Velocity, With<PlayerTank>>,
    mut enemy_velocity_query: Query<&mut Velocity, (With<EnemyTank>, Without<PlayerTank>)>,
) {
    let font = common::get_font(&font_resources, *language);

    // 停止所有坦克的移动
    crate::utils::stop_all_tanks_velocity(&mut player_velocity_query, &mut enemy_velocity_query);

    commands.spawn((
        PauseUI,
        Text2d(PAUSED_TITLE.get(*language).to_string()),
        common::create_text_font(&font, FONT_SIZE_GAME_OVER),
        TextColor(COLOR_YELLOW),
        Transform::from_xyz(0.0, 0.0, Z_UI),
    ));

    commands.spawn((
        PauseUI,
        Text2d(PAUSED_INSTRUCTION.get(*language).to_string()),
        common::create_text_font(&font, FONT_SIZE_UI),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, -100.0, Z_UI),
    ));
}

/// 销毁暂停界面
pub fn despawn_pause_ui(mut commands: Commands, query: Query<Entity, With<PauseUI>>) {
    crate::utils::cleanup_entities(&mut commands, query.iter());
}

/// 处理游戏中的输入（暂停和退出）
pub fn handle_game_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    // Space 键暂停
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Paused);
    }
    // Esc 键退出
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let _ = app_exit.write(AppExit::Success);
    }
}

/// 处理暂停界面的输入
pub fn handle_pause_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    // Space 键恢复游戏
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
    // B 键返回菜单
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        next_state.set(GameState::StartScreen);
    }
    // Esc 键退出
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let _ = app_exit.write(AppExit::Success);
    }
}

/// 生成游戏结束界面
pub fn spawn_game_over_ui(
    mut commands: Commands,
    font_resources: Res<GameTextureResources>,
    language: Res<Language>,
    mut player_velocity_query: Query<&mut Velocity, With<PlayerTank>>,
    mut enemy_velocity_query: Query<&mut Velocity, (With<EnemyTank>, Without<PlayerTank>)>,
) {
    let font = common::get_font(&font_resources, *language);

    // 停止所有坦克的移动
    crate::utils::stop_all_tanks_velocity(&mut player_velocity_query, &mut enemy_velocity_query);

    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_TITLE.get(*language).to_string()),
        TextFont {
            font_size: FONT_SIZE_GAME_OVER,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_RED),
        Transform::from_xyz(0.0, 100.0, Z_UI),
    ));

    // Restart 选项
    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_RESTART.get(*language).to_string()),
        common::create_text_font(&font, FONT_SIZE_OPTION),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, 0.0, Z_UI),
        MenuOption { index: 0 },
    ));

    // Back to Menu 选项
    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_MENU.get(*language).to_string()),
        common::create_text_font(&font, FONT_SIZE_OPTION),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, -60.0, Z_UI),
        MenuOption { index: 1 },
    ));

    // Exit 选项
    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_EXIT.get(*language).to_string()),
        common::create_text_font(&font, FONT_SIZE_OPTION),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, -120.0, Z_UI),
        MenuOption { index: 2 },
    ));

    // 操作说明
    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_INSTRUCTION.get(*language).to_string()),
        common::create_text_font(&font, FONT_SIZE_UI),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, -180.0, Z_UI),
    ));
}

/// 处理游戏结束界面的输入
pub fn handle_game_over_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu_selection: ResMut<CurrentMenuSelection>,
    mut app_exit: MessageWriter<AppExit>,
) {
    // 使用通用菜单导航函数
    common::handle_menu_navigation(
        &keyboard_input,
        &mut menu_selection.selected_index,
        2, // 最大索引（3个选项：0-2）
        common::NavigationWrap::Clamped,
    );

    // Space 键确认选择
    if keyboard_input.just_pressed(KeyCode::Space) {
        match menu_selection.selected_index {
            0 => {
                // Restart: 进入关卡介绍页面
                next_state.set(GameState::StageIntro);
            }
            1 => {
                // Back to Menu: 返回开始界面
                next_state.set(GameState::StartScreen);
            }
            2 => {
                // Exit: 退出游戏
                let _ = app_exit.write(AppExit::Success);
            }
            _ => {}
        }
    }
}

/// 销毁游戏结束界面
pub fn despawn_game_over_ui(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    crate::utils::cleanup_entities(&mut commands, query.iter());
}

/// 生成能量不足提示
pub fn spawn_insufficient_energy_warning(
    commands: &mut Commands,
    font_resources: &GameTextureResources,
    tank_type: TankType,
    language: Language,
) {
    let text = INSUFFICIENT_ENERGY.get(language);

    // 根据玩家类型选择 X 位置（玩家1在左侧，玩家2在右侧）
    let x_pos = match tank_type {
        TankType::Player1 => WINDOW_LEFT_X + crate::ui::constants::HUD_PLAYER_OFFSET,
        TankType::Player2 => WINDOW_RIGHT_X - crate::ui::constants::HUD_PLAYER_OFFSET,
        TankType::Enemy => unreachable!(),
    };

    // Y 位置在效果和名称中间
    let y_pos = common::hud_y_position(HudYPosition::InsufficientEnergy);

    let font = common::get_font(&font_resources, language);

    // 生成文本实体
    let _entity = commands.spawn((
        InsufficientEnergyText,
        Text2d(text.to_string()),
        common::create_text_font(&font, FONT_SIZE_INSUFFICIENT_ENERGY),
        TextColor(COLOR_GOLD),
        Transform::from_xyz(x_pos, y_pos, Z_UI),
        // 使用新的通用闪烁动画系统，设置 despawn_on_complete 为 true
        // 动画完成后会自动销毁整个实体
        common::BlinkAnimation::gold_blink_despawn(INSUFFICIENT_ENERGY_DISPLAY_DURATION),
    )).id();
}



/// 销毁所有能量不足提示
pub fn despawn_insufficient_energy_warnings(
    mut commands: Commands,
    query: Query<Entity, With<InsufficientEnergyText>>,
) {
    crate::utils::cleanup_entities(&mut commands, query.iter());
}

/// 清理开始界面的UI元素
/// 销毁开始界面
pub fn despawn_start_screen_ui(mut commands: Commands, query: Query<Entity, With<StartScreenUI>>) {
    common::despawn_by_marker::<StartScreenUI>(&mut commands, query);
}

/// 销毁关于界面
pub fn despawn_about_screen(mut commands: Commands, query: Query<Entity, With<AboutUI>>) {
    common::despawn_by_marker::<AboutUI>(&mut commands, query);
}

/// 销毁致谢界面
pub fn despawn_credits_screen(mut commands: Commands, query: Query<Entity, With<CreditsUI>>) {
    common::despawn_by_marker::<CreditsUI>(&mut commands, query);
}

/// 销毁所有道具
pub fn despawn_powerups(
    mut commands: Commands,
    powerups: Query<Entity, With<crate::powerup::PowerUp>>,
) {
    common::despawn_by_marker::<crate::powerup::PowerUp>(&mut commands, powerups);
}
