# RFC-012: UI 行为定制系统

- **状态**: Accepted
- **前置**: RFC-010 (可定制 UI 系统), RFC-011 (UI 系统后续增强)

## 背景

RFC-010/011 建立了通过 `layout.json` 定制 UI 样式（布局、颜色、字号、素材）的机制。但按钮列表、动作映射、可见性条件、背景切换等**行为**仍硬编码在 Rust 源码中。这意味着为新项目定制 UI（例如新增章节入口按钮、修改菜单项顺序、更换背景切换逻辑）必须修改引擎代码。

## 目标

- 通过声明式 JSON 配置定义各界面的按钮列表、动作映射、可见性条件和背景切换规则
- 引入 `UiRenderContext` 消除 `build_*` 函数对 `&AppState` 的直接依赖
- 泛化 `StartWinter` 为 `StartAtLabel(String)`，消除项目专用 Action
- 无 `screens.json` 时默认值与当前硬编码行为等价（向后兼容）

## 非目标

- 不覆盖复杂界面（save_load、settings、history、confirm、ingame 对话框）
- 不引入脚本引擎或表达式解析器
- 不处理 i18n 文案层

## 方案

### 配置文件

新增 `assets/ui/screens.json`，与 `layout.json` 并列。缺失时使用引擎默认值。

JSON 结构定义四个界面区域：`title`、`ingame_menu`、`quick_menu`、`game_menu`。每个区域可声明按钮列表（label + action + visible + confirm）、条件背景和 overlay。

### 动作词汇表

固定的动作 ID 映射到 `EguiAction`：`start_game`、`continue_game`、`open_load`、`open_save`、`navigate_settings`、`navigate_history`、`replace_settings`、`replace_history`（游戏菜单同级切换用）、`quick_save`、`quick_load`、`toggle_skip`、`toggle_auto`、`go_back`、`return_to_title`、`return_to_game`、`exit`、`{ "start_at_label": "X" }`。

`confirm` 字段自动包裹为 `ShowConfirm { message, on_confirm }`。

### 条件系统

极简单变量布尔判断：`$has_continue`（存在可继续存档）、`$persistent.KEY`（持久化变量为 truthy）、`!$persistent.KEY`（取反）。字段缺失视为始终可见。

### UiRenderContext

合并 `(layout, assets, scale, screen_defs, conditions)` 为统一上下文包，使所有 `build_*` 函数不再接触 `AppState`。`ConditionContext`（含 `has_continue` 与 `&PersistentStore`）在 `host_app.rs` 每帧预求值后传入。

### 数据模型

`ScreenDefinitions`、`TitleScreenDef`、`GameMenuDef`、`ButtonListDef`、`ButtonDef`、`ActionDef`、`ConditionDef`、`ConditionalAsset`。全部实现 serde 反序列化 + Default（等价于当前硬编码值）。

## 风险

- **表达力有限**：极简条件系统无法处理复合条件。缓解：覆盖当前所有已知需求；未来可扩展 `and`/`or` 组合。
- **新动作需改引擎**：新增 `ActionDef` 变体需同步修改 `EguiAction` 和 `handle_egui_action`。缓解：动作词汇表已覆盖常见操作。

## 实施完成（2026-03）

1. 引入 `UiRenderContext` 与 `ConditionContext`，合并公共参数
2. 定义 `ScreenDefinitions` 等数据模型 + serde + Default
3. 实现条件求值器（`ConditionDef::evaluate`）与 `ActionDef` → `EguiAction` 转换
4. 改造 title、ingame_menu、ingame（quick_menu）、game_menu 消费 `ScreenDefinitions` + `UiRenderContext`；settings、save_load、history、confirm、skip_indicator、toast 改为接收 `UiRenderContext`
5. `history::build_history_content` 改为接收 `&[HistoryEvent]` 而非 `&AppState`
6. 编写 `docs/engine/ui/screens-customization.md` 与示例

## 验收标准（已达成）

- `cargo check-all` 通过
- 不提供 `screens.json` 时行为与改造前完全一致
- 提供 `screens.json` 可成功覆盖按钮列表、动作、可见性条件、背景切换
- 条件求值和动作转换有单元测试覆盖（`host/src/ui/screen_defs.rs`）
