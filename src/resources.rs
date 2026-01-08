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

#[derive(Resource, Default)]
pub struct MenuBlinkTimer(pub Timer);
