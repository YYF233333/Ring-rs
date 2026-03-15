# RFC 索引

本仓库使用 RFC 流程制定中长期计划与跨模块方案。  
提案入口为 `RFCs/`，不再使用 `ROADMAP.md`。

## 使用约定

- 新提案：新增 `rfc-<topic>.md`，状态为 `Proposed`
- 实施中：标记为 `Active`
- 已落地：标记为 `Accepted`，移动到Accepted子文件夹下
- RFC文件更新时同步更新本文档中目录
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
| RFC-008 | RenderBackend Trait -- 渲染后端抽象层 | `rfc-rendering-backend-trait.md` | Accepted |
| RFC-009 | Cutscene 视频播放 | `rfc-cutscene-video-playback.md` | Accepted |
| RFC-010 | 可定制 UI 系统 | `rfc-customizable-ui-system.md` | Accepted |
| RFC-011 | UI 系统后续增强 | `rfc-ui-enhancements.md` | Accepted |
| RFC-012 | UI 行为定制系统 | `rfc-ui-behavior-customization.md` | Accepted |
| RFC-013 | 配置默认值外部化 | `rfc-config-externalization.md` | Accepted |