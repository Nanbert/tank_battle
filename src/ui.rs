//! UI 模块
//!
//! 处理所有用户界面相关的功能，包括开始界面、暂停界面、游戏结束界面、关于界面、致谢界面等

use bevy::prelude::*;
use bevy::app::AppExit;
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[allow(clippy::wildcard_imports)]
use crate::constants::*;
#[allow(clippy::wildcard_imports)]
use crate::resources::*;

/// 加载开始界面的动画资源
pub fn load_start_animation_assets(
    asset_server: &Res<AssetServer>,
    animation_frames: &mut ResMut<StartAnimationFrames>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) {
    // 使用3个较小的精灵图加载背景动画（15帧，每部分5帧）
    // 拆分以支持GPU纹理尺寸限制（最大16384）
    let background_texture1: Handle<Image> = asset_server.load("background/background_sprite_part1.png");
    let background_texture2: Handle<Image> = asset_server.load("background/background_sprite_part2.png");
    let background_texture3: Handle<Image> = asset_server.load("background/background_sprite_part3.png");

    let background_tile_size = UVec2::new(BACKGROUND_ANIMATION_TILE_WIDTH as u32, BACKGROUND_ANIMATION_TILE_HEIGHT as u32); // 每帧的尺寸（窗口大小）

    // 创建3个纹理图集，每个5帧
    let atlas1 = TextureAtlasLayout::from_grid(background_tile_size, 5, 1, None, None);
    let atlas2 = TextureAtlasLayout::from_grid(background_tile_size, 5, 1, None, None);
    let atlas3 = TextureAtlasLayout::from_grid(background_tile_size, 5, 1, None, None);

    let layout1 = texture_atlas_layouts.add(atlas1);
    let layout2 = texture_atlas_layouts.add(atlas2);
    let layout3 = texture_atlas_layouts.add(atlas3);

    // 存储到资源中
    animation_frames.texture_atlas_layouts = vec![layout1, layout2, layout3];
    animation_frames.textures = vec![background_texture1, background_texture2, background_texture3];
}

/// 生成开始界面的背景动画
pub fn spawn_start_screen_background(
    commands: &mut Commands,
    animation_frames: &ResMut<StartAnimationFrames>,
) {
    let animation_indices = AnimationIndices { first: 0, last: 14 };

    // 使用第一个纹理和图集初始化
    let texture_atlas_layout = animation_frames.texture_atlas_layouts[0].clone();
    let texture = animation_frames.textures[0].clone();

    commands.spawn((
        StartScreenUI,
        Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: 0,
            }),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
        animation_indices,
        AnimationTimer(Timer::from_seconds(ANIMATION_FRAME_START_BACKGROUND, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));
}

/// 生成开始界面的标题和菜单选项
pub fn spawn_start_screen_title(
    commands: &mut Commands,
    font: Handle<Font>,
) {
    commands.spawn((
        StartScreenUI,
        Text2d("For Communism!!".to_string()),
        TextFont {
            font_size: FONT_SIZE_TITLE,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 400.0, 1.0),
    ));

    // 1 Player 选项
    commands.spawn((
        StartScreenUI,
        Text2d("1 Player".to_string()),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 初始选中，黄色
        Transform::from_xyz(0.0, 50.0, 1.0),
        MenuOption { index: 0 },
    ));

    // 2 Player 选项
    commands.spawn((
        StartScreenUI,
        Text2d("2 Player".to_string()),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -50.0, 1.0),
        MenuOption { index: 1 },
    ));

    // About 选项
    commands.spawn((
        StartScreenUI,
        Text2d("About".to_string()),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -150.0, 1.0),
        MenuOption { index: 2 },
    ));

    // Credits 选项
    commands.spawn((
        StartScreenUI,
        Text2d("Credits".to_string()),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -250.0, 1.0),
        MenuOption { index: 3 },
    ));

    // EXIT 选项
    commands.spawn((
        StartScreenUI,
        Text2d("EXIT".to_string()),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)), // 白色
        Transform::from_xyz(0.0, -350.0, 1.0),
        MenuOption { index: 4 },
    ));
}

/// 生成开始界面的操作说明
pub fn spawn_start_screen_instructions(commands: &mut Commands, font: &Handle<Font>) {
    // 玩家1操作说明
    commands.spawn((
        StartScreenUI,
        Text2d("Player 1 (Li Yun Long): WASD to move | J to shoot | I to recall | K to dash | L to laser".to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(Color::srgb(0.0, 0.5, 1.0)), // 蓝色
        Transform::from_xyz(0.0, -450.0, 1.0),
    ));

    // 玩家2操作说明
    commands.spawn((
        StartScreenUI,
        Text2d("Player 2 (Chu Yun Fei): Arrow Keys to move | 1 to shoot | 4 to recall | 2 to dash | 3 to laser".to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)), // 红色
        Transform::from_xyz(0.0, -480.0, 1.0),
    ));

    // 添加通用操作说明
    commands.spawn((
        StartScreenUI,
        Text2d("W/S to select | SPACE to select/pause | ESC to exit".to_string()),
        TextFont {
            font_size: FONT_SIZE_INFO,
            font: font.clone(),
            font_smoothing: default(),
            line_height: default(),
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色
        Transform::from_xyz(0.0, -510.0, 1.0),
    ));
}

/// 生成完整的开始界面
pub fn spawn_start_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut animation_frames: ResMut<StartAnimationFrames>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // 加载所有动画帧
    load_start_animation_assets(&asset_server, &mut animation_frames, &mut texture_atlas_layouts);

    // 添加动画背景
    spawn_start_screen_background(&mut commands, &animation_frames);

    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load(FONT_EN);

    // 添加标题文字
    spawn_start_screen_title(&mut commands, custom_font.clone());

    // 添加操作说明
    spawn_start_screen_instructions(&mut commands, &custom_font);
}

/// 生成关于界面
pub fn spawn_about_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load(FONT_EN);

    // 添加白色背景覆盖
    commands.spawn((
        AboutUI,
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // 添加标题
    commands.spawn((
        AboutUI,
        Text2d("About".to_string()),
        TextFont {
            font_size: FONT_SIZE_CREDITS_TITLE,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 600.0, 1.0),
    ));

    // 使用多行文本显示所有信息
    let about_text = "Nanbert\n\n\
        Email: 2726905171@qq.com\n\n\
        Copyright © 2025 Nanbert\n\
        All rights reserved.\n\n\
        This is a tank battle game inspired by Battle City 1990.\n\
        Built with Rust and Bevy game engine.\n\n\
        Special thanks to iFlow for invaluable assistance.\n\n\
        License: MIT License";

    commands.spawn((
        AboutUI,
        Text2d(about_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        TextLayout::new_with_justify(bevy::text::Justify::Center),
        Transform::from_xyz(0.0, 350.0, 1.0),
    ));

    // 添加收款码文案
    let support_text = "If you enjoyed the game,\nplease buy me a coffee! ☕️\n(Caffeine is a programmer's fuel)";

    commands.spawn((
        AboutUI,
        Text2d(support_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_MEDIUM,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        TextLayout::new_with_justify(bevy::text::Justify::Center),
        Transform::from_xyz(0.0, 50.0, 1.0),
    ));

    // 加载收款码图片
    let alipay_image: Handle<Image> = asset_server.load("alipay.png");
    let wechat_image: Handle<Image> = asset_server.load("wechat.png");

    // 图片大小统一为 400x400 像素
    let qr_size = PAYMENT_CODE_SIZE;

    // 支付宝收款码
    commands.spawn((
        AboutUI,
        Sprite {
            image: alipay_image,
            custom_size: Some(Vec2::new(qr_size, qr_size)),
            ..default()
        },
        Transform::from_xyz(-250.0, -250.0, 1.0),
    ));

    // 支付宝标签
    commands.spawn((
        AboutUI,
        Text2d("Alipay".to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(-250.0, -470.0, 1.0),
    ));

    // 微信收款码
    commands.spawn((
        AboutUI,
        Sprite {
            image: wechat_image,
            custom_size: Some(Vec2::new(qr_size, qr_size)),
            ..default()
        },
        Transform::from_xyz(250.0, -250.0, 1.0),
    ));

    // 微信标签
    commands.spawn((
        AboutUI,
        Text2d("WeChat".to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(250.0, -470.0, 1.0),
    ));

// 添加返回提示
    commands.spawn((
        AboutUI,
        Text2d("Press SPACE to return".to_string()),
        TextFont {
            font_size: FONT_SIZE_MEDIUM,
            font: custom_font,
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, -550.0, 1.0),
    ));
}

/// 销毁关于界面
pub fn despawn_about_screen(
    mut commands: Commands,
    query: Query<Entity, With<AboutUI>>,
) {
    for entity in query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
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
    asset_server: Res<AssetServer>,
) {
    // 加载自定义字体
    let custom_font: Handle<Font> = asset_server.load(FONT_EN);

    // 添加白色背景覆盖
    commands.spawn((
        CreditsUI,
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
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
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 500.0, 1.0),
    ));

    // 使用多行文本显示素材来源
    let credits_text = "Asset Credits\n\n\
        OpenGameArt.org:\n\
        • Bubbles by HorrorPen (CC-BY 3.0)\n\
        • Explosion by Sinestesia (CC0 1.0)\n\
        • Laser by netcake3 (CC-BY-SA 3.0/4.0)\n\
        • Enemy Born by JoesAlotofthings (CC-BY 4.0)\n\
        • Player/Enemy Tanks by irmirx (CC-BY 3.0)\n\
        • Smoke by Skorpio (CC-BY 3.0)\n\
        • Steel Hit by Sinestesia (CC0 1.0)\n\
        • Bullets by Wenrexa (CC0 1.0)\n\n\
        通义千问 (AI Generated):\n\
        • Background, Music Notes (CC0 1.0)\n\
        • Maps (Brick, Steel, Sea, Tree, Barrier) (CC0 1.0)\n\
        • Power-ups (10 types) (CC0 1.0)\n\
        • Avatars & Commander (CC0 1.0)\n\n\
        Fonts:\n\
        • ChelaOne by Latinotype\n\
        • Corben\n\
        • Matemasie\n\
        • LiuHuanKaTongShouShu by 刘欢\n\n\
        See COPYRIGHT file for full details.";

    commands.spawn((
        CreditsUI,
        Text2d(credits_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_INSTRUCTION,
            font: custom_font.clone(),
            ..default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
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
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, -500.0, 1.0),
    ));
}

/// 销毁致谢界面
pub fn despawn_credits_screen(
    mut commands: Commands,
    query: Query<Entity, With<CreditsUI>>,
) {
    for entity in query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
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
            4
        };
    }
    // S 键向下选择
    if keyboard_input.just_pressed(KeyCode::KeyS) {
        menu_selection.selected_index = (menu_selection.selected_index + 1) % 5;
    }
    // Space 键确认选择
    if keyboard_input.just_pressed(KeyCode::Space) {
        match menu_selection.selected_index {
            0 => {
                *game_mode = GameMode::OnePlayer;
                next_state.set(GameState::FadingOut); // 1 Player
            }
            1 => {
                *game_mode = GameMode::TwoPlayers;
                next_state.set(GameState::FadingOut); // 2 Player
            }
            2 => {
                next_state.set(GameState::About); // About
            }
            3 => {
                next_state.set(GameState::Credits); // Credits
            }
            4 => {
                let _ = app_exit.write(AppExit::Success);
            } // EXIT
            _ => {}
        }
    }
}

/// 动画化开始界面的背景
pub fn animate_start_screen(
    time: Res<Time>,
    mut query: Query<(&AnimationIndices, &mut AnimationTimer, &mut Sprite, &mut CurrentAnimationFrame), With<StartScreenUI>>,
    animation_frames: Res<StartAnimationFrames>,
) {
    for (indices, mut timer, mut sprite, mut current_frame) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            let current = current_frame.0;
            let next_index = if current == indices.last {
                indices.first
            } else {
                current + 1
            };
            current_frame.0 = next_index;

            // 计算使用哪个纹理图集（每个图集5帧）
            let atlas_index = next_index / 5;
            let frame_in_atlas = next_index % 5;

            // 更新纹理和图集
            if atlas_index < animation_frames.textures.len() {
                sprite.image = animation_frames.textures[atlas_index].clone();
                if let Some(texture_atlas) = &mut sprite.texture_atlas {
                    texture_atlas.layout = animation_frames.texture_atlas_layouts[atlas_index].clone();
                    texture_atlas.index = frame_in_atlas;
                }
            }
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
            text_color.0 = Color::srgb(1.0, 1.0, 0.0);
        } else {
            text_color.0 = Color::srgb(1.0, 1.0, 1.0); // 白色
        }
    }
}

/// 生成暂停界面
pub fn spawn_pause_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut player_velocity_query: Query<&mut Velocity, With<PlayerTank>>,
    mut enemy_velocity_query: Query<&mut Velocity, (With<EnemyTank>, Without<PlayerTank>)>,
) {
    let font: Handle<Font> = asset_server.load(FONT_EN);

    // 停止玩家坦克的移动
    for mut velocity in &mut player_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    // 停止敌方坦克的移动
    for mut velocity in &mut enemy_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    commands.spawn((
        PauseUI,
        Text2d("PAUSED".to_string()),
        TextFont {
            font_size: FONT_SIZE_GAME_OVER,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)),
        Transform::from_xyz(0.0, 0.0, Z_UI),
    ));

    commands.spawn((
        PauseUI,
        Text2d("Press SPACE to resume | B to menu | ESC to exit".to_string()),
        TextFont {
            font_size: FONT_SIZE_UI,
            font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -100.0, Z_UI),
    ));
}

/// 销毁暂停界面
pub fn despawn_pause_ui(mut commands: Commands, query: Query<Entity, With<PauseUI>>) {
    for entity in query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
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
    asset_server: Res<AssetServer>,
    mut player_velocity_query: Query<&mut Velocity, With<PlayerTank>>,
    mut enemy_velocity_query: Query<&mut Velocity, (With<EnemyTank>, Without<PlayerTank>)>,
) {
    let font: Handle<Font> = asset_server.load(FONT_EN);

    // 停止玩家坦克的移动
    for mut velocity in &mut player_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    // 停止敌方坦克的移动
    for mut velocity in &mut enemy_velocity_query {
        velocity.linvel = Vec2::ZERO;
    }

    commands.spawn((
        GameOverUI,
        Text2d("GAME OVER".to_string()),
        TextFont {
            font_size: FONT_SIZE_GAME_OVER,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)),
        Transform::from_xyz(0.0, 100.0, Z_UI),
    ));

    // Restart 选项
    commands.spawn((
        GameOverUI,
        Text2d("RESTART".to_string()),
        TextFont {
            font_size: FONT_SIZE_OPTION,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, 0.0, Z_UI),
        MenuOption { index: 0 },
    ));

    // Back to Menu 选项
    commands.spawn((
        GameOverUI,
        Text2d("BACK TO MENU".to_string()),
        TextFont {
            font_size: FONT_SIZE_OPTION,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -60.0, Z_UI),
        MenuOption { index: 1 },
    ));

    // Exit 选项
    commands.spawn((
        GameOverUI,
        Text2d("EXIT".to_string()),
        TextFont {
            font_size: FONT_SIZE_OPTION,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -120.0, Z_UI),
        MenuOption { index: 2 },
    ));

    // 操作说明
    commands.spawn((
        GameOverUI,
        Text2d("W/S to select | SPACE to confirm".to_string()),
        TextFont {
            font_size: FONT_SIZE_UI,
            font,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(0.0, -200.0, Z_UI),
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
                // Restart: 重置游戏状态并重新开始
                next_state.set(GameState::Playing);
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
    for entity in query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}

/// 清理开始界面的UI元素
pub fn cleanup_start_screen_ui(
    mut commands: Commands,
    sprite_query: Query<(Entity, &mut Sprite), With<StartScreenUI>>,
    text_query: Query<(Entity, &mut TextColor, Option<&MenuOption>), With<StartScreenUI>>,
) {
    for (entity, _) in sprite_query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
    for (entity, _, _) in text_query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}

/// 更新 Sprite 的透明度
pub fn update_sprite_alpha(alpha: f32, sprite: &mut Sprite) {
    let linear = sprite.color.to_linear();
    sprite.color = Color::srgba(linear.red, linear.green, linear.blue, alpha);
}

/// 更新 `TextColor` 的透明度
pub fn update_text_color_alpha(alpha: f32, text_color: &mut TextColor) {
    let linear = text_color.0.to_linear();
    text_color.0 = Color::srgba(linear.red, linear.green, linear.blue, alpha);
}

/// 淡出屏幕效果
pub fn fade_out_screen(
    commands: Commands,
    time: Res<Time>,
    mut fading_out: ResMut<FadingOut>,
    mut next_state: ResMut<NextState<GameState>>,
    menu_selection: Res<CurrentMenuSelection>,
    mut sprite_query: Query<(Entity, &mut Sprite), With<StartScreenUI>>,
    mut text_query: Query<(Entity, &mut TextColor, Option<&MenuOption>), With<StartScreenUI>>,
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
        cleanup_start_screen_ui(commands, sprite_query, text_query);
    }
}

/// 生成关卡介绍界面
pub fn spawn_stage_intro(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut stage_intro_timer: ResMut<StageIntroTimer>,
    mut clear_color: ResMut<ClearColor>,
    stage_level: Res<StageLevel>,
) {
    // 设置背景色为白色
    clear_color.0 = Color::srgb(1.0, 1.0, 1.0);

    // 初始化计时器
    stage_intro_timer.fade_in = Timer::from_seconds(STAGE_FADE_IN_DURATION, TimerMode::Once);
    stage_intro_timer.stay = Timer::from_seconds(STAGE_FADE_HOLD_DURATION, TimerMode::Once);
    stage_intro_timer.fade_out = Timer::from_seconds(STAGE_FADE_OUT_DURATION, TimerMode::Once);

    // 创建全屏白色背景方块，遮挡所有游戏元素
    commands.spawn((
        StageIntroUI,
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 1.0), // 白色
            custom_size: Some(Vec2::new(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_STAGE_INTRO_BG), // z=100.0 确保在所有游戏元素之上
    ));

    // 加载字体
    let en_font: Handle<Font> = asset_server.load(FONT_EN);
    let zh_font: Handle<Font> = asset_server.load(FONT_CN);

    // Stage 标题（显示在屏幕中心）
    commands.spawn((
        StageIntroUI,
        Text2d(format!("Stage {}", stage_level.0)),
        TextFont {
            font_size: FONT_SIZE_MENU,
            font: en_font,
            ..default()
        },
        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)), // 黑色，初始透明度为0
        Transform::from_xyz(0.0, 100.0, Z_STAGE_INTRO_TEXT), // z=101.0 在白色背景之上
    ));

    // 描述文字（随机选择一条俏皮话）
    let mut rng = rand::rng();
    let quote_index = rng.random_range(0..STAGE_QUOTES_CN.len());
    let quote_text = STAGE_QUOTES_CN[quote_index];
    commands.spawn((
        StageIntroUI,
        Text2d(quote_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_SCORE,
            font: zh_font,
            ..default()
        },
        TextColor(Color::srgba(0.3, 0.3, 0.3, 0.0)), // 暗灰色，初始透明度为0
        TextLayout::new_with_justify(Justify::Center),
        Transform::from_xyz(0.0, -50.0, Z_STAGE_INTRO_TEXT), // z=101.0 在白色背景之上
    ));
}

/// 处理关卡介绍界面的计时器
pub fn handle_stage_intro_timer(
    time: Res<Time>,
    mut stage_intro_timer: ResMut<StageIntroTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut text_query: Query<&mut TextColor, With<StageIntroUI>>,
) {
    // 淡入阶段
    if !stage_intro_timer.fade_in.is_finished() {
        stage_intro_timer.fade_in.tick(time.delta());
        let progress = stage_intro_timer.fade_in.elapsed_secs() / stage_intro_timer.fade_in.duration().as_secs_f32();
        let alpha = progress.min(1.0);
        for mut text_color in &mut text_query {
            // 获取当前颜色（不包含透明度）
            let color = text_color.0;
            // 只更新透明度，保持原始颜色
            text_color.0 = color.with_alpha(alpha);
        }
    }
    // 停留阶段
    else if !stage_intro_timer.stay.is_finished() {
        stage_intro_timer.stay.tick(time.delta());
    }
    // 淡出阶段
    else if !stage_intro_timer.fade_out.is_finished() {
        stage_intro_timer.fade_out.tick(time.delta());
        let progress = stage_intro_timer.fade_out.elapsed_secs() / stage_intro_timer.fade_out.duration().as_secs_f32();
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
    clear_color.0 = BACKGROUND_COLOR;
    
    for entity in stage_intro_query.iter() {
        let () = commands.entity(entity).try_despawn();
    }
}
