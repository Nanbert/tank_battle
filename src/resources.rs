//! Game resources for the Tank Battle game

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Resource, Deref, DerefMut)]
pub struct Score(pub usize);

#[derive(Resource, Deref, DerefMut)]
pub struct Life(pub usize);

#[derive(Resource, Default)]
pub struct PlayerInfoData {
    pub speed: usize,
    pub fire_speed: usize,
    pub protection: usize,
    pub shells: usize,
    pub penetrate: bool,
    pub track_chain: bool,
    pub air_cushion: bool,
    pub fire_shell: bool,
}

#[derive(Resource, Default)]
pub struct ColliderEventSet{
    pub entities: HashSet<Entity>,
}

#[derive(Resource, Default)]
pub struct PlayerSpeed(pub usize);

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

#[derive(Resource, Default)]
pub struct MenuBlinkTimer(pub Timer);

#[derive(Resource, Default)]
pub struct ShouldCleanup(pub bool);

#[derive(Resource, Deref, DerefMut)]
pub struct PlayerRespawnTimer(pub Timer);