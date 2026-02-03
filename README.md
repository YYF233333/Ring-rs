# Ring-rs

一个使用 Rust 构建的视觉小说引擎，采用 Runtime/Host 分离架构。

## 项目结构

```
Ring-rs/
├── vn-runtime/          # 核心运行时（引擎无关）
│   ├── src/
│   │   ├── command.rs   # Runtime → Host 命令定义
│   │   ├── input.rs     # Host → Runtime 输入定义
│   │   ├── state.rs     # 运行时状态
│   │   ├── save.rs      # 存档数据模型
│   │   ├── history.rs   # 历史记录数据模型
│   │   ├── script/      # 脚本解析器
│   │   └── runtime/     # 执行引擎
│   └── Cargo.toml
├── host/                # macroquad 适配层
│   ├── src/
│   │   ├── main.rs      # 入口与主循环
│   │   ├── renderer/    # 渲染系统
│   │   ├── audio/       # 音频系统
│   │   ├── input/       # 输入处理
│   │   ├── resources/   # 资源管理
│   │   ├── command_executor/  # 命令执行器
│   │   ├── save_manager/     # 存档管理
│   │   ├── config/      # 配置管理
│   │   └── manifest/    # 立绘布局配置
│   └── Cargo.toml
├── assets/              # 游戏资源
│   ├── backgrounds/     # 背景图片
│   ├── characters/      # 角色立绘
│   ├── bgm/             # 背景音乐
│   ├── fonts/           # 字体文件
│   ├── scripts/         # 脚本文件
│   └── manifest.json    # 立绘布局配置
├── saves/               # 存档目录（自动创建）
│   ├── continue.json    # 专用"继续"存档（返回标题时自动保存）
│   ├── slot_001.json    # 玩家手动存档（槽位 1-99）
│   └── ...
├── config.json          # 应用配置文件（必须配置 start_script_path）
├── user_settings.json   # 用户设置（音量、显示等）
└── docs/                # 文档
    ├── script_syntax_spec.md   # 脚本语法规范
    ├── save_format.md   # 存档格式说明
    ├── manifest_guide.md  # 立绘布局配置说明
    └── resource_management.md  # 资源管理系统使用指南
```

## 快速开始

### 环境要求

- Rust 1.70+
- 支持 OpenGL 的图形环境
- **必须配置 `config.json`** 中的 `start_script_path` 字段

### 配置（首次运行前）

编辑 `config.json`，**必须设置**：

```json
{
  "start_script_path": "scripts/your_game.md",  // ← 必须配置入口脚本路径
  ...
}
```

未配置 `start_script_path` 将导致程序 panic。

### Dev Mode：启动时脚本自动检查（推荐）

Host 启动时可自动运行脚本静态检查（语法 / 未定义 label / 资源存在性），默认 **debug build 开启、release build 关闭**。

- **开启**：在 `config.json` 中设置 `debug.script_check = true`
- **关闭**：在 `config.json` 中设置 `debug.script_check = false`
- **命令行手动检查**：`cargo script-check`（不运行游戏）

### 运行

```bash
cd host
cargo run
```

首次运行将启动 **标题界面**，点击"开始游戏"加载 `start_script_path` 指定的脚本。

### 操作说明

**主菜单（Title Screen）**

- 使用鼠标点击按钮：开始游戏 / 继续 / 读档 / 设置 / 退出
- **继续**按钮在没有 Continue 存档时置灰

**游戏进行中（In-Game）**

| 按键 | 功能 |
|------|------|
| **空格/鼠标点击** | 前进对话 / 完成打字机效果 |
| **↑↓ 或鼠标悬停** | 选择分支时切换选项 |
| **回车/鼠标点击选项** | 确认选择 |
| **ESC** | 打开系统菜单（存档/读档/设置/历史/返回标题/退出） |

**开发者快捷键（Debug Build）**

| 按键 | 功能 |
|------|------|
| **F5** | 快速保存（存到当前槽位） |
| **F9** | 快速读取（从当前槽位） |

## 架构设计

### Runtime/Host 分离

- **vn-runtime**：纯 Rust 核心，不依赖任何图形/音频库
  - 脚本解析与执行
  - 状态管理
  - 通过 Command 与 Host 通信

- **host**：具体的渲染实现（macroquad）
  - 窗口管理
  - 图形渲染
  - 音频播放
  - 输入处理

### 通信模型

```
┌──────────────┐    Command     ┌──────────────┐
│  vn-runtime  │ ─────────────► │     host     │
│              │                │              │
│  脚本解析    │                │  渲染/音频   │
│  状态管理    │ ◄───────────── │  输入处理    │
└──────────────┘  RuntimeInput  └──────────────┘
```

## 脚本语法

脚本采用 Markdown 格式，支持 Typora 等编辑器预览。  
**注意：脚本内所有素材路径（图片/音频）都应相对于脚本文件自身**，以保证在 Typora 中可直接预览。

### 章节标题

```markdown
# 第一章：相遇
## 1.1 早晨
```

### 对话

```markdown
角色名："对话内容"
："这是旁白文本。"
```

### 背景切换

```markdown
changeBG <img src="../backgrounds/scene.jpg" /> with dissolve
```

### 角色显示

```markdown
show <img src="../characters/char.png" /> as alias at center with dissolve
hide alias with fade
```

### 音频

```markdown
<audio src="../bgm/Signal.mp3"></audio> loop   # BGM（循环）
<audio src="../bgm/click.mp3"></audio>        # SFX（播放一次）
stopBGM                                       # 停止 BGM（淡出）
```

### 立绘布局（manifest）

立绘的对齐点/预处理缩放/站位预设由 `assets/manifest.json` 控制，便于不同尺寸立绘保持构图一致（无需改代码）。

### 存档系统

- **F5**：快速保存到槽位 1
- **F9**：快速读取槽位 1
- 存档文件位于 `saves/slot_XXX.json`
- 存档包含：脚本位置、角色状态、背景、BGM、历史记录

### 配置文件

运行配置通过 `config.json` 管理：

```json
{
  "assets_root": "assets",
  "saves_dir": "saves",
  "window": { "width": 1920, "height": 1080 },
  "audio": { "master_volume": 1.0, "bgm_volume": 0.8 }
}
```

### 选择分支

```markdown
| 选择 |  |
| --- | --- |
| 选项A | label_a |
| 选项B | label_b |

**label_a**
："选择了 A"

**label_b**
："选择了 B"
```

详细语法参见 [docs/script_syntax_spec.md](docs/script_syntax_spec.md)

## 功能特性

### 已实现

- ✅ Markdown 脚本解析
- ✅ 背景切换与过渡效果（dissolve/fade/fadewhite）
- ✅ 角色立绘显示（支持多位置）
- ✅ 对话系统（打字机效果）
- ✅ 选择分支与标签跳转
- ✅ `goto **label**` 无条件跳转
- ✅ 章节标题显示
- ✅ 音频系统（BGM/SFX，支持 MP3/WAV/FLAC/OGG）
- ✅ 脚本音频语法（`<audio ...></audio>` / `loop` / `stopBGM`）
- ✅ 鼠标与键盘输入
- ✅ 中文字体支持
- ✅ 立绘布局系统（`assets/manifest.json`：anchor/pre_scale/preset）
- ✅ 存档/读档系统（F5 保存 / F9 读取）
- ✅ 历史记录（对话/章节/选择事件）
- ✅ 运行配置文件（`config.json`）

### 待实现

- ⏳ 脚本变量系统（数字/字符串/布尔）
- ⏳ 条件分支（`if/elseif/else`）
- ⏳ 循环（`while`）
- ⏳ 立绘动画（淡入/淡出、移动、缩放）
- ⏳ 对话框样式增强
- ⏳ 跨平台支持（Linux、macOS）

## 开发文档

- [PLAN.md](PLAN.md) - 架构设计文档
- [ROADMAP.md](ROADMAP.md) - 开发路线图
- [docs/navigation_map.md](docs/navigation_map.md) - 仓库导航地图（从哪里改起/常见改动索引）

### 用户文档

- [docs/script_syntax_spec.md](docs/script_syntax_spec.md) - 脚本语法规范
- [docs/save_format.md](docs/save_format.md) - 存档格式说明
- [docs/manifest_guide.md](docs/manifest_guide.md) - 立绘布局配置说明
- [docs/resource_management.md](docs/resource_management.md) - 资源管理系统使用指南（动态加载、缓存、打包）

## License

MIT
