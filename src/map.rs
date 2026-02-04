//! 地图和地形生成模块
//!
//! 包含地图数据定义（TerrainType、网格系统）和地形实体生成功能
//! 处理墙壁、砖块、钢块、森林、海等地形元素的生成与管理

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::{
    MapResources, StageLevel, TerrainAtlasLayouts,
};

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
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(COLOR_WHITE, Vec2::ONE),
        Transform { translation: position, scale, ..default() },
    ));
}

/// 生成边界线（3条白线：顶部、左侧、右侧）
pub fn spawn_walls(commands: &mut Commands) {
    // 左边界线
    spawn_wall_line(
        commands,
        Vec3::new(MAP_LEFT_X, (MAP_TOP_Y + MAP_BOTTOM_Y) / 2.0, 0.0),
        Vec3::new(5.0, MAP_HEIGHT, 1.0),
    );

    // 右边界线
    spawn_wall_line(
        commands,
        Vec3::new(MAP_RIGHT_X, (MAP_TOP_Y + MAP_BOTTOM_Y) / 2.0, 0.0),
        Vec3::new(5.0, MAP_HEIGHT, 1.0),
    );

    // 上边界线
    spawn_wall_line(
        commands,
        Vec3::new(0.0, MAP_TOP_Y, 0.0),
        Vec3::new(MAP_WIDTH + 4.0, 5.0, 1.0),
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
    map_resources: &Res<MapResources>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    position: Vec2,
    tile_type: TerrainTileType,
) -> Entity {
    match tile_type {
        TerrainTileType::Brick => {
            let brick_texture = map_resources.brick.clone();
            commands
                .spawn((
                    Brick,
                    PlayingEntity,
                    Sprite {
                        image: brick_texture,
                        custom_size: Some(Vec2::new(BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT)),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 0.0),
                    RigidBody::Fixed,
                    Collider::cuboid(BRICK_COLLIDER_WIDTH / 2.0, BRICK_COLLIDER_HEIGHT / 2.0),
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::all(),
                ))
                .id()
        }
        TerrainTileType::Steel => {
            let steel_texture = map_resources.steel.clone();
            commands
                .spawn((
                    Steel,
                    PlayingEntity,
                    Sprite {
                        image: steel_texture,
                        custom_size: Some(Vec2::new(STEEL_TEXTURE_WIDTH, STEEL_TEXTURE_HEIGHT)),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 0.0),
                    RigidBody::Fixed,
                    Collider::cuboid(STEEL_COLLIDER_WIDTH / 2.0, STEEL_COLLIDER_HEIGHT / 2.0),
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::all(),
                ))
                .id()
        }
        TerrainTileType::Forest => {
            let forest_texture = map_resources.tree.clone();
            let forest_animation_indices = AnimationIndices { first: 0, last: 9 };
            commands
                .spawn((
                    Forest,
                    PlayingEntity,
                    AnimationMode::Looping,
                    Sprite::from_atlas_image(
                        forest_texture,
                        TextureAtlas {
                            layout: atlas_layouts.forest.clone(),
                            index: forest_animation_indices.first,
                        },
                    ),
                    Transform::from_xyz(position.x, position.y, Z_FOREST),
                    forest_animation_indices,
                    AnimationTimer(Timer::from_seconds(
                        ANIMATION_FRAME_FOREST,
                        TimerMode::Repeating,
                    )),
                    CurrentAnimationFrame(0),
                    Collider::cuboid(FOREST_COLLIDER_HALF / 2.0, FOREST_COLLIDER_HALF / 2.0),
                    RigidBody::Fixed,
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                ))
                .id()
        }
        TerrainTileType::Sea => {
            let sea_texture = map_resources.sea.clone();
            let sea_animation_indices = AnimationIndices { first: 0, last: 2 };
            commands
                .spawn((
                    Sea,
                    PlayingEntity,
                    AnimationMode::Looping,
                    Sprite::from_atlas_image(
                        sea_texture,
                        TextureAtlas {
                            layout: atlas_layouts.sea.clone(),
                            index: sea_animation_indices.first,
                        },
                    ),
                    Transform::from_xyz(position.x, position.y, Z_SEA),
                    sea_animation_indices,
                    AnimationTimer(Timer::from_seconds(
                        ANIMATION_FRAME_SEA,
                        TimerMode::Repeating,
                    )),
                    CurrentAnimationFrame(0),
                    RigidBody::Fixed,
                    Collider::cuboid(DETECTION_RADIUS / 2.0, DETECTION_RADIUS / 2.0),
                    CollisionGroups::new(SEA_GROUP, Group::all()),
                ))
                .id()
        }
        TerrainTileType::Barrier => {
            let barrier_texture = map_resources.barrier.clone();
            commands
                .spawn((
                    Barrier,
                    PlayingEntity,
                    Sprite {
                        image: barrier_texture,
                        custom_size: Some(Vec2::new(BARRIER_WIDTH, BARRIER_HEIGHT)),
                        ..default()
                    },
                    Transform::from_xyz(position.x, position.y, 0.0),
                    RigidBody::Fixed,
                    Collider::cuboid(BARRIER_WIDTH / 2.0, BARRIER_HEIGHT / 2.0),
                    Sensor,
                    ActiveEvents::COLLISION_EVENTS,
                    ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
                ))
                .id()
        }
    }
}

/// 地形瓦片布局类型
#[derive(Clone, Copy)]
pub(crate) enum TileLayout {
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
    map_resources: &Res<MapResources>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
    tile_type: TerrainTileType,
    layout: TileLayout,
) -> Vec<Entity> {
    let offset = BRICK_GROUP_OFFSET;
    let positions = match layout {
        TileLayout::Full => vec![
            Vec2::new(-offset, offset),
            Vec2::new(offset, offset),
            Vec2::new(-offset, -offset),
            Vec2::new(offset, -offset),
        ],
        TileLayout::Left => vec![Vec2::new(-offset, offset), Vec2::new(-offset, -offset)],
        TileLayout::Right => vec![Vec2::new(offset, offset), Vec2::new(offset, -offset)],
        TileLayout::Top => vec![Vec2::new(-offset, offset), Vec2::new(offset, offset)],
        TileLayout::Bottom => vec![Vec2::new(-offset, -offset), Vec2::new(offset, -offset)],
    };

    positions
        .into_iter()
        .map(|pos| {
            spawn_terrain_tile(
                commands,
                map_resources,
                atlas_layouts,
                center_position + pos,
                tile_type,
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
    map_resources: &Res<MapResources>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    level_assets: &mut crate::levels::LevelAssets,
    stage_level: usize,
) {
    let level_map = crate::levels::get_level_from_assets(level_assets, stage_level);

    for (row, row_data) in level_map.iter().enumerate().take(MAP_ROWS) {
        for (col, terrain) in row_data.iter().enumerate().take(MAP_COLS) {
            if *terrain == TerrainType::Empty {
                continue;
            }

            let pos = grid_to_world(row, col);

            if let Some((tile_type, layout)) = get_terrain_config(*terrain) {
                match layout {
                    Some(layout_type) => {
                        spawn_tile_group(commands, map_resources, atlas_layouts, pos, tile_type, layout_type);
                    }
                    None => {
                        spawn_terrain_tile(commands, map_resources, atlas_layouts, pos, tile_type);
                    }
                }
            }
        }
    }
}

/// 生成游戏地图（包括围墙、地形和司令官堡垒）
pub fn spawn_map(
    mut commands: Commands,
    map_resources: Res<MapResources>,
    atlas_layouts: Res<TerrainAtlasLayouts>,
    mut level_assets: ResMut<crate::levels::LevelAssets>,
    mut clear_color: ResMut<ClearColor>,
    stage_level: Res<StageLevel>,
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

    // 设置背景色为黑色
    clear_color.0 = COLOR_BACKGROUND;

    // 生成围墙
    spawn_walls(&mut commands);

    // 根据地图数组生成地形
    spawn_map_terrain(
        &mut commands,
        &map_resources,
        &atlas_layouts,
        &mut level_assets,
        stage_level.0,
    );

    // 生成司令官堡垒墙
    spawn_commander_fortress(&mut commands, &map_resources, &atlas_layouts);
}

/// 生成一列砖块墙
fn spawn_wall_column(
    commands: &mut Commands,
    map_resources: &Res<MapResources>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    fixed_axis: f32,
    start_pos: f32,
    brick_size: f32,
    count: usize,
    is_vertical: bool,
) {
    for i in 0..count {
        let pos = (i as f32).mul_add(brick_size, start_pos + brick_size / 2.0);
        let position = if is_vertical {
            Vec2::new(fixed_axis, pos)
        } else {
            Vec2::new(pos, fixed_axis)
        };
        spawn_terrain_tile(commands, map_resources, atlas_layouts, position, TerrainTileType::Brick);
    }
}

/// 生成包围司令官的三面砖块堡垒墙（左、右、上）
pub fn spawn_commander_fortress(
    commands: &mut Commands,
    map_resources: &Res<MapResources>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
) {
    let commander_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT / 2.0;

    // 司令官边界
    let commander_left = -COMMANDER_WIDTH / 2.0;
    let commander_right = COMMANDER_WIDTH / 2.0;
    let commander_top = commander_y + COMMANDER_HEIGHT / 2.0;
    let commander_bottom = commander_y - COMMANDER_HEIGHT / 2.0;

    let brick_size = COMMANDER_BRICK_SIZE;

    // 左墙：3块砖
    spawn_wall_column(
        commands,
        map_resources,
        atlas_layouts,
        commander_left - brick_size / 2.0,
        commander_bottom,
        brick_size,
        3,
        true,
    );

    // 右墙：3块砖
    spawn_wall_column(
        commands,
        map_resources,
        atlas_layouts,
        commander_right + brick_size / 2.0,
        commander_bottom,
        brick_size,
        3,
        true,
    );

    // 上墙：2块砖
    spawn_wall_column(
        commands,
        map_resources,
        atlas_layouts,
        commander_top + brick_size / 2.0,
        -COMMANDER_WIDTH / 2.0,
        brick_size,
        2,
        false,
    );
}

