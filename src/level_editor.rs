//! 关卡编辑器模块
//!
//! 提供可视化的关卡编辑功能，支持拖拽地形元素到网格

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;

use crate::constants::*;
use crate::map::{TerrainType, MAP_COLS, MAP_ROWS, GRID_SIZE, grid_to_world};
use crate::resources::*;
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;

// ============================================================================
// 常量配置
// ============================================================================

/// 地形按钮尺寸
const TERRAIN_BUTTON_SIZE: f32 = 90.0;

/// 地形按钮间距
const TERRAIN_BUTTON_SPACING: f32 = 130.0;

/// 地形按钮起始 Y 坐标
const TERRAIN_BUTTON_START_Y: f32 = 500.0;

/// 左侧面板 X 坐标
const LEFT_PANEL_X: f32 = -930.0;

/// 右侧面板 X 坐标
const RIGHT_PANEL_X: f32 = 930.0;

/// 地形按钮内边距（用于缩小预览显示）
const TERRAIN_BUTTON_PADDING: f32 = 10.0;

/// 文本垂直偏移（地形名称）
const TEXT_Y_OFFSET: f32 = TERRAIN_BUTTON_SIZE / 2.0 + 15.0;

/// Z轴层级
const Z_UI_BASE: f32 = 10.0;
const Z_UI_PREVIEW: f32 = Z_UI_BASE + 0.1;
const Z_UI_TEXT: f32 = 20.0;

/// 地形按钮点击检测半径（像素）
const TERRAIN_BUTTON_CLICK_RADIUS: f32 = 45.0;

/// 最大文件名长度
const MAX_FILENAME_LENGTH: usize = 3;

// ============================================================================
// 编辑器标记组件
// ============================================================================

/// 编辑器 UI 标记
#[derive(Component)]
pub struct LevelEditorUI;

/// 地形元素按钮标记
#[derive(Component)]
pub struct TerrainButton {
    pub terrain_type: TerrainType,
}

/// 地形元素预览标记
#[derive(Component)]
pub struct TerrainPreview;

/// 当前选择地形文本标记
#[derive(Component)]
pub struct CurrentTerrainText;

/// 地形显示标记（网格单元格中的地形显示）
#[derive(Component)]
pub struct TerrainDisplay;

/// 文件名输入框标记
#[derive(Component)]
pub struct FilenameInput;

/// 文件名显示文本标记
#[derive(Component)]
pub struct FilenameDisplay;

/// 网格单元格标记
#[derive(Component)]
pub struct GridCell {
    pub row: usize,
    pub col: usize,
}

/// 当前选中的地形类型（用于拖拽）
#[derive(Resource, Default)]
pub struct SelectedTerrain {
    pub terrain_type: Option<TerrainType>,
}

/// 输入的文件名
#[derive(Resource, Default)]
pub struct InputFilename {
    pub name: String,
}

impl InputFilename {
    pub fn new() -> Self {
        Self {
            name: String::from("1"), // 默认关卡号
        }
    }
}

/// 当前编辑的地图数据
#[derive(Resource, Clone)]
pub struct EditorMapData {
    pub data: [[TerrainType; MAP_COLS]; MAP_ROWS],
}

impl Default for EditorMapData {
    fn default() -> Self {
        Self {
            data: [[TerrainType::Empty; MAP_COLS]; MAP_ROWS],
        }
    }
}

impl EditorMapData {
    /// 获取指定位置的地形类型
    pub fn get(&self, row: usize, col: usize) -> TerrainType {
        if row < MAP_ROWS && col < MAP_COLS {
            self.data[row][col]
        } else {
            TerrainType::Empty
        }
    }

    /// 设置指定位置的地形类型
    pub fn set(&mut self, row: usize, col: usize, terrain: TerrainType) {
        if row < MAP_ROWS && col < MAP_COLS {
            self.data[row][col] = terrain;
        }
    }

    /// 清空地图
    pub fn clear(&mut self) {
        self.data = [[TerrainType::Empty; MAP_COLS]; MAP_ROWS];
    }

    /// 从现有关卡数据加载
    pub fn load_from_level(&mut self, level_data: &[[TerrainType; MAP_COLS]; MAP_ROWS]) {
        self.data = *level_data;
    }
}

// ============================================================================
// 编辑器布局常量
// ============================================================================

/// 地形元素定义（14种）
// ============================================================================

/// 左侧面板地形元素（7种）
const LEFT_PANEL_TERRAINS: [TerrainType; 7] = [
    TerrainType::Empty,
    TerrainType::Sea,
    TerrainType::Forest,
    TerrainType::Barrier,
    TerrainType::Steel,
    TerrainType::SteelTop,
    TerrainType::SteelBottom,
];

/// 右侧面板地形元素（7种）
const RIGHT_PANEL_TERRAINS: [TerrainType; 7] = [
    TerrainType::SteelLeft,
    TerrainType::SteelRight,
    TerrainType::Brick,
    TerrainType::BrickTop,
    TerrainType::BrickBottom,
    TerrainType::BrickLeft,
    TerrainType::BrickRight,
];

// ============================================================================
// 地形名称映射
// ============================================================================

/// 获取地形的显示名称
fn get_terrain_name(terrain: TerrainType) -> &'static str {
    match terrain {
        TerrainType::Empty => "空地",
        TerrainType::Sea => "海洋",
        TerrainType::Forest => "树林",
        TerrainType::Barrier => "屏障",
        TerrainType::Steel => "钢铁",
        TerrainType::SteelTop => "钢铁-上",
        TerrainType::SteelBottom => "钢铁-下",
        TerrainType::SteelLeft => "钢铁-左",
        TerrainType::SteelRight => "钢铁-右",
        TerrainType::Brick => "砖块",
        TerrainType::BrickTop => "砖块-上",
        TerrainType::BrickBottom => "砖块-下",
        TerrainType::BrickLeft => "砖块-左",
        TerrainType::BrickRight => "砖块-右",
    }
}

/// 地形显示尺寸类型
enum TerrainDisplaySize {
    Full,
    HalfTop,
    HalfBottom,
    HalfLeft,
    HalfRight,
}

/// 根据地形类型获取显示尺寸
fn get_terrain_display_size(terrain: TerrainType) -> TerrainDisplaySize {
    match terrain {
        TerrainType::Empty | TerrainType::Sea | TerrainType::Forest |
        TerrainType::Barrier | TerrainType::Brick | TerrainType::Steel => {
            TerrainDisplaySize::Full
        }
        TerrainType::SteelTop | TerrainType::BrickTop => TerrainDisplaySize::HalfTop,
        TerrainType::SteelBottom | TerrainType::BrickBottom => TerrainDisplaySize::HalfBottom,
        TerrainType::SteelLeft | TerrainType::BrickLeft => TerrainDisplaySize::HalfLeft,
        TerrainType::SteelRight | TerrainType::BrickRight => TerrainDisplaySize::HalfRight,
    }
}

/// 根据尺寸类型和基准尺寸计算实际尺寸和位置偏移
fn calculate_size_and_offset(
    size_type: TerrainDisplaySize,
    base_size: Vec2,
) -> (Vec2, Vec2) {
    match size_type {
        TerrainDisplaySize::Full => (base_size, Vec2::ZERO),
        TerrainDisplaySize::HalfTop => {
            let half_size = Vec2::new(base_size.x, base_size.y / 2.0);
            let offset = Vec2::new(0.0, half_size.y / 2.0);
            (half_size, offset)
        }
        TerrainDisplaySize::HalfBottom => {
            let half_size = Vec2::new(base_size.x, base_size.y / 2.0);
            let offset = Vec2::new(0.0, -half_size.y / 2.0);
            (half_size, offset)
        }
        TerrainDisplaySize::HalfLeft => {
            let half_size = Vec2::new(base_size.x / 2.0, base_size.y);
            let offset = Vec2::new(-half_size.x / 2.0, 0.0);
            (half_size, offset)
        }
        TerrainDisplaySize::HalfRight => {
            let half_size = Vec2::new(base_size.x / 2.0, base_size.y);
            let offset = Vec2::new(half_size.x / 2.0, 0.0);
            (half_size, offset)
        }
    }
}

/// 获取纹理句柄
fn get_texture_handle(
    terrain: TerrainType,
    texture_resources: &GameTextureResources,
) -> Handle<Image> {
    match terrain {
        TerrainType::Sea => texture_resources.sea.clone(),
        TerrainType::Forest => texture_resources.tree.clone(),
        TerrainType::Barrier => texture_resources.barrier.clone(),
        TerrainType::Steel | TerrainType::SteelTop | TerrainType::SteelBottom |
        TerrainType::SteelLeft | TerrainType::SteelRight => texture_resources.steel.clone(),
        TerrainType::Brick | TerrainType::BrickTop | TerrainType::BrickBottom |
        TerrainType::BrickLeft | TerrainType::BrickRight => texture_resources.brick.clone(),
        TerrainType::Empty => Handle::default(),
    }
}

/// 获取精灵图布局和索引
fn get_atlas_info(terrain: TerrainType, atlas_layouts: &GameAtlasLayoutResources) -> Option<(Handle<TextureAtlasLayout>, usize)> {
    match terrain {
        TerrainType::Sea => Some((atlas_layouts.sea.clone(), 0)),
        TerrainType::Forest => Some((atlas_layouts.forest.clone(), 0)),
        _ => None,
    }
}

// ============================================================================
// 编辑器系统函数
// ============================================================================

/// 进入编辑器状态时初始化
pub fn on_enter_level_editor(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    level_assets: Res<crate::levels::LevelAssets>,
    mut editor_map: ResMut<EditorMapData>,
    mut input_filename: ResMut<InputFilename>,
    stage_level: Res<StageLevel>,
    start_screen_entities: Query<Entity, With<crate::ui::StartScreenUI>>,
    playing_entities: Query<Entity, With<crate::ui::PlayingEntity>>,
) {
    info!("进入关卡编辑器");

    // 清理开始界面UI元素
    for entity in start_screen_entities.iter() {
        commands.entity(entity).despawn();
    }

    // 清理游戏中的实体
    for entity in playing_entities.iter() {
        commands.entity(entity).despawn();
    }

    // 设置背景色为黑色
    commands.insert_resource(ClearColor(crate::ui::COLOR_BLACK));
    
    // 初始化文件名输入
    input_filename.name = "1".to_string();

    // 清空地图，从空地图开始编辑
    editor_map.clear();

    // 生成编辑器 UI
    spawn_editor_ui(&mut commands, &texture_resources, &atlas_layouts);

    // 生成网格
    spawn_editor_grid(&mut commands);
}

/// 退出编辑器状态时清理
pub fn on_exit_level_editor(
    mut commands: Commands,
    mut clear_color: ResMut<ClearColor>,
    editor_entities: Query<Entity, With<LevelEditorUI>>,
) {
    info!("退出关卡编辑器");

    // 恢复背景色
    clear_color.0 = crate::ui::COLOR_GRAY;

    // 清理编辑器UI元素
    for entity in editor_entities.iter() {
        commands.entity(entity).despawn();
    }
}

/// 生成编辑器 UI
fn spawn_editor_ui(
    commands: &mut Commands, 
    texture_resources: &GameTextureResources, 
    atlas_layouts: &GameAtlasLayoutResources,
) {
    // 设置背景色为游戏背景色（蓝绿色）
    commands.insert_resource(ClearColor(crate::ui::COLOR_BACKGROUND));

    // 顶部标题
    commands.spawn((
        LevelEditorUI,
        Text2d("关卡编辑器".to_string()),
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
        LEFT_PANEL_X,
        TERRAIN_BUTTON_START_Y,
        &LEFT_PANEL_TERRAINS,
    );

    // 右侧面板地形元素
    spawn_terrain_panel(
        commands,
        texture_resources,
        atlas_layouts,
        RIGHT_PANEL_X,
        TERRAIN_BUTTON_START_Y,
        &RIGHT_PANEL_TERRAINS,
    );

    // 底部操作说明
    spawn_instructions(commands);
}

/// 生成地形面板
fn spawn_terrain_panel(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    panel_x: f32,
    start_y: f32,
    terrains: &[TerrainType],
) {
    for (i, terrain_type) in terrains.iter().enumerate() {
        let y = start_y - (i as f32) * TERRAIN_BUTTON_SPACING;

        // 生成地形预览
        spawn_terrain_preview(commands, texture_resources, atlas_layouts, panel_x, y, *terrain_type);

        // 生成地形名称文本
        let name = get_terrain_name(*terrain_type);
        commands.spawn((
            LevelEditorUI,
            Text2d(name.to_string()),
            TextFont {
                font_size: FONT_SIZE_SMALL,
                ..default()
            },
            TextColor(COLOR_WHITE),
            Transform::from_xyz(panel_x, y - TEXT_Y_OFFSET, Z_UI_TEXT),
        ));

        // 生成地形按钮（点击选择）
        commands.spawn((
            LevelEditorUI,
            TerrainButton {
                terrain_type: *terrain_type,
            },
            Pickable::default(),
            Sprite {
                color: Color::srgba(1.0, 1.0, 1.0, 0.3),
                custom_size: Some(Vec2::new(TERRAIN_BUTTON_SIZE, TERRAIN_BUTTON_SIZE)),
                ..default()
            },
            Transform::from_xyz(panel_x, y, Z_UI_BASE),
        ));
    }
}

/// 生成地形预览
fn spawn_terrain_preview(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    x: f32,
    y: f32,
    terrain_type: TerrainType,
) {
    // 空地形不显示
    if terrain_type == TerrainType::Empty {
        return;
    }

    let base_size = Vec2::new(TERRAIN_BUTTON_SIZE - TERRAIN_BUTTON_PADDING, TERRAIN_BUTTON_SIZE - TERRAIN_BUTTON_PADDING);
    let size_type = get_terrain_display_size(terrain_type);
    let (display_size, offset) = calculate_size_and_offset(size_type, base_size);

    let texture = get_texture_handle(terrain_type, texture_resources);
    let atlas_info = get_atlas_info(terrain_type, atlas_layouts);

    let mut sprite = Sprite {
        image: texture,
        custom_size: Some(display_size),
        ..default()
    };

    if let Some((layout, index)) = atlas_info {
        sprite.texture_atlas = Some(TextureAtlas { layout, index });
    }

    commands.spawn((
        LevelEditorUI,
        TerrainPreview,
        sprite,
        Transform::from_xyz(x + offset.x, y + offset.y, Z_UI_PREVIEW),
    ));
}

/// 生成网格
fn spawn_editor_grid(commands: &mut Commands) {
    // 绘制网格线（只绘制白色网格线）
    for row in 0..=MAP_ROWS {
        let y = MAP_TOP_Y - (row as f32) * GRID_SIZE;
        commands.spawn((
            LevelEditorUI,
            Sprite {
                color: COLOR_WHITE,
                custom_size: Some(Vec2::new(MAP_WIDTH, 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, y, Z_UI - 0.1),
        ));
    }

    for col in 0..=MAP_COLS {
        let x = MAP_LEFT_X + (col as f32) * GRID_SIZE;
        let y = (MAP_TOP_Y + MAP_BOTTOM_Y) / 2.0;
        commands.spawn((
            LevelEditorUI,
            Sprite {
                color: COLOR_WHITE,
                custom_size: Some(Vec2::new(1.0, MAP_HEIGHT)),
                ..default()
            },
            Transform::from_xyz(x, y, Z_UI - 0.1),
        ));
    }

    // 生成网格单元格（用于点击检测，完全透明）
    for row in 0..MAP_ROWS {
        for col in 0..MAP_COLS {
            let pos = grid_to_world(row, col);
            commands.spawn((
                LevelEditorUI,
                GridCell { row, col },
                Pickable::default(),
                Sprite {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0), // 完全透明
                    custom_size: Some(Vec2::new(GRID_SIZE, GRID_SIZE)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, Z_UI - 0.2),
            ));
        }
    }
}

/// 生成操作说明
fn spawn_instructions(commands: &mut Commands) {
    let instructions = vec![
        "点击地形元素选择",
        "点击网格放置地形",
        "ESC 退出编辑器",
        "S 导出关卡文件",
    ];

    // 左上角位置：MAP_LEFT_X 左侧 + 400像素，MAP_TOP_Y 上方 + 40像素
    let left_x = MAP_LEFT_X - 200.0 + 400.0;
    let top_y = MAP_TOP_Y + 20.0 + 40.0;

    for (i, text) in instructions.iter().enumerate() {
        // 分两列显示
        let col = i % 2;
        let row = i / 2;
        
        let x = left_x + (col as f32) * 200.0;
        let y = top_y - (row as f32) * 30.0;
        
        commands.spawn((
            LevelEditorUI,
            Text2d(text.to_string()),
            TextFont {
                font_size: FONT_SIZE_SMALL,
                ..default()
            },
            TextColor(COLOR_WHITE),
            Transform::from_xyz(x, y, Z_UI_TEXT),
        ));
    }

    // 在右边对称区域添加当前选择地形图标
    let right_x = -left_x - 200.0; // 向左移动400像素，再向右移动200像素，净移动-200像素
    let selected_y = top_y;
    
    // 添加"当前选择:"文字（在图标左边）
    commands.spawn((
        LevelEditorUI,
        Text2d("当前选择:".to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(right_x - TERRAIN_BUTTON_SIZE / 2.0 - 20.0, selected_y, Z_UI_TEXT),
    ));
    
    // 创建当前选择地形图标容器
    commands.spawn((
        LevelEditorUI,
        CurrentTerrainText,
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.3), // 半透明白色背景
            custom_size: Some(Vec2::new(TERRAIN_BUTTON_SIZE, TERRAIN_BUTTON_SIZE)),
            ..default()
        },
        Transform::from_xyz(right_x + TERRAIN_BUTTON_SIZE / 2.0 + 20.0 - 6.0, selected_y, Z_UI),
    ));
    
    // 添加文件名输入提示（在最右侧）
    let prompt_text = "输出到关卡:";
    let prompt_x = right_x + TERRAIN_BUTTON_SIZE + 100.0; // 在图标右侧
    let prompt_y = selected_y;
    
    // 提示文字
    commands.spawn((
        LevelEditorUI,
        Text2d(prompt_text.to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(prompt_x, prompt_y, Z_UI_TEXT),
    ));
    
    // 文件名输入框背景
    let input_box_width = 80.0; // 缩短为80像素，约3个数字宽度
    let input_box_height = 30.0;
    let input_box_x = prompt_x + 80.0 + 45.0 - 6.0; // 向右移动60像素，再向左移动15像素，再向左移动6像素，净移动+39像素
    let input_box_y = prompt_y; // 向上移动5像素
    
    commands.spawn((
        LevelEditorUI,
        FilenameInput,
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.5), // 半透明黑色背景
            custom_size: Some(Vec2::new(input_box_width, input_box_height)),
            ..default()
        },
        Transform::from_xyz(input_box_x, input_box_y, Z_UI - 0.05),
    ));
    
    // 文件名输入文本
    commands.spawn((
        LevelEditorUI,
        FilenameDisplay,
        Text2d("1".to_string()),
        TextFont {
            font_size: FONT_SIZE_SMALL,
            ..default()
        },
        TextColor(COLOR_WHITE),
        Transform::from_xyz(input_box_x, input_box_y, Z_UI_TEXT + 0.1),
    ));
}

/// 处理地形按钮点击（使用鼠标位置检测）
pub fn handle_terrain_button_click(
    mut commands: Commands,
    button_query: Query<(&Transform, &TerrainButton)>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut selected_terrain: ResMut<SelectedTerrain>,
    current_terrain_query: Query<Entity, With<CurrentTerrainText>>,
    terrain_display_entities: Query<(Entity, &ChildOf), With<TerrainDisplay>>,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let cursor_position = window.cursor_position();
    let Some(cursor_position) = cursor_position else {
        return;
    };

    let Ok((_camera, _camera_transform)) = camera_q.single() else {
        return;
    };

    info!("鼠标点击检测: 鼠标位置 {:?}", cursor_position);

    // 将屏幕坐标转换为世界坐标（简化版）
    let world_position: Vec2 = {
        let width = window.width();
        let height = window.height();
        let x = cursor_position.x - width / 2.0;
        let y = -(cursor_position.y - height / 2.0); // Y轴翻转
        
        Vec2::new(x, y)
    };

    info!("世界坐标: {:?}", world_position);
    info!("地形按钮总数: {}", button_query.iter().count());

    // 检查是否点击了地形按钮
    for (transform, button) in button_query.iter() {
        let button_pos = transform.translation.truncate();
        let distance = button_pos.distance(world_position);

        if distance < TERRAIN_BUTTON_CLICK_RADIUS {
            selected_terrain.terrain_type = Some(button.terrain_type);
            info!("✓ 成功选中地形: {:?}", button.terrain_type);
            
            // 更新当前选择地形图标
            if let Ok(current_terrain_entity) = current_terrain_query.single() {
                // 先移除所有的子实体（旧的图标）
                let child_entities: Vec<Entity> = terrain_display_entities
                    .iter()
                    .filter(|(_, parent)| parent.0 == current_terrain_entity)
                    .map(|(entity, _)| entity)
                    .collect();
                for child in child_entities {
                    commands.entity(child).despawn();
                }
                
                // 直接生成地形显示
                if button.terrain_type != TerrainType::Empty {
                    let base_size = Vec2::new(TERRAIN_BUTTON_SIZE - TERRAIN_BUTTON_PADDING, TERRAIN_BUTTON_SIZE - TERRAIN_BUTTON_PADDING);
                    let size_type = get_terrain_display_size(button.terrain_type);
                    let (display_size, offset) = calculate_size_and_offset(size_type, base_size);
                    
                    let texture = get_texture_handle(button.terrain_type, &texture_resources);
                    let atlas_info = get_atlas_info(button.terrain_type, &atlas_layouts);
                    
                    let mut sprite = Sprite {
                        image: texture,
                        custom_size: Some(display_size),
                        color: Color::WHITE,
                        ..default()
                    };
                    
                    if let Some((layout, index)) = atlas_info {
                        sprite.texture_atlas = Some(TextureAtlas { layout, index });
                    }
                    
                    commands.spawn((
                        LevelEditorUI,
                        TerrainDisplay,
                        ChildOf(current_terrain_entity),
                        sprite,
                        Transform::from_xyz(offset.x, offset.y, 0.1),
                    ));
                }
            }
            
            break;
        }
    }
}

/// 处理网格点击（放置地形）- 使用鼠标位置检测
pub fn handle_grid_click(
    mut commands: Commands,
    grid_query: Query<(&Transform, &GridCell)>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    selected_terrain: Res<SelectedTerrain>,
    mut editor_map: ResMut<EditorMapData>,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    grid_entities: Query<(Entity, &GridCell, &Transform), With<GridCell>>,
    terrain_display_entities: Query<(Entity, &ChildOf), With<TerrainDisplay>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let cursor_position = window.cursor_position();
    let Some(cursor_position) = cursor_position else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_q.single() else {
        return;
    };

    // 将屏幕坐标转换为世界坐标（简化版）
    let world_position: Vec2 = {
        let width = window.width();
        let height = window.height();
        let x = cursor_position.x - width / 2.0;
        let y = -(cursor_position.y - height / 2.0); // Y轴翻转
        
        Vec2::new(x, y)
    };

    // 检查是否点击了网格单元格
    for (transform, cell) in grid_query.iter() {
        let cell_pos = transform.translation.truncate();
        let distance = cell_pos.distance(world_position);

        if distance < GRID_SIZE / 2.0 {
            if let Some(terrain_type) = selected_terrain.terrain_type {
                // 更新地图数据
                editor_map.set(cell.row, cell.col, terrain_type);

                // 更新网格单元格显示
                update_grid_cell(
                    &mut commands,
                    &texture_resources,
                    &atlas_layouts,
                    cell.row,
                    cell.col,
                    terrain_type,
                    &grid_entities,
                    &terrain_display_entities,
                );

                info!("放置地形: {:?} at ({}, {})", terrain_type, cell.row, cell.col);
            }
        }
    }
}

/// 更新网格单元格的地形显示
fn update_grid_cell(
    commands: &mut Commands,
    texture_resources: &GameTextureResources,
    atlas_layouts: &GameAtlasLayoutResources,
    row: usize,
    col: usize,
    terrain_type: TerrainType,
    grid_entities: &Query<(Entity, &GridCell, &Transform), With<GridCell>>,
    terrain_display_entities: &Query<(Entity, &ChildOf), With<TerrainDisplay>>,
) {
    // 找到对应的网格单元格实体
    if let Some((cell_entity, _, _)) = grid_entities
        .iter()
        .find(|(_, cell, _)| cell.row == row && cell.col == col)
    {
        // 查找并移除旧的地形显示子实体
        for (display_entity, parent) in terrain_display_entities.iter() {
            if parent.0 == cell_entity {
                commands.entity(display_entity).despawn();
            }
        }

        // 创建新的地形显示子实体
        let base_size = Vec2::new(GRID_SIZE, GRID_SIZE);
        // 树林使用游戏中的实际尺寸
        let custom_base_size = if terrain_type == TerrainType::Forest {
            Vec2::new(131.0, 131.0)
        } else {
            base_size
        };
        
        // 直接生成地形显示，避免可变借用冲突
        if terrain_type != TerrainType::Empty {
            let actual_base_size = custom_base_size;
            let size_type = get_terrain_display_size(terrain_type);
            let (display_size, offset) = calculate_size_and_offset(size_type, actual_base_size);
            
            let texture = get_texture_handle(terrain_type, texture_resources);
            let atlas_info = get_atlas_info(terrain_type, atlas_layouts);
            
            let mut sprite = Sprite {
                image: texture,
                custom_size: Some(display_size),
                color: Color::WHITE,
                ..default()
            };
            
            if let Some((layout, index)) = atlas_info {
                sprite.texture_atlas = Some(TextureAtlas { layout, index });
            }
            
            commands.spawn((
                LevelEditorUI,
                TerrainDisplay,
                ChildOf(cell_entity),
                sprite,
                Transform::from_xyz(offset.x, offset.y, 0.1),
            ));
        }
    }
}

/// 处理编辑器键盘输入
pub fn handle_editor_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    editor_map: Res<EditorMapData>,
    mut input_filename: ResMut<InputFilename>,
    mut filename_query: Query<&mut Text2d, With<FilenameDisplay>>,
) {
    // ESC 退出编辑器
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::StartScreen);
        info!("退出编辑器");
    }

    // S 导出关卡文件
    if keyboard.just_pressed(KeyCode::KeyS) {
        export_level(&editor_map, &input_filename.name);
    }
    
    // 处理文件名输入
    for key in keyboard.get_just_pressed() {
        match key {
            KeyCode::Backspace => {
                input_filename.name.pop();
            }
            key_char @ (KeyCode::Digit0 | KeyCode::Digit1 | KeyCode::Digit2 | 
                       KeyCode::Digit3 | KeyCode::Digit4 | KeyCode::Digit5 | 
                       KeyCode::Digit6 | KeyCode::Digit7 | KeyCode::Digit8 | KeyCode::Digit9) => {
                if input_filename.name.len() < MAX_FILENAME_LENGTH {
                    let digit_char = match key_char {
                        KeyCode::Digit0 => '0',
                        KeyCode::Digit1 => '1',
                        KeyCode::Digit2 => '2',
                        KeyCode::Digit3 => '3',
                        KeyCode::Digit4 => '4',
                        KeyCode::Digit5 => '5',
                        KeyCode::Digit6 => '6',
                        KeyCode::Digit7 => '7',
                        KeyCode::Digit8 => '8',
                        KeyCode::Digit9 => '9',
                        _ => unreachable!(),
                    };
                    input_filename.name.push(digit_char);
                }
            }
            _ => {}
        }
    }
    
    // 更新文件名显示
    if let Ok(mut text) = filename_query.single_mut() {
        text.0 = input_filename.name.clone();
    }
}

/// 导出关卡文件
fn export_level(editor_map: &EditorMapData, filename: &str) {
    let mut content = String::new();

    // 调试：打印前几行数据
    info!("导出地图数据，文件名: {}", filename);
    for row in 0..3.min(MAP_ROWS) {
        let row_data: Vec<String> = (0..MAP_COLS)
            .map(|col| {
                let terrain = editor_map.get(row, col);
                terrain_to_symbol(terrain).to_string()
            })
            .collect();
        info!("第{}行: {}", row + 1, row_data.join(" "));
    }

    for row in 0..MAP_ROWS {
        for col in 0..MAP_COLS {
            let terrain = editor_map.get(row, col);
            let symbol = terrain_to_symbol(terrain);
            if col > 0 {
                content.push(' ');
            }
            content.push_str(symbol);
        }
        content.push('\n');
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::fs;
        let levels_dir = if std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .as_deref()
            == Some(std::path::Path::new("/usr/bin"))
        {
            "/usr/share/tank-battle/levels"
        } else {
            "levels"
        };

        let file_path = format!("{}/{}.txt", levels_dir, filename);
        match fs::write(&file_path, content) {
            Ok(_) => info!("关卡已导出到: {}", file_path),
            Err(e) => warn!("导出关卡失败: {}", e),
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        info!("关卡数据（Web端无法直接导出文件）:\n{}", content);
    }
}

/// 将地形类型转换为符号
fn terrain_to_symbol(terrain: TerrainType) -> &'static str {
    match terrain {
        TerrainType::Empty => ".",
        TerrainType::Forest => "t",
        TerrainType::Sea => "s",
        TerrainType::Brick => "b",
        TerrainType::BrickLeft => "bl",
        TerrainType::BrickRight => "br",
        TerrainType::BrickTop => "bt",
        TerrainType::BrickBottom => "bb",
        TerrainType::Steel => "i",
        TerrainType::SteelLeft => "il",
        TerrainType::SteelRight => "ir",
        TerrainType::SteelTop => "it",
        TerrainType::SteelBottom => "ib",
        TerrainType::Barrier => "a",
    }
}