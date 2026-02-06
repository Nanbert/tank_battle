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
use crate::resources::*;
use super::common;

// ==================== 本地化文本常量 ====================

const PAUSED_TITLE: LocalizedText = LocalizedText {
    cn: "已暂停",
    en: "PAUSED",
};

const PAUSED_INSTRUCTION: LocalizedText = LocalizedText {
    cn: "按 SPACE 继续 | B 返回菜单 | ESC 退出",
    en: "Press SPACE to resume | B to menu | ESC to exit",
};

const GAME_OVER_TITLE: LocalizedText = LocalizedText {
    cn: "游戏结束",
    en: "GAME OVER",
};

const GAME_OVER_RESTART: LocalizedText = LocalizedText {
    cn: "重新开始",
    en: "RESTART",
};

const GAME_OVER_MENU: LocalizedText = LocalizedText {
    cn: "返回菜单",
    en: "MENU",
};

const GAME_OVER_EXIT: LocalizedText = LocalizedText {
    cn: "退出",
    en: "EXIT",
};

const GAME_OVER_INSTRUCTION: LocalizedText = LocalizedText {
    cn: "W/S 选择 | SPACE 确认",
    en: "W/S select | SPACE confirm",
};

const INSUFFICIENT_ENERGY_CN: &str = "能量不足！";
const INSUFFICIENT_ENERGY_EN: &str = "Insufficient Energy!";

// ==================== 辅助函数 ====================

/// 更新 Sprite 的透明度
fn update_sprite_alpha(alpha: f32, sprite: &mut Sprite) {
    let linear = sprite.color.to_linear();
    sprite.color = Color::srgba(linear.red, linear.green, linear.blue, alpha);
}

/// 更新 `TextColor` 的透明度
fn update_text_color_alpha(alpha: f32, text_color: &mut TextColor) {
    let linear = text_color.0.to_linear();
    text_color.0 = Color::srgba(linear.red, linear.green, linear.blue, alpha);
}

/// 淡出屏幕效果
pub fn fade_out_screen(
    mut commands: Commands,
    time: Res<Time>,
    mut fading_out: ResMut<FadingOut>,
    mut next_state: ResMut<NextState<GameState>>,
    menu_selection: Res<CurrentMenuSelection>,
    mut sprite_query: Query<(Entity, &mut Sprite), With<StartScreenUI>>,
    mut text_query: Query<(Entity, &mut TextColor, Option<&MenuOption>), With<StartScreenUI>>,
    start_screen_query: Query<Entity, With<StartScreenUI>>,
) {
    // 减少透明度
    fading_out.alpha -= time.delta_secs() * (1.0 / FADE_OUT_SPEED); // 淡出速度，1.5秒完成

    // 更新所有 Sprite 元素的透明度
    for (_, mut sprite) in &mut sprite_query {
        update_sprite_alpha(fading_out.alpha, &mut sprite);
    }

    // 更新所有 Text 元素的颜色（选中的选项由 update_menu_blink 处理闪烁，但需要跟随淡出）
    let selected_index = menu_selection.selected_index;

    for (_, mut text_color, menu_option) in &mut text_query {
        // 如果是当前选中的选项，跳过透明度更新（闪烁由 update_menu_blink 处理）
        if menu_option.is_some_and(|opt| opt.index == selected_index) {
            continue;
        }
        update_text_color_alpha(fading_out.alpha, &mut text_color);
    }

    // 淡出完成，切换到 StageIntro 状态并清理所有 StartScreenUI 元素
    if fading_out.alpha <= 0.0 {
        next_state.set(GameState::StageIntro);
        crate::utils::cleanup_entities(&mut commands, start_screen_query.iter());
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
    commands.spawn((
        StageIntroUI,
        Text2d(stage_text),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font: stage_font,
            ..default()
        },
        TextColor(COLOR_TRANSPARENT_BLACK), // 黑色，初始透明度为0
        Transform::from_xyz(0.0, 100.0, Z_STAGE_INTRO_TEXT), // z=101.0 在白色背景之上
    ));

    // 描述文字（俏皮话）
    commands.spawn((
        StageIntroUI,
        Text2d(quote_text),
        TextFont {
            font_size: FONT_SIZE_SCORE,
            font: quote_font,
            ..default()
        },
        TextColor(COLOR_DARK_GRAY.with_alpha(0.0)), // 暗灰色，初始透明度为0
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, -50.0, Z_STAGE_INTRO_TEXT), // z=101.0 在白色背景之上
    ));
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
        let progress = game_timers.stage_intro.fade_in.elapsed_secs()
            / game_timers.stage_intro.fade_in.duration().as_secs_f32();
        let alpha = progress.min(1.0);
        for mut text_color in &mut text_query {
            // 获取当前颜色（不包含透明度）
            let color = text_color.0;
            // 只更新透明度，保持原始颜色
            text_color.0 = color.with_alpha(alpha);
        }
    }
    // 停留阶段
    else if !game_timers.stage_intro.stay.is_finished() {
        game_timers.stage_intro.stay.tick(time.delta());
    }
    // 淡出阶段
    else if !game_timers.stage_intro.fade_out.is_finished() {
        game_timers.stage_intro.fade_out.tick(time.delta());
        let progress = game_timers.stage_intro.fade_out.elapsed_secs()
            / game_timers.stage_intro.fade_out.duration().as_secs_f32();
        let alpha = 1.0 - progress.min(1.0);
        for mut text_color in &mut text_query {
            // 获取当前颜色（不包含透明度）
            let color = text_color.0;
            // 只更新透明度，保持原始颜色
            text_color.0 = color.with_alpha(alpha);
        }
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
        TextFont {
            font_size: FONT_SIZE_GAME_OVER,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_YELLOW),
        Transform::from_xyz(0.0, 0.0, Z_UI),
    ));

    commands.spawn((
        PauseUI,
        Text2d(PAUSED_INSTRUCTION.get(*language).to_string()),
        TextFont {
            font_size: FONT_SIZE_UI,
            font,
            ..default()
        },
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
        TextFont {
            font_size: FONT_SIZE_OPTION,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, 0.0, Z_UI),
        MenuOption { index: 0 },
    ));

    // Back to Menu 选项
    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_MENU.get(*language).to_string()),
        TextFont {
            font_size: FONT_SIZE_OPTION,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, -60.0, Z_UI),
        MenuOption { index: 1 },
    ));

    // Exit 选项
    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_EXIT.get(*language).to_string()),
        TextFont {
            font_size: FONT_SIZE_OPTION,
            font: font.clone(),
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(0.0, -120.0, Z_UI),
        MenuOption { index: 2 },
    ));

    // 操作说明
    commands.spawn((
        GameOverUI,
        Text2d(GAME_OVER_INSTRUCTION.get(*language).to_string()),
        TextFont {
            font_size: FONT_SIZE_UI,
            font,
            ..default()
        },
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
    // W 键向上选择
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        menu_selection.selected_index = if menu_selection.selected_index == 0 {
            2
        } else {
            menu_selection.selected_index - 1
        };
    }
    // S 键向下选择
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        menu_selection.selected_index = (menu_selection.selected_index + 1) % 3;
    }
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
    mut commands: Commands,
    font_cn: Handle<Font>,
    font_en: Handle<Font>,
    tank_type: TankType,
    language: Language,
) {
    let font = match language {
        Language::Chinese => font_cn,
        Language::English => font_en,
    };

    let text = match language {
        Language::Chinese => INSUFFICIENT_ENERGY_CN,
        Language::English => INSUFFICIENT_ENERGY_EN,
    };

    // 根据玩家类型选择 X 位置（玩家1在左侧，玩家2在右侧）
    let x_pos = match tank_type {
        TankType::Player1 => WINDOW_LEFT_X + 115.0,
        TankType::Player2 => WINDOW_RIGHT_X - 115.0,
        TankType::Enemy => unreachable!(),
    };

    // Y 位置在效果和名称中间
    let y_pos = WINDOW_TOP_Y - HUD_Y_POSITIONS[HudYPosition::InsufficientEnergy as usize];

    // 使用金色字体
    commands.spawn((
        InsufficientEnergyText,
        Text2d(text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INSUFFICIENT_ENERGY,
            font,
            ..default()
        },
        TextColor(COLOR_GOLD),
        Transform::from_xyz(x_pos, y_pos, Z_UI),
        InsufficientEnergyTimer(Timer::from_seconds(
            INSUFFICIENT_ENERGY_DISPLAY_DURATION,
            TimerMode::Once,
        )),
    ));
}

/// 更新能量不足提示计时器
pub fn update_insufficient_energy_warnings(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut InsufficientEnergyTimer, &mut TextColor)>,
) {
    for (entity, mut timer, mut color) in &mut query {
        timer.tick(time.delta());

        // 计算闪烁效果：2秒内闪烁2次
        // 闪烁周期 = 2秒 / 2次 = 1秒
        let elapsed = timer.elapsed_secs();
        let blink_period = INSUFFICIENT_ENERGY_DISPLAY_DURATION / 2.0; // 1秒

        // 计算当前在闪烁周期中的位置
        let cycle_progress = (elapsed % blink_period) / blink_period;

        // 前50%显示金色，后50%隐藏
        if cycle_progress < 0.5 {
            color.0 = COLOR_GOLD; // 显示金色
        } else {
            color.0 = COLOR_TRANSPARENT_BLACK; // 隐藏
        }

        if timer.just_finished() {
            let () = commands.entity(entity).try_despawn();
        }
    }
}

/// 销毁所有能量不足提示
pub fn despawn_insufficient_energy_warnings(
    mut commands: Commands,
    query: Query<Entity, With<InsufficientEnergyText>>,
) {
    crate::utils::cleanup_entities(&mut commands, query.iter());
}

/// 清理开始界面的UI元素
pub fn cleanup_start_screen_ui(mut commands: Commands, query: Query<Entity, With<StartScreenUI>>) {
    crate::utils::cleanup_entities(&mut commands, query.iter());
}

/// 销毁关于界面
pub fn despawn_about_screen(mut commands: Commands, query: Query<Entity, With<AboutUI>>) {
    crate::utils::cleanup_entities(&mut commands, query.iter());
}

/// 销毁致谢界面
pub fn despawn_credits_screen(mut commands: Commands, query: Query<Entity, With<CreditsUI>>) {
    crate::utils::cleanup_entities(&mut commands, query.iter());
}

/// 销毁所有道具
pub fn despawn_powerups(
    mut commands: Commands,
    powerups: Query<Entity, With<crate::powerup::PowerUp>>,
) {
    crate::utils::cleanup_entities(&mut commands, powerups.iter());
}
