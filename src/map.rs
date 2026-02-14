//! 地图和地形生成模块
//!
//! 包含地图数据定义（TerrainType、网格系统）和地形实体生成功能
//! 处理墙壁、砖块、钢块、森林、海等地形元素的生成与管理

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use avian2d::prelude::*;

use crate::constants::*;
use crate::resources::{GameAtlasLayoutResources, GameTextureResources, StageLevel};
#[allow(clippy::wildcard_imports)]
use crate::ui::constants::*;

/// 地形类型枚举
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainType {
    /// 空地（可通行）
    Empty,
    /// 树林（坦克可穿过，提供掩护）
    Forest,
    /// 海（子弹可穿过，坦克不可）
    Sea,
    /// 砖块（可破坏，1发子弹）
    Brick,
    /// 砖块左半（50×100）
    BrickLeft,
    /// 砖块右半（50×100）
    BrickRight,
    /// 砖块上半（100×50）
    BrickTop,
    /// 砖块下半（100×50）
    BrickBottom,
    /// 钢铁（不可破坏）
    Steel,
    /// 钢铁左半（50×100）
    SteelLeft,
    /// 钢铁右半（50×100）
    SteelRight,
    /// 钢铁上半（100×50）
    SteelTop,
    /// 钢铁下半（100×50）
    SteelBottom,
    /// 屏障（可破坏，2发子弹）
    Barrier,
}

impl TerrainType {
    /// 从字符串转换为地形类型
    pub fn from_str(s: &str) -> Self {
        match s {
            "t" => Self::Forest,
            "s" => Self::Sea,
            "b" => Self::Brick,
            "bl" => Self::BrickLeft,
            "br" => Self::BrickRight,
            "bt" => Self::BrickTop,
            "bb" => Self::BrickBottom,
            "i" => Self::Steel,
            "il" => Self::SteelLeft,
            "ir" => Self::SteelRight,
            "it" => Self::SteelTop,
            "ib" => Self::SteelBottom,
            "a" => Self::Barrier,
            _ => Self::Empty,
        }
    }

    /// 转换为符号字符串（用于导出关卡）
    pub fn to_symbol(self) -> &'static str {
        match self {
            Self::Empty => ".",
            Self::Forest => "t",
            Self::Sea => "s",
            Self::Brick => "b",
            Self::BrickLeft => "bl",
            Self::BrickRight => "br",
            Self::BrickTop => "bt",
            Self::BrickBottom => "bb",
            Self::Steel => "i",
            Self::SteelLeft => "il",
            Self::SteelRight => "ir",
            Self::SteelTop => "it",
            Self::SteelBottom => "ib",
            Self::Barrier => "a",
        }
    }

    /// 获取地形的中英文名称（用于UI显示）
    pub fn to_display_name(&self, language: crate::resources::Language) -> &'static str {
        match self {
            Self::Empty => match language {
                crate::resources::Language::Chinese => "空地",
                crate::resources::Language::English => "Empty",
            },
            Self::Forest => match language {
                crate::resources::Language::Chinese => "树林",
                crate::resources::Language::English => "Forest",
            },
            Self::Sea => match language {
                crate::resources::Language::Chinese => "海洋",
                crate::resources::Language::English => "Sea",
            },
            Self::Brick => match language {
                crate::resources::Language::Chinese => "砖块",
                crate::resources::Language::English => "Brick",
            },
            Self::BrickLeft => match language {
                crate::resources::Language::Chinese => "砖块-左",
                crate::resources::Language::English => "Brick-Left",
            },
            Self::BrickRight => match language {
                crate::resources::Language::Chinese => "砖块-右",
                crate::resources::Language::English => "Brick-Right",
            },
            Self::BrickTop => match language {
                crate::resources::Language::Chinese => "砖块-上",
                crate::resources::Language::English => "Brick-Top",
            },
            Self::BrickBottom => match language {
                crate::resources::Language::Chinese => "砖块-下",
                crate::resources::Language::English => "Brick-Bottom",
            },
            Self::Steel => match language {
                crate::resources::Language::Chinese => "钢铁",
                crate::resources::Language::English => "Steel",
            },
            Self::SteelLeft => match language {
                crate::resources::Language::Chinese => "钢铁-左",
                crate::resources::Language::English => "Steel-Left",
            },
            Self::SteelRight => match language {
                crate::resources::Language::Chinese => "钢铁-右",
                crate::resources::Language::English => "Steel-Right",
            },
            Self::SteelTop => match language {
                crate::resources::Language::Chinese => "钢铁-上",
                crate::resources::Language::English => "Steel-Top",
            },
            Self::SteelBottom => match language {
                crate::resources::Language::Chinese => "钢铁-下",
                crate::resources::Language::English => "Steel-Bottom",
            },
            Self::Barrier => match language {
                crate::resources::Language::Chinese => "屏障",
                crate::resources::Language::English => "Barrier",
            },
        }
    }
}

/// 地图配置常量
pub const MAP_ROWS: usize = 12;
pub const MAP_COLS: usize = 16;
pub const GRID_SIZE: f32 = 100.0; // 每个网格的像素大小

/// 将网格坐标转换为世界坐标
pub fn grid_to_world(row: usize, col: usize) -> Vec2 {
    let x = (col as f32).mul_add(GRID_SIZE, MAP_LEFT_X) + GRID_SIZE / 2.0;
    let y = (row as f32).mul_add(-GRID_SIZE, MAP_TOP_Y) - GRID_SIZE / 2.0;
    Vec2::new(x, y)
}

/// 获取所有地图实体
fn query_all_map_entities(
    bricks: Query<Entity, With<Brick>>,
    steels: Query<Entity, With<Steel>>,
    forests: Query<Entity, With<Forest>>,
    seas: Query<Entity, With<Sea>>,
    barriers: Query<Entity, With<Barrier>>,
    walls: Query<Entity, With<Wall>>,
) -> Vec<Entity> {
    bricks
        .iter()
        .chain(steels.iter())
        .chain(forests.iter())
        .chain(seas.iter())
        .chain(barriers.iter())
        .chain(walls.iter())
        .collect()
}

/// 销毁所有地图元素（围墙、砖块、钢、森林、海、障碍物）
pub fn despawn_map(
    mut commands: Commands,
    bricks: Query<Entity, With<Brick>>,
    steels: Query<Entity, With<Steel>>,
    forests: Query<Entity, With<Forest>>,
    seas: Query<Entity, With<Sea>>,
    barriers: Query<Entity, With<Barrier>>,
    walls: Query<Entity, With<Wall>>,
) {
    for entity in query_all_map_entities(bricks, steels, forests, seas, barriers, walls) {
        let () = commands.entity(entity).try_despawn();
    }
}

/// 生成单条边界线
fn spawn_wall_line(commands: &mut Commands, position: Vec3, scale: Vec3) {
    let collider_size = scale.truncate(); // Vec3 -> Vec2
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(COLOR_WHITE, Vec2::ONE),
        Transform {
            translation: position,
            scale,
            ..default()
        },
        // 添加物理碰撞体
        RigidBody::Static,
        Collider::rectangle(collider_size.x, collider_size.y),
        CollisionEventsEnabled,
    ));
}

/// 生成边界线（3条白线：顶部、左侧、右侧）
pub fn spawn_walls(commands: &mut Commands) {
    // 左边界线 - 放在地图边界外侧 1.0 像素（碰撞体厚度 2.0）
    spawn_wall_line(
        commands,
        Vec3::new(MAP_LEFT_X - 1.0, (MAP_TOP_Y + MAP_BOTTOM_Y) / 2.0, 0.0),
        Vec3::new(2.0, MAP_HEIGHT + 4.0, 1.0),
    );

    // 右边界线 - 放在地图边界外侧 1.0 像素（碰撞体厚度 2.0）
    spawn_wall_line(
        commands,
        Vec3::new(MAP_RIGHT_X + 1.0, (MAP_TOP_Y + MAP_BOTTOM_Y) / 2.0, 0.0),
        Vec3::new(2.0, MAP_HEIGHT + 4.0, 1.0),
    );

    // 上边界线 - 放在地图边界外侧 1.0 像素（碰撞体厚度 2.0）
    spawn_wall_line(
        commands,
        Vec3::new(0.0, MAP_TOP_Y + 1.0, 0.0),
        Vec3::new(MAP_WIDTH + 4.0, 2.0, 1.0),
    );

    // 下边界线 - 放在地图边界外侧 1.0 像素（碰撞体厚度 2.0）
    spawn_wall_line(
        commands,
        Vec3::new(0.0, MAP_BOTTOM_Y - 1.0, 0.0),
        Vec3::new(MAP_WIDTH + 4.0, 2.0, 1.0),
    );
}

/// 地形瓦片类型（用于 `spawn_terrain_tile`）
#[derive(Clone, Copy)]
pub enum TerrainTileType {
    Brick,
    Steel,
    Forest,
    Sea,
    Barrier,
}

/// 生成单个地形瓦片（优化的4参数版本）
///
/// 参数：
/// - commands: 命令队列
/// - `asset_server`: 资源服务器
/// - `atlas_layouts`: 地形纹理图集布局资源
/// - position: 生成位置
/// - `tile_type`: 地形类型
pub fn spawn_terrain_tile(
    commands: &mut Commands,
    texture_resources: &Res<GameTextureResources>,
    atlas_layouts: &Res<GameAtlasLayoutResources>,
    position: Vec2,
    tile_type: TerrainTileType,
    tree_color: crate::resources::TreeColor,
) -> Entity {
    match tile_type {
        TerrainTileType::Brick => {
            let brick_texture = texture_resources.brick.clone();
            
            let entity = commands
                .spawn((
                    Brick,
                    PlayingEntity,
                    Sprite {
                        image: brick_texture,
                        custom_size: Some(WALL_TEXTURE_SIZE),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 0.0),
                ))
                .id();

            // 应用物理配置
            crate::physics_config::WALL_PHYSICS.apply_to_entity(&mut commands.entity(entity));

            entity
        }
        TerrainTileType::Steel => {
            let steel_texture = texture_resources.steel.clone();
            
            let entity = commands
                .spawn((
                    Steel,
                    PlayingEntity,
                    Sprite {
                        image: steel_texture,
                        custom_size: Some(WALL_TEXTURE_SIZE),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 0.0),
                ))
                .id();

            // 应用物理配置
            crate::physics_config::WALL_PHYSICS.apply_to_entity(&mut commands.entity(entity));

            entity
        }
        TerrainTileType::Forest => {
            let (tree_texture, forest_layout) = match tree_color {
                crate::resources::TreeColor::Green => {
                    (texture_resources.tree.clone(), atlas_layouts.forest.clone())
                }
                crate::resources::TreeColor::Yellow => {
                    (texture_resources.tree_yellow.clone(), atlas_layouts.forest_yellow.clone())
                }
            };

            let entity = crate::utils::spawn_animated_sprite(
                commands,
                tree_texture,
                forest_layout,
                crate::atlas::FOREST_ATLAS.animation_indices_full(),
                ANIMATION_FRAME_FOREST,
                Transform::from_translation(Vec3::new(position.x, position.y, Z_FOREST)),
                crate::atlas::FOREST_ATLAS.display_size,
                (Forest, PlayingEntity, AnimationMode::Looping),
            );
            crate::physics_config::FOREST_PHYSICS.apply_to_entity(&mut commands.entity(entity));
            entity
        }
        TerrainTileType::Sea => {
            let entity = crate::utils::spawn_animated_sprite(
                commands,
                texture_resources.sea.clone(),
                atlas_layouts.sea.clone(),
                crate::atlas::SEA_ATLAS.animation_indices_full(),
                ANIMATION_FRAME_SEA,
                Transform::from_translation(Vec3::new(position.x, position.y, Z_SEA)),
                crate::atlas::SEA_ATLAS.display_size,
                (Sea, PlayingEntity, AnimationMode::Looping),
            );
            crate::physics_config::sea_physics().apply_to_entity(&mut commands.entity(entity));
            entity
        }
        TerrainTileType::Barrier => {
            let barrier_texture = texture_resources.barrier.clone();
            
            let entity = commands
                .spawn((
                    Barrier,
                    PlayingEntity,
                    Sprite {
                        image: barrier_texture,
                        custom_size: Some(BARRIER_SIZE),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 0.0),
                ))
                .id();

            // 应用物理配置
            crate::physics_config::BARRIER_PHYSICS.apply_to_entity(&mut commands.entity(entity));

            entity
        }
    }
}

/// 地形瓦片布局类型
#[derive(Clone, Copy)]
pub enum TileLayout {
    /// 2x2 网格（4个瓦片）
    Full,
    /// 左半（2x1 网格，2个瓦片）
    Left,
    /// 右半（2x1 网格，2个瓦片）
    Right,
    /// 上半（1x2 网格，2个瓦片）
    Top,
    /// 下半（1x2 网格，2个瓦片）
    Bottom,
}

/// 生成地形瓦片组（通用函数）
///
/// 参数：
/// - commands: 命令队列
/// - asset_server: 资源服务器
/// - atlas_layouts: 地形纹理图集布局资源
/// - center_position: 中心位置
/// - tile_type: 地形类型（砖块或钢块）
/// - layout: 瓦片布局类型
///
/// 返回：生成的实体数组（2或4个，取决于布局类型）
pub fn spawn_tile_group(
    commands: &mut Commands,
    texture_resources: &Res<GameTextureResources>,
    atlas_layouts: &Res<GameAtlasLayoutResources>,
    center_position: Vec2,
    tile_type: TerrainTileType,
    layout: TileLayout,
    tree_color: crate::resources::TreeColor,
) -> Vec<Entity> {
    let positions = match layout {
        TileLayout::Full => vec![
            BRICK_GROUP_TOP_LEFT,
            BRICK_GROUP_TOP_RIGHT,
            BRICK_GROUP_BOTTOM_LEFT,
            BRICK_GROUP_BOTTOM_RIGHT,
        ],
        TileLayout::Left => vec![BRICK_GROUP_TOP_LEFT, BRICK_GROUP_BOTTOM_LEFT],
        TileLayout::Right => vec![BRICK_GROUP_TOP_RIGHT, BRICK_GROUP_BOTTOM_RIGHT],
        TileLayout::Top => vec![BRICK_GROUP_TOP_LEFT, BRICK_GROUP_TOP_RIGHT],
        TileLayout::Bottom => vec![BRICK_GROUP_BOTTOM_LEFT, BRICK_GROUP_BOTTOM_RIGHT],
    };

    positions
        .into_iter()
        .map(|pos| {
            spawn_terrain_tile(
                commands,
                texture_resources,
                atlas_layouts,
                center_position + pos,
                tile_type,
                tree_color,
            )
        })
        .collect()
}

/// 地形配置：将 TerrainType 映射到 (TerrainTileType, Option<TileLayout>)
fn get_terrain_config(terrain: TerrainType) -> Option<(TerrainTileType, Option<TileLayout>)> {
    match terrain {
        TerrainType::Brick => Some((TerrainTileType::Brick, Some(TileLayout::Full))),
        TerrainType::BrickLeft => Some((TerrainTileType::Brick, Some(TileLayout::Left))),
        TerrainType::BrickRight => Some((TerrainTileType::Brick, Some(TileLayout::Right))),
        TerrainType::BrickTop => Some((TerrainTileType::Brick, Some(TileLayout::Top))),
        TerrainType::BrickBottom => Some((TerrainTileType::Brick, Some(TileLayout::Bottom))),
        TerrainType::Steel => Some((TerrainTileType::Steel, Some(TileLayout::Full))),
        TerrainType::SteelLeft => Some((TerrainTileType::Steel, Some(TileLayout::Left))),
        TerrainType::SteelRight => Some((TerrainTileType::Steel, Some(TileLayout::Right))),
        TerrainType::SteelTop => Some((TerrainTileType::Steel, Some(TileLayout::Top))),
        TerrainType::SteelBottom => Some((TerrainTileType::Steel, Some(TileLayout::Bottom))),
        TerrainType::Forest => Some((TerrainTileType::Forest, None)),
        TerrainType::Sea => Some((TerrainTileType::Sea, None)),
        TerrainType::Barrier => Some((TerrainTileType::Barrier, None)),
        TerrainType::Empty => None,
    }
}

fn spawn_map_terrain(
    commands: &mut Commands,
    texture_resources: &Res<GameTextureResources>,
    atlas_layouts: &Res<GameAtlasLayoutResources>,
    level_assets: &mut crate::levels::LevelAssets,
    stage_level: usize,
    tree_color: crate::resources::TreeColor,
) {
    let level_map = level_assets.get(stage_level).unwrap_or_else(|| {
        [[TerrainType::Empty; MAP_COLS]; MAP_ROWS]
    });

    for (row, row_data) in level_map.iter().enumerate().take(MAP_ROWS) {
        for (col, terrain) in row_data.iter().enumerate().take(MAP_COLS) {
            if *terrain == TerrainType::Empty {
                continue;
            }

            let pos = grid_to_world(row, col);

            if let Some((tile_type, layout)) = get_terrain_config(*terrain) {
                match layout {
                    Some(layout_type) => {
                        spawn_tile_group(
                            commands,
                            texture_resources,
                            atlas_layouts,
                            pos,
                            tile_type,
                            layout_type,
                            tree_color,
                        );
                    }
                    None => {
                        spawn_terrain_tile(
                            commands,
                            texture_resources,
                            atlas_layouts,
                            pos,
                            tile_type,
                            tree_color,
                        );
                    }
                }
            }
        }
    }
}

/// 生成游戏地图（包括围墙、地形和司令官堡垒）
pub fn spawn_map(
    mut commands: Commands,
    texture_resources: Res<GameTextureResources>,
    atlas_layouts: Res<GameAtlasLayoutResources>,
    mut level_assets: ResMut<crate::levels::LevelAssets>,
    mut clear_color: ResMut<ClearColor>,
    stage_level: Res<StageLevel>,
    mut tree_color: ResMut<crate::resources::TreeColor>,
    weather: Res<crate::weather::CurrentWeather>,
    mut global_rng: ResMut<crate::global_rng::GlobalRng>,
    bricks: Query<Entity, With<Brick>>,
    steels: Query<Entity, With<Steel>>,
    forests: Query<Entity, With<Forest>>,
    seas: Query<Entity, With<Sea>>,
    barriers: Query<Entity, With<Barrier>>,
    walls: Query<Entity, With<Wall>>,
) {
    // 防御性编程：先清理所有可能存在的地图实体
    for entity in query_all_map_entities(bricks, steels, forests, seas, barriers, walls) {
        let () = commands.entity(entity).try_despawn();
    }

    // 根据天气选择树木颜色：下雪时强制使用黄色（秋冬季节）
    *tree_color = if weather.weather_type == crate::weather::WeatherType::Snow {
        crate::resources::TreeColor::Yellow
    } else {
        if global_rng.gen_bool() {
            crate::resources::TreeColor::Green
        } else {
            crate::resources::TreeColor::Yellow
        }
    };

    // 设置背景色为黑色
    clear_color.0 = COLOR_BACKGROUND;

    // 生成围墙
    spawn_walls(&mut commands);

    // 根据地图数组生成地形
    spawn_map_terrain(
        &mut commands,
        &texture_resources,
        &atlas_layouts,
        &mut level_assets,
        stage_level.0,
        *tree_color,
    );

    // 生成司令官堡垒墙
    spawn_commander_fortress(&mut commands, &texture_resources, &atlas_layouts, *tree_color);
}

/// 生成一列砖块墙
fn spawn_wall_column(
    commands: &mut Commands,
    texture_resources: &Res<GameTextureResources>,
    atlas_layouts: &Res<GameAtlasLayoutResources>,
    fixed_axis: f32,
    start_pos: f32,
    brick_size: f32,
    count: usize,
    is_vertical: bool,
    tree_color: crate::resources::TreeColor,
) {
    for i in 0..count {
        let pos = (i as f32).mul_add(brick_size, start_pos + brick_size / 2.0);
        let position = if is_vertical {
            Vec2::new(fixed_axis, pos)
        } else {
            Vec2::new(pos, fixed_axis)
        };
        spawn_terrain_tile(
            commands,
            texture_resources,
            atlas_layouts,
            position,
            TerrainTileType::Brick,
            tree_color,
        );
    }
}

/// 生成包围司令官的三面砖块堡垒墙（左、右、上）
pub fn spawn_commander_fortress(
    commands: &mut Commands,
    texture_resources: &Res<GameTextureResources>,
    atlas_layouts: &Res<GameAtlasLayoutResources>,
    tree_color: crate::resources::TreeColor,
) {
    let commander_y = MAP_BOTTOM_Y + COMMANDER_SIZE.y / 2.0;

    // 司令官边界
    let commander_left = -COMMANDER_SIZE.x / 2.0;
    let commander_right = COMMANDER_SIZE.x / 2.0;
    let commander_top = commander_y + COMMANDER_SIZE.y / 2.0;
    let commander_bottom = commander_y - COMMANDER_SIZE.y / 2.0;

    let brick_size = WALL_TEXTURE_SIZE.x;

    // 左墙：3块砖
    spawn_wall_column(
        commands,
        texture_resources,
        atlas_layouts,
        commander_left - brick_size / 2.0,
        commander_bottom,
        brick_size,
        3,
        true,
        tree_color,
    );

    // 右墙：3块砖
    spawn_wall_column(
        commands,
        texture_resources,
        atlas_layouts,
        commander_right + brick_size / 2.0,
        commander_bottom,
        brick_size,
        3,
        true,
        tree_color,
    );

    // 上墙：2块砖
    spawn_wall_column(
        commands,
        texture_resources,
        atlas_layouts,
        commander_top + brick_size / 2.0,
        -COMMANDER_SIZE.x / 2.0,
        brick_size,
        2,
        false,
        tree_color,
    );
}
