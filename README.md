# Ring-rs

一个使用 Rust 构建的视觉小说引擎。

如果你是来写脚本、准备素材、调整配置的，优先看内容作者文档；如果你是来改引擎实现的，优先看引擎开发文档。

## 快速开始

### 环境要求

- Rust **stable**（本仓库使用 `edition = "2024"`，建议使用较新的 toolchain）
- 支持 WebView2 的桌面环境（Windows 10+；macOS/Linux 需对应 WebView 运行时）

### 运行

在仓库根目录运行：

```bash
cargo run
```

Debug build 默认启用 Debug Server（HTTP REST API），可用于自动化调试与 AI 集成：

```bash
# 健康检查
curl http://127.0.0.1:9876/api/ping

# 推进对话
curl -X POST http://127.0.0.1:9876/api/click
```

Headless harness（无窗口批量执行）通过环境变量驱动，详见 [Headless 测试模式使用指南](docs/testing/headless-guide.md)。

## 文档入口

- [文档中心](docs/README.md)
- [内容作者文档](docs/authoring/README.md)
- [引擎开发文档](docs/engine/README.md)
- [测试与调试](docs/testing/README.md)
- [维护文档](docs/maintenance/README.md)
- [架构约束](ARCH.md)
- [RFC 索引](RFCs/README.md)

## 仓库结构（Workspace）

```
Ring-rs/
├── vn-runtime/          # 纯逻辑 Runtime：脚本解析/执行/状态/等待/存档（不依赖引擎与 IO）
├── host-dioxus/         # 渲染宿主：Dioxus 0.7 Desktop（执行 Runtime 产出的 Command）
├── tools/xtask/         # 本地自检与 CI 共用的门禁/覆盖率/脚本检查（cargo alias：check-all/script-check/...）
├── tools/asset-packer/  # 资源打包/发行版生成（cargo alias：pack）
├── tools/debug-mcp/     # Debug MCP Server（Node.js，封装 HTTP REST API）
├── assets/              # 游戏资源（scripts/backgrounds/characters/bgm/fonts/manifest.json...）
├── config.json          # 运行配置（入口脚本、资源来源、窗口等）
└── docs/                # 规范与指南
```

## 指标仪表盘

每次推送到 `main` 时，CI 自动采集覆盖率、构建耗时、二进制大小等指标并生成趋势图：

**[查看仪表盘](https://yyf233333.github.io/Ring-rs/)**（GitHub Pages，数据在 `gh-pages` 分支）

## 贡献与开发

本地提交前建议跑：`cargo check-all`（fmt → clippy → tests）；CI 也会执行同一门禁并在仓库变脏时失败。更多见：[CONTRIBUTING.md](CONTRIBUTING.md)。
