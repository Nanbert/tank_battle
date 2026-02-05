//! 菜单界面模块
//!
//! 处理开始界面、关于界面、致谢界面等菜单相关功能

use bevy::app::AppExit;
use bevy::prelude::*;

#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;

/// 生成开始界面的背景动画
pub fn spawn_start_screen_background(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
) {
    commands.spawn((
        StartScreenUI,
        Sprite {
            image: texture_resources.background.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: atlas_layouts.background.clone(),
                index: 0,
            }),
            custom_size: Some(Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        AnimationIndices {
            first: 0,
            last: BACKGROUND_TOTAL_FRAMES - 1,
        },
        AnimationTimer(Timer::from_seconds(BACKGROUND_ANIMATION_FRAME, TimerMode::Repeating)),
    ));
}

/// 生成开始界面的标题和菜单选项
pub fn spawn_start_screen_title(commands: &mut Commands, font_en: Handle<Font>, font_cn: Handle<Font>, language: Language) {
    let (title, menu_text, use_cn_font) = match language {
        Language::Chinese => ("钢铁指令", vec![
            "单人游戏",
            "双人对战",
            "语言 / Language",  // 双语显示
            "关于",
            "制作人员",
            "退出",
        ], true),
        Language::English => ("Steel Command", vec![
            "1 Player",
            "2 Player",
            "语言 / Language",  // 双语显示
            "About",
            "Credits",
            "EXIT",
        ], false),
    };

    let title_font = if use_cn_font { font_cn.clone() } else { font_en.clone() };
    let menu_font = if use_cn_font { font_cn.clone() } else { font_en.clone() };

    commands.spawn((
        StartScreenUI,
        Text2d(title.to_string()),
        TextFont {
            font_size: FONT_SIZE_TITLE,
            font: title_font,
            ..default()
        },
        TextColor(COLOR_RED),
        Transform::from_xyz(0.0, 550.0, 1.0),
    ));

    // 菜单选项，从上到下 0-5
    let y_positions = [250.0, 150.0, 50.0, -50.0, -150.0, -250.0];
    for (i, text) in menu_text.iter().enumerate() {
        // 语言选项使用英文字体（因为文本是双语的），其他选项使用对应语言的字体
        let item_font = if i == 2 {
            font_en.clone()
        } else {
            menu_font.clone()
        };

        commands.spawn((
            StartScreenUI,
            Text2d(text.to_string()),
            TextFont {
                font_size: FONT_SIZE_MENU,
                font: item_font,
                ..default()
            },
            TextColor(if i == 0 { COLOR_YELLOW } else { COLOR_WHITE }),
            Transform::from_xyz(0.0, y_positions[i], 1.0),
            MenuOption { index: i },
        ));
    }
}

/// 生成开始界面的操作说明
pub fn spawn_start_screen_instructions(commands: &mut Commands, font_en: &Handle<Font>, font_cn: &Handle<Font>, language: Language) {
    let (p1_text, p2_text, general_text, use_cn_font) = match language {
        Language::Chinese => (
            "玩家1 (李云龙): WASD 移动 | J 射击 | I 召回 | K 冲刺 | L 激光",
            "玩家2 (楚云飞): 方向键 移动 | 1 射击 | 4 召回 | 2 冲刺 | 3 激光",
            "W/S 选择 | SPACE 确认/暂停 | ESC 退出",
            true,
        ),
        Language::English => (
            "Player 1 (Li Yun Long): WASD to move | J to shoot | I to recall | K to dash | L to laser",
            "Player 2 (Chu Yun Fei): Arrow Keys to move | 1 to shoot | 4 to recall | 2 to dash | 3 to laser",
            "W/S to select | SPACE to select/pause | ESC to exit",
            false,
        ),
    };

    let font = if use_cn_font { font_cn.clone() } else { font_en.clone() };

    // 玩家1操作说明
    commands.spawn((
        StartScreenUI,
        Text2d(p1_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(COLOR_BLUE), // 蓝色
        Transform::from_xyz(0.0, -450.0, 1.0),
    ));

    // 玩家2操作说明
    commands.spawn((
        StartScreenUI,
        Text2d(p2_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(COLOR_RED), // 红色
        Transform::from_xyz(0.0, -480.0, 1.0),
    ));

    // 添加通用操作说明
    commands.spawn((
        StartScreenUI,
        Text2d(general_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INFO,
            font,
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(COLOR_YELLOW), // 黄色
        Transform::from_xyz(0.0, -510.0, 1.0),
    ));
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
    let font_en_loaded = asset_server.is_loaded(&font_resources.en);
    let font_cn_loaded = asset_server.is_loaded(&font_resources.cn);
    let bg_loaded = asset_server.is_loaded(&texture_resources.background);

    // 如果资源未完全加载，跳过生成（下次 Update 会重试）
    // 注意：atlas_layouts.background 是动态创建的资源，不需要检查 is_loaded
    if !font_en_loaded || !font_cn_loaded || !bg_loaded {
        return;
    }

    // 添加动态背景
    spawn_start_screen_background(&mut commands, &texture_resources, &atlas_layouts);

    // 添加标题文字
    spawn_start_screen_title(&mut commands, font_resources.en.clone(), font_resources.cn.clone(), *language);

    // 添加操作说明
    spawn_start_screen_instructions(&mut commands, &font_resources.en, &font_resources.cn, *language);
}

/// 生成关于界面
pub fn spawn_about_screen(mut commands: Commands, font_resources: Res<GameTextureResources>, asset_server: Res<AssetServer>, language: Res<Language>) {
    // 加载自定义字体
    let cn_font = font_resources.cn.clone();
    let en_font = font_resources.en.clone();

    let (title, about_text, support_text, return_text, use_cn_font) = match *language {
        Language::Chinese => (
            "关于",
            "开发者: 南敬文\n\n        邮箱: 2726905171@qq.com\n\n        版权所有 (c) 2026 南敬文\n        保留所有权利\n\n        本游戏是受《坦克大战 1990》启发的坦克对战游戏.\n        使用 Rust 和 Bevy 游戏引擎开发.\n\n        特别感谢 iFlow 提供的宝贵帮助.\n\n        许可证: MIT 许可证",
            "如果你喜欢这个游戏,\n请给我买杯咖啡! (咖啡是程序员的燃料)",
            "按 SPACE 返回",
            true,
        ),
        Language::English => (
            "About",
            "Developer: Nanbert\n\n        Email: 2726905171@qq.com\n\n        Copyright © 2026 Nanbert\n        All rights reserved.\n\n        This is a tank battle game inspired by Battle City 1990.\n        Built with Rust and Bevy game engine.\n\n        Special thanks to iFlow for invaluable assistance.\n\n        License: MIT License",
            "If you enjoyed the game,\nplease buy me a coffee! ☕️\n(Caffeine is a programmer's fuel)",
            "Press SPACE to return",
            false,
        ),
    };

    let title_font = if use_cn_font { cn_font.clone() } else { en_font.clone() };
    let body_font = if use_cn_font { cn_font.clone() } else { en_font.clone() };
    let label_font = if use_cn_font { cn_font.clone() } else { en_font.clone() };
    let return_font = if use_cn_font { cn_font.clone() } else { en_font.clone() };

    // 添加白色背景覆盖
    commands.spawn((
        AboutUI,
        Sprite {
            color: COLOR_WHITE,
            custom_size: Some(Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 添加标题（位置不变）
    commands.spawn((
        AboutUI,
        Text2d(title.to_string()),
        TextFont {
            font_size: FONT_SIZE_CREDITS_TITLE,
            font: title_font,
            ..default()
        },
        TextColor(COLOR_BLACK),
        Transform::from_xyz(0.0, 600.0, 1.0),
    ));

    // 显示信息（向下平移10像素）
    commands.spawn((
        AboutUI,
        Text2d(about_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: body_font.clone(),
            ..default()
        },
        TextColor(COLOR_BLACK),
        TextLayout::new_with_justify(bevy::text::Justify::Center),
        Transform::from_xyz(0.0, 340.0, 1.0),
    ));

    // 添加收款码文案（向下平移10像素）
    commands.spawn((
        AboutUI,
        Text2d(support_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_MEDIUM,
            font: body_font.clone(),
            ..default()
        },
        TextColor(COLOR_BLACK),
        TextLayout::new_with_justify(bevy::text::Justify::Center),
        Transform::from_xyz(0.0, 40.0, 1.0),
    ));

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
        Transform::from_xyz(-250.0, -260.0, 1.0),
    ));

    // 支付宝标签（向下平移10像素）
    commands.spawn((
        AboutUI,
        Text2d(match *language {
            Language::Chinese => "支付宝",
            Language::English => "Alipay",
        }.to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            font: label_font.clone(),
            ..default()
        },
        TextColor(COLOR_BLACK),
        Transform::from_xyz(-250.0, -480.0, 1.0),
    ));

    // 微信收款码（向下平移10像素）
    commands.spawn((
        AboutUI,
        Sprite {
            image: wechat_image,
            custom_size: Some(Vec2::new(qr_size, qr_size)),
            ..default()
        },
        Transform::from_xyz(250.0, -260.0, 1.0),
    ));

    // 微信标签（向下平移10像素）
    commands.spawn((
        AboutUI,
        Text2d(match *language {
            Language::Chinese => "微信",
            Language::English => "WeChat",
        }.to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            font: label_font,
            ..default()
        },
        TextColor(COLOR_BLACK),
        Transform::from_xyz(250.0, -480.0, 1.0),
    ));

    // 添加返回提示（向下平移10像素）
    commands.spawn((
        AboutUI,
        Text2d(return_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_MEDIUM,
            font: return_font,
            ..default()
        },
        TextColor(COLOR_BLACK),
        Transform::from_xyz(0.0, -560.0, 1.0),
    ));
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
pub fn spawn_credits_screen(mut commands: Commands, font_resources: Res<GameTextureResources>) {
    // 加载自定义字体
    let custom_font = font_resources.en.clone();

    // 添加白色背景覆盖
    commands.spawn((
        CreditsUI,
        Sprite {
            color: COLOR_WHITE,
            custom_size: Some(Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 添加标题
    commands.spawn((
        CreditsUI,
        Text2d("Credits".to_string()),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(COLOR_BLACK),
        Transform::from_xyz(0.0, 500.0, 1.0),
    ));

    // 使用多行文本显示素材来源
    let credits_text = "Asset Credits\n\n\n        OpenGameArt.org:\n        • Bubbles by HorrorPen (CC-BY 3.0)\n        • Explosion by Sinestesia (CC0 1.0)\n        • Laser by netcake3 (CC-BY-SA 3.0/4.0)\n        • Enemy Born by Skorpio (CC-BY 3.0)\n        • Fire Effect by JoesAlotofthings (CC-BY 4.0)\n        • Player/Enemy Tanks & Barrels by irmirx (CC-BY 3.0)\n        • Smoke by Skorpio (CC-BY 3.0)\n        • Steel Hit by Sinestesia (CC0 1.0)\n        • Bullets by Wenrexa (CC0 1.0)\n\n\n        通义千问 (AI Generated):\n        • Background, Music Notes (CC0 1.0)\n        • Maps (Brick, Steel, Sea, Tree, Barrier) (CC0 1.0)\n        • Power-ups (10 types) (CC0 1.0)\n        • Track Train (CC0 1.0)\n        • Avatars & Commander (CC0 1.0)\n\n\n        Fonts:\n        • ChelaOne by Latinotype\n        • Corben\n        • Matemasie\n        • LiuHuanKaTongShouShu by 刘欢\n\n\n        See COPYRIGHT file for full details.";

    commands.spawn((
        CreditsUI,
        Text2d(credits_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(COLOR_BLACK),
        TextLayout::new_with_justify(bevy::text::Justify::Left),
        Transform::from_xyz(-400.0, 100.0, 1.0),
    ));

    // 添加返回提示
    commands.spawn((
        CreditsUI,
        Text2d("Press SPACE to return".to_string()),
        TextFont {
            font_size: FONT_SIZE_UI,
            font: custom_font,
            ..default()
        },
        TextColor(COLOR_BLACK),
        Transform::from_xyz(0.0, -500.0, 1.0),
    ));
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
    mut app_exit: MessageWriter<AppExit>,
) {
    // Esc 键退出游戏
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let _ = app_exit.write(AppExit::Success);
    }

    // W 键向上选择
    if keyboard_input.just_pressed(KeyCode::KeyW) {
        menu_selection.selected_index = if menu_selection.selected_index > 0 {
            menu_selection.selected_index - 1
        } else {
            5
        };
    }
    // S 键向下选择
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        menu_selection.selected_index = (menu_selection.selected_index + 1) % 6;
    }
    // Space 键确认选择
    if keyboard_input.just_pressed(KeyCode::Space) {
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
                let _ = app_exit.write(AppExit::Success);
            } // EXIT / 退出
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

/// 开始界面背景动画系统
pub fn animate_start_screen_background(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices), With<StartScreenUI>>,
) {
    for (mut timer, mut sprite, indices) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished()
            && let Some(ref mut atlas) = sprite.texture_atlas {
                atlas.index = if atlas.index < indices.last {
                    atlas.index + 1
                } else {
                    indices.first
                };
            }
    }
}
