# RFC: UI 系统后续增强

## 元信息

- 编号：RFC-011
- 状态：Accepted
- 作者：Ring-rs 开发组
- 日期：2026-03-15
- 前置：RFC-010（可定制 UI 系统）
- 相关范围：`host/src/ui/`、`host/src/egui_screens/`、`host/src/backend/`、`host/src/app/`、`host/src/save_manager/`、`assets/gui/`

---

## 1. 背景

RFC-010 构建了数据驱动的可定制 UI 系统，完成了核心架构（`UiLayoutConfig` + `ScaleContext` + `UiAssetCache` + `NinePatch`）和所有主要页面的改造。但部分功能因优先级、技术依赖或工作量原因被推迟。

本 RFC 将这些后续工作统一管理，按优先级分阶段实施。

---

## 2. 目标与非目标

### 2.1 目标

- **季节切换完整接入**：标题画面和游戏菜单根据 persistent store 状态切换夏篇/冬篇背景
- **存读档增强**：分页导航、存档截图缩略图
- **视觉精细化**：NotoSansSC 字体加载、notify.png 背景、自定义滑块样式
- **文档补全**：UI 定制指南（如何通过 `layout.json` + 素材替换自定义 UI）

### 2.2 非目标

- NVL 模式 UI（如需另开 RFC）
- Rollback 功能及对应"回退"按钮（依赖 runtime 支持，不在本 RFC 范围）
- 运行时 UI 热重载
- 像素级还原 ref-project

---

## 3. 工作项

### 3.1 高优先级

#### 3.1.1 季节切换逻辑

**现状**：✅ **已实现**。`title.rs` 与 `game_menu.rs` 通过 `app_state.persistent_store().get("complete_summer")` 判断，选用 `main_winter` / `main_summer` 背景键；`UiLayoutConfig` 提供 `main_summer` / `main_winter` 路径（默认 `gui/main_summer.jpg`、`gui/main_winter.jpg`），素材由 `UiAssetCache` 统一加载。

**目标**（已达成）：
- 读取 persistent store 的 `complete_summer` 标志
- 标题画面与游戏菜单按标志切换冬/夏背景
- 两套季节背景路径在 layout 中配置，由 asset cache 加载

**涉及文件**：`host/src/egui_screens/title.rs`、`host/src/egui_screens/game_menu.rs`、`host/src/ui/layout.rs`、`host/src/host_app.rs`（传入 is_winter）

#### 3.1.2 冬篇入口按钮

**现状**：✅ **已实现**。标题画面在 `is_winter == true` 时显示「冬篇」按钮，绑定 `EguiAction::StartWinter`，由宿主处理跳转冬篇起始脚本。

**目标**（已达成）：
- `complete_summer = true` 时显示「冬篇」按钮
- 点击触发冬篇入口逻辑

**涉及文件**：`host/src/egui_screens/title.rs`、`host/src/egui_actions.rs`、`host/src/host_app.rs`

### 3.2 中优先级

#### 3.2.1 存读档分页导航

**现状**：✅ **已实现**。`SaveLoadPage` 枚举（A / Q / 1–9）在 `app_mode.rs` 定义，`save_load.rs` 底部分页栏实现 `<` / `A` `Q` `1`–`9` / `>`，每页 6 槽（`slot_range()`），当前页高亮。

**目标**（已达成）：
- 底部分页按钮栏：`<` `A` `Q` `1`–`9` `>`
- `A` = Auto，`Q` = Quick，`1`–`9` = 手动存档页，每页 6 槽，共 66 槽
- 当前页按钮高亮

**涉及文件**：`host/src/egui_screens/save_load.rs`、`host/src/app/app_mode.rs`

#### 3.2.2 NotoSansSC 字体加载

**现状**：CJK 字体通过 `config.default_font` 配置（默认 `fonts/simhei.ttf`），在 `main.rs` 中读取后以 `font_data` 传入 `HostApp`，由 `host/src/backend/mod.rs` 的 `configure_fonts` 注册为 egui 默认字体。**NotoSansSC 尚未作为默认或专用字体**；若需与 ref-project 一致，可改为默认加载 `assets/fonts/NotoSansSC-Regular.otf` 或通过配置指定。

**目标**：
- 启动时从 `assets/fonts/NotoSansSC-Regular.otf` 加载（或保持可配置）
- 注册为 egui 的 proportional font family
- 确保 `UiLayoutConfig` 中的字号配置（33/45/75px）生效

**涉及文件**：`host/src/main.rs`、`host/src/backend/mod.rs`、`host/src/config/mod.rs`

### 3.3 低优先级

#### 3.3.1 存档截图缩略图

**现状**：✅ **已实现**。保存时 `host_app` 设置 `pending_thumbnail_slot` 并调用 `backend.request_screenshot()`，下一帧 `backend.take_screenshot()` 回读帧缓冲；`save_manager.save_thumbnail()` 将 RGBA 缩放到 384×216 并写入 `saves/thumb_NNN.png`。读档界面从 `save_manager.load_thumbnail_bytes()` 加载并显示为 `thumbnails` 纹理。

**目标**（已达成）：
- 保存时截取当前画面缩略图（384×216）
- 缩略图文件与存档槽位对应，存读档界面显示

**涉及文件**：`host/src/egui_screens/save_load.rs`、`host/src/host_app.rs`、`host/src/save_manager/mod.rs`、`host/src/backend/mod.rs`

#### 3.3.2 notify.png 背景

**现状**：✅ **已实现**。Toast 在 `toast.rs` 中通过 `assets.get("notify")` 获取纹理，使用 `NinePatch` 渲染背景；`UiLayoutConfig.notify` 提供 `ypos`、`frame_borders` 及路径（默认 `gui/notify.png`）。

**目标**（已达成）：
- Toast 使用 `gui/notify.png` 作为 NinePatch 背景
- Y 偏移与边框由 layout 配置

**涉及文件**：`host/src/egui_screens/toast.rs`、`host/src/ui/layout.rs`

#### 3.3.3 自定义滑块样式

**现状**：✅ **已实现**。`host/src/ui/image_slider.rs` 提供基于 NinePatch 轨道与图片拇指的滑块；`settings.rs` 通过 `SliderTextures`（`slider_idle_thumb` / `slider_hover_thumb` 及 bar 纹理）调用，支持 idle/hover 状态。`UiLayoutConfig` 中配置 `gui/slider/horizontal_idle_thumb.png`、`horizontal_hover_thumb.png` 等路径。

**目标**（已达成）：
- 使用 `gui/slider/` 素材渲染轨道与手柄
- 支持 idle/hover 状态

**涉及文件**：`host/src/egui_screens/settings.rs`、`host/src/ui/image_slider.rs`、`host/src/ui/layout.rs`

#### 3.3.4 UI 定制文档

**现状**：✅ **已实现**。`docs/ui_customization.md` 已存在，包含配置文件位置（`assets/ui/layout.json`）、完整结构示例、字体/颜色/布局字段说明、素材路径与 NinePatch 边框等，面向游戏开发者。

**目标**（已达成）：
- `docs/ui_customization.md` 描述 layout 字段、素材替换与路径映射
- 面向游戏开发者

---

## 4. 风险

| 风险 | 等级 | 缓解 |
|------|------|------|
| persistent store 接口尚未稳定 | 中 | 季节切换逻辑通过抽象读取接口隔离，不直接依赖存储格式 |
| GPU 帧回读（存档截图）性能与兼容性 | 中 | 降级策略：截图失败时保持灰色占位，不阻断保存流程 |
| 自定义滑块 egui Painter 工作量 | 低 | 可分步：先实现静态图片滑块，再添加动画/交互细节 |
| NotoSansSC 字体文件较大（~15MB） | 低 | 可考虑 subset 裁剪仅保留常用字符集 |

---

## 5. 分阶段计划

### Phase A：季节切换 + 冬篇入口（高优先级）

- [x] `title.rs` 读取 persistent store 切换背景图
- [x] `title.rs` 条件显示冬篇按钮
- [x] `game_menu.rs` 同步季节背景切换
- [x] 验收：`complete_summer = true` 时切换为冬篇 UI

### Phase B：存读档分页 + 字体加载（中优先级）

- [x] `save_load.rs` 添加分页按钮栏 + 页面切换状态
- [ ] 字体加载入口注册 NotoSansSC（当前为 config 可配置 CJK 字体，默认 simhei）
- [x] 验收：分页可切换；（CJK 字体已可配置，NotoSansSC 为可选）

### Phase C：视觉细节 + 文档（低优先级）

- [x] 存档截图功能（GPU 帧回读 + 缩略图编码）
- [x] Toast 使用 `notify.png` NinePatch 背景
- [x] 自定义滑块样式
- [x] 撰写 `docs/ui_customization.md`

---

## 6. 验收标准

| # | 标准 | Phase | 状态 |
|---|------|-------|------|
| 1 | `complete_summer` 后标题画面/游戏菜单自动切换为冬篇背景 | A | ✅ 已验收 |
| 2 | 冬篇按钮仅在解锁后显示，点击可进入冬篇 | A | ✅ 已验收 |
| 3 | 存读档界面支持分页切换（A/Q/1-9），每页 6 个槽位 | B | ✅ 已验收 |
| 4 | 对话/界面/标题文字使用 NotoSansSC 字体且字号正确 | B | ⏳ 未做（当前为可配置 CJK 字体） |
| 5 | 存档槽位显示截图缩略图 | C | ✅ 已验收 |
| 6 | Toast 通知使用 notify.png 背景 | C | ✅ 已验收 |
| 7 | 设置页面滑块使用自定义图片样式 | C | ✅ 已验收 |
| 8 | UI 定制文档完整可用 | C | ✅ 已验收 |

---

## 7. 相关 RFC

| RFC | 标题 | 关系 |
|-----|------|------|
| RFC-002 | ref-project 重制体验等价计划 | 本 RFC 延续 P2-4 "UI 风格持续贴近原作" |
| RFC-010 | 可定制 UI 系统 | 本 RFC 的直接前置，承接其未完成工作 |

---

## 8. 实施状态（与仓库同步）

**更新日期**：2026-03-15。以下与当前仓库实现一致。

| 工作项 | 状态 | 说明 |
|--------|------|------|
| 3.1.1 季节切换逻辑 | ✅ 已实现 | title/game_menu 按 persistent store 的 complete_summer 切换 main_winter/main_summer |
| 3.1.2 冬篇入口按钮 | ✅ 已实现 | 标题画面 is_winter 时显示「冬篇」按钮，EguiAction::StartWinter |
| 3.2.1 存读档分页导航 | ✅ 已实现 | SaveLoadPage A/Q/1-9，save_load 底部分页栏，每页 6 槽 |
| 3.2.2 NotoSansSC 字体加载 | ⏳ 未做 | 当前为 config.default_font（默认 simhei），NotoSansSC 可选 |
| 3.3.1 存档截图缩略图 | ✅ 已实现 | 帧回读 + save_manager 缩略图读写 + save_load 显示 |
| 3.3.2 notify.png 背景 | ✅ 已实现 | toast.rs 使用 notify 纹理 + NinePatch |
| 3.3.3 自定义滑块样式 | ✅ 已实现 | image_slider.rs + settings 使用 slider 素材 |
| 3.3.4 UI 定制文档 | ✅ 已实现 | docs/ui_customization.md 已存在且完整 |

**待办**：仅 3.2.2 NotoSansSC 作为默认或专用字体尚未实现；若需与 ref-project 一致，可在 config 或资源中默认使用 NotoSansSC。
