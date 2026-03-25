# host-tauri/render_state

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

定义完整的可序列化渲染状态——通过 Tauri IPC 以 JSON 形式推送给 Vue 前端，是前后端之间的唯一数据契约。

## 关键类型/结构

### 顶层结构

| 类型 | 说明 |
|------|------|
| `RenderState` | 当前帧的完整渲染快照（14 个字段） |

### 子状态

| 类型 | 说明 |
|------|------|
| `CharacterSprite` | 角色立绘显示状态（路径、位置、alpha、过渡时长等） |
| `DialogueState` | 对话框打字机状态（speaker、content、visible_chars、inline effects） |
| `ChoicesState` / `ChoiceItem` | 选择界面状态 |
| `ChapterMarkState` / `ChapterMarkPhase` | 章节标记动画（FadeIn→Visible→FadeOut） |
| `TitleCardState` | 字卡显示（text + duration + elapsed） |
| `SceneEffectState` | 场景效果瞬时值（shake、blur、dim） |
| `BackgroundTransition` | 背景 dissolve 过渡（old_background + progress） |
| `SceneTransition` / `SceneTransitionKind` / `SceneTransitionPhaseState` | 场景遮罩过渡（Fade/FadeWhite/Rule，5 阶段状态机） |
| `CutsceneState` | 视频过场（video_path + is_playing） |
| `NvlEntry` | NVL 模式累积文本条目 |
| `PlaybackMode` | 播放模式（Normal/Auto/Skip） |
| `InlineWait` | 内联等待标记剩余时间 |
| `EffectiveCps` | 当前生效的文字速度覆盖（Absolute/Relative） |

### RenderState 字段一览

```
RenderState {
    current_background:     Option<String>,
    visible_characters:     HashMap<String, CharacterSprite>,
    dialogue:               Option<DialogueState>,
    chapter_mark:           Option<ChapterMarkState>,
    choices:                Option<ChoicesState>,
    ui_visible:             bool,
    title_card:             Option<TitleCardState>,
    scene_effect:           SceneEffectState,
    text_mode:              TextMode,           // ADV | NVL
    nvl_entries:            Vec<NvlEntry>,
    background_transition:  Option<BackgroundTransition>,
    scene_transition:       Option<SceneTransition>,
    cutscene:               Option<CutsceneState>,
    playback_mode:          PlaybackMode,
}
```

## 数据流

```
CommandExecutor::execute()          state.rs::process_tick()
  │ (写入 RenderState 字段)          │ (推进动画/打字机)
  ▼                                  ▼
RenderState ──── serde::Serialize ────→ JSON ────→ 前端 RenderState TS 类型
                                                      │
                                                      ▼
                                               Vue 组件 props 渲染
```

### 打字机方法链

- `start_typewriter()` → 设置 dialogue，visible_chars = 0
- `advance_typewriter()` → visible_chars++，触发 inline effects (wait/cps)
- `complete_typewriter()` → visible_chars = total, 清除 wait/cps
- `extend_dialogue()` → 追加文本，偏移 inline effects 位置
- `effective_text_speed()` → 根据 EffectiveCps 计算实际 CPS

### 章节标记动画

三阶段：FadeIn(0.5s) → Visible(2.0s) → FadeOut(0.5s)，`update_chapter_mark(dt)` 推进。

### 场景过渡状态机

五阶段：FadeIn → Blackout(0.2s hold) → FadeOut → UIFadeIn(0.3s) → Completed（清除）。

## TypeScript 镜像

`src/types/render-state.ts` 手动维护 Rust 结构的 TypeScript 镜像，确保前后端类型一致。

| Rust 类型 | TS 类型 | 注意 |
|-----------|---------|------|
| `Option<T>` | `T \| null` | |
| `HashMap<String, T>` | `Record<string, T>` | 加 `Readonly` 包装 |
| `enum { A, B }` | 联合字符串 `"A" \| "B"` | |
| `enum { A { field } }` | `{ A: { field } }` | Tagged union 风格 |
| `Vec<T>` | `readonly T[]` | |

## 关键不变量

- RenderState 是**前端唯一的数据源**——前端不持有独立游戏逻辑状态
- 所有字段都 `#[derive(Serialize)]`，确保可序列化
- `visible_characters` 以 alias 为 key，保证同一角色不重复
- `dialogue.visible_chars` 单调递增直到 `is_complete`
- `background_transition` 和 `scene_transition` 互斥使用不同的过渡语义
- TS 镜像**必须手动同步**——Rust 侧新增/修改字段后需同步更新 `render-state.ts`

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `command_executor.rs` | 被修改：execute 的主要输出目标 |
| `state.rs` | 被持有 + 被修改：process_tick 推进动画 |
| `commands.rs` | 被返回：大多数 IPC command 返回 RenderState |
| 前端 `types/render-state.ts` | 镜像：TypeScript 类型定义 |
| 前端 `vn/` 组件 | 消费：渲染各字段 |
