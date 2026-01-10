//! Game resources for the Tank Battle game

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Default)]
pub struct CanFire(pub HashSet<Entity>);

#[derive(Resource, Default)]
pub struct BulletOwners {
    pub owners: HashMap<Entity, Entity>, // 子弹实体 -> 坦克实体
}

#[derive(Resource, Default)]
pub struct StartAnimationFrames {
    pub frames: Vec<Handle<Image>>,
}

#[derive(Resource)]
pub struct FadingOut {
    pub alpha: f32,
}

impl Default for FadingOut {
    fn default() -> Self {
        Self { alpha: 1.0 }
    }
}

#[derive(Resource, Default)]
pub struct CurrentMenuSelection {
    pub selected_index: usize, // 0 = PLAY, 1 = EXIT
}

#[derive(Resource, Default)]
pub struct GameStarted(pub bool);

#[derive(Resource)]
pub struct EnemyCount {
    pub total_spawned: usize, // 已生成的敌方坦克总数
    pub max_count: usize,     // 最大敌方坦克数量
}

impl Default for EnemyCount {
    fn default() -> Self {
        Self {
            total_spawned: 0,
            max_count: 20,
        }
    }
}

#[derive(Resource, Default)]
pub struct MenuBlinkTimer(pub Timer);

#[derive(Resource, Default)]
pub struct StageIntroTimer {
    pub fade_in: Timer,
    pub stay: Timer,
    pub fade_out: Timer,
}
