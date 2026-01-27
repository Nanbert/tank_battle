//! 地形和实体生成模块
//!
//! 处理墙壁、地图地形、指挥官、玩家坦克等实体生成

#![allow(clippy::wildcard_imports)]

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constants::*;
use crate::resources::{
    StageLevel, TerrainAtlasLayouts,
};

/// 生成墙壁
pub fn spawn_walls(commands: &mut Commands) {
    // 左墙（在原游戏区域左边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, MAP_TOP_Y / 100.0),
        Transform {
            translation: Vec3::new(MAP_LEFT_X - WALL_POSITION_OFFSET_2, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(WALL_SCALE, MAP_HEIGHT, 1.0),
            ..default()
        },
    ));

    // 右墙（在原游戏区域右边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, MAP_TOP_Y / 100.0),
        Transform {
            translation: Vec3::new(MAP_RIGHT_X + WALL_POSITION_OFFSET_2, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(WALL_SCALE, MAP_HEIGHT, 1.0),
            ..default()
        },
    ));

    // 上墙（在原游戏区域上边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(MAP_RIGHT_X / 100.0, 0.1),
        Transform {
            translation: Vec3::new(0.0, MAP_TOP_Y + WALL_POSITION_OFFSET_2, 0.0),
            scale: Vec3::new(MAP_WIDTH, WALL_SCALE, 1.0),
            ..default()
        },
    ));

    // 下墙（在原游戏区域下边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(MAP_RIGHT_X / 100.0, 0.1),
        Transform {
            translation: Vec3::new(0.0, MAP_BOTTOM_Y - WALL_POSITION_OFFSET_2, 0.0),
            scale: Vec3::new(MAP_WIDTH, WALL_SCALE, 1.0),
            ..default()
        },
    ));
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
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    position: Vec2,
    tile_type: TerrainTileType,
) -> Entity {
    match tile_type {
        TerrainTileType::Brick => {
            let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
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
            let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
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
            let forest_texture: Handle<Image> = asset_server.load("maps/tree.png");
            let forest_animation_indices = AnimationIndices { first: 0, last: 9 };
            commands
                .spawn((
                    Forest,
                    PlayingEntity,
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
            let sea_texture: Handle<Image> = asset_server.load(TEXTURE_SEA);
            let sea_animation_indices = AnimationIndices { first: 0, last: 2 };
            commands
                .spawn((
                    Sea,
                    PlayingEntity,
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
            let barrier_texture: Handle<Image> = asset_server.load(TEXTURE_BARRIER);
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
    asset_server: &Res<AssetServer>,
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
                asset_server,
                atlas_layouts,
                center_position + pos,
                tile_type,
            )
        })
        .collect()
}

/// 生成砖块组（2x2网格，100x100）
pub fn spawn_brick_group(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Brick,
        TileLayout::Full,
    )
}

/// 生成钢块组（2x2网格，100x100）
pub fn spawn_steel_group(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Steel,
        TileLayout::Full,
    )
}

/// 生成砖块左半（2x1网格，50x100）
pub fn spawn_brick_left(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Brick,
        TileLayout::Left,
    )
}

/// 生成砖块右半（2x1网格，50x100）
pub fn spawn_brick_right(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Brick,
        TileLayout::Right,
    )
}

/// 生成砖块上半（1x2网格，100x50）
pub fn spawn_brick_top(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Brick,
        TileLayout::Top,
    )
}

/// 生成砖块下半（1x2网格，100x50）
pub fn spawn_brick_bottom(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Brick,
        TileLayout::Bottom,
    )
}

/// 生成钢块左半（2x1网格，50x100）
pub fn spawn_steel_left(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Steel,
        TileLayout::Left,
    )
}

/// 生成钢块右半（2x1网格，50x100）
pub fn spawn_steel_right(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Steel,
        TileLayout::Right,
    )
}

/// 生成钢块上半（1x2网格，100x50）
pub fn spawn_steel_top(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Steel,
        TileLayout::Top,
    )
}

/// 生成钢块下半（1x2网格，100x50）
pub fn spawn_steel_bottom(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    center_position: Vec2,
) -> Vec<Entity> {
    spawn_tile_group(
        commands,
        asset_server,
        atlas_layouts,
        center_position,
        TerrainTileType::Steel,
        TileLayout::Bottom,
    )
}

fn spawn_map_terrain(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
    level_assets: &crate::levels::LevelAssets,
    stage_level: usize,
) {
    use crate::map::{MAP_COLS, MAP_ROWS, TerrainType, grid_to_world};

    let level_map = crate::levels::get_level_from_assets(level_assets, stage_level);

    for (row, row_data) in level_map.iter().enumerate().take(MAP_ROWS) {
        for (col, terrain) in row_data.iter().enumerate().take(MAP_COLS) {
            if *terrain == TerrainType::Empty {
                continue;
            }

            let pos = grid_to_world(row, col);

            match terrain {
                TerrainType::Forest => {
                    spawn_terrain_tile(
                        commands,
                        asset_server,
                        atlas_layouts,
                        pos,
                        TerrainTileType::Forest,
                    );
                }
                TerrainType::Sea => {
                    spawn_terrain_tile(
                        commands,
                        asset_server,
                        atlas_layouts,
                        pos,
                        TerrainTileType::Sea,
                    );
                }
                TerrainType::Brick => {
                    spawn_brick_group(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::BrickLeft => {
                    spawn_brick_left(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::BrickRight => {
                    spawn_brick_right(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::BrickTop => {
                    spawn_brick_top(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::BrickBottom => {
                    spawn_brick_bottom(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::Steel => {
                    spawn_steel_group(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::SteelLeft => {
                    spawn_steel_left(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::SteelRight => {
                    spawn_steel_right(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::SteelTop => {
                    spawn_steel_top(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::SteelBottom => {
                    spawn_steel_bottom(commands, asset_server, atlas_layouts, pos);
                }
                TerrainType::Barrier => {
                    spawn_terrain_tile(
                        commands,
                        asset_server,
                        atlas_layouts,
                        pos,
                        TerrainTileType::Barrier,
                    );
                }
                TerrainType::Empty => {}
            }
        }
    }
}

/// 生成游戏地图（包括围墙、地形和司令官堡垒）
pub fn spawn_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    atlas_layouts: Res<TerrainAtlasLayouts>,
    level_assets: Res<crate::levels::LevelAssets>,
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
    for entity in bricks
        .iter()
        .chain(steels.iter())
        .chain(forests.iter())
        .chain(seas.iter())
        .chain(barriers.iter())
        .chain(walls.iter())
    {
        let () = commands.entity(entity).try_despawn();
    }

    // 设置背景色为黑色
    clear_color.0 = BACKGROUND_COLOR;

    // 生成围墙
    spawn_walls(&mut commands);

    // 根据地图数组生成地形
    spawn_map_terrain(
        &mut commands,
        &asset_server,
        &atlas_layouts,
        &level_assets,
        stage_level.0,
    );

    // 生成司令官堡垒墙
    spawn_commander_fortress(&mut commands, &asset_server, &atlas_layouts);
}

/// 生成包围司令官的三面砖块堡垒墙（左、右、上）
pub fn spawn_commander_fortress(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &Res<TerrainAtlasLayouts>,
) {
    let commander_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT / 2.0;

    // 司令官边界
    let commander_left = -COMMANDER_WIDTH / 2.0;
    let commander_right = COMMANDER_WIDTH / 2.0;
    let commander_top = commander_y + COMMANDER_HEIGHT / 2.0;
    let commander_bottom = commander_y - COMMANDER_HEIGHT / 2.0;

    let brick_size = COMMANDER_BRICK_SIZE;

    // 左墙：3块砖，紧贴司令官左侧
    for i in 0..3 {
        let y = (i as f32).mul_add(brick_size, commander_bottom + brick_size / 2.0);
        spawn_terrain_tile(
            commands,
            asset_server,
            atlas_layouts,
            Vec2::new(commander_left - brick_size / 2.0, y),
            TerrainTileType::Brick,
        );
    }

    // 右墙：3块砖，紧贴司令官右侧
    for i in 0..3 {
        let y = (i as f32).mul_add(brick_size, commander_bottom + brick_size / 2.0);
        spawn_terrain_tile(
            commands,
            asset_server,
            atlas_layouts,
            Vec2::new(commander_right + brick_size / 2.0, y),
            TerrainTileType::Brick,
        );
    }

    // 上墙：2块砖封顶，紧贴司令官顶部
    for i in 0..2 {
        let x = (i as f32).mul_add(brick_size, -brick_size / 2.0);
        spawn_terrain_tile(
            commands,
            asset_server,
            atlas_layouts,
            Vec2::new(x, commander_top + brick_size / 2.0),
            TerrainTileType::Brick,
        );
    }
}

