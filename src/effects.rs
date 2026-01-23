//! 特效动画模块
//!
//! 处理爆炸、烟雾、火花、激光、森林火焰等特效动画

#![allow(clippy::wildcard_imports)]

use bevy::audio::Volume;
use bevy::prelude::*;

use crate::bullet::BulletOwner;
use crate::constants::*;

pub fn spawn_explosion(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载爆炸精灵图（8x8，共64帧，每帧512x512）
    let explosion_texture: Handle<Image> = asset_server.load(TEXTURE_EXPLOSION);
    let explosion_tile_size = UVec2::new(EXPLOSION_TILE_SIZE as u32, EXPLOSION_TILE_SIZE as u32);
    let explosion_texture_atlas =
        TextureAtlasLayout::from_grid(explosion_tile_size, 8, 8, None, None);
    let explosion_texture_atlas_layout = texture_atlas_layouts.add(explosion_texture_atlas);
    let explosion_animation_indices = AnimationIndices { first: 0, last: 63 };

    commands.spawn((
        Explosion,
        PlayingEntity,
        Sprite {
            image: explosion_texture,
            texture_atlas: Some(TextureAtlas {
                layout: explosion_texture_atlas_layout,
                index: explosion_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(300.0, 300.0)),
            ..default()
        },
        Transform::from_translation(position),
        explosion_animation_indices,
        AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_EXPLOSION,
            TimerMode::Repeating,
        )),
        CurrentAnimationFrame(0),
    ));

    // 播放爆炸音效
    let explosion_sound: Handle<AudioSource> = asset_server.load(SOUND_EXPLOSION);
    commands.spawn((
        AudioPlayer::new(explosion_sound),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(VOLUME_HALF)),
    ));
}

pub fn spawn_forest_fire(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载树林燃烧精灵图（10帧，每帧131x131，1.5秒播完）
    let forest_fire_texture: Handle<Image> = asset_server.load("maps/tree_fire_sheet.png");
    let forest_fire_tile_size = UVec2::new(131, 131);
    let forest_fire_texture_atlas =
        TextureAtlasLayout::from_grid(forest_fire_tile_size, 10, 1, None, None);
    let forest_fire_texture_atlas_layout = texture_atlas_layouts.add(forest_fire_texture_atlas);
    let forest_fire_animation_indices = AnimationIndices { first: 0, last: 9 };

    commands.spawn((
        ForestFire,
        PlayingEntity,
        Sprite::from_atlas_image(
            forest_fire_texture,
            TextureAtlas {
                layout: forest_fire_texture_atlas_layout,
                index: forest_fire_animation_indices.first,
            },
        ),
        Transform::from_translation(position),
        forest_fire_animation_indices,
        AnimationTimer(Timer::from_seconds(
            FOREST_FIRE_DURATION / 10.0,
            TimerMode::Repeating,
        )), // 1.5秒播完10帧
        CurrentAnimationFrame(0),
    ));

    // 播放树林燃烧音效
    let burn_tree_sound: Handle<AudioSource> = asset_server.load(SOUND_BURN_TREE);
    commands.spawn(AudioPlayer::new(burn_tree_sound));
}

pub fn spawn_spark(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载打击效果图片（4x4，共16帧，每帧1024x1024）
    let spark_texture: Handle<Image> = asset_server.load(TEXTURE_STEEL_HIT);
    let spark_tile_size = UVec2::new(SPARK_TILE_SIZE as u32, SPARK_TILE_SIZE as u32);
    let spark_texture_atlas = TextureAtlasLayout::from_grid(spark_tile_size, 4, 4, None, None);
    let spark_texture_atlas_layout = texture_atlas_layouts.add(spark_texture_atlas);
    let spark_animation_indices = AnimationIndices { first: 0, last: 15 };

    commands.spawn((
        Spark,
        PlayingEntity,
        Sprite {
            image: spark_texture,
            texture_atlas: Some(TextureAtlas {
                layout: spark_texture_atlas_layout,
                index: spark_animation_indices.first,
            }),
            custom_size: Some(Vec2::new(200.0, 200.0)),
            ..default()
        },
        Transform::from_translation(position),
        spark_animation_indices,
        AnimationTimer(Timer::from_seconds(
            ANIMATION_FRAME_SPARK,
            TimerMode::Repeating,
        )),
        CurrentAnimationFrame(0),
    ));
}

pub fn animate_explosion(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Explosion>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁爆炸实体
                let () = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

pub fn animate_laser(
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Laser>,
    >,
    despawn_entities: Query<(Entity, &Transform), With<DespawnMarker>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁激光实体和所有标记的实体
                for (despawn_entity, transform) in despawn_entities.iter() {
                    // 在被销毁实体的位置播放烟雾效果
                    let smoke_texture: Handle<Image> = asset_server.load(TEXTURE_SMOKE);
                    let smoke_tile_size = UVec2::new(100, 100);
                    let smoke_texture_atlas =
                        TextureAtlasLayout::from_grid(smoke_tile_size, 5, 3, None, None);
                    let smoke_texture_atlas_layout = texture_atlas_layouts.add(smoke_texture_atlas);
                    let smoke_animation_indices = AnimationIndices { first: 0, last: 14 };

                    commands.spawn((
                        PlayingEntity,
                        Smoke,
                        Sprite {
                            image: smoke_texture,
                            texture_atlas: Some(TextureAtlas {
                                layout: smoke_texture_atlas_layout,
                                index: smoke_animation_indices.first,
                            }),
                            custom_size: Some(Vec2::new(SMOKE_SIZE, SMOKE_SIZE)),
                            ..default()
                        },
                        Transform::from_xyz(
                            transform.translation.x,
                            transform.translation.y,
                            Z_DEFAULT,
                        ),
                        smoke_animation_indices,
                        AnimationTimer(Timer::from_seconds(
                            ANIMATION_FRAME_SMOKE,
                            TimerMode::Repeating,
                        )),
                        CurrentAnimationFrame(0),
                    ));

                    let () = commands.entity(despawn_entity).try_despawn();
                }
                let () = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

/// 处理烟雾动画
pub fn animate_smoke(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Smoke>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁烟雾实体
                let () = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

/// 处理后坐力效果
pub fn handle_recoil_force(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut RecoilForce)>,
) {
    for (entity, mut transform, mut recoil) in &mut query {
        recoil.timer.tick(time.delta());

        // 使用平滑插值应用后坐力位移
        let progress = recoil.timer.elapsed_secs() / recoil.timer.duration().as_secs_f32();
        let current_offset = recoil.target_offset * (1.0 - progress);

        // 从原始位置插值到当前位置
        transform.translation.x = recoil.original_pos.x + current_offset.x;
        transform.translation.y = recoil.original_pos.y + current_offset.y;

        // 后坐力时间结束，移除组件
        if recoil.timer.just_finished() {
            commands.entity(entity).remove::<RecoilForce>();
        }
    }
}

/// 激光碰撞检测系统（只收集实体，不立即销毁）
pub fn animate_forest_fire(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<ForestFire>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁森林燃烧实体
                let () = commands.entity(entity).try_despawn();
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                let next_index = current + 1;
                current_frame.0 = next_index;
                atlas.index = next_index;
            }
        }
    }
}

pub fn animate_forest(
    time: Res<Time>,
    mut query: Query<
        (
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Forest>,
    >,
) {
    for (mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());

        if timer.just_finished() {
            let current = current_frame.0;
            let next_index = if current == indices.last {
                indices.first
            } else {
                current + 1
            };
            current_frame.0 = next_index;
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = next_index;
            }
        }
    }
}

pub fn animate_spark(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut AnimationTimer,
            &mut Sprite,
            &AnimationIndices,
            &mut CurrentAnimationFrame,
        ),
        With<Spark>,
    >,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁实体
                let () = commands.entity(entity).try_despawn();
            } else {
                // 继续播放动画
                let next_index = current + 1;
                current_frame.0 = next_index;
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = next_index;
                }
            }
        }
    }
}

/// 激光碰撞检测系统（只收集实体，不立即销毁）
pub fn laser_collision_system(
    mut commands: Commands,
    mut frame_count: Local<u32>,
    lasers: Query<
        (
            Entity,
            &Transform,
            &CurrentAnimationFrame,
            &AnimationIndices,
        ),
        With<Laser>,
    >,
    enemies: Query<(Entity, &Transform), With<EnemyTank>>,
    bullets: Query<(Entity, &Transform), With<BulletOwner>>,
    bricks: Query<(Entity, &Transform), With<Brick>>,
    steels: Query<(Entity, &Transform), With<Steel>>,
    forests: Query<(Entity, &Transform), With<Forest>>,
    barriers: Query<(Entity, &Transform), With<Barrier>>,
    seas: Query<(Entity, &Transform), With<Sea>>,
) {
    // 每5帧执行一次碰撞检测
    *frame_count += 1;
    if !(*frame_count).is_multiple_of(LASER_COLLISION_FRAME_INTERVAL) {
        return;
    }

    for (_laser_entity, laser_transform, _, _) in &lasers {
        let laser_bounds = calculate_laser_bounds(laser_transform);

        check_and_mark_collisions(&mut commands, &enemies, laser_bounds, TANK_WIDTH, TANK_HEIGHT);
        check_and_mark_collisions(&mut commands, &bullets, laser_bounds, BULLET_SIZE, BULLET_SIZE);
        check_and_mark_collisions(&mut commands, &bricks, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &steels, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &forests, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &barriers, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
        check_and_mark_collisions(&mut commands, &seas, laser_bounds, BRICK_TEXTURE_WIDTH, BRICK_TEXTURE_HEIGHT);
    }
}

/// 激光边界框
#[derive(Clone, Copy)]
struct LaserBounds {
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
}

/// 计算激光旋转后的边界框
fn calculate_laser_bounds(transform: &Transform) -> LaserBounds {
    let laser_half_width = LASER_COLLIDER_HALF_WIDTH;
    let laser_half_height = LASER_COLLIDER_HALF_HEIGHT;
    let rotation = transform.rotation;

    let corners = [
        Vec2::new(-laser_half_width, -laser_half_height),
        Vec2::new(laser_half_width, -laser_half_height),
        Vec2::new(laser_half_width, laser_half_height),
        Vec2::new(-laser_half_width, laser_half_height),
    ];

    let rotated_corners: Vec<Vec2> = corners
        .iter()
        .map(|corner| {
            let rotated = rotation.mul_vec3(corner.extend(0.0));
            Vec2::new(rotated.x, rotated.y) + Vec2::new(transform.translation.x, transform.translation.y)
        })
        .collect();

    LaserBounds {
        left: rotated_corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min),
        right: rotated_corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max),
        bottom: rotated_corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min),
        top: rotated_corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max),
    }
}

/// 检查并标记碰撞实体
fn check_and_mark_collisions<T: Component>(
    commands: &mut Commands,
    entities: &Query<(Entity, &Transform), With<T>>,
    laser_bounds: LaserBounds,
    entity_width: f32,
    entity_height: f32,
) {
    let half_width = entity_width / 2.0;
    let half_height = entity_height / 2.0;

    for (entity, transform) in entities.iter() {
        let entity_left = transform.translation.x - half_width;
        let entity_right = transform.translation.x + half_width;
        let entity_bottom = transform.translation.y - half_height;
        let entity_top = transform.translation.y + half_height;

        if laser_bounds.left < entity_right
            && laser_bounds.right > entity_left
            && laser_bounds.bottom < entity_top
            && laser_bounds.top > entity_bottom
        {
            let _ = commands.entity(entity).try_insert(DespawnMarker);
        }
    }
}
