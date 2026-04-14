# RFC: RenderState Signal 细粒度优化

## Meta

- Number: RFC-036
- Status: Accepted
- Author: Ring-rs 开发组
- Date: 2026-04-14
- Scope: `host-dioxus/src/main.rs`, `host-dioxus/src/vn/`, `host-dioxus/src/render_state.rs`
- Prerequisites: 无（可独立于 RFC-035 实施）

---

## Background

当前 host-dioxus 使用单个 `Signal<RenderState>` 驱动所有 VN 组件渲染。tick loop（~30 FPS）每帧执行 `render_state.set(inner.render_state.clone())`，导致所有 18 个订阅该 Signal 的组件每帧都被通知重新检查渲染。

组件→字段依赖分析显示大量浪费：

| 组件 | 依赖字段数 | RenderState 总字段 | 无关更新比例 |
|------|-----------|-------------------|------------|
| ChoicePanel | 1 (choices) | 17 | 94% |
| CharacterLayer | 1 (visible_characters) | 17 | 94% |
| AudioBridge | 1 (audio) | 17 | 94% |
| SkipIndicator | 1 (playback_mode) | 17 | 94% |
| DialogueBox | 3 (text_mode, ui_visible, dialogue) | 17 | 82% |

---

## Goals & Non-Goals

### Goals

- 减少每帧触发的不必要组件重渲染检查
- 保持行为完全不变
- 允许渐进式迁移（不需要一次性改完所有组件）

### Non-Goals

- 不改变 AppStateInner 或 RenderState 的结构
- 不引入外部状态管理库
- 不做性能基准测试框架（本 RFC 仅做结构优化）

---

## Design

### 方案对比

| 方案 | 复杂度 | 收益 | 渐进性 |
|------|--------|------|--------|
| A: 组件内 `use_memo` | 低 | 中（减少 DOM diff，不减少通知） | 可逐组件迁移 |
| B: 拆分为多个 Signal | 高 | 高（减少通知 + DOM diff） | 需要修改 tick loop + 所有组件 |

### 推荐方案：A（use_memo）优先，B 作为后续

方案 A 以最低成本获得主要收益，且可以逐个组件迁移。方案 B 在性能分析证明 Signal 通知本身是瓶颈后再考虑。

### 方案 A 实现

在每个组件内使用 `use_memo` 提取所需数据，使组件仅在实际依赖数据变化时重新渲染：

```rust
// 当前
#[component]
pub fn ChoicePanel(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();
    let choices = match &rs.choices { ... };
}

// 优化后
#[component]
pub fn ChoicePanel(render_state: Signal<RenderState>) -> Element {
    let choices = use_memo(move || render_state.read().choices.clone());
    let choices = choices.read();
    // 仅当 choices 实际变化时才重新渲染
}
```

### 迁移优先级

按"更新频率 × 渲染成本"排序，优先迁移高收益组件：

| 优先级 | 组件 | 理由 |
|--------|------|------|
| P0 | CharacterLayer | 多个精灵的 transform 计算成本高 |
| P0 | AudioBridge | 每帧检查但几乎不变 |
| P1 | ChoicePanel | 仅在选择支出现时有意义 |
| P1 | VideoOverlay | 仅在播放视频时有意义 |
| P1 | TransitionOverlay / RuleTransitionCanvas | 仅在过渡期间有意义 |
| P2 | DialogueBox | 打字机每帧变化，但 memo 粒度需仔细选择 |
| P2 | 其余组件 | 收益较小 |

### 方案 B 预留设计（如需要）

如方案 A 不足，可进一步拆分为 7 个 Signal：

1. **scene_state**: host_screen, text_mode, ui_visible
2. **visual_state**: current_background, background_transition, visible_characters, title_card, chapter_mark
3. **dialogue_state**: dialogue, nvl_entries
4. **choice_state**: choices
5. **transition_state**: scene_transition, scene_effect
6. **audio_state**: audio
7. **playback_ui_state**: playback_mode, active_ui_mode, cutscene

tick loop 改为分别更新 7 个 Signal。组件 Props 改为接收所需的 Signal 子集。

---

## Impact

| Module | Change | Risk |
|--------|--------|------|
| `vn/*.rs` (18 组件) | 添加 `use_memo` 包装 | 低（渐进式，每次改一个） |
| `main.rs` | 方案 A 无变更 | 无 |
| `render_state.rs` | 方案 A 无变更 | 无 |

---

## Migration Plan

1. 选择 1-2 个 P0 组件（CharacterLayer、AudioBridge）添加 `use_memo`
2. 验证行为不变 + 使用 debug_server 截图对比
3. 逐步迁移剩余组件
4. 如需方案 B，在全部方案 A 完成后评估

---

## Acceptance Criteria

- [ ] P0 组件（CharacterLayer、AudioBridge）完成 `use_memo` 迁移
- [ ] P1 组件完成迁移
- [ ] `cargo check-all` 通过
- [ ] 通过 debug_server 截图验证渲染结果无变化
- [ ] 无功能变更（纯优化）
