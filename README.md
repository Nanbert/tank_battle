# Tank Battle - For Communism!

一个使用 Rust 和 Bevy 游戏引擎开发的经典坦克大战（Battle City 1990）简化版实现。

## 游戏引擎

### 技术栈

- **编程语言**: Rust (Edition 2024)
- **游戏引擎**: Bevy 0.17.3
- **物理引擎**: bevy_rapier2d 0.32
- **架构模式**: ECS (Entity Component System)

### 核心依赖

- `bevy`: 基础游戏引擎，提供渲染、音频、输入等功能
- `bevy_rapier2d`: 2D 物理模拟，处理碰撞检测和刚体运动
- `rand`: 随机数生成器，用于敌方坦克AI和地图生成
- `log`: 日志系统

## 游戏玩法

### 游戏模式

- **1 Player**: 单人模式
- **2 Player**: 双人合作模式

### 玩家控制

#### 玩家1 (Li Yun Long)
- **WASD**: 移动坦克
- **J**: 发射炮弹
- **B**: 回城技能（4秒）
- **K**: 冲刺技能（0.2秒，可撞碎砖块、铁块、敌方坦克）

#### 玩家2 (Chu Yun Fei)
- **方向键**: 移动坦克
- **Numpad1**: 发射炮弹
- **Numpad4**: 回城技能（4秒）
- **Numpad2**: 冲刺技能（0.2秒，可撞碎砖块、铁块、敌方坦克）

#### 通用操作
- **W/S**: 菜单选择
- **SPACE**: 确认选择 / 暂停游戏
- **ESC**: 退出游戏

### 游戏元素

#### 地形类型
- **砖块** (Brick): 可被炮弹破坏
- **钢铁** (Steel): 无法破坏（除非 protection >= 100%）
- **海** (Sea): 坦克无法通过
- **树林** (Forest): 坦克可隐藏，提供掩护
- **屏障** (Barrier): 靠近会伤害坦克（减少 speed 和 protection）

#### 游戏角色
- **玩家坦克**: 可移动、射击、使用技能
- **敌方坦克**: 随机移动和射击
- **指挥官**: 需要保护，有生命值

#### 道具系统
- **Speed Up**: 提升移动速度
- **Protection**: 提升护甲
- **Fire Speed**: 提升射击速度
- **Fire Shell**: 火焰炮弹
- **Track Chain**: 履带链（免疫屏障伤害）
- **Penetrate**: 穿透能力
- **Repair**: 修复坦克
- **Hamburger**: 恢复生命

### 游戏机制

- **关卡系统**: 多个关卡，难度递增
- **属性系统**: 速度、射击速度、保护等属性可升级
- **技能系统**: 回城和冲刺技能
- **碰撞检测**: 基于 Rapier 物理引擎的精确碰撞
- **动画系统**: 丰富的精灵图动画效果

#### 冲刺技能机制
- **撞碎砖块**: 扣除玩家 1/3 血条（根据 protection 调整）
- **撞碎铁块**: protection >= 100% 时可撞碎，否则玩家死亡
- **撞碎敌方坦克**: 扣除玩家 1/3 血条（根据 protection 调整）
- **扣血规则**: protection < 40% 扣 2/3，40-80% 扣 1/3，>= 80% 不扣血

#### 屏障伤害机制
- 靠近屏障（70像素内）会触发伤害
- speed 和 protection 各减少 20
- 2 秒冷却时间
- 拥有 track_chain 特效时免疫伤害

#### 特效抵挡机制
- 拥有 fire_shell、track_chain 或 penetrate 时可抵挡一次敌方子弹
- 中弹后移除对应特效，不扣血

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

## 游戏常量

- **竞技场尺寸**: 1600 x 1200 像素
- **坦克尺寸**: 87 x 87 像素
- **玩家坦克速度**: 150 像素/秒
- **敌方坦克速度**: 200 像素/秒
- **炮弹速度**: 600-900 像素/秒

## 开发约定

- 使用 Clippy 的 `pedantic` 和 `nursery` lint 级别
- 基于 Rapier 物理引擎，比例：100 像素 = 1 米
- 敌方坦克AI：随机移动方向，碰撞时自动转向

## 项目结构

```
tank_battle/
├── assets/              # 游戏资源
│   ├── texture/         # 纹理资源
│   ├── maps/            # 地图资源
│   ├── power_up/        # 道具资源
│   └── start_scene/     # 开始场景动画
├── src/
│   ├── main.rs          # 主程序入口
│   ├── constants.rs     # 游戏常量定义
│   └── resources.rs     # 资源定义
├── Cargo.toml           # 项目依赖配置
└── rust-toolchain.toml  # Rust 工具链配置
```