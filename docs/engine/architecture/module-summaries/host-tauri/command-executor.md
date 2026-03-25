# host-tauri/command_executor

> LastVerified: 2026-03-26
> Owner: Claude

## 职责

将 vn-runtime 的 `Command` 翻译为 `RenderState` 变更，并将音频副作用记录到 `CommandOutput`。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `CommandExecutor` | 命令执行器，持有 `last_output` |
| `ExecuteResult` | 执行结果枚举：Ok / WaitForClick / WaitForChoice / WaitForTime / WaitForCutscene / Error |
| `AudioCommand` | 音频副作用枚举：PlayBgm / StopBgm / BgmDuck / BgmUnduck / PlaySfx |
| `CommandOutput` | 单次执行输出（result + audio_command） |
| `TransitionKind` | 过渡效果分类（内部）：None / Dissolve / Fade / FadeWhite / Move / Rule |

## 数据流

```
Command (from vn-runtime)
  │
  ▼
CommandExecutor::execute(cmd, &mut render_state, manifest)
  │
  ├─ 视觉命令 → 直接修改 RenderState 字段
  │   ├─ ShowBackground/ChangeScene → set_background + 过渡状态
  │   ├─ ShowCharacter/HideCharacter → visible_characters 增删改（`execute()` 接受 `&Manifest`，将 `Position` 枚举解析为归一化坐标并写入 `CharacterSprite`）
  │   ├─ ShowText/ExtendText → start_typewriter / extend_dialogue
  │   ├─ PresentChoices → set_choices
  │   ├─ ChapterMark/TitleCard → 设置标记状态
  │   ├─ TextBox{Hide,Show,Clear} → ui_visible / clear_dialogue
  │   ├─ ClearCharacters → hide_all_characters
  │   └─ SetTextMode → text_mode + 清理 NVL 条目
  │
  └─ 音频命令 → 记录到 last_output.audio_command
      ├─ PlayBgm / StopBgm / PlaySfx
      └─ BgmDuck / BgmUnduck
```

### 过渡效果处理

`resolve_transition()` 解析 `Transition` 为 `(TransitionKind, duration)`：

| 过渡名 | 效果类型 | 默认时长 |
|--------|---------|---------|
| `dissolve` | Dissolve | 0.3s |
| `fade` | Fade | 0.5s |
| `fadewhite` | FadeWhite | 0.5s |
| `move` / `slide` | Move | 0.3s |
| `rule` | Rule(mask_path, reversed) | 0.5s |
| `none` | None | 0.0s |
| 其他 | 回退为 Dissolve | 0.3s |

- `ShowBackground`：Dissolve/Move → 设置 `background_transition`；其他 → 直接切换
- `ChangeScene`：Dissolve/Move → `background_transition`；Fade/FadeWhite/Rule → `scene_transition`（多阶段状态机）

## 关键不变量

- `execute()` 每次先重置 `last_output`，避免跨命令泄漏
- 执行器只做状态翻译，不直接播放音频——AudioCommand 由 `state.rs` 的 `dispatch_audio_command` 消费
- `execute_batch()` 顺序执行所有命令，返回最后一个非 Ok 的 ExecuteResult
- `Cutscene` 命令直接返回 `WaitForCutscene`，不修改 RenderState（由 state.rs 设置 cutscene 字段）
- `FullRestart` 和 `RequestUI` 返回 Ok（Tauri 宿主暂不处理）

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `state.rs` | 被持有：`AppStateInner.command_executor` |
| `render_state.rs` | 修改：execute 的主要输出目标 |
| `vn-runtime::Command` | 输入：execute 的命令来源 |
| `audio.rs` | 间接：通过 AudioCommand 中转 |
