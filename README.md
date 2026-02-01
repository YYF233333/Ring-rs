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
│   │   └── command_executor/  # 命令执行器
│   └── Cargo.toml
├── assets/              # 游戏资源
│   ├── backgrounds/     # 背景图片
│   ├── characters/      # 角色立绘
│   ├── bgm/             # 背景音乐
│   ├── fonts/           # 字体文件
│   └── scripts/         # 脚本文件
└── docs/                # 文档
    ├── script_syntax_spec.md   # 脚本语法规范
    └── script_language_showcase.md
```

## 快速开始

### 环境要求

- Rust 1.70+
- 支持 OpenGL 的图形环境

### 运行

```bash
cd host
cargo run
```

### 操作说明

| 按键 | 功能 |
|------|------|
| **空格/鼠标点击** | 前进对话 |
| **↑↓ 或鼠标悬停** | 选择分支时切换选项 |
| **回车/鼠标点击选项** | 确认选择 |
| **F1** | 显示/隐藏调试信息 |
| **F2** | 切换到命令演示模式 |
| **F3** | 切换到脚本模式 |
| **F4** | 切换脚本文件 |
| **M** | 静音/取消静音 |
| **ESC** | 退出 |

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

### 待实现

- ⏳ `rule` 遮罩过渡效果
- ⏳ 对话框样式增强
- ⏳ 存档/读档系统
- ⏳ 历史对话回看

## 开发文档

- [PLAN.md](PLAN.md) - 架构设计文档
- [ROADMAP.md](ROADMAP.md) - 开发路线图
- [docs/script_syntax_spec.md](docs/script_syntax_spec.md) - 脚本语法规范

## License

MIT
