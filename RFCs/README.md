# RFC 索引

本仓库使用 RFC 流程制定中长期计划与跨模块方案。  
提案入口为 `RFCs/`，不再使用 `ROADMAP.md`。

相关文档入口：

- 文档总首页：[`docs/README.md`](../docs/README.md)
- 架构约束：[`ARCH.md`](../ARCH.md)
- 贡献指南：[`CONTRIBUTING.md`](../CONTRIBUTING.md)

## 使用约定

- 新提案：新增 `rfc-<topic>.md`，状态为 `Proposed`
- 实施中：标记为 `Active`
- 已落地：标记为 `Accepted`，移动到 `Accepted/` 子目录
- 「文件」列只填写文件名，不填写路径前缀
- 状态为 `Accepted` 时，默认到 `RFCs/Accepted/` 查找该文件；其余状态默认在 `RFCs/` 当前目录
- RFC 文件迁移、重命名或状态变更时，同步更新本文档
- RFC 编号按“提出顺序”递增分配，格式为 `RFC-XXX`（三位数）

## 当前 RFC

| RFC 编号 | 名称 | 文件 | 状态 |
|---|---|---|---|
| RFC-001 | 对话语音标注与自动播放管线 | `rfc-dialogue-voice-pipeline.md` | Proposed |
| RFC-002 | ref-project 重制体验等价计划 | `rfc-remake-experience-equivalence.md` | Active |
| RFC-003 | `show` 语义收敛与人体工学优先 | `rfc-show-unification-ergonomics.md` | Accepted |
| RFC-004 | 扩展 API 与 Mod 化效果管理 | `rfc-extension-api-mod-effect-management.md` | Active |
| RFC-005 | 对话内联差分语法 | `rfc-inline-sprite-in-dialogue.md` | Proposed |
| RFC-006 | 节奏标签与 extend 台词续接 | `rfc-rhythm-tags.md` | Accepted |
| RFC-007 | 渲染后端迁移 (macroquad → winit+wgpu+egui) | `rfc-rendering-backend-migration.md` | Accepted |
| RFC-008 | RenderBackend Trait -- 渲染后端抽象层 | `rfc-render-backend-trait.md` | Accepted |
| RFC-009 | Cutscene 视频播放 | `rfc-cutscene-video-playback.md` | Accepted |
| RFC-010 | 可定制 UI 系统 | `rfc-customizable-ui-system.md` | Accepted |
| RFC-011 | UI 系统后续增强 | `rfc-ui-enhancements.md` | Accepted |
| RFC-012 | UI 行为定制系统 | `rfc-ui-behavior-customization.md` | Accepted |
| RFC-013 | 配置默认值外部化 | `rfc-config-externalization.md` | Accepted |
| RFC-014 | 测试分层与维护策略 | `rfc-test-tiering.md` | Accepted |
| RFC-015 | 调试状态快照热键 | `rfc-debug-state-snapshot.md` | Proposed |
| RFC-016 | 输入录制与 AI 自动调试管线 | `rfc-input-recording-replay.md` | Accepted |
| RFC-017 | 调试覆盖层 | `rfc-debug-overlay.md` | Proposed |
| RFC-018 | 结构化事件流调试基础设施 | `rfc-structured-event-stream.md` | Accepted |
| RFC-019 | Headless 测试模式 | `rfc-headless-testing-mode.md` | Accepted |
| RFC-020 | 双向 UI-Script 通信协议 | `rfc-bidirectional-ui-script-communication.md` | Accepted |
| RFC-021 | WebView 小游戏集成 | `rfc-webview-minigame-integration.md` | Superseded（由 RFC-024 VN+ Hub 等后续方案取代） |
| RFC-022 | UI Mode Plugin System | `rfc-ui-mode-plugin-system.md` | Accepted |
| RFC-023 | HTTP Bridge API | `rfc-http-bridge-api.md` | Superseded（由 RFC-024 宿主/桥接路线取代） |
| RFC-024 | VN+ Hub 架构愿景 | `rfc-vnplus-hub-architecture.md` | Active |
| RFC-025 | 共享服务层提取 | `rfc-shared-services-extraction.md` | Accepted |
| RFC-026 | 统一 Game Mode 框架 | `rfc-unified-game-mode-framework.md` | Superseded（将基于 Tauri 重新设计） |
| RFC-027 | 玩法脚本层集成 | `rfc-gameplay-scripting-layer.md` | Proposed |
| RFC-028 | 脚本预览编辑器 | `rfc-script-preview-editor.md` | Accepted |
| RFC-029 | 前端媒体统一——动画模型收敛与音频前端化 | `rfc-frontend-media-unification.md` | Accepted |
| RFC-030 | Tauri 前端 UI 主题与客制化架构 | `rfc-tauri-ui-theming.md` | Proposed |
| RFC-031 | ZIP 资源自定义协议 | `rfc-zip-asset-protocol.md` | Active |
| RFC-032 | host-tauri Harness 能力对齐 | `rfc-host-tauri-harness-parity.md` | Proposed |