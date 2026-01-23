//! 地形和实体生成模块
//!
//! 处理墙壁、地图地形、指挥官、玩家坦克、道具等实体生成

use bevy::prelude::*;
use bevy::audio::Volume;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::constants::*;
use crate::resources::*;

/// 生成墙壁
pub fn spawn_walls(commands: &mut Commands) {
    // 左墙（在原游戏区域左边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, MAP_TOP_Y / 100.0),
        Transform{
            translation: Vec3::new(MAP_LEFT_X - 5.0, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(10.0 , MAP_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 右墙（在原游戏区域右边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(0.1, MAP_TOP_Y / 100.0),
        Transform{
            translation: Vec3::new(MAP_RIGHT_X + 5.0, VERTICAL_OFFSET, 0.0),
            scale: Vec3::new(10.0 , MAP_HEIGHT, 1.0),
            ..default()
        }
    ));

    // 上墙（在原游戏区域上边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(MAP_RIGHT_X / 100.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, MAP_TOP_Y + 5.0, 0.0),
            scale: Vec3::new(MAP_WIDTH, 10.0, 1.0),
            ..default()
        }
    ));

    // 下墙（在原游戏区域下边界，向下平移40像素）
    commands.spawn((
        Wall,
        PlayingEntity,
        Sprite::from_color(Color::srgb(0.8, 0.8, 0.8), Vec2::ONE),
        RigidBody::Fixed,
        Collider::cuboid(MAP_RIGHT_X / 100.0, 0.1),
        Transform{
            translation: Vec3::new(0.0, MAP_BOTTOM_Y -5.0, 0.0),
            scale: Vec3::new(MAP_WIDTH, 10.0 , 1.0),
            ..default()
        }
    ));
}

pub fn is_stat_at_max_value(text: &str, player_stats: &PlayerStats) -> bool {
    match text {
        s if s.starts_with("Speed") => player_stats.speed >= 100,
        s if s.starts_with("Shells") => player_stats.shells >= 5,
        s if s.starts_with("Protection") => player_stats.protection >= 100,
        s if s.starts_with("Fire Speed") => player_stats.fire_speed >= 100,
        _ => false,
    }
}
fn spawn_map_terrain(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    stage_level: usize,
) {
    use crate::map::{TerrainType, grid_to_world, MAP_ROWS, MAP_COLS};

    let level_map = crate::levels::get_level(stage_level);

    for row in 0..MAP_ROWS {
        for col in 0..MAP_COLS {
            let terrain = level_map[row][col];
            if terrain == TerrainType::Empty {
                continue;
            }

            let pos = grid_to_world(row, col);

            match terrain {
                TerrainType::Forest => {
                    let forest_texture: Handle<Image> = asset_server.load("maps/tree.png");
                    let forest_tile_size = UVec2::new(131, 131);
                    let forest_texture_atlas = TextureAtlasLayout::from_grid(forest_tile_size, 10, 1, None, None);
                    let forest_texture_atlas_layout = texture_atlas_layouts.add(forest_texture_atlas);
                    let forest_animation_indices = AnimationIndices { first: 0, last: 9 };

                    commands.spawn((
                        Forest,
                        PlayingEntity,
                        Sprite::from_atlas_image(
                            forest_texture,
                            TextureAtlas {
                                layout: forest_texture_atlas_layout,
                                index: forest_animation_indices.first,
                            }
                        ),
                        Transform::from_xyz(pos.x, pos.y, 1.0),
                        forest_animation_indices,
                        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                        CurrentAnimationFrame(0),
                        Collider::cuboid(131.0 / 2.0, 131.0 / 2.0),
                        RigidBody::Fixed,
                        Sensor,
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_KINEMATIC,
                    ));
                }
                TerrainType::Sea => {
                    let sea_texture: Handle<Image> = asset_server.load(TEXTURE_SEA);
                    let sea_tile_size = UVec2::new(100, 100);
                    let sea_texture_atlas = TextureAtlasLayout::from_grid(sea_tile_size, 3, 1, None, None);
                    let sea_texture_atlas_layout = texture_atlas_layouts.add(sea_texture_atlas);
                    let sea_animation_indices = AnimationIndices { first: 0, last: 2 };

                    commands.spawn((
                        Sea,
                        PlayingEntity,
                        Sprite::from_atlas_image(
                            sea_texture,
                            TextureAtlas {
                                layout: sea_texture_atlas_layout,
                                index: sea_animation_indices.first,
                            }
                        ),
                        Transform::from_xyz(pos.x, pos.y, -0.5),
                        sea_animation_indices,
                        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                        CurrentAnimationFrame(0),
                        RigidBody::Fixed,
                        Collider::cuboid(100.0 / 2.0, 100.0 / 2.0),
                        CollisionGroups::new(SEA_GROUP, Group::all()),
                    ));
                }
                TerrainType::Brick => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    // 生成4块砖块组成100x100的网格
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickLeft => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(-offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickRight => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(offset, offset),
                        Vec2::new(offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickTop => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::BrickBottom => {
                    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for brick_pos in positions {
                        commands.spawn((
                            Brick,
                            PlayingEntity,
                            Sprite {
                                image: brick_texture.clone(),
                                custom_size: Some(Vec2::new(BRICK_WIDTH, BRICK_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + brick_pos.x, pos.y + brick_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(BRICK_WIDTH / 2.0, BRICK_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::Steel => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    // 生成4块钢铁组成100x100的网格
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelLeft => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(-offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelRight => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(offset, offset),
                        Vec2::new(offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelTop => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, offset),
                        Vec2::new(offset, offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::SteelBottom => {
                    let steel_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL);
                    let offset = 25.0;
                    let positions = [
                        Vec2::new(-offset, -offset),
                        Vec2::new(offset, -offset),
                    ];
                    for steel_pos in positions {
                        commands.spawn((
                            Steel,
                            PlayingEntity,
                            Sprite {
                                image: steel_texture.clone(),
                                custom_size: Some(Vec2::new(STEEL_WIDTH, STEEL_HEIGHT)),
                                ..default()
                            },
                            Transform::from_xyz(pos.x + steel_pos.x, pos.y + steel_pos.y, 0.0),
                            RigidBody::Fixed,
                            Collider::cuboid(STEEL_WIDTH / 2.0, STEEL_HEIGHT / 2.0),
                            ActiveEvents::COLLISION_EVENTS,
                            ActiveCollisionTypes::all(),
                        ));
                    }
                }
                TerrainType::Barrier => {
                    let barrier_texture: Handle<Image> = asset_server.load(TEXTURE_BARRIER);
                    commands.spawn((
                        Barrier,
                        PlayingEntity,
                        Sprite {
                            image: barrier_texture,
                            custom_size: Some(Vec2::new(BARRIER_WIDTH, BARRIER_HEIGHT)),
                            ..default()
                        },
                        Transform::from_xyz(pos.x, pos.y, 0.0),
                        RigidBody::Fixed,
                        Collider::cuboid(BARRIER_WIDTH / 2.0, BARRIER_HEIGHT / 2.0),
                        Sensor,
                        ActiveEvents::COLLISION_EVENTS,
                        ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC,
                    ));
                }
                TerrainType::Empty => {}
            }
        }
    }
}






fn spawn_commander(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) {
    let commander_texture: Handle<Image> = asset_server.load(TEXTURE_COMMANDER);
    // commander.png 实际尺寸: 1400x1200, 每帧 140x120, 10列 x 10行, 共100帧
    let commander_tile_size = UVec2::new(140, 120);
    let commander_texture_atlas = TextureAtlasLayout::from_grid(commander_tile_size, 10, 10, None, None);
    let commander_texture_atlas_layout = texture_atlas_layouts.add(commander_texture_atlas);
    let commander_animation_indices = AnimationIndices { first: 0, last: 99 };

    let commander_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT / 2.0;
    let commander_x = 0.0;

    // 创建包围司令官的砖块堡垒墙
    let brick_texture: Handle<Image> = asset_server.load(TEXTURE_BRICK);
    let brick_size = 50.0; // 每块砖的大小

    // 司令官边界
    let commander_left = -COMMANDER_WIDTH / 2.0;
    let commander_right = COMMANDER_WIDTH / 2.0;
    let commander_top = commander_y + COMMANDER_HEIGHT / 2.0;
    let commander_bottom = commander_y - COMMANDER_HEIGHT / 2.0;

    // 左墙：3块砖，紧贴司令官左侧
    for i in 0..3 {
        let y = commander_bottom + brick_size / 2.0 + i as f32 * brick_size;
        commands.spawn((
            Brick,
            PlayingEntity,
            Sprite {
                image: brick_texture.clone(),
                custom_size: Some(Vec2::new(brick_size, brick_size)),
                ..default()
            },
            Transform::from_xyz(commander_left - brick_size / 2.0, y, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(brick_size / 2.0, brick_size / 2.0),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::all(),
        ));
    }

    // 右墙：3块砖，紧贴司令官右侧
    for i in 0..3 {
        let y = commander_bottom + brick_size / 2.0 + i as f32 * brick_size;
        commands.spawn((
            Brick,
            PlayingEntity,
            Sprite {
                image: brick_texture.clone(),
                custom_size: Some(Vec2::new(brick_size, brick_size)),
                ..default()
            },
            Transform::from_xyz(commander_right + brick_size / 2.0, y, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(brick_size / 2.0, brick_size / 2.0),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::all(),
        ));
    }

    // 上墙：2块砖封顶，紧贴司令官顶部
    for i in 0..2 {
        let x = -brick_size / 2.0 + i as f32 * brick_size;
        commands.spawn((
            Brick,
            PlayingEntity,
            Sprite {
                image: brick_texture.clone(),
                custom_size: Some(Vec2::new(brick_size, brick_size)),
                ..default()
            },
            Transform::from_xyz(x, commander_top + brick_size / 2.0, 0.0),
            RigidBody::Fixed,
            Collider::cuboid(brick_size / 2.0, brick_size / 2.0),
            ActiveEvents::COLLISION_EVENTS,
            ActiveCollisionTypes::all(),
        ));
    }

    commands.spawn((
        Commander,
        PlayingEntity,
        Sprite {
            image: commander_texture,
            texture_atlas: Some(TextureAtlas {
                layout: commander_texture_atlas_layout,
                index: commander_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(COMMANDER_WIDTH, COMMANDER_HEIGHT)),
            ..default()
        },
        Transform::from_xyz(commander_x, commander_y, 0.0),
        commander_animation_indices,
        AnimationTimer(Timer::from_seconds(0.15, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
        RigidBody::Fixed,
        Collider::cuboid(COMMANDER_WIDTH / 2.0, COMMANDER_HEIGHT / 2.0),
        ActiveEvents::COLLISION_EVENTS,
    ));

    // 创建音乐动画精灵（一直播放）
    let music_texture: Handle<Image> = asset_server.load(TEXTURE_MUSIC_NOTE);
    let music_tile_size = UVec2::new(140, 120);
    let music_texture_atlas = TextureAtlasLayout::from_grid(music_tile_size, 10, 1, None, None);
    let music_texture_atlas_layout = texture_atlas_layouts.add(music_texture_atlas);
    let music_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        CommanderMusicAnimation,
        PlayingEntity,
        Sprite {
            image: music_texture,
            texture_atlas: Some(TextureAtlas {
                layout: music_texture_atlas_layout,
                index: music_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(70.0, 60.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(commander_x, commander_y, 1.0)), // z=1.0 使动画在 Commander 上方
        music_animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)), // 每0.1秒切换一帧
        CurrentAnimationFrame(0),
    ));
}

fn spawn_player1_tank(
    commands: &mut Commands,
    texture: Handle<Image>,
    texture_atlas_layout: Handle<TextureAtlasLayout>,
    animation_indices: AnimationIndices,
) -> Entity {
    let player_tank = PlayerTank { tank_type: TankType::Player1 };

    

    commands.spawn_empty()
        .insert(player_tank)
        .insert(PlayingEntity)
        .insert(TankFireConfig::default())
        .insert(RotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))
        .insert(TargetRotation { angle: 0.0_f32.to_radians() })
        .insert(Sprite {
            image: texture,
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: animation_indices.first,
            }),
            custom_size: Some(Vec2::new(80.0, 90.0)),
            ..default()
        })
        .insert(Transform::from_xyz(-TANK_WIDTH / 2.0 - COMMANDER_WIDTH/2.0 - 50.0, MAP_BOTTOM_Y+TANK_HEIGHT / 2.0, 0.0))
        .insert(Velocity{ linvel: Vec2::default(), angvel: 0.0 })
        .insert(animation_indices)
        .insert(AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)))
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::cuboid(35.0, 35.0))
        .insert(ActiveEvents::COLLISION_EVENTS)
        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC | ActiveCollisionTypes::KINEMATIC_KINEMATIC)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            filter_groups: None,
            autostep: Some(bevy_rapier2d::prelude::CharacterAutostep {
                max_height: CharacterLength::Absolute(5.0),
                min_width: CharacterLength::Absolute(0.5),
                include_dynamic_bodies: false,
            }),
            ..default()
        })
        .id()
}

pub fn spawn_game_entities_if_needed(
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    mut clear_color: ResMut<ClearColor>,
    _enemy_spawn_state: Res<EnemySpawnState>,
    mut player_info: ResMut<PlayerInfo>,
    stage_level: Res<StageLevel>,
    game_mode: Res<GameMode>,
    mut entities_spawned: ResMut<GameEntitiesSpawned>,
) {
    // 如果游戏实体已经生成，则跳过
    if entities_spawned.0 {
        return;
    }

    // 标记游戏实体已生成
    entities_spawned.0 = true;

    // 设置背景色为黑色
    clear_color.0 = BACKGROUND_COLOR;

    // 生成墙壁
    spawn_walls(&mut commands);

    // 根据地图数组生成地形
    spawn_map_terrain(&mut commands, &asset_server, &mut texture_atlas_layouts, stage_level.0);

    // 生成司令官
    spawn_commander(&mut commands, &asset_server, &mut texture_atlas_layouts);

    // 加载玩家坦克纹理和创建精灵图
    let player1_texture = asset_server.load(TEXTURE_PLAYER_TANK1);
    let player2_texture = asset_server.load(TEXTURE_PLAYER_TANK2);
    let player_tile_size = UVec2::new(293, 328);
    let player_texture_atlas = TextureAtlasLayout::from_grid(player_tile_size, 2, 1, None, None);
    let player_texture_atlas_layout = texture_atlas_layouts.add(player_texture_atlas);
    let player_animation_indices = AnimationIndices { first: 0, last: 1 };

    // 根据游戏模式生成玩家

        match *game_mode {

            GameMode::OnePlayer => {

                // 单人模式：只生成玩家1

                let _player1_tank_entity = spawn_player1_tank(

                    &mut commands,

                    player1_texture,

                    player_texture_atlas_layout,

                    player_animation_indices,

                );

    

                                // 初始化玩家1信息

    

                                player_info.players.insert(TankType::Player1, PlayerStats {

    

                                    name: "Li Yun Long".to_string(),

    

                                    speed: 40,

    

                                    fire_speed: 40,

    

                                    protection: 40,

    

                                    shells: 1,

    

                                    penetrate: false,

    

                                    track_chain: false,

    

                                    air_cushion: false,

    

                                    fire_shell: false,

    

                                    life_red_bar: 3,

    

                                    energy_blue_bar: 3,

                    score: 0,

                });

    

                }

            GameMode::TwoPlayers => {

                // 双人模式：生成玩家1和玩家2

                let _player1_tank_entity = spawn_player1_tank(

                    &mut commands,

                    player1_texture,

                    player_texture_atlas_layout.clone(),

                    player_animation_indices,

                );



                let _player2_tank_entity = commands.spawn_empty()

                    .insert(PlayerTank { tank_type: TankType::Player2 })

                    .insert(PlayingEntity)

                    .insert(TankFireConfig::default())

                    .insert(RotationTimer(Timer::from_seconds(0.1, TimerMode::Once)))

                    .insert(TargetRotation { angle: 0.0_f32.to_radians() })

                    .insert(Sprite {
                        image: player2_texture,
                        texture_atlas: Some(TextureAtlas {
                            layout: player_texture_atlas_layout,
                            index: player_animation_indices.first,
                        }),
                        custom_size: Some(Vec2::new(80.0, 90.0)),
                        ..default()
                    })

                    .insert(Transform::from_xyz(TANK_WIDTH / 2.0 + COMMANDER_WIDTH/2.0 + 50.0, MAP_BOTTOM_Y+TANK_HEIGHT / 2.0, 0.0))

                    .insert(Velocity{ linvel: Vec2::default(), angvel: 0.0 })

                    .insert(player_animation_indices)

                    .insert(AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)))

                    .insert(RigidBody::KinematicPositionBased)

                                        .insert(Collider::cuboid(TANK_WIDTH/2.0, TANK_HEIGHT/2.0))

                                        .insert(ActiveEvents::COLLISION_EVENTS)

                                        .insert(ActiveCollisionTypes::default() | ActiveCollisionTypes::KINEMATIC_STATIC | ActiveCollisionTypes::KINEMATIC_KINEMATIC)

                                        .insert(LockedAxes::ROTATION_LOCKED)

                                        .insert(KinematicCharacterController {

                                            offset: CharacterLength::Absolute(0.01),

                                            filter_groups: None,

                                            autostep: Some(bevy_rapier2d::prelude::CharacterAutostep {

                                                max_height: CharacterLength::Absolute(5.0),

                                                min_width: CharacterLength::Absolute(0.5),

                                                include_dynamic_bodies: false,

                                            }),

                                            ..default()

                                        })

                                        .id();

    

                // 初始化玩家1信息

                player_info.players.insert(TankType::Player1, PlayerStats {

                    name: "Li Yun Long".to_string(),

                    speed: 40,

                    fire_speed: 40,

                    protection: 40,

                    shells: 1,

                    penetrate: false,

                    track_chain: false,

                    air_cushion: false,

                    fire_shell: false,

                    life_red_bar: 3,

                    energy_blue_bar: 3,

                    score: 0,

                });

    

                // 初始化玩家2信息

                player_info.players.insert(TankType::Player2, PlayerStats {

                    name: "Chu Yun Fei".to_string(),

                    speed: 40,

                    fire_speed: 40,

                    protection: 40,

                    shells: 1,

                    penetrate: false,

                    track_chain: false,

                    air_cushion: false,

                    fire_shell: false,

                    life_red_bar: 3,

                    energy_blue_bar: 3,

                    score: 0,

                });

            }
    }

    // 加载字体
    let font: Handle<Font> = asset_server.load(FONT_EN);

    // 根据游戏模式生成UI
    match *game_mode {
        GameMode::OnePlayer => {
            // 单人模式：只生成玩家1的UI
            for config in PLAYER1_UI_ELEMENTS {
                spawn_ui_element_from_config(&mut commands, &font, &asset_server, &mut texture_atlas_layouts, config, &player_info, TankType::Player1);
            }
        }
        GameMode::TwoPlayers => {
            // 双人模式：生成玩家1和玩家2的UI
            spawn_player_info(&mut commands, &font, &asset_server, &mut texture_atlas_layouts, &player_info);
        }
    }

    spawn_top_text_info(&mut commands, &font, stage_level.0);

    // 生成道具
    spawn_power_ups(&mut commands, &asset_server, &mut texture_atlas_layouts, &stage_level);
}
fn spawn_player_info(
    commands: &mut Commands,
    font: &Handle<Font>,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    player_info: &PlayerInfo,
) {
    // 生成玩家1 UI 元素
    for config in PLAYER1_UI_ELEMENTS {
        spawn_ui_element_from_config(commands, font, asset_server, texture_atlas_layouts, config, player_info, TankType::Player1);
    }
    // 生成玩家2 UI 元素
    for config in PLAYER2_UI_ELEMENTS {
        spawn_ui_element_from_config(commands, font, asset_server, texture_atlas_layouts, config, player_info, TankType::Player2);
    }
}

fn spawn_top_text_info(
    commands: &mut Commands,
    font: &Handle<Font>,
    stage_level: usize,
) {
    // 其他游戏信息 UI 元素配置
    let commander_text_x = WINDOW_LEFT_X + 435.0; // 往左平移30像素

    // 关卡信息显示在顶部中心
    commands.spawn((
        PlayingEntity,
        Text2d(format!("Stage {stage_level}")),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色
        Transform::from_xyz(0.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));

    commands.spawn((
        PlayingEntity,
        Text2d("Commander Life:".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(commander_text_x - 42.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
    // Commander 血条（与玩家血条长度相同：160像素），放在文字正右方
    commands.spawn((
        PlayingEntity,
        CommanderHealthBar,
        CommanderHealthBarOriginalPosition(commander_text_x + 172.0), // 文字右侧
        Sprite {
            color: Color::srgb(1.0, 0.0, 0.0),
            custom_size: Some(Vec2::new(160.0, 10.0)),
            ..default()
        },
        Transform::from_xyz(commander_text_x + 172.0, WINDOW_TOP_Y - 50.0, 1.0), // 与文字同一Y坐标
    ));
    commands.spawn((
        PlayingEntity,
        EnemyCountText,
        Text2d("Enemy Left: 20/20".to_string()),
        TextFont {
            font_size: 28.0,
            font: font.clone(),
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Transform::from_xyz(WINDOW_RIGHT_X - 465.0, WINDOW_TOP_Y - 50.0, 1.0),
    ));
}

fn spawn_ui_element_from_config(
    commands: &mut Commands,
    font: &Handle<Font>,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    config: &UIElementConfig,
    player_info: &PlayerInfo,
    tank_type: TankType,
) {
    let player_stats = &player_info.players[&tank_type];
    match config.element_type {
        UIElementType::NormalText(f) => {
            let text = f(player_stats);
            // 检查属性是否达到最大值或On状态，如果是则设置红色
            let text_color = if is_stat_at_max_value(&text, player_stats) {
                Color::srgb(1.0, 0.0, 0.0) // 红色
            } else {
                Color::srgb(1.0, 1.0, 1.0) // 白色
            };

            commands.spawn((
                PlayerUI { player_type: tank_type },
                PlayingEntity,
                Text2d(text),
                TextFont {
                    font_size: config.font_size,
                    font: font.clone(),
                    ..default()
                },
                TextColor(text_color),
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
        UIElementType::PlayerAvatar => {
            let player_avatar_texture: Handle<Image> = asset_server.load(TEXTURE_AVATAR);
            let player_avatar_tile_size = UVec2::new(160, 147);
            let player_avatar_texture_atlas = TextureAtlasLayout::from_grid(player_avatar_tile_size, 13, 3, None, None);
            let player_avatar_texture_atlas_layout = texture_atlas_layouts.add(player_avatar_texture_atlas);
            let player_avatar_animation_indices = AnimationIndices { first: 0, last: 32 };
            commands.spawn((
                PlayerUI { player_type: tank_type },
                PlayerAvatar,
                PlayingEntity,
                Sprite {
                    image: player_avatar_texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: player_avatar_texture_atlas_layout,
                        index: 0,
                    }),
                    custom_size: Some(Vec2::new(160.0, 147.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
                player_avatar_animation_indices,
                AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                CurrentAnimationFrame(0),
            ));
        }
        UIElementType::HealthBar => {
            commands.spawn((
                PlayerUI { player_type: tank_type },
                HealthBar,
                HealthBarOriginalPosition(config.x_pos),
                PlayingEntity,
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    custom_size: Some(Vec2::new(160.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
        UIElementType::BlueBar => {
            commands.spawn((
                PlayerUI { player_type: tank_type },
                BlueBar,
                BlueBarOriginalPosition(config.x_pos),
                PlayingEntity,
                Sprite {
                    color: Color::srgb(0.0, 0.5, 1.0),
                    custom_size: Some(Vec2::new(160.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(config.x_pos, config.y_pos, 1.0),
            ));
        }
    }
}

fn spawn_power_ups(commands: &mut Commands, asset_server: &AssetServer, texture_atlas_layouts: &mut Assets<TextureAtlasLayout>, stage_level: &StageLevel) {
    let powerup_type = if stage_level.0 == 1 {
        // 第一关强制生成 air_cushion 道具
        PowerUp::AirCushion
    } else {
        // 其他关卡随机选择一个道具类型
        let powerup_types = [
            PowerUp::SpeedUp,
            PowerUp::Protection,
            PowerUp::FireSpeed,
            PowerUp::FireShell,
            PowerUp::TrackChain,
            PowerUp::Penetrate,
            PowerUp::Repair,
            PowerUp::Hamburger,
            PowerUp::AirCushion,
            PowerUp::Shell,
        ];

        let mut rng = rand::rng();
        powerup_types[rng.random_range(0..powerup_types.len())]
    };

    // 定义禁止区域
    // 上方：坦克高度区域（MAP_TOP_Y - TANK_HEIGHT 到 MAP_TOP_Y）
    // 下方：commander高度区域（MAP_BOTTOM_Y 到 MAP_BOTTOM_Y + COMMANDER_HEIGHT）
    let top_forbidden_y = MAP_TOP_Y - TANK_HEIGHT;
    let bottom_forbidden_y = MAP_BOTTOM_Y + COMMANDER_HEIGHT;

    // 在随机位置生成道具（在地图范围内），避开禁止区域
    let mut rng = rand::rng();
    let x = rng.random_range(MAP_LEFT_X + 100.0..MAP_RIGHT_X - 100.0);
    let y = rng.random_range(bottom_forbidden_y + 100.0..top_forbidden_y - 100.0);
    let position = Vec3::new(x, y, 0.0);

    spawn_powerup_batch(commands, asset_server, texture_atlas_layouts, powerup_type, powerup_type.texture_path(), &[position]);
}

fn spawn_powerup_batch(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    powerup_type: PowerUp,
    texture_path: &'static str,
    positions: &[Vec3],
) {
    let texture: Handle<Image> = asset_server.load(texture_path);
    let tile_size = UVec2::new(87, 69);
    let texture_atlas = TextureAtlasLayout::from_grid(tile_size, 3, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(texture_atlas);
    let animation_indices = AnimationIndices { first: 0, last: 2 };

    for pos in positions {
        commands.spawn((
            powerup_type,
            PlayingEntity,
            Sprite::from_atlas_image(
                texture.clone(),
                TextureAtlas {
                    layout: texture_atlas_layout.clone(),
                    index: animation_indices.first,
                }
            ),
            Transform::from_xyz(pos.x, pos.y, 0.8), // z=0.8 使道具高于除了树之外的所有图层
            animation_indices,
            AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            CurrentAnimationFrame(0),
            RigidBody::Fixed,
            Collider::cuboid(87.0 / 2.0, 69.0 / 2.0),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
        ));
    }
}

