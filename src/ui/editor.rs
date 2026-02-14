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
use crate::ui::common;
use crate::level_editor::LevelEditorUI;

// ============================================================================
// UI 生成函数
// ============================================================================

/// 生成关卡编辑器完整UI
pub fn spawn_editor_ui(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    font_resources: &GameTextureResources,
    language: Language,
) {
    commands.insert_resource(ClearColor(crate::ui::COLOR_BACKGROUND));

    let font = common::get_font(font_resources, language);

    // 标题
    commands.spawn((
        LevelEditorUI,
        Text2d(EDITOR_TITLE.get(language).to_string()),
        common::create_text_font(&font, FONT_SIZE_TITLE),
        TextColor(COLOR_YELLOW),
        Transform::from_xyz(0.0, WINDOW_TOP_Y - 50.0, Z_UI_TEXT),
    ));

    // 生成左右两侧的地形选择面板
    spawn_terrain_panel(
        commands,
        texture_resources,
        atlas_layouts,
        crate::level_editor::LEFT_PANEL_X,
        crate::level_editor::TERRAIN_BUTTON_START_Y,
        &crate::level_editor::LEFT_PANEL_TERRAINS,
        language,
        &font,
    );

    spawn_terrain_panel(
        commands,
        texture_resources,
        atlas_layouts,
        crate::level_editor::RIGHT_PANEL_X,
        crate::level_editor::TERRAIN_BUTTON_START_Y,
        &crate::level_editor::RIGHT_PANEL_TERRAINS,
        language,
        &font,
    );

    // 生成操作说明
    spawn_instructions(commands, language, &font);

    // 生成当前选择和文件名输入
    spawn_current_selection_and_filename(commands, language, &font);
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
    font: &Handle<Font>,
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
            common::create_text_font(font, FONT_SIZE_SMALL),
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
pub fn spawn_instructions(
    commands: &mut Commands,
    language: Language,
    font: &Handle<Font>,
) {
    let instructions = vec![
        EDITOR_INSTRUCTION_CLICK_SELECT,
        EDITOR_INSTRUCTION_EXIT,
    ];

    // 左上角位置计算
    const INSTRUCTIONS_OFFSET_X: f32 = 200.0;  // 相对于 MAP_LEFT_X 的偏移
    const INSTRUCTIONS_OFFSET_Y: f32 = 69.0;   // 相对于 MAP_TOP_Y 的偏移（20+40+9）
    const INSTRUCTIONS_LINE_SPACING: f32 = 30.0;  // 行间距

    let left_x = MAP_LEFT_X + INSTRUCTIONS_OFFSET_X;
    let top_y = MAP_TOP_Y + INSTRUCTIONS_OFFSET_Y;

    for (i, text) in instructions.iter().enumerate() {
        let x = left_x;
        let y = top_y - (i as f32) * INSTRUCTIONS_LINE_SPACING;

        commands.spawn((
            LevelEditorUI,
            Text2d(text.get(language).to_string()),
            common::create_text_font(font, FONT_SIZE_SMALL),
            TextColor(COLOR_WHITE),
            Transform::from_xyz(x, y, Z_UI_TEXT),
        ));
    }
}

/// 生成当前选择和文件名输入UI
pub fn spawn_current_selection_and_filename(
    commands: &mut Commands,
    language: Language,
    font: &Handle<Font>,
) {
    // 顶部右边位置，与操作提示对称
    const MAP_RIGHT_X: f32 = -crate::constants::MAP_LEFT_X; // 地图右边界
    const INSTRUCTIONS_OFFSET_X: f32 = 200.0;  // 与操作提示相同的偏移量
    const INSTRUCTIONS_OFFSET_Y: f32 = 69.0;   // 与操作提示相同的偏移量
    const HORIZONTAL_SPACING: f32 = 150.0;     // 水平间距
    const RIGHT_PANEL_OFFSET_X: f32 = -200.0;  // 右上角面板向左移动200像素
    const RIGHT_PANEL_OFFSET_Y: f32 = 10.0;    // 右上角面板向下移动10像素
    const ICON_OFFSET_X: f32 = -60.0;          // 地形图标向左移动60像素
    const ICON_OFFSET_Y: f32 = 40.0;           // 地形图标向上移动40像素

    let right_x = MAP_RIGHT_X - INSTRUCTIONS_OFFSET_X + RIGHT_PANEL_OFFSET_X;
    let top_y = MAP_TOP_Y + INSTRUCTIONS_OFFSET_Y + RIGHT_PANEL_OFFSET_Y;

    // 当前选择文本和地形图标（第一行，水平排列）
    const LABEL_ICON_SPACING: f32 = 20.0;  // 标签和图标之间的间距
    
    // 计算标签和图标的总宽度，使它们居中显示
    let label_text = EDITOR_CURRENT_SELECTION.get(language).to_string();
    let label_width = label_text.len() as f32 * FONT_SIZE_SMALL * 0.6; // 估算标签宽度
    let icon_size = crate::level_editor::TERRAIN_BUTTON_SIZE;
    let total_width = label_width + LABEL_ICON_SPACING + icon_size;
    let start_x = right_x - total_width / 2.0; // 调整起始位置
    
    // 当前选择文本（左）
    commands.spawn((
        LevelEditorUI,
        Text2d(label_text),
        common::create_text_font(font, FONT_SIZE_SMALL),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(start_x + label_width / 2.0 - 46.0, top_y - 10.0, Z_UI_TEXT),
    ));
    
    // 地形图标（右）
    let icon_x = start_x + label_width + LABEL_ICON_SPACING + icon_size / 2.0 + ICON_OFFSET_X;
    commands.spawn((
        LevelEditorUI,
        crate::level_editor::CurrentTerrainText,
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.3), // 半透明白色背景
            custom_size: Some(Vec2::new(icon_size, icon_size)),
            ..default()
        },
        Transform::from_xyz(icon_x, top_y - icon_size / 2.0 - 5.0 + 25.0, crate::level_editor::Z_UI_BASE),
    ));

    // 添加文件名输入提示（第二列）
    let prompt_x = right_x + HORIZONTAL_SPACING + 15.0;

    // 提示文字
    commands.spawn((
        LevelEditorUI,
        Text2d(EDITOR_OUTPUT_PROMPT.get(language).to_string()),
        common::create_text_font(font, FONT_SIZE_SMALL),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(prompt_x, top_y, Z_UI_TEXT),
    ));

    // 文件名输入框
    let input_box_y = top_y - 30.0;
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
        Transform::from_xyz(prompt_x, input_box_y, crate::level_editor::Z_UI_BASE),
    ));
    
    // 文件名输入文本
    commands.spawn((
        LevelEditorUI,
        crate::level_editor::FilenameDisplay,
        Text2d("1".to_string()),
        common::create_text_font(font, FONT_SIZE_SMALL),
        TextColor(COLOR_WHITE),
        Transform::from_xyz(prompt_x, input_box_y, Z_UI_TEXT + 0.1),
    ));
}