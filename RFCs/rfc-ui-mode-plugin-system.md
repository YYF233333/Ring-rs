# RFC: UI Mode Plugin System

## 元信息

- 编号：RFC-022
- 状态：Implemented
- 作者：claude-4.6-opus
- 日期：2026-03-23
- 相关范围：`host`（ui_modes / app / ui / host_app / build_ui）
- 前置：RFC-020（双向 UI-Script 通信协议）

---

## 背景

RFC-020 建立了 `requestUI` / `UIResult` 双向通信协议，脚本可通过 `requestUI "mode"` 请求 Host 展示自定义 UI。当前 Host 侧的 mode 分发采用硬编码 if-else 链：

```rust
// host/src/app/update/script.rs（现状）
if mode == "call_game" {
    // WebView 小游戏逻辑
} else if mode == "show_map" {
    // 地图加载逻辑
}
```

这导致：

1. **新增 UI 模式必须修改核心代码**：每个新 mode 都要在 `script.rs` 中添加分支
2. **无统一生命周期**：各 mode 的激活/渲染/清理逻辑散布在不同位置
3. **地图等复杂 UI 缺少渲染实现**：`MapDefinition` 数据模型已存在，但无 egui 渲染、无命中检测

引擎正从 AVG 扩展到 RPG，预期将有更多自定义 UI 模式（地图、背包、任务面板等）。需要一个类似 `EffectExtension` / `ExtensionRegistry` 的 plugin 机制，使引擎维护者可以通过实现 trait + 注册的方式添加新 UI 模式，而非侵入核心流程。

---

## 目标与非目标

### 目标

- 定义 `UiModeHandler` trait 作为 UI 模式的标准接口（activate / render / deactivate）
- 定义 `UiModeRegistry` 管理注册与运行时调度
- 将现有 `show_map` 的 if-else 分支迁移为首个 handler 实现
- 实现地图颜色掩码命中检测（支持纹理轮廓定义的异形可选区域）
- `call_game`（WebView）保留独立处理路径（生命周期特殊性，不纳入 UiModeHandler）
- 新增 UI 模式只需：实现 trait → 注册 → 脚本调用

### 非目标

- 第三方 plugin 加载（仅引擎内建 handler）
- 多模式叠加（同时只有一个 UI 模式活跃）
- UI 模式的热重载
- 通用 CustomScreen JSON DSL（后续需求驱动时再设计）
- 扩展生命周期钩子（on_load / on_scene_enter 等，当前不需要）

---

## 方案设计

### 核心类型

#### UiModeHandler trait

```rust
// host/src/ui_modes/mod.rs

/// UI 模式处理器
///
/// 实现此 trait 以注册自定义 UI 模式。
/// 每个模式对应 `requestUI` 命令的一个 `mode` 值。
pub trait UiModeHandler: std::fmt::Debug + Send {
    /// 模式标识符（与 `Command::RequestUI.mode` 匹配）
    fn mode_id(&self) -> &str;

    /// 收到 `Command::RequestUI` 时激活此模式
    ///
    /// `key` 用于回传 `RuntimeInput::UIResult` 时的匹配。
    /// `params` 是脚本传入的参数。
    /// `resources` 用于加载模式所需的资源。
    fn activate(
        &mut self,
        key: String,
        params: &HashMap<String, VarValue>,
        resources: &ResourceManager,
    ) -> Result<(), UiModeError>;

    /// 每帧渲染
    ///
    /// 在 egui context 内调用。返回 `Active` 表示继续渲染，
    /// 返回 `Completed(value)` 表示用户完成交互，携带结果值。
    fn render(
        &mut self,
        ctx: &egui::Context,
        scale: &ScaleContext,
    ) -> UiModeStatus;

    /// 模式结束或被取消后清理内部状态和资源
    fn deactivate(&mut self);
}
```

#### UiModeStatus

```rust
/// UI 模式每帧渲染返回的状态
pub enum UiModeStatus {
    /// 模式仍然活跃，继续渲染
    Active,
    /// 用户完成交互，携带结果值
    Completed(VarValue),
    /// 用户取消（Esc 等），无结果
    Cancelled,
}
```

#### UiModeError

```rust
#[derive(Debug, thiserror::Error)]
pub enum UiModeError {
    #[error("unknown UI mode: {0}")]
    UnknownMode(String),
    #[error("another UI mode is already active")]
    AlreadyActive,
    #[error("resource load failed: {0}")]
    ResourceLoadFailed(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
}
```

#### UiModeRegistry

```rust
/// UI 模式注册表与运行时调度
pub struct UiModeRegistry {
    handlers: HashMap<String, Box<dyn UiModeHandler>>,
    /// 当前活跃的模式 ID
    active_mode: Option<String>,
    /// 当前活跃模式的 request key（用于回传 UIResult）
    active_key: Option<String>,
}

impl UiModeRegistry {
    pub fn new() -> Self;

    /// 注册一个 UI 模式 handler
    pub fn register(&mut self, handler: Box<dyn UiModeHandler>);

    /// 激活指定模式
    pub fn activate(
        &mut self,
        mode: &str,
        key: String,
        params: &HashMap<String, VarValue>,
        resources: &ResourceManager,
    ) -> Result<(), UiModeError>;

    /// 每帧渲染当前活跃模式
    ///
    /// 返回 `Some((key, value))` 表示模式完成，需要发送 UIResult。
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        scale: &ScaleContext,
    ) -> Option<(String, VarValue)>;

    /// 是否有活跃模式
    pub fn is_active(&self) -> bool;

    /// 强制取消当前活跃模式
    pub fn cancel_current(&mut self);
}
```

### 集成设计

#### RequestUI 处理流程变更

```
收到 Command::RequestUI { key, mode, params }
  ├── mode == "call_game"
  │     → 保持现有 WebView 路径（生命周期特殊）
  └── 其他 mode
        → ui_mode_registry.activate(mode, key, params, resources)
        → 失败则 warn 并忽略
```

#### 每帧渲染集成

```
帧循环（build_ui 或 host_app）
  ├── ui_mode_registry.is_active()?
  │     → ui_mode_registry.render(egui_ctx, scale)
  │     → 如果返回 Some((key, value))
  │          → run_script_tick(RuntimeInput::UIResult { key, value })
  │          → ui_mode_registry 内部自动 deactivate
  └── 否则
        → 正常渲染 ingame/menu 等页面
```

活跃 UI 模式优先级高于普通游戏画面，覆盖在最上层。

### MapModeHandler：地图颜色掩码实现

#### 扩展后的 MapDefinition

```rust
// host/src/ui/map.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapDefinition {
    pub title: String,
    #[serde(default)]
    pub background: Option<String>,
    /// 命中检测掩码图路径（同尺寸，每个区域涂唯一纯色）
    #[serde(default)]
    pub hit_mask: Option<String>,
    pub locations: Vec<MapLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapLocation {
    pub id: String,
    pub label: String,
    /// 掩码图中此区域的颜色（如 "#FF0000"）
    #[serde(default)]
    pub mask_color: Option<String>,
    /// 标签显示位置
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub condition: Option<String>,
}
```

#### JSON 格式

```json
{
  "title": "世界地图",
  "background": "maps/world_bg.png",
  "hit_mask": "maps/world_mask.png",
  "locations": [
    {
      "id": "beach",
      "label": "海边",
      "mask_color": "#FF0000",
      "x": 360.0,
      "y": 700.0,
      "enabled": true
    },
    {
      "id": "town",
      "label": "小镇",
      "mask_color": "#00FF00",
      "x": 960.0,
      "y": 500.0,
      "enabled": true
    },
    {
      "id": "forest",
      "label": "森林",
      "mask_color": "#0000FF",
      "x": 1450.0,
      "y": 320.0,
      "enabled": true,
      "condition": "visited_town"
    }
  ]
}
```

#### MapModeHandler 内部状态

```rust
// host/src/ui_modes/map_handler.rs

#[derive(Debug)]
pub struct MapModeHandler {
    active: Option<MapActiveState>,
}

struct MapActiveState {
    definition: MapDefinition,
    request_key: String,
    /// 背景图 egui 纹理
    background_texture: egui::TextureHandle,
    /// 掩码图原始像素（CPU 侧，用于命中检测）
    mask_pixels: Vec<u8>,
    mask_width: u32,
    mask_height: u32,
    /// RGB → location 索引映射表
    color_to_location: HashMap<[u8; 3], usize>,
    /// 当前悬停的 location 索引
    hovered: Option<usize>,
}
```

#### 命中检测算法

```rust
fn hit_test(&self, base_x: f32, base_y: f32) -> Option<usize> {
    let px = base_x as u32;
    let py = base_y as u32;
    if px >= self.mask_width || py >= self.mask_height {
        return None;
    }
    let offset = ((py * self.mask_width + px) * 4) as usize;
    let r = self.mask_pixels[offset];
    let g = self.mask_pixels[offset + 1];
    let b = self.mask_pixels[offset + 2];
    let a = self.mask_pixels[offset + 3];
    if a < 128 {
        return None; // 透明区域
    }
    self.color_to_location.get(&[r, g, b]).copied()
}
```

#### render() 流程

1. 全屏绘制 `background_texture`
2. 读取 `ctx.input()` 获取鼠标位置
3. 通过 `ScaleContext` 反算基准分辨率坐标
4. 调用 `hit_test()` 判定悬停区域
5. 如果悬停到 enabled 区域：显示高亮标签（`egui::Area` 浮层）
6. 如果点击 enabled 区域：返回 `UiModeStatus::Completed(VarValue::String(location.id))`
7. 如果按 Esc：返回 `UiModeStatus::Cancelled`

#### 向后兼容

现有 `MapDefinition` 新增的 `hit_mask` 和 `mask_color` 字段均为 `Option`（`#[serde(default)]`），不影响已有地图 JSON 文件。无 hit_mask 时回退到基于 (x, y) 坐标 + 固定半径的矩形命中检测（保持当前行为）。

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host/src/ui_modes/` (新增) | UiModeHandler trait + UiModeRegistry + MapModeHandler | 低：纯新增 |
| `host/src/ui/map.rs` | MapDefinition 扩展 hit_mask / mask_color 字段 | 低：纯新增 Optional 字段 |
| `host/src/app/mod.rs` | AppState 新增 `ui_mode_registry: UiModeRegistry` | 低：新增字段 |
| `host/src/app/update/script.rs` | RequestUI 分发从 if-else 改为 registry 路由 | 中：重构现有逻辑 |
| `host/src/build_ui.rs` 或 `host/src/host_app.rs` | 帧循环中插入 ui_mode_registry.render() | 中：需确定渲染时机 |
| `host/src/ui/map.rs` | 移除 MapDisplayState（迁入 MapModeHandler） | 低：内部重构 |
| `host/src/renderer/render_state` | 移除 `map_display` 字段（迁入 registry） | 低：解耦 |

---

## 迁移计划

纯新增 + 内部重构，不影响脚本语法和外部行为。

1. 新建 `ui_modes/` 模块，定义 trait + registry + error/status 类型
2. 实现 MapModeHandler（色块掩码命中检测 + egui 渲染）
3. 在 AppState 初始化时创建 registry 并注册 MapModeHandler
4. 将 script.rs 中 `show_map` 分支改为 `registry.activate()`
5. 在帧循环中插入 `registry.render()` 调用
6. 移除 render_state 中的 `map_display` 字段
7. 更新地图 JSON 格式（新增 hit_mask + mask_color）

---

## 验收标准

- [x] UiModeHandler trait 定义完整
- [x] UiModeRegistry 支持 register / activate / render / cancel
- [x] MapModeHandler 正确加载背景 + 掩码图
- [x] 颜色掩码命中检测准确（含透明区域跳过）
- [x] 地图 UI 正确渲染（背景 + 悬停标签 + 禁用状态灰显）
- [x] 点击返回正确的 location.id 作为 UIResult
- [x] Esc 取消返回空字符串
- [x] 无 hit_mask 时回退到坐标矩形检测（向后兼容）
- [x] script.rs 中 show_map 不再硬编码（通过 registry 路由）
- [x] render_state 中 map_display 字段移除
- [x] `cargo check-all` 通过
- [x] 模块摘要文档更新
