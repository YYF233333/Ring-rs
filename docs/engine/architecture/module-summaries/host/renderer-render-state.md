# host/renderer/render_state 摘要

## Purpose

`renderer/render_state` 是 Host 侧的可渲染快照：背景、角色、对话、选项、章节标记、场景效果与文本模式都在这里汇合，供更新循环与 `Renderer` 共享。

## PublicSurface

- 模块入口：`host/src/renderer/render_state/mod.rs`
- 核心类型：`RenderState`、`CharacterSprite`、`DialogueState`、`ChoicesState`、`ChoiceItem`
- 对话附属类型：`InlineWait`、`EffectiveCps`、`NvlEntry`
- 常用接口：打字机推进、对话续接、内联等待更新、有效字速查询

## KeyFlow

1. `CommandExecutor` 写入背景、角色、对话与选项等状态。
2. `app/update` 逐帧推进打字机、章节标记和淡出回收。
3. `Renderer` 只读消费这些字段生成绘制命令。

## Dependencies

- 依赖 `vn_runtime::command::Position` 表达角色站位
- 依赖 `character_animation::AnimatableCharacter` 持有角色动画属性

## Invariants

- `RenderState` 只保存可渲染状态，不耦合输入或脚本执行器逻辑。
- 打字机与章节标记状态推进必须可在帧级迭代。
- `DialogueState` 扩展字段：`inline_effects`（位置索引内联效果）、`no_wait`（自动推进）、`inline_wait`（当前内联等待状态）、`effective_cps`（当前字速覆盖）。
- `advance_typewriter` 在推进字符时自动检测并激活对应位置的 `InlineEffect`。
- `extend_dialogue` 追加文本时自动偏移新增内联效果的位置索引。
- `text_mode: TextMode` 字段控制 ADV/NVL 渲染模式。
- `nvl_entries: Vec<NvlEntry>` 累积 NVL 对话行。

## FailureModes

- 角色淡出标记与移除流程不一致，导致残留或提前消失。
- 选择状态索引越界，导致输入处理与渲染不一致。

## WhenToReadSource

- 需要新增渲染实体或 UI 状态字段时。
- 需要排查“命令已执行但渲染结果不符合预期”问题时。

## RelatedDocs

- [renderer 摘要](renderer.md)
- [command_executor 摘要](command-executor.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4