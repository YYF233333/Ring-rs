# Ring-rs

一个使用 Rust 构建的视觉小说引擎。

如果你是来 **做内容（脚本/素材/配置）** 的：你不需要改仓库代码。

## Getting Started（内容作者）

- **从零开始做内容**：[docs/getting_started.md](docs/getting_started.md)
- **脚本语法规范**：[docs/script_syntax_spec.md](docs/script_syntax_spec.md)
- **资源/打包（fs/zip）**：[docs/resource_management.md](docs/resource_management.md)
- **立绘布局（manifest）**：[docs/manifest_guide.md](docs/manifest_guide.md)

### 环境要求

- Rust **stable**（本仓库使用 `edition = "2024"`，建议使用较新的 toolchain）
- 支持 OpenGL 的图形环境

### 运行

在仓库根目录运行：

```bash
cargo run
```

首次做内容/配置/打包发行建议按完整流程走：[docs/getting_started.md](docs/getting_started.md)（包含 `config.json` 配置、脚本检查、打包发布与自检）。

运行配置字段说明见：[docs/config_guide.md](docs/config_guide.md)。

## 文档入口

- **内容制作入门**：[docs/getting_started.md](docs/getting_started.md)
- **运行配置（config.json）**：[docs/config_guide.md](docs/config_guide.md)
- **脚本规范**：[docs/script_syntax_spec.md](docs/script_syntax_spec.md)
- **脚本示例集**：[docs/script_language_showcase.md](docs/script_language_showcase.md)
- **资源系统与打包**：[docs/resource_management.md](docs/resource_management.md)
- **存档格式**：[docs/save_format.md](docs/save_format.md)
- **仓库导航地图（改引擎时看）**：[docs/navigation_map.md](docs/navigation_map.md)
- **RFC 计划索引**：[RFCs/README.md](RFCs/README.md)

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

## 贡献与开发

提交前建议跑：`cargo check-all`（fmt → clippy → tests）。更多见：[CONTRIBUTING.md](CONTRIBUTING.md)。
