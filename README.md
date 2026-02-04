# Ring-rs

一个使用 Rust 构建的视觉小说引擎（Visual Novel Engine）。

如果你是来 **做内容（脚本/素材/配置）** 的：你不需要改仓库代码。

## Getting Started（内容作者）

- **从零开始做内容**：[docs/getting_started.md](docs/getting_started.md)
- **脚本语法规范**：[docs/script_syntax_spec.md](docs/script_syntax_spec.md)
- **资源/打包（fs/zip）**：[docs/resource_management.md](docs/resource_management.md)
- **立绘布局（manifest）**：[docs/manifest_guide.md](docs/manifest_guide.md)

## 快速开始（运行引擎，测试你的内容）

### 环境要求

- Rust **stable**（本仓库使用 `edition = "2024"`，建议使用较新的 toolchain）
- 支持 OpenGL 的图形环境

### 配置 `config.json`

将仓库根目录下的`config.json`中的`start_script_path`修改为实际入口脚本的路径（**相对于 `assets_root` 的路径**）：

```json
{
  "assets_root": "assets",
  "start_script_path": "scripts/main.md"
}
```

其余配置详见 [docs/config_guide.md](docs/config_guide.md)

### 运行

在仓库根目录运行：

```bash
cargo run
```

> Windows PowerShell 不支持 `&&`，需要用 `;`。

## 内容制作闭环（检查 → 运行 → 打包发布）

### 1) 静态检查脚本（推荐每次改动都跑）

```bash
cargo script-check
```

也可以检查单个文件或目录：

```bash
cargo script-check assets/scripts/main.md
```

检查内容：语法错误、未定义 label（`goto`/choice 目标）、资源引用是否存在（背景/立绘/音频）。

### 2) 开发模式运行（资源来自文件系统）

```bash
cargo run
```

开发模式默认使用 `asset_source: "fs"`（见 [docs/resource_management.md](docs/resource_management.md)）。

### 3) 打包发布（生成 dist/ 发行版）

```bash
cargo pack release --zip
```

该命令会：

- 打包 `assets/` 为 `game.zip`
- 编译 `host` release
- 生成 `dist/` 发行版目录（包含 `*.exe`/`game.zip`/`config.json`）
- 可选将整个发行版目录再打成 `GameName.zip`

## 文档入口

- **内容制作入门**：[docs/getting_started.md](docs/getting_started.md)
- **运行配置（config.json）**：[docs/config_guide.md](docs/config_guide.md)
- **脚本规范**：[docs/script_syntax_spec.md](docs/script_syntax_spec.md)
- **脚本示例集**：[docs/script_language_showcase.md](docs/script_language_showcase.md)
- **资源系统与打包**：[docs/resource_management.md](docs/resource_management.md)
- **存档格式**：[docs/save_format.md](docs/save_format.md)
- **仓库导航地图（改引擎时看）**：[docs/navigation_map.md](docs/navigation_map.md)
- **路线图**：[ROADMAP.md](ROADMAP.md)

## 仓库结构（Workspace）

```
Ring-rs/
├── vn-runtime/          # 纯逻辑 Runtime：脚本解析/执行/状态/等待/存档（不依赖引擎与 IO）
├── host/                # 宿主：渲染/音频/输入/资源（执行 Runtime 产出的 Command）
├── tools/xtask/         # 本地门禁/覆盖率/脚本检查（cargo alias：check-all/script-check/...）
├── tools/asset-packer/  # 资源打包/发行版生成（cargo alias：pack）
├── assets/              # 游戏资源（scripts/backgrounds/characters/bgm/fonts/manifest.json...）
├── config.json          # 运行配置（入口脚本、资源来源、窗口等）
└── docs/                # 规范与指南
```

## 架构概要（Runtime/Host 分离）

- **`vn-runtime`**：只做确定性逻辑推进，**只产出** `Command`，不做渲染/音频/IO
- **`host`**：只消费 `Command` 产生画面/音频/UI，输入以 `RuntimeInput` 回传 Runtime

## 贡献与开发

提交前建议跑：`cargo check-all`（fmt → clippy → tests）。更多见：[CONTRIBUTING.md](CONTRIBUTING.md)。
