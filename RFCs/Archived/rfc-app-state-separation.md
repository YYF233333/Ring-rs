# RFC: AppStateInner 关注点分离

## Meta

- Number: RFC-035
- Status: Accepted
- Author: Ring-rs 开发组
- Date: 2026-04-14
- Scope: `host-dioxus/src/state/`
- Prerequisites: 无

---

## Background

`AppStateInner`（`host-dioxus/src/state/mod.rs`）当前有 21 个字段，横跨动画计时、游戏核心状态、session 管理、播放控制等多个关注点。字段访问矩阵分析显示存在明确的内聚分组，部分字段组仅在 1-2 个模块中被使用。

当前的平铺结构导致：
- 阅读代码时难以快速识别哪些字段属于哪个子系统
- `tick.rs` 中的动画推进逻辑与 `interaction.rs` 中的用户交互逻辑共享同一个扁平命名空间
- Session 管理（`client_owner`、`next_client_id`）与游戏逻辑混在一起，但这两组字段在访问模式上完全隔离

---

## Goals & Non-Goals

### Goals

- 将 AppStateInner 的 21 个字段按职责分组为子结构，提升代码可读性
- 保持行为完全不变（纯重构，无功能变更）
- 保持测试通过，无需修改测试逻辑

### Non-Goals

- 不改变 AppStateInner 的生命周期或所有权模型（仍是 `Arc<Mutex<AppStateInner>>`）
- 不拆分到独立的 Mutex（避免增加锁复杂度）
- 不引入 trait 抽象或泛型
- 不涉及 RenderState 的内部重构（另见 P2-2）

---

## Design

### 字段访问矩阵（摘要）

| 字段组 | tick.rs | interaction.rs | game_lifecycle.rs | save_load.rs | session.rs |
|--------|---------|---------------|-------------------|-------------|-----------|
| 动画计时器 (4) | 读写 | 读写(reset) | 写(reset) | — | — |
| Session (2) | — | — | — | — | 读写 |
| 播放控制 (2) | 读写 | 读写 | 写(reset) | — | — |
| 核心游戏状态 (13) | 读写 | 读写 | 读写 | 读写 | 读 |

### 分组方案

提取 2 个子结构，保留核心状态在 AppStateInner 中：

```rust
/// 动画/过渡计时器，仅在 tick 和 interaction 中使用
pub struct AnimationTimers {
    pub bg_transition_elapsed: f32,
    pub scene_transition_elapsed: f32,
    pub active_shake: Option<ShakeAnimation>,
    pub scene_effect_active: bool,
}

/// Debug session 管理，完全隔离在 session.rs 中
pub struct SessionAuthority {
    pub client_owner: Option<SessionOwner>,
    pub next_client_id: u64,
}

pub struct AppStateInner {
    // ── 核心游戏状态 ──
    pub runtime: Option<vn_runtime::VNRuntime>,
    pub command_executor: CommandExecutor,
    pub render_state: RenderState,
    pub host_screen: HostScreen,
    pub waiting: WaitingFor,
    pub script_finished: bool,
    pub services: Option<Services>,
    pub history: Vec<HistoryEntry>,
    pub user_settings: UserSettings,
    pub persistent_store: PersistentStore,
    pub snapshot_stack: SnapshotStack,

    // ── 播放控制 ──
    pub playback_mode: PlaybackMode,
    pub auto_timer: f32,
    pub typewriter_timer: f32,
    pub text_speed: f32,

    // ── 子结构 ──
    pub anim: AnimationTimers,
    pub session: SessionAuthority,
}
```

### 为什么不提取 PlaybackControl 子结构

`playback_mode`、`auto_timer`、`typewriter_timer`、`text_speed` 在 `tick.rs` 和 `interaction.rs` 中与核心状态（`waiting`、`render_state`）高度交叉引用。提取为子结构会导致大量 `self.playback.auto_timer` vs `self.waiting` 的跨结构访问，增加代码噪声而无真正收益。保留在 AppStateInner 顶层，用注释分组即可。

### 变更影响范围

| 文件 | 变更类型 |
|------|---------|
| `state/mod.rs` | 结构体定义变更 + AnimationTimers/SessionAuthority 定义 |
| `state/tick.rs` | `self.bg_transition_elapsed` → `self.anim.bg_transition_elapsed` 等（约 16 处） |
| `state/interaction.rs` | `self.scene_effect_active` → `self.anim.scene_effect_active` 等（约 7 处） |
| `state/game_lifecycle.rs` | 动画字段 reset 改用 `self.anim = AnimationTimers::default()` |
| `state/session.rs` | `self.client_owner` → `self.session.client_owner` 等（约 4 处） |
| `state/tests.rs` | 构造函数更新 |

### 迁移方式

纯机械替换 + `Default` derive。Rust 编译器会捕获所有遗漏的字段访问路径。不需要分阶段迁移。

---

## Impact

| Module | Change | Risk |
|--------|--------|------|
| `state/mod.rs` | 结构体拆分 | 低 |
| `state/tick.rs` | 字段路径重写（16 处） | 低（编译器保证） |
| `state/interaction.rs` | 字段路径重写（7 处） | 低 |
| `state/game_lifecycle.rs` | Reset 逻辑简化 | 低 |
| `state/session.rs` | 字段路径重写（4 处） | 低 |
| `state/tests.rs` | 构造更新 | 低 |

---

## Migration Plan

1. 定义 `AnimationTimers` 和 `SessionAuthority`，derive `Default`
2. 在 `AppStateInner` 中替换字段为子结构
3. 批量替换所有 `self.field` 路径（编译器驱动）
4. `game_lifecycle.rs` 的 reset 逻辑改为 `self.anim = AnimationTimers::default()`
5. `cargo check-all` 验证

---

## Acceptance Criteria

- [ ] `AnimationTimers` 和 `SessionAuthority` 子结构定义完毕
- [ ] AppStateInner 字段重组完毕
- [ ] 所有 `state/` 模块中的字段访问路径更新
- [ ] `cargo check-all` 通过（fmt + clippy + 全部 398 测试）
- [ ] 无功能变更（纯重构）
