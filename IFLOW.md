# Steel Command - 钢铁指令

## 项目概述

这是一个使用 Rust 和 Bevy 游戏引擎开发的经典坦克大战（Battle City 1990）现代化实现。该项目是一个功能完整的 2D 游戏，支持单人/双人合作模式，包含丰富的游戏机制：玩家坦克控制、敌方 AI、射击系统、技能系统（回城/冲刺/激光）、道具系统、多关卡、可破坏地形、指挥官保护、天气系统、关卡编辑器等。

### 主要技术栈

- **编程语言**: Rust (Edition 2024)
- **游戏引擎**: Bevy 0.18
- **物理引擎**: Avian2D 0.5
- **Rust 工具链**: Nightly channel
- **链接器**: Linux 使用 clang + mold，Windows 使用 rust-lld
- **Web 支持**: WebAssembly (wasm32-unknown-unknown)

### 核心依赖

- `bevy`: 基础游戏引擎，启用了 `dynamic_linking` 特性以加快编译速度
- `log`: 日志库，开发环境启用 debug 级别，发布环境启用 warn 级别
- `rand`: 随机数生成器
- `avian2d`: 2D 物理模拟（碰撞检测、刚体运动）
- Web 目标依赖: `wasm-bindgen`, `web-sys`, `wasm-bindgen-futures`, `console_error_panic_hook`, `getrandom`

### 架构特点

- 使用 ECS (Entity Component System) 架构
- 基于 Avian2D 物理引擎实现精确的碰撞检测和坦克移动
- 精灵图动画系统（Texture Atlas），支持多种动画模式（循环/一次性/循环范围）
- 模块化代码结构：每个功能（玩家、敌人、子弹、道具、特效等）独立模块
- 状态机驱动的游戏流程管理
- 双玩家支持，独立按键绑定
- 丰富的特效系统（爆炸、烟雾、火花、能量球、激光）
- 跨平台支持：桌面端（Linux/Windows）和 Web 端
- 本地化支持（中文/英文）
- 天气系统（雨/雪）
- 关卡编辑器（桌面端专属）

## 构建和运行

### 前置要求

- Rust Nightly 工具链
- Linux: clang, mold 链接器
- Windows: rust-lld（Rust 自带）
- Web 构建: wasm-pack

### 桌面端常用命令

```bash
# 开发模式构建（启用动态链接）
cargo build --features dev

# 运行开发版本
cargo run --features dev

# 发布版本构建（体积优化）
cargo build --release

# 运行发布版本
cargo run --release

# 清理构建产物
cargo clean
```

### Web 端构建命令

```bash
# 安装 wasm-pack（首次运行）
cargo install wasm-pack

# 构建 Web 版本
bash build_web.sh

# 运行 Web 版本（在 dist 目录下启动 HTTP 服务器）
cd dist && python -m http.server 8000
# 或使用 Node.js
cd dist && npx http-server -p 8000

# 访问 http://localhost:8000
```

### 自动化部署

项目支持 Vercel 自动部署：
- 推送代码到 GitHub 后，Vercel 会自动构建 Web 版本
- 配置文件: `vercel.json`
- 输出目录: `dist/`

### 编译优化配置

项目使用自定义的编译配置以平衡开发体验和性能：

**开发模式:**
- `[profile.dev]`: 主代码 opt-level = 1
- `[profile.dev.package."*"]`: 依赖项 opt-level = 3
- 启用 `dynamic_linking` 特性（通过 `--features dev`）

**发布模式:**
- `opt-level = "z"`: 优化体积
- `lto = true`: 启用链接时优化
- `codegen-units = 1`: 单代码单元，更好的优化
- `strip = true`: 移除调试符号
- `panic = "abort"`: 减少二进制体积

## 项目结构

```
tank_battle/
├── assets/                    # 游戏资源
│   ├── effect/               # 特效纹理（爆炸、烟雾、火花、能量球、激光）
│   ├── enemy_tank/           # 敌方坦克纹理
│   ├── font/                 # 字体文件（中英文）
│   ├── maps/                 # 地形纹理（砖块、钢铁、海、树林）
│   ├── music/                # 音效和背景音乐
│   ├── power_up/             # 道具纹理
│   ├── texture/              # 游戏纹理（玩家坦克、头像、子弹等）
│   ├── ambience/             # 环境特效（气泡、树叶等）
│   └── logo.png              # 游戏图标
├── docs/
│   └── controls.md           # 详细操作说明
├── levels/                   # 关卡文件（10个关卡）
│   ├── 1.txt ~ 10.txt        # 关卡1-10地图
│   └── README.md             # 关卡文件格式说明
├── src/
│   ├── app.rs                # 应用配置、系统注册
│   ├── atlas.rs              # 纹理图集管理
│   ├── ambience.rs           # 环境音效系统
│   ├── bullet.rs             # 子弹系统
│   ├── commander.rs          # 指挥官系统
│   ├── constants.rs          # 游戏常量（尺寸、颜色、路径等）
│   ├── dash.rs               # 冲刺技能系统
│   ├── effects.rs            # 特效系统
│   ├── enemy.rs              # 敌方坦克系统
│   ├── game_state.rs         # 游戏状态管理
│   ├── global_rng.rs         # 全局随机数生成器
│   ├── laser.rs              # 激光技能系统
│   ├── level_editor.rs       # 关卡编辑器（桌面端）
│   ├── levels.rs             # 关卡加载
│   ├── lib.rs                # Web 库入口
│   ├── main.rs               # 桌面端主程序入口
│   ├── map.rs                # 地图系统
│   ├── physics_config.rs     # 物理引擎配置
│   ├── player.rs             # 玩家系统
│   ├── powerup.rs            # 道具系统
│   ├── powerup_strategy.rs   # 道具效果策略模式
│   ├── resources.rs          # 游戏资源定义
│   ├── utils.rs              # 工具函数
│   ├── weather.rs            # 天气系统（雨/雪）
│   └── ui/                   # UI 模块
│       ├── mod.rs            # UI 模块入口
│       ├── common.rs         # 通用 UI 组件
│       ├── constants.rs      # UI 常量
│       ├── editor.rs         # 编辑器 UI
│       ├── localization.rs   # 本地化文本
│       ├── menus.rs          # 菜单 UI
│       ├── overlay.rs        # 覆盖层 UI
│       └── hud/              # HUD 子模块
│           ├── mod.rs        # HUD 模块入口
│           ├── blink.rs      # 闪烁效果
│           ├── spawn.rs      # HUD 生成
│           ├── stats.rs      # 统计信息
│           └── update.rs     # HUD 更新
├── .cargo/
│   └── config.toml           # Cargo配置（链接器设置）
├── .github/
│   └── workflows/            # GitHub Actions 工作流
│       ├── release.yml       # 发布流程
│       └── web-deploy.yml    # Web 部署流程
├── Cargo.toml                # 项目依赖配置
├── rust-toolchain.toml       # Rust工具链配置（nightly）
├── PKGBUILD                  # Arch Linux打包配置
├── tank-battle.desktop       # Linux桌面文件
├── build_web.sh              # Web 构建脚本
├── index.html                # Web 入口页面
├── vercel.json               # Vercel 部署配置
├── package.json              # npm 配置（用于 Web 构建）
└── IFLOW.md                  # 本文档
```

## 游戏常量

### 尺寸常量

| 参数 | 数值 | 说明 |
|------|------|------|
| 窗口尺寸 | 2060 x 1300 像素 | 总窗口大小 |
| 地图尺寸 | 1600 x 1200 像素 | 游戏区域 |
| 左右侧边距 | 230 像素 | HUD显示区域 |
| 顶部边距 | 100 像素 | 状态栏区域 |
| 玩家坦克 | 80 x 90 像素 | 显示尺寸 |
| 敌方坦克 | 80 x 90 像素 | 显示尺寸 |
| 子弹 | 60 x 40 像素 | 碰撞体积 |
| 物理比例 | 100 像素 = 1 米 | Rapier引擎 |

### 游戏数值常量

| 参数 | 数值 | 说明 |
|------|------|------|
| 玩家坦克速度 | 150 像素/秒 | 默认速度 |
| 敌方坦克速度 | 200 像素/秒 | 默认速度 |
| 玩家子弹速度 | 600 像素/秒 | |
| 敌方子弹速度 | 900 像素/秒 | |
| 最大敌方坦克数 | 4 | 场上同时存在 |
| 每关敌方坦克总数 | 10 | |
| 回城技能时间 | 2 秒 | |
| 冲刺技能时间 | 0.2 秒 | |
| 激光蓄力时间 | 3 秒 | |
| 最大生命值 | 3 | 玩家 |
| 指挥官生命值 | 3 | |
| 最大能量值 | 3 | 激光技能 |

## 核心系统架构

### 游戏状态

游戏通过 `GameState` 枚举管理不同的游戏阶段：

- `StartScreen`: 开始界面（菜单选择）
- `FadingOut`: 淡出过渡
- `StageIntro`: 关卡介绍（关卡名称和俏皮话）
- `Playing`: 游戏进行中
- `Paused`: 暂停
- `GameOver`: 游戏结束
- `About`: 关于页面
- `Credits`: 制作人员

### 系统调度集

为了优化性能，系统被分组到不同的调度集：

- `EnemySystems`: 敌方坦克相关系统
- `PlayerSystems`: 玩家坦克相关系统
- `BulletSystems`: 子弹相关系统
- `LaserSystems`: 激光相关系统
- `EffectsAndAnimationSystems`: 特效和动画
- `CommanderSystems`: 指挥官相关系统
- `HudSystems`: HUD界面系统
- `PowerUpSystems`: 道具系统
- `GameStateSystems`: 游戏状态管理
- `AmbienceSystems`: 环境音效系统

### 主要组件

**玩家相关:**
- `PlayerTank`: 玩家坦克标记（包含坦克类型）
- `PlayerKeyBindings`: 按键绑定配置
- `IsDashing`: 冲刺状态标记
- `IsRecalling`: 回城状态标记
- `RecoilForce`: 后坐力组件
- `LaserCharge`: 激光蓄力组件
- `TankFireConfig`: 坦克射击配置

**敌方相关:**
- `EnemyTank`: 敌方坦克标记（包含方向）
- `EnemyBornAnimation`: 出生动画标记

**子弹相关:**
- `FireEffect`: 火焰特效标记
- `PenetrateEffect`: 穿透特效标记

**地形相关:**
- `Wall`: 墙壁基类
- `Brick`: 砖块（可破坏）
- `Steel`: 钢铁（不可破坏）
- `Barrier`: 屏障（伤害型）
- `Forest`: 树林（掩体）
- `Sea`: 海洋（不可通过）

**指挥官:**
- `Commander`: 指挥官标记
- `CommanderLife`: 生命值资源

**天气相关:**
- `WeatherType`: 天气类型（None/Rain/Snow）
- `WeatherParticle`: 降水粒子
- `WeatherSystem`: 天气系统调度集

**环境音效:**
- `AmbienceType`: 环境类型（海洋/树林/指挥官）
- `AmbienceVolume`: 环境音量控制

**关卡编辑器:**
- `EditorUI`: 编辑器界面标记
- `SelectedTerrain`: 选中的地形类型
- `TerrainButton`: 地形按钮标记

**特效相关:**
- `Explosion`: 爆炸特效
- `Smoke`: 烟雾特效
- `Spark`: 火花特效
- `EnergyBall`: 能量球（激光技能）
- `Laser`: 激光束

**道具相关:**
- `PowerUp`: 道具基类
- `PowerUpStrategy`: 道具效果策略模式（零成本抽象）
- 各种道具类型：SpeedUp, Protection, FireSpeed, FireShell, TrackChain, Penetrate, Repair, Hamburger, AirCushion, Shell

**UI相关:**
- `StartScreenUI`, `MenuOption`, `MenuArrow`: 开始界面UI
- `PauseUI`: 暂停界面UI
- `GameOverUI`: 游戏结束界面UI
- `StageIntroUI`: 关卡介绍UI
- `PlayerUI`: 玩家HUD标记
- `Language`: 语言资源（中文/英文）
- `LocalizedText`: 本地化文本结构
- `PlayingEntity`: 游戏进行中的实体标记

**动画相关:**
- `AnimationIndices`: 动画帧范围
- `AnimationTimer`: 动画计时器
- `AnimationMode`: 动画播放模式（Looping/OneShot/LoopRange）
- `CurrentAnimationFrame`: 当前帧资源

### 地形系统

地图基于 12x16 的网格系统，每个格子 100x100 像素：

| 地形 | 符号 | 可破坏 | 坦克通过 | 子弹通过 | 特殊效果 |
|------|------|--------|----------|----------|----------|
| 空地 | `.` | - | ✓ | ✓ | - |
| 树林 | `t` | 否 | ✓ | ✓ | 提供掩护 |
| 海 | `s` | 否 | ✗ | ✓ | 不可通过 |
| 砖块 | `b` | 1发子弹 | ✗ | ✗ | - |
| 钢铁 | `i` | 否 | ✗ | ✗ | protection>=100%可破坏 |
| 屏障 | `a` | 否 | ✗ | ✓ | 靠近伤害坦克 |

## 游戏机制

### 玩家控制

**玩家1 (Li Yun Long):**
- 移动: WASD
- 射击: J
- 回城: I
- 冲刺: K
- 激光: L

**玩家2 (Chu Yun Fei):**
- 移动: 方向键
- 射击: Numpad1
- 回城: Numpad4
- 冲刺: Numpad2
- 激光: Numpad3

**通用操作:**
- 菜单选择: W/S
- 确认/暂停: Space
- 退出: ESC

### 技能系统

#### 回城技能
- 按键: I / Numpad4
- 持续时间: 2秒
- 效果: 将坦克传送回初始位置
- 显示: 进度条显示传送进度

#### 冲刺技能
- 按键: K / Numpad2
- 持续时间: 0.2秒
- 冲刺距离: 两个坦克长度
- 撞击效果:
  - 砖块: 可撞碎，扣血（根据 protection）
  - 钢铁: protection>=100% 可撞碎，否则死亡
  - 敌方坦克: 可撞碎，扣血（根据 protection）
- 扣血规则:
  - protection < 40%: 扣 2/3 血量
  - protection 40-80%: 扣 1/3 血量
  - protection >= 80%: 不扣血

#### 激光技能
- 按键: L / Numpad3
- 蓄力时间: 4秒
- 消耗: 1点能量
- 效果: 发射激光束，穿透多个目标
- 动画: 能量球蓄力 → 激光发射

### 道具系统

| 道具 | 属性影响 | 特殊效果 |
|------|----------|----------|
| Speed Up | 速度+20 | - |
| Protection | 护甲+20 | - |
| Fire Speed | 射速+20 | - |
| Fire Shell | - | 可抵挡一次敌方子弹 |
| Track Chain | - | 免疫屏障伤害，可抵挡子弹 |
| Penetrate | - | 子弹可穿透，可抵挡子弹 |
| Repair | - | 恢复1点生命 |
| Hamburger | - | 恢复1点生命 |
| Air Cushion | - | 气垫特效（特殊效果） |

### 屏障伤害机制
- 靠近屏障（100像素内）触发伤害
- speed 和 protection 各减少 20
- 2秒冷却时间
- 拥有 track_chain 特效时免疫伤害

### 敌方AI
- 随机移动方向
- 碰撞时自动转向（基于法线方向）
- 定期随机转向（每2秒）
- 随机射击（1%概率/帧）

### 碰撞检测
- 基于 Avian2D 物理引擎
- 敌方坦克碰撞缓存机制优化性能
- 精确的碰撞接触信息用于转向决策

### 天气系统
**视觉效果**
- **雨天**: 垂直下落的雨滴粒子，每帧生成 5 个
- **雪天**: 缓慢飘落的雪花粒子，每帧生成 3 个
- **无天气**: 默认状态

**天气特性**
- 雨滴尺寸: 3x15 像素，下落速度快
- 雪花尺寸: 4x4 像素，下落速度慢
- 粒子从地图顶部上方生成，底部下方销毁
- 纯视觉效果，不影响游戏玩法

### 本地化支持
**支持语言**
- 中文（默认）
- 英文

**本地化内容**
- 所有菜单文本
- 关卡介绍文本
- HUD 显示文本
- 游戏提示信息

**切换方式**
- 主菜单: "语言 / Language" 选项
- 实时生效，无需重启

## 开发约定

### 代码风格

- 启用 Clippy 的 `pedantic` 和 `nursery` lint 级别
- 允许 `missing_docs_in_private_items`（私有项无需文档）
- 允许 `float_arithmetic`（浮点运算）
- 允许 `needless_pass_by_value`（按值传递）
- 允许 `wildcard_imports`（通配符导入用于简化）

### 物理配置

- 使用 Avian2D 物理引擎，比例：100 像素 = 1 米
- 重力设置为 ZERO（俯视视角游戏）
- 墙壁使用 `RigidBody::Static`
- 敌方坦克使用 `RigidBody::Kinematic`
- 玩家坦克使用 `CharacterController` 实现移动
- 碰撞检测使用 Avian2D 的碰撞系统

### 动画系统

- 支持三种动画模式：
  - `Looping`: 循环播放（敌方坦克、森林、海洋等）
  - `OneShot`: 播放一次后停止（爆炸、烟雾等）
  - `LoopRange`: 播放一次后循环指定范围（能量球蓄力→激光）
- 统一的动画计时器系统
- 帧速率可配置

### 资源管理

- 使用 Bevy 资源系统管理全局状态
- 主要资源：
  - `StageLevel`: 当前关卡
  - `GameMode`: 游戏模式（单人/双人）
  - `PlayerInfo`: 玩家信息（生命、能量、分数等）
  - `CommanderLife`: 指挥官生命值
  - `EnemyCollisionCache`: 敌方碰撞缓存
  - `Language`: 语言设置（中文/英文）
  - `GlobalRng`: 全局随机数生成器
  - `WeatherType`: 当前天气类型

## 关卡系统

### 关卡文件格式

- 位置: `levels/` 目录
- 格式: 纯文本，每行 16 个符号，共 12 行
- 符号见"地形系统"表格
- 支持 50x50 的半块地形（bl, br, bt, bb, il, ir, it, ib）
- 当前关卡数: 10 个（1.txt ~ 10.txt）

### 关卡俏皮话

每个关卡有中英文俏皮话（共17句），在关卡介绍界面显示。

### 关卡编辑器

**桌面端专属功能**

- 通过主菜单进入"关卡编辑器"（仅桌面端菜单显示此选项）
- 左侧面板：地形选择（空地、树林、海、砖块、钢铁、屏障）
- 右侧面板：编辑操作（保存、加载、清空、返回）
- 鼠标操作：点击地形选择，点击网格放置
- 支持导出关卡到文件
- 支持从文件加载关卡
- **注意**: Web 端完全不支持关卡编辑功能（菜单中不显示此选项）

## 音效系统

### 音效文件

- `brick_hit.ogg`: 砖块被击中
- `burn_tree.ogg`: 树林燃烧
- `commander_death.ogg`: 指挥官死亡
- `commander_get_shot.ogg`: 指挥官被击中
- `explosion_sound.ogg`: 爆炸
- `hit_sound.ogg`: 击中
- `laser_charge.ogg`: 激光蓄力
- `laser.ogg`: 激光发射
- `metal_crash.ogg`: 金属碰撞
- `player_shot.ogg` / `player_shot.mp3`: 玩家射击
- `enemy_shot.ogg` / `enemy_shot.mp3`: 敌方射击
- `sea_ambience.ogg`: 海洋环境音
- `tree_ambience.ogg`: 树林环境音
- `bubbles.ogg`: 气泡音效
- `rain.ogg`: 雨声
- `commander_music.ogg`: 指挥官音乐
- `powerup_sound.ogg`: 道具拾取音效
- `dash.ogg` / `dash.mp3`: 冲刺音效
- `music_note_*.ogg`: 音乐音符（4个）
- `commander_music_*.ogg`: 指挥官音乐（4首）

### 环境音效

- 自动播放：靠近海洋/树林/指挥官时播放对应环境音
- 音量控制：不同音效有独立音量设置
- 音效淡入淡出：平滑切换环境音效
- Web 端限制：需要用户交互后才能播放音频

### 特效音效
- 关卡俏皮话显示时播放音符音效
- 指挥官附近播放特殊音乐

## 待办事项

- [ ] 添加更多关卡（当前10关）
- [ ] 实现存档/读档功能
- [ ] 添加网络多人对战
- [ ] 优化性能（减少内存分配）
- [ ] 添加更多道具类型
- [ ] 实现成就系统
- [ ] 添加排行榜
- [ ] 添加游戏教程
- [ ] 支持更多语言本地化（当前支持中英文）
- [ ] 添加更多天气类型（当前支持雨/雪）
- [ ] 改进 Web 端音频支持
- [ ] 添加移动端触控支持

## 版本历史

### v0.6.0
- 升级到 Bevy 0.18
- 迁移物理引擎到 Avian2D
- 添加 WebAssembly 支持（Web 端可运行）
- 添加天气系统（雨/雪）
- 添加关卡编辑器（桌面端）
- 关卡数量增加到 10 个
- UI 模块化重构
- 添加本地化支持（中英文）
- 优化代码结构和性能

### v0.3.0
- 完整的双玩家支持
- 技能系统（回城、冲刺、激光）
- 道具系统（9种道具）
- 关卡系统（4个关卡）
- 指挥官保护机制
- 丰富的特效系统
- 完整的UI系统
- 音效和背景音乐

### 早期版本
- 基础坦克移动和碰撞
- 单玩家控制
- 简单的射击系统
- 基础地形（砖块、钢铁）

## 平台支持

### 桌面端
- Linux (x86_64)
- Windows (x86_64)

### Web 端
- 支持现代浏览器（Chrome, Firefox, Safari, Edge）
- 需要用户交互后才能播放音频
- 使用 WebAssembly 编译

## 部署

### GitHub Actions
- **release.yml**: 自动构建发布版本
- **web-deploy.yml**: 自动部署到 Vercel

### Vercel 部署
- 自动从 GitHub 仓库部署
- 输出目录: `dist/`
- 缓存策略优化
- 支持自定义域名