//! 菜单界面模块
//!
//! 处理开始界面、关于界面、致谢界面等菜单相关功能

#[cfg(not(target_arch = "wasm32"))]
use bevy::app::AppExit;
use bevy::prelude::*;

use super::common;
#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;
// 从 localization 模块导入本地化常量
use super::localization::*;

// ==================== 开始界面相关函数 ====================

/// 生成开始界面的背景动画
pub fn spawn_start_screen_background(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
) {
    crate::utils::spawn_animated_sprite(
        commands,
        texture_resources.background.clone(),
        atlas_layouts.background.clone(),
        crate::atlas::BACKGROUND_ATLAS.animation_indices_full(),
        BACKGROUND_ANIMATION_FRAME,
        Transform::from_translation(Vec3::new(0.0, 0.0, Z_UI_BACKGROUND)),
        WINDOW_SIZE,
        (StartScreenUI, AnimationMode::Looping),
    );
}

/// 生成开始界面的标题和菜单选项
pub fn spawn_start_screen_title(commands: &mut Commands, font: Handle<Font>, language: Language) {
    let title = MENU_TITLE.get(language);

    common::spawn_simple_text_with_marker(
        commands,
        title.to_string(),
        &font,
        FONT_SIZE_TITLE,
        Vec3::new(0.0, MENU_TITLE_Y, Z_UI_TEXT),
        COLOR_RED,
        StartScreenUI,
        Z_UI_TEXT,
    );

    // 菜单选项，从上到下
    #[cfg(not(target_arch = "wasm32"))]
    let menu_count = 7;
    #[cfg(target_arch = "wasm32")]
    let menu_count = 6;
    let y_positions =
        common::generate_menu_y_positions(MENU_START_Y, MENU_OPTION_SPACING, menu_count);
    for (i, option_text) in MENU_OPTIONS.iter().enumerate() {
        commands.spawn((
            StartScreenUI,
            Text2d(option_text.get(language).to_string()),
            common::create_text_font(&font, FONT_SIZE_MENU),
            TextColor(if i == 0 { COLOR_YELLOW } else { COLOR_WHITE }),
            Transform::from_xyz(0.0, y_positions[i], Z_UI_TEXT),
            MenuOption { index: i },
        ));
    }
}

/// 生成开始界面的操作说明
pub fn spawn_start_screen_instructions(
    commands: &mut Commands,
    font: Handle<Font>,
    language: Language,
) {
    let p1_text = CONTROLS_P1.get(language);
    let p2_text = CONTROLS_P2.get(language);
    let general_text = CONTROLS_GENERAL.get(language);

    // 玩家1操作说明 - 蓝色
    common::spawn_simple_text_with_marker(
        commands,
        p1_text.to_string(),
        &font,
        FONT_SIZE_INSTRUCTION,
        Vec3::new(0.0, CONTROLS_P1_Y, Z_UI_TEXT),
        COLOR_BLUE,
        StartScreenUI,
        Z_UI_TEXT,
    );

    // 玩家2操作说明 - 红色
    common::spawn_simple_text_with_marker(
        commands,
        p2_text.to_string(),
        &font,
        FONT_SIZE_INSTRUCTION,
        Vec3::new(0.0, CONTROLS_P2_Y, Z_UI_TEXT),
        COLOR_RED,
        StartScreenUI,
        Z_UI_TEXT,
    );

    // 通用操作说明 - 深灰色
    common::spawn_simple_text_with_marker(
        commands,
        general_text.to_string(),
        &font,
        FONT_SIZE_INSTRUCTION,
        Vec3::new(0.0, CONTROLS_GENERAL_Y, Z_UI_TEXT),
        COLOR_DARK_GRAY,
        StartScreenUI,
        Z_UI_TEXT,
    );
}

/// 生成完整的开始界面
pub fn spawn_start_screen(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    font_resources: Res<GameTextureResources>,
    asset_server: Res<AssetServer>,
    language: Res<Language>,
) {
    // 检查资源是否已加载完成
    // 注意：atlas_layouts.background 是动态创建的资源，不需要检查 is_loaded
    if !common::ensure_assets_loaded(
        &asset_server,
        &[&font_resources.en, &font_resources.cn],
        &[&texture_resources.background],
    ) {
        return;
    }

    // 根据语言选择字体
    let font = common::get_font(&font_resources, *language);

    // 添加动态背景
    spawn_start_screen_background(&mut commands, &texture_resources, &atlas_layouts);

    // 添加标题文字
    spawn_start_screen_title(&mut commands, font.clone(), *language);

    // 添加操作说明
    spawn_start_screen_instructions(&mut commands, font, *language);
}

/// 生成关于界面
pub fn spawn_about_screen(
    mut commands: Commands,
    font_resources: Res<GameTextureResources>,
    asset_server: Res<AssetServer>,
    language: Res<Language>,
) {
    // 加载自定义字体
    let font = common::get_font(&font_resources, *language);

    // 添加白色背景覆盖
    common::spawn_overlay_background(&mut commands, COLOR_WHITE, 0.0, WINDOW_SIZE, AboutUI);

    // 添加标题
    common::spawn_simple_text_with_marker(
        &mut commands,
        ABOUT_TITLE.get(*language).to_string(),
        &font,
        FONT_SIZE_CREDITS_TITLE,
        Vec3::new(0.0, ABOUT_TITLE_Y, Z_UI_TEXT),
        COLOR_BLACK,
        AboutUI,
        Z_UI_TEXT,
    );

    // 显示信息
    common::spawn_text_with_justify_and_marker(
        &mut commands,
        ABOUT_TEXT.get(*language).to_string(),
        &font,
        FONT_SIZE_INSTRUCTION,
        Vec3::new(0.0, ABOUT_TEXT_Y, Z_UI_TEXT),
        COLOR_BLACK,
        AboutUI,
        bevy::text::Justify::Center,
        Z_UI_TEXT,
    );

    // 添加收款码文案
    common::spawn_text_with_justify_and_marker(
        &mut commands,
        ABOUT_SUPPORT.get(*language).to_string(),
        &font,
        FONT_SIZE_MEDIUM,
        Vec3::new(0.0, ABOUT_SUPPORT_Y, Z_UI_TEXT),
        COLOR_BLACK,
        AboutUI,
        bevy::text::Justify::Center,
        Z_UI_TEXT,
    );

    // 加载收款码图片
    let alipay_image: Handle<Image> = asset_server.load(IMAGE_ALIPAY);
    let wechat_image: Handle<Image> = asset_server.load(IMAGE_WECHAT);

    // 图片大小统一为 400x400 像素
    let qr_size = PAYMENT_CODE_SIZE;

    // 支付宝收款码（向下平移10像素）
    commands.spawn((
        AboutUI,
        Sprite {
            image: alipay_image,
            custom_size: Some(Vec2::new(qr_size, qr_size)),
            ..default()
        },
        Transform::from_xyz(ABOUT_QR_ALIPAY_X, ABOUT_QR_Y, Z_UI_TEXT),
    ));

    // 支付宝标签
    common::spawn_simple_text_with_marker(
        &mut commands,
        PAYMENT_METHOD_ALIPAY.get(*language).to_string(),
        &font,
        FONT_SIZE_SMALL,
        Vec3::new(ABOUT_QR_ALIPAY_X, ABOUT_PAYMENT_LABEL_Y, Z_UI_TEXT),
        COLOR_BLACK,
        AboutUI,
        Z_UI_TEXT,
    );

    // 微信收款码
    commands.spawn((
        AboutUI,
        Sprite {
            image: wechat_image,
            custom_size: Some(Vec2::new(qr_size, qr_size)),
            ..default()
        },
        Transform::from_xyz(ABOUT_QR_WECHAT_X, ABOUT_QR_Y, Z_UI_TEXT),
    ));

    // 微信标签
    common::spawn_simple_text_with_marker(
        &mut commands,
        PAYMENT_METHOD_WECHAT.get(*language).to_string(),
        &font,
        FONT_SIZE_SMALL,
        Vec3::new(ABOUT_QR_WECHAT_X, ABOUT_PAYMENT_LABEL_Y, Z_UI_TEXT),
        COLOR_BLACK,
        AboutUI,
        Z_UI_TEXT,
    );

    // 添加返回提示
    common::spawn_simple_text_with_marker(
        &mut commands,
        ABOUT_RETURN.get(*language).to_string(),
        &font,
        FONT_SIZE_UI,
        Vec3::new(0.0, ABOUT_RETURN_Y, Z_UI_TEXT),
        COLOR_BLACK,
        AboutUI,
        Z_UI_TEXT,
    );
}

/// 处理关于界面的输入
pub fn handle_about_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Space 键返回开始界面
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::StartScreen);
    }
}

/// 生成致谢界面
pub fn spawn_credits_screen(
    mut commands: Commands,
    font_resources: Res<GameTextureResources>,
    language: Res<Language>,
) {
    // 加载自定义字体
    let font = common::get_font(&font_resources, *language);

    // 添加白色背景覆盖
    common::spawn_overlay_background(&mut commands, COLOR_WHITE, 0.0, WINDOW_SIZE, CreditsUI);

    // 添加标题
    common::spawn_simple_text_with_marker(
        &mut commands,
        CREDITS_TITLE.get(*language).to_string(),
        &font,
        FONT_SIZE_MENU,
        Vec3::new(0.0, CREDITS_TITLE_Y, Z_UI_TEXT),
        COLOR_BLACK,
        CreditsUI,
        Z_UI_TEXT,
    );

    // 使用多行文本显示素材来源
    common::spawn_text_with_justify_and_marker(
        &mut commands,
        CREDITS_TEXT.get(*language).to_string(),
        &font,
        FONT_SIZE_INSTRUCTION,
        Vec3::new(CREDITS_TEXT_X, CREDITS_TEXT_Y, Z_UI_TEXT),
        COLOR_BLACK,
        CreditsUI,
        bevy::text::Justify::Left,
        Z_UI_TEXT,
    );

    // 添加返回提示
    common::spawn_simple_text_with_marker(
        &mut commands,
        CREDITS_RETURN.get(*language).to_string(),
        &font,
        FONT_SIZE_UI,
        Vec3::new(0.0, CREDITS_RETURN_Y, Z_UI_TEXT),
        COLOR_BLACK,
        CreditsUI,
        Z_UI_TEXT,
    );
}

/// 处理致谢界面的输入
pub fn handle_credits_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Space 键返回开始界面
    if keyboard_input.just_pressed(KeyCode::Space) {
        next_state.set(GameState::StartScreen);
    }
}

/// 处理开始界面的输入
pub fn handle_start_screen_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut menu_selection: ResMut<CurrentMenuSelection>,
    mut game_mode: ResMut<GameMode>,
    mut language: ResMut<Language>,
    #[cfg(not(target_arch = "wasm32"))] mut app_exit: MessageWriter<AppExit>,
) {
    // Esc 键退出游戏（Web 端无效果，桌面端退出）
    #[cfg(not(target_arch = "wasm32"))]
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let _ = app_exit.write(AppExit::Success);
    }

    // 菜单选项数量因平台而异
    #[cfg(not(target_arch = "wasm32"))]
    let menu_count = 7;
    #[cfg(target_arch = "wasm32")]
    let menu_count = 6;

    // 使用通用菜单导航函数
    common::handle_menu_navigation(
        &keyboard_input,
        &mut menu_selection.selected_index,
        menu_count - 1, // 最大索引
        common::NavigationWrap::WrapAround,
    );

    // Space 键确认选择
    if keyboard_input.just_pressed(KeyCode::Space) {
        #[cfg(not(target_arch = "wasm32"))]
        match menu_selection.selected_index {
            0 => {
                *game_mode = GameMode::OnePlayer;
                next_state.set(GameState::FadingOut); // 1 Player / 单人游戏
            }
            1 => {
                *game_mode = GameMode::TwoPlayers;
                next_state.set(GameState::FadingOut); // 2 Player / 双人对战
            }
            2 => {
                next_state.set(GameState::LevelEditor); // Level Editor / 关卡编辑器
            }
            3 => {
                // 切换语言
                *language = match *language {
                    Language::Chinese => Language::English,
                    Language::English => Language::Chinese,
                };
                // 语言切换后重新生成菜单以更新文本
            }
            4 => {
                next_state.set(GameState::About); // About / 关于
            }
            5 => {
                next_state.set(GameState::Credits); // Credits / 制作人员
            }
            6 => {
                // EXIT / 退出（桌面端退出）
                let _ = app_exit.write(AppExit::Success);
            }
            _ => {}
        }

        #[cfg(target_arch = "wasm32")]
        match menu_selection.selected_index {
            0 => {
                *game_mode = GameMode::OnePlayer;
                next_state.set(GameState::FadingOut); // 1 Player / 单人游戏
            }
            1 => {
                *game_mode = GameMode::TwoPlayers;
                next_state.set(GameState::FadingOut); // 2 Player / 双人对战
            }
            2 => {
                // 切换语言
                *language = match *language {
                    Language::Chinese => Language::English,
                    Language::English => Language::Chinese,
                };
                // 语言切换后重新生成菜单以更新文本
            }
            3 => {
                next_state.set(GameState::About); // About / 关于
            }
            4 => {
                next_state.set(GameState::Credits); // Credits / 制作人员
            }
            5 => {
                // EXIT / 退出（Web 端返回菜单）
                next_state.set(GameState::StartScreen);
            }
            _ => {}
        }
    }
}

/// 更新菜单选项的颜色
pub fn update_option_colors(
    menu_selection: Res<CurrentMenuSelection>,
    mut text_query: Query<(&MenuOption, &mut TextColor)>,
) {
    for (option, mut text_color) in &mut text_query {
        if option.index == menu_selection.selected_index {
            // 选中的选项使用黄色
            text_color.0 = COLOR_YELLOW;
        } else {
            text_color.0 = COLOR_WHITE; // 白色
        }
    }
}
