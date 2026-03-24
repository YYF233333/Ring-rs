# RFC: 统一 Game Mode 框架

## 元信息

- 编号：RFC-026
- 状态：Proposed
- 作者：claude-4.6-opus
- 日期：2026-03-24
- 影响范围：`host`（ui_modes / game_mode / app / host_app）
- 前置：RFC-025（共享服务层提取）
- 父 RFC：RFC-024（VN+ Hub 架构愿景）

---

## 背景

当前 Host 中存在两套并行的"模态"机制：

1. **`ui_modes`（RFC-022）**：`UiModeHandler` trait + `UiModeRegistry`，用于 `show_map` 等 egui 渲染的 UI 模态。生命周期简单（activate → render 循环 → complete/cancel），在 egui 帧内渲染。

2. **`game_mode`**：`GameMode` + `BridgeServer` + WebView，用于 `call_game` 小游戏。生命周期复杂（启动 HTTP 服务器 → 创建 WebView → 异步通信 → 关闭），跨越多帧且涉及外部进程。

两套机制在 `app/update/script.rs` 中通过 if-else 分发：

```rust
if mode == "call_game" {
    // WebView 路径
} else {
    // UiModeRegistry 路径
}
```

随着 Hub 架构演进，还将出现第三类模态：**原生玩法 mode**（Rust 框架 + 脚本逻辑，RFC-027），它比 `UiModeHandler` 重（有自己的 update 循环和状态持久化），但比 `game_mode` 轻（不需要 WebView/HTTP）。

需要一个统一的 `GameMode` trait 将三者收敛，避免 if-else 持续膨胀。

**实施时机：等待第一个非 VN 项目进入预生产。当前仅作架构预案。**

---

## 目标与非目标

### 目标

- 定义统一的 `GameMode` trait，覆盖 UI 模态、WebView 模态、原生玩法模态三种场景
- 统一模态调度：进入/退出/每帧更新/渲染 通过同一个 registry 管理
- 明确叙事 ↔ 模态的数据传递协议（参数传入 + 结果回传）
- VN 叙事保持为"默认态"，不强制封装为 mode（避免过度抽象）

### 非目标

- 多模态并行（同时只有一个非 VN 模态活跃）
- 第三方 mode 插件加载（仅引擎内建）
- 模态间直接调用（卡牌 mode 不应调用战棋 mode）
- 现在就实施——本 RFC 是预案，等真实需求触发

---

## 方案草案

### GameMode trait

```rust
pub trait GameMode: std::fmt::Debug + Send {
    /// 模态标识符，与脚本中 `call_mode "xxx"` 匹配
    fn mode_id(&self) -> &str;

    /// 激活模态
    ///
    /// `params` 是脚本传入的参数（从 RuntimeState.variables 解析）。
    /// `services` 提供共享服务访问（音频/资源/存档等）。
    fn activate(
        &mut self,
        key: String,
        params: HashMap<String, VarValue>,
        services: &SharedServices,
    ) -> Result<(), GameModeError>;

    /// 每帧更新（逻辑）
    ///
    /// 返回 Running 或 Completed(result)。
    /// 对于 UI-only 的 mode（原 UiModeHandler），可在 render 中处理逻辑。
    fn update(&mut self, dt: f32, services: &mut SharedServices) -> GameModeStatus;

    /// 每帧渲染
    ///
    /// 提供 egui context 和 GPU 渲染能力。
    /// 简单 mode 只用 egui，复杂 mode 可同时使用 wgpu sprite 渲染。
    fn render(&mut self, ctx: &RenderContext);

    /// 模态结束后清理
    fn deactivate(&mut self);

    /// 是否需要独占渲染（隐藏 VN 画面）
    ///
    /// 默认 true。overlay 型 mode（如 show_map）可返回 false。
    fn is_fullscreen(&self) -> bool { true }
}
```

### 与现有机制的关系

| 现有机制 | 迁移方案 |
|---------|---------|
| `UiModeHandler`（show_map 等） | 适配为 `GameMode` 实现，`update` 为空，逻辑在 `render` 中完成 |
| `game_mode`（WebView） | 适配为 `GameMode` 实现，`activate` 启动 Bridge/WebView，`update` 轮询完成状态 |
| 原生玩法 mode（未来） | 直接实现 `GameMode`，`update` 驱动状态机，`render` 渲染游戏画面 |

### 调度器

```rust
pub struct GameModeDispatcher {
    modes: HashMap<String, Box<dyn GameMode>>,
    active: Option<String>,
    active_key: Option<String>,
}

impl GameModeDispatcher {
    /// 注册 mode
    pub fn register(&mut self, mode: Box<dyn GameMode>);

    /// 从脚本 call_mode 激活
    pub fn activate(&mut self, mode_id: &str, key: String, params: HashMap<String, VarValue>, services: &SharedServices) -> Result<(), GameModeError>;

    /// 每帧调用
    pub fn tick(&mut self, dt: f32, services: &mut SharedServices) -> Option<GameModeResult>;

    /// 渲染活跃 mode
    pub fn render(&mut self, ctx: &RenderContext);
}
```

### VN 叙事的定位

VN 不封装为 GameMode。原因：

1. VN 叙事由 vn-runtime 驱动，有自己的 tick 循环和 Command 管线，与 GameMode 的 update/render 模型不匹配
2. VN 是"默认态"，GameMode 是"中断态"——进入 mode 时暂停 VN tick，退出时恢复
3. 强行将 VN 封装为 mode 会引入大量适配代码，收益不明

VN 和 GameMode 的关系是：**VN 是主循环，GameMode 是插入式中断。**

---

## 待解决问题（实施前需回答）

1. **SharedServices 的精确接口**：需要从 RFC-025 的成果中确定哪些服务暴露给 mode
2. **RenderContext 的设计**：mode 可以用 egui、也可以用 wgpu sprite 渲染——如何统一提供？
3. **模态持久化状态**：如果玩家在 mode 中途存档，mode 状态怎么序列化？通过 SaveData.mode_data（RFC-025）还是 mode 自行管理？
4. **脚本语法**：`call_mode` 是新增语法还是复用 `requestUI`？参数传递格式？
5. **UiModeHandler 的迁移时机**：是否在 RFC-026 中一并迁移，还是先共存再逐步替换？

这些问题需要在第一个具体 mode 需求明确后，结合实际约束回答。

---

## 验收标准

- [ ] `GameMode` trait 定义完成
- [ ] `GameModeDispatcher` 实现注册/激活/tick/render 生命周期
- [ ] 现有 `UiModeHandler`（show_map）迁移为 `GameMode` 实现
- [ ] 现有 `game_mode`（WebView）迁移为 `GameMode` 实现
- [ ] `app/update/script.rs` 中 mode 分发收敛为统一调度
- [ ] 至少一个原生玩法 mode 原型通过 trait 接入
- [ ] 存档中 mode 状态可序列化/恢复
- [ ] `cargo check-all` 通过
- [ ] 模块摘要更新
