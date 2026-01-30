# Tank Battle - For Communism!

一个使用 Rust 和 Bevy 游戏引擎开发的经典坦克大战（Battle City 1990）简化版实现。

## 游戏预览

![Gameplay 1](assets/sample1.gif)
![Gameplay 2](assets/sample2.gif)

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

- **1 Player**: 单人模式
- **2 Player**: 双人合作模式

## 完整游戏指南

### 🎮 游戏模式

- **1 Player**: 单人模式
- **2 Player**: 双人合作模式

### ⌨️ 玩家控制

#### 玩家1 (Li Yun Long)

| 按键 | 功能 |
|------|------|
| **WASD** | 移动坦克 |
| **J** | 发射炮弹 |
| **I** | 回城技能（4秒） |
| **K** | 冲刺技能（0.2秒） |
| **L** | 激光技能 |

#### 玩家2 (Chu Yun Fei)

| 按键 | 功能 |
|------|------|
| **方向键** | 移动坦克 |
| **Numpad1** | 发射炮弹 |
| **Numpad4** | 回城技能（4秒） |
| **Numpad2** | 冲刺技能（0.2秒） |
| **Numpad3** | 激光技能 |

#### 通用操作

| 按键 | 功能 |
|------|------|
| **W/S** | 菜单选择（上/下） |
| **SPACE** | 确认选择 / 暂停游戏 |
| **ESC** | 退出游戏 |

### 🗺️ 地形类型

| 地形 | 描述 |
|------|------|
| **砖块** (Brick) | 可被炮弹破坏 |
| **钢铁** (Steel) | 无法破坏（除非 protection >= 100%） |
| **海** (Sea) | 坦克无法通过 |
| **树林** (Forest) | 坦克可隐藏，提供掩护 |
| **屏障** (Barrier) | 靠近会伤害坦克（减少 speed 和 protection） |

### 👥 游戏角色

| 角色 | 描述 |
|------|------|
| **玩家坦克** | 可移动、射击、使用技能 |
| **敌方坦克** | 随机移动和射击 |
| **指挥官** | 需要保护，有生命值 |

### 🎁 道具系统

| 道具 | 效果 |
|------|------|
| **Speed Up** | 提升移动速度 |
| **Protection** | 提升护甲 |
| **Fire Speed** | 提升射击速度 |
| **Fire Shell** | 火焰炮弹 |
| **Track Chain** | 履带链（免疫屏障伤害） |
| **Penetrate** | 穿透能力 |
| **Repair** | 修复坦克 |
| **Hamburger** | 恢复生命 |
| **Air Cushion** | 气垫（特殊效果） |

### ⚙️ 游戏机制

#### 关卡系统
- 多个关卡，难度递增
- 每关有固定的敌人和地形布局

#### 属性系统
- **速度**: 影响坦克移动速度
- **射击速度**: 影响炮弹发射频率
- **保护**: 影响受到的伤害和特殊能力

#### 技能系统

**回城技能 (I / Numpad4)**
- 持续 4 秒
- 将坦克传送回初始位置

**冲刺技能 (K / Numpad2)**
- 持续 0.2 秒
- 可撞碎砖块、铁块、敌方坦克
- 撞击时扣除玩家血量（根据 protection 调整）

**激光技能 (L / Numpad3)**
- 发射激光束
- 可穿透多个目标

#### 冲刺技能机制

| 撞击目标 | 条件 | 效果 |
|----------|------|------|
| **砖块** | 总是可撞碎 | 扣除玩家 1/3 血条（根据 protection 调整） |
| **铁块** | protection >= 100% | 可撞碎，否则玩家死亡 |
| **敌方坦克** | 总是可撞碎 | 扣除玩家 1/3 血条（根据 protection 调整） |

**扣血规则**（基于 protection）:
- protection < 40%: 扣 2/3 血量
- protection 40-80%: 扣 1/3 血量
- protection >= 80%: 不扣血

#### 屏障伤害机制
- 靠近屏障（70像素内）会触发伤害
- speed 和 protection 各减少 20
- 2 秒冷却时间
- 拥有 track_chain 特效时免疫伤害

#### 特效抵挡机制
拥有以下特效时可抵挡一次敌方子弹：
- **fire_shell** (火焰炮弹)
- **track_chain** (履带链)
- **penetrate** (穿透)

中弹后移除对应特效，不扣血

### 📊 游戏常量

| 参数 | 数值 |
|------|------|
| **竞技场尺寸** | 2060 x 1300 像素 |
| **坦克尺寸** | 87 x 87 像素 |
| **玩家坦克速度** | 150 像素/秒 |
| **敌方坦克速度** | 200 像素/秒 |
| **炮弹速度** | 600-900 像素/秒 |
| **物理比例** | 100 像素 = 1 米 |

## 下载安装

![Latest Release](https://img.shields.io/github/v/release/Nanbert/tank_battle?style=flat-square)

### Linux (Arch Linux)
```bash
# 直接下载
wget https://github.com/Nanbert/tank_battle/releases/latest/download/tank-battle-*-x86_64.pkg.tar.zst

# 使用 pacman 安装
sudo pacman -U tank-battle-*-x86_64.pkg.tar.zst
```

### Linux (通用)
下载最新版本：[tank_battle_linux_x64.tar.gz](https://github.com/Nanbert/tank_battle/releases/latest/download/tank_battle_linux_x64.tar.gz)

解压后运行：
```bash
tar xzf tank_battle_linux_x64.tar.gz
cd tank_battle
./tank_battle
```

### Windows
下载最新版本：[tank_battle_windows_x64.zip](https://github.com/Nanbert/tank_battle/releases/latest/download/tank_battle_windows_x64.zip)

解压后运行 `tank_battle.exe`

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

## 项目结构

```
tank_battle/
├── assets/              # 游戏资源
│   ├── background/      # 背景动画（3部分精灵图）
│   ├── effect/          # 特效（爆炸、激光、烟雾等）
│   ├── enemy_tank/      # 敌方坦克纹理
│   ├── font/            # 字体文件
│   ├── maps/            # 地图元素（砖块、钢铁、海、树林等）
│   ├── music/           # 音效和背景音乐
│   ├── power_up/        # 道具图标
│   ├── texture/         # 玩家坦克、子弹、头像等纹理
│   ├── alipay.png       # 支付宝收款码
│   ├── sample1.gif      # 游戏预览1
│   └── sample2.gif      # 游戏预览2
├── docs/                # 文档
│   └── controls.md      # 游戏操作说明
├── levels/              # 关卡文件（1-4关）
├── src/                 # 源代码
│   ├── main.rs          # 主程序入口
│   ├── app.rs           # 应用配置和插件注册
│   ├── bullet.rs        # 子弹系统
│   ├── commander.rs     # 指挥官系统
│   ├── constants.rs     # 游戏常量定义
│   ├── dash.rs          # 冲刺技能系统
│   ├── effects.rs       # 特效系统
│   ├── enemy.rs         # 敌方坦克系统
│   ├── game_state.rs    # 游戏状态管理
│   ├── hud_ui.rs        # HUD界面（分数、生命值等）
│   ├── laser.rs         # 激光系统
│   ├── levels.rs        # 关卡加载系统
│   ├── map.rs           # 地图渲染和碰撞系统
│   ├── menus_ui.rs      # 菜单界面
│   ├── overlay_ui.rs    # 遮罩层界面
│   ├── player.rs        # 玩家坦克系统
│   ├── powerup.rs       # 道具系统
│   └── resources.rs     # 资源定义
├── .cargo/              # Cargo 配置
│   └── config.toml      # 链接器设置
├── .github/workflows/   # GitHub Actions 配置
│   └── release.yml      # 自动发布工作流
├── PKGBUILD             # Arch Linux 包配置
├── Cargo.lock           # 依赖锁定文件
├── Cargo.toml           # 项目依赖配置
├── IFLOW.md             # iFlow 项目文档
├── rust-toolchain.toml  # Rust 工具链配置（nightly）
├── tank-battle.desktop  # Linux 桌面启动器
├── COPYRIGHT            # 素材版权声明
├── LICENSE              # 项目许可证
└── README.md            # 项目说明
```

## 版权声明

本项目使用的游戏素材均遵循各自的开源许可协议。详细的素材版权信息请参阅 [COPYRIGHT](COPYRIGHT) 文件。

### 🙏 致谢免费素材提供者

本游戏能够顺利完成，离不开以下优秀创作者的无私分享。在此向他们致以最诚挚的感谢！

#### OpenGameArt.org 贡献者
- **HorrorPen** - 气泡特效 (CC-BY 3.0)
- **Sinestesia** - 爆炸效果、钢铁击中特效 (CC0 1.0)
- **netcake3** - 激光特效 (CC-BY-SA 3.0/4.0)
- **JoesAlotofthings** - 敌方坦克出生动画 (CC-BY 4.0)
- **irmirx** - 玩家坦克纹理、敌方坦克纹理 (CC-BY 3.0)
- **Skorpio** - 烟雾特效 (CC-BY 3.0)
- **Wenrexa** - 激光子弹素材包 (CC0 1.0)

#### AI 生成素材
- **通义千问** - 背景动画、音符特效、地图元素、道具素材、头像和司令官纹理 (CC0 1.0)

#### 字体作者
- **Latinotype** - ChelaOne 字体
- **Corben 字体作者** - Corben 字体
- **刘欢** - 刘欢卡通手书字体
- **Matemasie 字体作者** - Matemasie 字体

### 许可说明

本项目使用的素材遵循以下开源许可协议：
- **CC-BY 3.0/4.0**: 需要署名，可商业使用
- **CC-BY-SA 3.0/4.0**: 需要署名，相同方式共享
- **CC0 1.0**: 公共领域，可自由使用、修改和分发

请在使用本项目时遵守相应的开源许可协议要求。特别感谢 OpenGameArt.org 平台为游戏开发者提供了如此丰富的免费素材资源！

## 推荐工具

如果你也想开发游戏项目，强烈推荐使用 **iFlow**——一个免费的人工智能智能体。

本项目在 iFlow 的协助下，仅用 20 多天就完成了原本可能需要 2-3 个月的工作。iFlow 能够帮助你：
- 快速定位和修复 bug
- 优化代码架构和性能
- 提供专业的技术建议
- 24小时随时待命，完全免费

试试 iFlow，让你的开发效率提升数倍！

## 支持作者

If you enjoyed the game, please buy me a coffee! ☕️
(Caffeine is a programmer's fuel)

![Alipay](assets/alipay.png)
![WeChat](assets/wechat.png)