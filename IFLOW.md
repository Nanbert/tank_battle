# Tank Battle 项目

## 项目概述

这是一个使用 Rust 和 Bevy 游戏引擎开发的经典坦克大战（Battle City 1990）简化版实现。该项目是一个 2D 游戏原型，包含玩家坦克和敌方坦克的基本移动和碰撞检测功能。

### 主要技术栈

- **编程语言**: Rust (Edition 2024)
- **游戏引擎**: Bevy 0.17.3
- **物理引擎**: bevy_rapier2d 0.32
- **Rust 工具链**: Nightly channel
- **链接器**: Linux 使用 clang + mold，Windows 使用 rust-lld

### 核心依赖

- `bevy`: 基础游戏引擎，启用了 `dynamic_linking` 特性以加快编译速度
- `log`: 日志库，开发环境启用 debug 级别，发布环境启用 warn 级别
- `rand`: 随机数生成器
- `bevy_rapier2d`: 2D 物理模拟

### 架构特点

- 使用 ECS (Entity Component System) 架构
- 基于 Rapier 物理引擎实现碰撞检测和坦克移动
- 精灵图动画系统（Texture Atlas）
- 敌方坦克 AI：随机移动方向，碰撞时自动转向

## 构建和运行

### 前置要求

- Rust Nightly 工具链
- Linux: clang, mold 链接器
- Windows: rust-lld（Rust 自带）

### 常用命令

```bash
# 构建项目
cargo build

# 运行项目
cargo run

# 发布版本构建
cargo build --release

# 运行发布版本
cargo run --release

# 清理构建产物
cargo clean
```

### 编译优化配置

项目使用自定义的编译配置以平衡开发体验和性能：

- `[profile.dev]`: 主代码 opt-level = 1
- `[profile.dev.package."*"]`: 依赖项 opt-level = 3
- 启用动态链接以加快编译速度

## 项目结构

```
tank_battle/
├── assets/texture/        # 纹理资源
│   ├── sprite_sheet.png   # 精灵图
│   └── tank_player.png    # 玩家坦克纹理
├── src/
│   └── main.rs            # 主程序入口（当前所有代码）
├── .cargo/
│   └── config.toml        # Cargo 配置（链接器设置）
├── Cargo.toml             # 项目依赖配置
├── rust-toolchain.toml    # Rust 工具链配置（nightly）
└── .gitignore             # Git 忽略规则
```

## 游戏常量

- **竞技场尺寸**: 1600 x 1200 像素
- **坦克尺寸**: 87 x 103 像素
- **坦克速度**: 120 像素/秒
- **敌方坦克出生点**: 3个固定位置（顶部左、中、右）

## 核心系统

### 主要组件

- `Score`: 分数资源
- `Life`: 生命值资源
- `AnimationIndices`: 精灵图动画索引
- `AnimationTimer`: 动画计时器
- `MoveTimer`: 移动计时器
- `EnemyTank`: 敌方坦克标记
- `Wall`: 墙壁标记

### 主要系统

1. **animate_sprite**: 处理精灵图动画
2. **setup**: 初始化游戏场景（相机、坦克、墙壁）
3. **collect_collision**: 收集碰撞事件
4. **move_enemy_tanks**: 控制敌方坦克移动和转向
5. **collect_contact_info**: 收集碰撞接触信息用于坦克转向决策

## 开发约定

### 代码风格

- 启用 Clippy 的 `pedantic` 和 `nursery` lint 级别
- 允许 `missing_docs_in_private_items`（私有项无需文档）
- 允许 `float_arithmetic`（浮点运算）
- 允许 `needless_pass_by_value`（按值传递）

### 物理配置

- 使用 Rapier 物理引擎，比例：100 像素 = 1 米
- 墙壁使用 `RigidBody::Fixed`
- 敌方坦克使用 `RigidBody::KinematicVelocityBased`
- 玩家坦克当前为静态（无刚体组件）

### 碰撞处理

- 敌方坦克碰撞时优先根据接触深度法线方向转向
- 若无接触信息，则随机选择转向（逆时针90°、顺时针90°或180°）
- 每6秒或碰撞时重新计算移动方向

## 待办事项

- [ ] 实现玩家坦克控制（键盘输入）
- [ ] 添加射击功能
- [ ] 实现可破坏墙壁
- [ ] 添加游戏状态管理（开始、暂停、结束）
- [ ] 实现分数和生命值系统
- [ ] 添加音效和背景音乐
- [ ] 优化性能和代码结构