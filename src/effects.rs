//! 特效动画模块
//!
//! 处理爆炸、烟雾、火花、激光、森林火焰等特效动画

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy::audio::Volume;

use crate::constants::*;
use crate::resources::*;
use crate::bullet::BulletOwner;

pub fn spawn_explosion(
    commands: &mut Commands,
    asset_server: &AssetServer,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    position: Vec3,
) {
    // 加载爆炸精灵图（8x8，共64帧，每帧512x512）
    let explosion_texture: Handle<Image> = asset_server.load(TEXTURE_EXPLOSION);
    let explosion_tile_size = UVec2::new(512, 512);
    let explosion_texture_atlas = TextureAtlasLayout::from_grid(explosion_tile_size, 8, 8, None, None);
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
        AnimationTimer(Timer::from_seconds(0.01, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));

    // 播放爆炸音效
    let explosion_sound: Handle<AudioSource> = asset_server.load(SOUND_EXPLOSION);
    commands.spawn((
        AudioPlayer::new(explosion_sound),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(0.5)),
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
    let forest_fire_texture_atlas = TextureAtlasLayout::from_grid(forest_fire_tile_size, 10, 1, None, None);
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
            }
        ),
        Transform::from_translation(position),
        forest_fire_animation_indices,
        AnimationTimer(Timer::from_seconds(1.5 / 10.0, TimerMode::Repeating)), // 1.5秒播完10帧
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
    let spark_tile_size = UVec2::new(1024, 1024);
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
        AnimationTimer(Timer::from_seconds(0.02, TimerMode::Repeating)),
        CurrentAnimationFrame(0),
    ));
}

pub fn animate_explosion(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁爆炸实体
                let _ = commands.entity(entity).try_despawn();
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
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Laser>>,
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
                    let smoke_texture_atlas = TextureAtlasLayout::from_grid(smoke_tile_size, 5, 3, None, None);
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
                            custom_size: Some(Vec2::new(100.0, 100.0)),
                            ..default()
                        },
                        Transform::from_xyz(transform.translation.x, transform.translation.y, 1.0),
                        smoke_animation_indices,
                        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
                        CurrentAnimationFrame(0),
                    ));
                    
                    let _ = commands.entity(despawn_entity).try_despawn();
                }
                let _ = commands.entity(entity).try_despawn();
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
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Smoke>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁烟雾实体
                let _ = commands.entity(entity).try_despawn();
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
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<ForestFire>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁森林燃烧实体
                let _ = commands.entity(entity).try_despawn();
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
    mut query: Query<(&mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Forest>>,
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
    mut query: Query<(Entity, &mut AnimationTimer, &mut Sprite, &AnimationIndices, &mut CurrentAnimationFrame), With<Spark>>,
) {
    for (entity, mut timer, mut sprite, indices, mut current_frame) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let current = current_frame.0;
            if current >= indices.last {
                // 动画播放完毕，销毁实体
                let _ = commands.entity(entity).try_despawn();
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
    lasers: Query<(Entity, &Transform, &CurrentAnimationFrame, &AnimationIndices), With<Laser>>,
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
    if *frame_count % 5 != 0 {
        return;
    }
    
    for (_laser_entity, laser_transform, _, _) in &lasers {
        // 激光原始尺寸（未旋转）
        let laser_half_width = 35.0; // 70 / 2
        let laser_half_height = 683.0; // 1366 / 2 (1倍)
        
        // 获取激光的旋转角度
        let rotation = laser_transform.rotation;
        
        // 激光的四个角点（未旋转）
        let corners = [
            Vec2::new(-laser_half_width, -laser_half_height),
            Vec2::new(laser_half_width, -laser_half_height),
            Vec2::new(laser_half_width, laser_half_height),
            Vec2::new(-laser_half_width, laser_half_height),
        ];
        
        // 旋转每个角点并加上位置
        let rotated_corners: Vec<Vec2> = corners.iter()
            .map(|corner| {
                let rotated = rotation.mul_vec3(corner.extend(0.0));
                Vec2::new(rotated.x, rotated.y) + Vec2::new(laser_transform.translation.x, laser_transform.translation.y)
            })
            .collect();
        
        // 计算旋转后的边界框
        let laser_left = rotated_corners.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let laser_right = rotated_corners.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let laser_bottom = rotated_corners.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let laser_top = rotated_corners.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

        // 检测与敌方坦克的碰撞
        for (enemy_entity, enemy_transform) in &enemies {
            let enemy_left = enemy_transform.translation.x - TANK_WIDTH / 2.0;
            let enemy_right = enemy_transform.translation.x + TANK_WIDTH / 2.0;
            let enemy_top = enemy_transform.translation.y + TANK_HEIGHT / 2.0;
            let enemy_bottom = enemy_transform.translation.y - TANK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < enemy_right && laser_right > enemy_left &&
               laser_bottom < enemy_top && laser_top > enemy_bottom {
                // 标记敌方坦克为待销毁
                let _ = commands.entity(enemy_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与子弹的碰撞
        for (bullet_entity, bullet_transform) in &bullets {
            let bullet_left = bullet_transform.translation.x - BULLET_SIZE / 2.0;
            let bullet_right = bullet_transform.translation.x + BULLET_SIZE / 2.0;
            let bullet_top = bullet_transform.translation.y + BULLET_SIZE / 2.0;
            let bullet_bottom = bullet_transform.translation.y - BULLET_SIZE / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < bullet_right && laser_right > bullet_left &&
               laser_bottom < bullet_top && laser_top > bullet_bottom {
                // 标记子弹为待销毁
                let _ = commands.entity(bullet_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与砖块的碰撞
        for (brick_entity, brick_transform) in &bricks {
            let brick_left = brick_transform.translation.x - BRICK_WIDTH / 2.0;
            let brick_right = brick_transform.translation.x + BRICK_WIDTH / 2.0;
            let brick_top = brick_transform.translation.y + BRICK_HEIGHT / 2.0;
            let brick_bottom = brick_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < brick_right && laser_right > brick_left &&
               laser_bottom < brick_top && laser_top > brick_bottom {
                // 标记砖块为待销毁
                let _ = commands.entity(brick_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与钢块的碰撞
        for (steel_entity, steel_transform) in &steels {
            let steel_left = steel_transform.translation.x - BRICK_WIDTH / 2.0;
            let steel_right = steel_transform.translation.x + BRICK_WIDTH / 2.0;
            let steel_top = steel_transform.translation.y + BRICK_HEIGHT / 2.0;
            let steel_bottom = steel_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < steel_right && laser_right > steel_left &&
               laser_bottom < steel_top && laser_top > steel_bottom {
                // 标记钢块为待销毁
                let _ = commands.entity(steel_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与森林的碰撞
        for (forest_entity, forest_transform) in &forests {
            let forest_left = forest_transform.translation.x - BRICK_WIDTH / 2.0;
            let forest_right = forest_transform.translation.x + BRICK_WIDTH / 2.0;
            let forest_top = forest_transform.translation.y + BRICK_HEIGHT / 2.0;
            let forest_bottom = forest_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < forest_right && laser_right > forest_left &&
               laser_bottom < forest_top && laser_top > forest_bottom {
                // 标记森林为待销毁
                let _ = commands.entity(forest_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与障碍的碰撞
        for (barrier_entity, barrier_transform) in &barriers {
            let barrier_left = barrier_transform.translation.x - BRICK_WIDTH / 2.0;
            let barrier_right = barrier_transform.translation.x + BRICK_WIDTH / 2.0;
            let barrier_top = barrier_transform.translation.y + BRICK_HEIGHT / 2.0;
            let barrier_bottom = barrier_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < barrier_right && laser_right > barrier_left &&
               laser_bottom < barrier_top && laser_top > barrier_bottom {
                // 标记障碍为待销毁
                let _ = commands.entity(barrier_entity).try_insert(DespawnMarker);
            }
        }

        // 检测与sea的碰撞
        for (sea_entity, sea_transform) in &seas {
            let sea_left = sea_transform.translation.x - BRICK_WIDTH / 2.0;
            let sea_right = sea_transform.translation.x + BRICK_WIDTH / 2.0;
            let sea_top = sea_transform.translation.y + BRICK_HEIGHT / 2.0;
            let sea_bottom = sea_transform.translation.y - BRICK_HEIGHT / 2.0;

            // 简单的AABB碰撞检测
            if laser_left < sea_right && laser_right > sea_left &&
               laser_bottom < sea_top && laser_top > sea_bottom {
                // 标记sea为待销毁
                let _ = commands.entity(sea_entity).try_insert(DespawnMarker);
            }
        }
    }
}
