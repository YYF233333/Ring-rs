# RFC: Tauri 前端 UI 主题与客制化架构

## 元信息

- 编号：RFC-030
- 状态：Proposed
- 作者：Ring-rs 开发组
- 日期：2026-03-25
- 相关范围：`host-tauri/src/`、`assets/ui/`、`docs/engine/ui/`
- 前置：RFC-010（可定制 UI 系统）、RFC-011（UI 增强）、RFC-012（UI 行为定制）、RFC-013（配置外部化）

---

## 1. 背景

egui host 通过 RFC-010/011/012/013 建立了完整的数据驱动 UI 体系：

- `layout.json`：像素布局参数 + 素材路径 + 颜色（基于 1920×1080 + ScaleContext 缩放）
- `screens.json`：按钮/动作/条件可见性/条件背景

Tauri host 采用 Web-native 路线重新实现了等价 UI，但有自己的定制维度：

- 布局由 CSS 原生实现（`vw/vh/clamp()/flex/grid`），不消费 layout.json 的像素值
- 共享 `screens.json`（行为定义）和 `layout.json` 的 `assets`（素材路径）+ `colors`（颜色）
- Web 平台有 CSS 变量、CSS 动画、`backdrop-filter`、自定义字体等 egui 不具备的能力

本 RFC 定义 Tauri 前端的三层定制架构，使游戏作者和引擎开发者能在不同层级定制 UI。

---

## 2. 目标与非目标

### 2.1 目标

- 定义清晰的三层定制能力（跨 host 共享 / Web-only CSS / 自定义页面）
- 游戏作者仅通过配置文件和 CSS 即可定制 Tauri host 的 UI 外观
- 保持与 egui host 的行为一致性（screens.json 是唯一的按钮/动作/条件源）
- 当 GUI 素材缺失时优雅降级为纯 CSS 风格

### 2.2 非目标

- 不做 egui layout.json 像素值的 Web 端桥接
- 不实现运行时 UI 编辑器或热重载
- 不做 CSS-in-JS 方案（保持 Vue scoped CSS）
- 自定义页面系统的完整设计（属远期，本 RFC 仅定义接口预留）

---

## 3. 三层定制架构

### 第一层：跨 Host 共享配置

**影响范围**：egui host + Tauri host 同时生效

| 配置文件 | 定制内容 | 消费方 |
|----------|---------|--------|
| `layout.json` → `assets` | GUI 素材路径（textbox、namebox、overlay、背景等） | `useTheme().asset(key)` |
| `layout.json` → `colors` | 主题颜色（accent、hover、idle、text 等） | 写入 CSS 变量 |
| `screens.json` | 按钮文案/动作/条件可见性/条件背景 | `useScreens()` |

**游戏作者操作**：修改 `assets/ui/layout.json` 和 `assets/ui/screens.json`。

### 第二层：Web-only CSS 主题

**影响范围**：仅 Tauri host

游戏作者可在 `assets/gui/theme.css` 放置一个可选的 CSS 文件。`useTheme()` 在初始化时尝试加载它（不存在则静默跳过）。

可覆盖的 CSS 变量：

```css
:root {
  /* 颜色（第一层 layout.json colors 会初始化这些变量，theme.css 可覆盖） */
  --vn-color-accent: ...;
  --vn-color-idle: ...;
  --vn-color-hover: ...;
  --vn-color-selected: ...;
  --vn-color-text: ...;
  --vn-color-ui-text: ...;

  /* 字体 */
  --vn-font-body: ...;
  --vn-font-display: ...;

  /* 表面 */
  --vn-surface-primary: ...;
  --vn-surface-overlay: ...;
  --vn-surface-card: ...;

  /* Web-specific 令牌 */
  --vn-radius-sm: ...;
  --vn-radius-md: ...;
  --vn-transition: ...;
}
```

此外，theme.css 可以直接用类选择器覆盖组件样式（组件使用非 scoped 的语义类名暴露钩子）。

### 第三层：自定义页面（远期）

两个子维度：

**3a. 引擎开发者扩展**
- 在 `host-tauri/src/screens/` 添加 Vue 组件
- 在 `useNavigation` 注册新 screen name
- 在 `screens.json` 的 `game_menu.nav_buttons` 添加导航项
- 无需新基础设施

**3b. 游戏作者自定义页面**（需后续 RFC 详细设计）

初步方向：
- 脚本 `requestUI` 指令 + `assets/ui/pages/*.html` 自定义页面文件
- Tauri 可利用 WebView 原生渲染任意 HTML
- 通过 bridge API 与 runtime 交互（读写变量、触发动作）

---

## 4. IPC 接口

已实现的三个 IPC 命令：

| 命令 | 返回值 | 用途 |
|------|--------|------|
| `get_screen_definitions` | screens.json 全文 | 按钮/动作/条件 |
| `get_ui_assets` | `{ assets, colors }` | 素材路径 + 颜色 |
| `get_ui_condition_context` | `{ has_continue, persistent }` | 条件求值上下文 |

---

## 5. 前端基础设施

| Composable | 职责 |
|-----------|------|
| `useTheme()` | 加载 assets/colors，初始化 CSS 变量，提供 `asset(key)` 方法，尝试加载 theme.css |
| `useScreens()` | 加载 screens.json，提供按钮列表/动作映射/条件求值/条件背景解析 |

---

## 6. 降级策略

当 GUI 素材（`assets/gui/`）不可用时：

- `useTheme().asset(key)` 返回 `undefined`
- Vue 组件通过 `v-if` 判断素材 URL 是否存在，不存在时使用 CSS-only fallback
- 例：对话框无 textbox 图片时显示半透明深色背景 + backdrop-filter
- 例：标题无背景图时显示 CSS 渐变

这确保开发调试时即使没有 GUI 素材也能正常运行。

---

## 7. 风险

- **theme.css 的类名稳定性**：组件重构可能改变类名，破坏 theme.css。需要在文档中声明稳定的"主题钩子"类名列表。
- **layout.json 的 colors 段与 CSS 变量的优先级**：theme.css 加载时机在 layout.json 颜色之后，因此 theme.css 的值会覆盖 layout.json colors，这是预期行为。
- **screens.json 扩展格式**：当前 screens.json 定义了 title/ingame_menu/quick_menu/game_menu 四块。新增页面的导航项格式需要与 egui host 协调（或在 screens.json 中用可选字段）。

---

## 8. 验收标准

- [ ] `useTheme()` 和 `useScreens()` composable 正常工作
- [ ] 标题画面使用 screens.json 按钮 + 条件背景
- [ ] GameMenuFrame 使用 screens.json 导航 + 条件背景
- [ ] 对话框、选项使用 layout.json assets 中的 GUI 素材
- [ ] 当 GUI 素材缺失时优雅降级为纯 CSS 风格
- [ ] `gui/theme.css`（可选）能覆盖 CSS 变量
- [ ] 前端门禁通过（biome + vue-tsc）
