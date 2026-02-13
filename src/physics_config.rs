//! 物理引擎配置常量
//!
//! 统一管理 Avian2D 物理引擎的所有配置参数

use bevy::prelude::*;
use avian2d::prelude::*;

// ============================================================================
// 刚体配置常量
// ============================================================================

/// 刚体体类型常量
pub mod rigid_body_types {
    pub const STATIC: avian2d::prelude::RigidBody = avian2d::prelude::RigidBody::Static;
    pub const DYNAMIC: avian2d::prelude::RigidBody = avian2d::prelude::RigidBody::Dynamic;
    pub const KINEMATIC: avian2d::prelude::RigidBody = avian2d::prelude::RigidBody::Kinematic;
}

/// 锁定轴配置常量
pub mod locked_axes {
    pub const ROTATION_LOCKED: avian2d::prelude::LockedAxes = avian2d::prelude::LockedAxes::ROTATION_LOCKED;
}

// ============================================================================
// 碰撞体尺寸常量
// ============================================================================

/// 碰撞体尺寸常量（半尺寸）
pub mod collider_sizes {
    /// 玩家坦克碰撞体半尺寸
    pub const PLAYER: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(35.0, 35.0);
    /// 敌方坦克碰撞体半尺寸
    pub const ENEMY: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(38.0, 43.0);
    /// 子弹碰撞体半尺寸
    pub const BULLET: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(30.0, 20.0);
    /// 墙壁碰撞体半尺寸（砖块/钢铁）- 50x50像素的半尺寸
    pub const WALL: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(25.0, 25.0);
    /// 司令官碰撞体半尺寸
    pub const COMMANDER: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(50.0, 50.0);
    /// 森林碰撞体半尺寸（碰撞大小 120x120）
    pub const FOREST: bevy::prelude::Vec2 = bevy::prelude::Vec2::splat(60.0);
    /// 屏障碰撞体半尺寸
    pub const BARRIER: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(50.0, 50.0);
    /// 检测半径（用于海、指挥官、森林的检测）
    pub const DETECTION_RADIUS: bevy::prelude::Vec2 = bevy::prelude::Vec2::new(50.0, 50.0);
}

/// 材质属性常量
pub mod material {
    pub const FRICTION_NONE: f32 = 0.0;
    pub const RESTITUTION_NONE: f32 = 0.0;
}

// ============================================================================
// 预定义的物理实体配置
// ============================================================================

/// 玩家坦克物理配置
pub fn player_tank_physics() -> PlayerTankPhysics {
    PlayerTankPhysics {
        rigid_body: rigid_body_types::DYNAMIC,
        collider_half_size: collider_sizes::PLAYER,
        locked_axes: locked_axes::ROTATION_LOCKED,
        friction: material::FRICTION_NONE,
        restitution: material::RESTITUTION_NONE,
        collision_layers: CollisionLayers::new(
            LayerMask::from(0b01u32), // 属于 layer0
            LayerMask::from(0b11u32), // 与 layer0 和 layer1 都碰撞
        ),
    }
}

/// 敌方坦克物理配置
pub fn enemy_tank_physics() -> EnemyTankPhysics {
    EnemyTankPhysics {
        rigid_body: rigid_body_types::DYNAMIC,
        collider_half_size: collider_sizes::ENEMY,
        locked_axes: locked_axes::ROTATION_LOCKED,
        friction: material::FRICTION_NONE,
        restitution: material::RESTITUTION_NONE,
        collision_layers: CollisionLayers::new(
            LayerMask::from(0b01u32), // 属于 layer0
            LayerMask::from(0b11u32), // 与 layer0 和 layer1 都碰撞
        ),
    }
}

/// 子弹物理配置
pub const BULLET_PHYSICS: BulletPhysics = BulletPhysics {
    rigid_body: rigid_body_types::KINEMATIC,
    collider_half_size: collider_sizes::BULLET,
    locked_axes: locked_axes::ROTATION_LOCKED,
    is_sensor: true,
};

/// 墙壁物理配置（砖块/钢铁）
pub const WALL_PHYSICS: WallPhysics = WallPhysics {
    rigid_body: rigid_body_types::STATIC,
    collider_half_size: collider_sizes::WALL,
};

/// 森林物理配置
pub const FOREST_PHYSICS: ForestPhysics = ForestPhysics {
    rigid_body: rigid_body_types::STATIC,
    collider_half_size: collider_sizes::FOREST,
    is_sensor: true,
};

/// 海洋物理配置
pub fn sea_physics() -> SeaPhysics {
    SeaPhysics {
        rigid_body: rigid_body_types::STATIC,
        collider_half_size: collider_sizes::DETECTION_RADIUS,
        collision_layers: CollisionLayers::new(
            LayerMask::from(0b10u32), // 属于 layer1
            LayerMask::from(0b11u32), // 与 layer0 和 layer1 都碰撞
        ),
    }
}

/// 屏障物理配置
pub const BARRIER_PHYSICS: BarrierPhysics = BarrierPhysics {
    rigid_body: rigid_body_types::STATIC,
    collider_half_size: collider_sizes::BARRIER,
};

/// 司令官物理配置
pub const COMMANDER_PHYSICS: CommanderPhysics = CommanderPhysics {
    rigid_body: rigid_body_types::STATIC,
    collider_half_size: collider_sizes::COMMANDER,
};

// ============================================================================
// 物理配置结构体
// ============================================================================

/// 玩家坦克物理配置
#[derive(Clone, Copy)]
pub struct PlayerTankPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
    pub locked_axes: avian2d::prelude::LockedAxes,
    pub friction: f32,
    pub restitution: f32,
    pub collision_layers: CollisionLayers,
}

impl PlayerTankPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ))
            .insert(self.locked_axes)
            .insert(CollisionEventsEnabled)
            .insert(Friction::new(self.friction))
            .insert(Restitution::new(self.restitution))
            .insert(self.collision_layers);
    }
}

/// 敌方坦克物理配置
#[derive(Clone, Copy)]
pub struct EnemyTankPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
    pub locked_axes: avian2d::prelude::LockedAxes,
    pub friction: f32,
    pub restitution: f32,
    pub collision_layers: CollisionLayers,
}

impl EnemyTankPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ))
            .insert(self.locked_axes)
            .insert(CollisionEventsEnabled)
            .insert(Friction::new(self.friction))
            .insert(Restitution::new(self.restitution))
            .insert(self.collision_layers);
    }
}

/// 子弹物理配置
#[derive(Clone, Copy)]
pub struct BulletPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
    pub locked_axes: avian2d::prelude::LockedAxes,
    pub is_sensor: bool,
}

impl BulletPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ))
            .insert(self.locked_axes)
            .insert(CollisionEventsEnabled);
        
        if self.is_sensor {
            entity.insert(Sensor);
        }
    }
}

/// 墙壁物理配置
#[derive(Clone, Copy)]
pub struct WallPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
}

impl WallPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ))
            .insert(CollisionEventsEnabled);
    }
}

/// 森林物理配置
#[derive(Clone, Copy)]
pub struct ForestPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
    pub is_sensor: bool,
}

impl ForestPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ));
        
        if self.is_sensor {
            entity.insert(Sensor);
        }
    }
}

/// 海洋物理配置
#[derive(Clone, Copy)]
pub struct SeaPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
    pub collision_layers: CollisionLayers,
}

impl SeaPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ))
            .insert(self.collision_layers);
    }
}

/// 屏障物理配置
#[derive(Clone, Copy)]
pub struct BarrierPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
}

impl BarrierPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ))
            .insert(Sensor)
            .insert(CollisionEventsEnabled);
    }
}

/// 司令官物理配置
#[derive(Clone, Copy)]
pub struct CommanderPhysics {
    pub rigid_body: avian2d::prelude::RigidBody,
    pub collider_half_size: bevy::prelude::Vec2,
}

impl CommanderPhysics {
    /// 应用到实体
    pub fn apply_to_entity(&self, entity: &mut EntityCommands) {
        entity.insert(self.rigid_body)
            .insert(Collider::rectangle(
                self.collider_half_size.x * 2.0,
                self.collider_half_size.y * 2.0,
            ))
            .insert(CollisionEventsEnabled);
    }
}