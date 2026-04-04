# host-tauri（已归档）

> **状态：已归档** — 本目录保留为参考实现，不再参与构建或接受新功能开发。
> 替代方案见 [RFC-033: Dioxus 宿主迁移](../RFCs/rfc-dioxus-host-migration.md) 和 [`host-dioxus/`](../host-dioxus/)。

## 背景

host-tauri 是基于 Tauri 2 + Vue 3 + TypeScript 的视觉小说宿主实现，作为从旧 host（winit/wgpu/egui）迁移的中间方案。开发期间暴露了三个结构性问题：

1. **双语言工具链摩擦**：Rust + TypeScript/Vue 需要 cargo + pnpm + vite + biome + vue-tsc 五套工具链
2. **IPC 边界成本**：Tauri invoke 机制要求所有前后端交互经过 JSON 序列化，`render_state.rs` 与 `render-state.ts` 需手动同步
3. **调试架构矛盾**：debug_server 与 WebView 形成双客户端竞争

根本原因：Tauri 架构将 Rust 后端与 WebView 前端隔离为独立进程/线程，而本项目前端是纯渲染层（不持有游戏状态），该隔离只有成本没有收益。

## 源码结构

### Rust 后端 (`src-tauri/src/`)

| 文件 | 职责 |
|------|------|
| `lib.rs` | Tauri Builder 初始化、模块注册 |
| `state.rs` | AppState + tick/click/choose 逻辑 |
| `commands.rs` | 34 个 `#[command]` IPC 薄代理 |
| `command_executor.rs` | Runtime Command -> RenderState 翻译 |
| `render_state.rs` | 可序列化渲染状态（JSON -> 前端） |
| `audio.rs` | rodio 音频播放 |
| `resources.rs` | ResourceManager + LogicalPath |
| `config.rs` | AppConfig 加载 |
| `save_manager.rs` | 存档槽位管理 |
| `debug_server.rs` | 调试 HTTP 服务器 |
| `manifest.rs` | 游戏清单解析 |

### Vue 前端 (`src/`)

| 目录/文件 | 职责 |
|-----------|------|
| `App.vue` | 根组件，后端调用入口 |
| `vn/` | VN 渲染组件（BackgroundLayer, CharacterLayer, DialogueBox 等） |
| `screens/` | 系统 UI 页面（Title, SaveLoad, Settings, History, InGameMenu） |
| `composables/` | 可复用逻辑（useEngine, useAssets, useAudio 等） |
| `types/render-state.ts` | Rust RenderState 的 TypeScript 镜像 |

## 归档文档

- [开发指南](docs/dev-guide.md)
- [调试策略](docs/debugging.md)
- [学习路线](docs/learning-roadmap.md)
- [模块总览](docs/module-summary.md)

## 相关 RFC

- RFC-033: Dioxus 宿主迁移（Active，取代本方案）
- RFC-030: Tauri UI 主题（Superseded）
- RFC-032: host-tauri Harness 能力对齐（Superseded）
