//! 关卡编辑器 UI 模块
//!
//! 处理关卡编辑器的所有 UI 元素生成和更新

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;

use crate::constants::*;
use crate::map::TerrainType;
use crate::resources::{GameTextureResources, GameAtlasLayoutResources, Language};
use crate::ui::constants::*;
use crate::ui::localization::*;
use crate::level_editor::{LevelEditorUI, LevelEditorUI as EditorUI};

// ============================================================================
// UI 生成函数
// ============================================================================

/// 生成关卡编辑器UI
pub fn spawn_editor_ui(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    language: Language,
) {
    // 设置背景色为游戏背景色（蓝绿色）
    commands.insert_resource(ClearColor(crate::ui::COLOR_BACKGROUND));

    // 顶部标题
    commands.spawn((
        LevelEditorUI,
        Text2d(EDITOR_TITLE.get(language).to_string()),
        TextFont {
            font_size: FONT_SIZE_TITLE,
            ..default()
        },
        TextColor(COLOR_YELLOW),
        Transform::from_xyz(0.0, WINDOW_TOP_Y - 50.0, Z_UI_TEXT),
    ));

    // 左侧面板地形元素
    spawn_terrain_panel(
        commands,
        texture_resources,
        atlas_layouts,
        crate::level_editor::LEFT_PANEL_X,
        crate::level_editor::TERRAIN_BUTTON_START_Y,
        &crate::level_editor::LEFT_PANEL_TERRAINS,
        language,
    );

    // 右侧面板地形元素
    spawn_terrain_panel(
        commands,
        texture_resources,
        atlas_layouts,
        crate::level_editor::RIGHT_PANEL_X,
        crate::level_editor::TERRAIN_BUTTON_START_Y,
        &crate::level_editor::RIGHT_PANEL_TERRAINS,
        language,
    );

    // 底部操作说明
    spawn_instructions(commands, language);
    
    // 生成当前选择和文件名输入UI
    spawn_current_selection_and_filename(commands, language);
}

/// 生成地形面板
pub fn spawn_terrain_panel(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    panel_x: f32,
    start_y: f32,
    terrains: &[TerrainType],
    language: Language,
) {
    for (i, terrain_type) in terrains.iter().enumerate() {
        let y = start_y - (i as f32) * crate::level_editor::TERRAIN_BUTTON_SPACING;

        // 生成地形预览
        crate::level_editor::spawn_terrain_preview(
            commands,
            texture_resources,
            atlas_layouts,
            panel_x,
            y,
            *terrain_type
        );

        // 生成地形名称文本
        let name = terrain_type.to_display_name(language);
        commands.spawn((
            LevelEditorUI,
            Text2d(name.to_string()),
            TextFont {
                font_size: FONT_SIZE_SMALL,
                ..default()
            },
            TextColor(COLOR_WHITE),
            Transform::from_xyz(panel_x, y - crate::level_editor::TEXT_Y_OFFSET, Z_UI_TEXT),
        ));

        // 生成地形按钮（点击选择）
        commands.spawn((
            LevelEditorUI,
            crate::level_editor::TerrainButton {
                terrain_type: *terrain_type,
            },
            Pickable::default(),
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.3),
                custom_size: Some(Vec2::new(
                    crate::level_editor::TERRAIN_BUTTON_SIZE, 
                    crate::level_editor::TERRAIN_BUTTON_SIZE
                )),
                ..default()
            },
            Transform::from_xyz(panel_x, y, crate::level_editor::Z_UI_BASE),
        ));
    }
}

/// 生成操作说明
pub fn spawn_instructions(commands: &mut Commands, language: Language) {
    let instructions = vec![
        EDITOR_INSTRUCTION_CLICK_SELECT,
        EDITOR_INSTRUCTION_EXIT,
        EDITOR_INSTRUCTION_EXPORT,
    ];

    // 左上角位置：MAP_LEFT_X 左侧 + 400像素，MAP_TOP_Y 上方 + 40像素
    let left_x = MAP_LEFT_X - 200.0 + 400.0;
    let top_y = MAP_TOP_Y + 20.0 + 40.0;

    for (i, text) in instructions.iter().enumerate() {
        let x = left_x;
        let y = top_y - (i as f32) * 30.0;
        
        commands.spawn((
            LevelEditorUI,
            Text2d(text.get(language).to_string()),
            TextFont {
                font_size: FONT_SIZE_SMALL,
                ..default()
            },
            TextColor(COLOR_WHITE),
            Transform::from_xyz(x, y, Z_UI_TEXT),
        ));
    }
}

/// 生成当前选择和文件名输入UI
pub fn spawn_current_selection_and_filename(commands: &mut Commands, language: Language) {
    // 右侧面板下方位置
    let right_x = crate::level_editor::RIGHT_PANEL_X;
    let selected_y = crate::level_editor::TERRAIN_BUTTON_START_Y - 
                    (7.0 * crate::level_editor::TERRAIN_BUTTON_SPACING) - 100.0;
    
    // 当前选择文本
    commands.spawn((
        LevelEditorUI,
        Text2d(EDITOR_CURRENT_SELECTION.get(language).to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(
            right_x, 
            selected_y + crate::level_editor::TERRAIN_BUTTON_SIZE / 2.0 + 20.0, 
            Z_UI_TEXT
        ),
    ));
    
    // 创建当前选择地形图标容器
    commands.spawn((
        LevelEditorUI,
        crate::level_editor::CurrentTerrainText,
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.3), // 半透明白色背景
            custom_size: Some(Vec2::new(
                crate::level_editor::TERRAIN_BUTTON_SIZE, 
                crate::level_editor::TERRAIN_BUTTON_SIZE
            )),
            ..default()
        },
        Transform::from_xyz(
            right_x + crate::level_editor::TERRAIN_BUTTON_SIZE / 2.0 + 20.0 - 6.0, 
            selected_y, 
            crate::level_editor::Z_UI_BASE
        ),
    ));
    
    // 添加文件名输入提示
    let prompt_x = right_x + crate::level_editor::TERRAIN_BUTTON_SIZE + 100.0;
    let prompt_y = selected_y;
    
    // 提示文字
    commands.spawn((
        LevelEditorUI,
        Text2d(EDITOR_OUTPUT_PROMPT.get(language).to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(prompt_x, prompt_y, Z_UI_TEXT),
    ));
    
    // 文件名输入框
    let input_box_x = prompt_x + 160.0;
    let input_box_y = prompt_y;
    let input_box_width = 80.0;
    let input_box_height = 30.0;
    
    commands.spawn((
        LevelEditorUI,
        crate::level_editor::FilenameInput,
        Sprite {
            color: Color::srgba(0.2, 0.2, 0.2, 0.8),
            custom_size: Some(Vec2::new(input_box_width, input_box_height)),
            ..default()
        },
        Transform::from_xyz(input_box_x, input_box_y, crate::level_editor::Z_UI_BASE),
    ));
    
    // 文件名输入文本
    commands.spawn((
        LevelEditorUI,
        crate::level_editor::FilenameDisplay,
        Text2d("1".to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(input_box_x, input_box_y, Z_UI_TEXT + 0.1),
    ));
}