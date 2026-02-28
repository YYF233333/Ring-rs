# host/renderer/render_state 摘要

## Purpose

`renderer/render_state` 定义 Host 渲染态数据模型，集中表达当前背景、角色、对话、章节标记、选项和 UI 可见性。

## PublicSurface

- 模块入口：`host/src/renderer/render_state/mod.rs`
- 核心类型：`RenderState`、`CharacterSprite`、`DialogueState`、`ChoicesState`、`ChoiceItem`
- 关键接口：背景/角色/对话/选择状态读写与打字机推进方法

## KeyFlow

1. `CommandExecutor` 更新 `RenderState`（背景、角色、文本、选项等）。
2. `app/update` 每帧推进章节标记和角色淡出回收。
3. `Renderer` 读取 `RenderState` 进行无副作用渲染。

## Dependencies

- 依赖 `vn_runtime::command::Position` 表达角色站位
- 依赖 `character_animation::AnimatableCharacter` 持有角色动画属性

## Invariants

- `RenderState` 只保存可渲染状态，不耦合输入或脚本执行器逻辑。
- 打字机与章节标记状态推进必须可在帧级迭代。

## FailureModes

- 角色淡出标记与移除流程不一致，导致残留或提前消失。
- 选择状态索引越界，导致输入处理与渲染不一致。

## WhenToReadSource

- 需要新增渲染实体或 UI 状态字段时。
- 需要排查“命令已执行但渲染结果不符合预期”问题时。

## RelatedDocs

- [renderer 摘要](renderer.md)
- [command_executor 摘要](command_executor.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
