# RFC: 可定制 UI 系统

## 元信息

- 编号：RFC-010
- 状态：Accepted
- 作者：Ring-rs 开发组
- 日期：2026-03-15
- 实施日期：2026-03-15
- 相关范围：`host/src/ui/`、`host/src/egui_screens/`、`host/src/backend/`、`assets/gui/`

---

## 1. 背景

当前引擎的 UI 是在 RFC-007 迁移到 egui 后的"能用即可"状态：所有页面使用硬编码颜色、尺寸和纯文本控件，无图片素材支持，也未真正使用已有的 Theme token 系统。与 ref-project 的 Ren'Py 原作 UI 存在显著的视觉差距。

ref-project 使用 Ren'Py 的 `gui.rpy` / `screens.rpy` 体系，通过声明式配置变量 + 图片素材 + screen 布局定义了完整的视觉风格。所有 UI 素材已在 `assets/ref-project/gui/` 中可用。

本 RFC 的目标是构建一个**数据驱动的可定制 UI 系统**，使引擎能够：
1. 复用 ref-project 的 GUI 素材实现与原作视觉风格一致的 UI
2. 支持未来不同游戏项目通过替换配置 + 素材来定制 UI 外观

---

## 2. 目标与非目标

### 2.1 目标

- **视觉等价**：核心页面（标题/对话框/快捷菜单/游戏菜单/设置/存读档/历史/选项/确认弹窗）的视觉风格与 ref-project 主观一致
- **数据驱动**：布局参数（位置/尺寸/间距/颜色/字号）和图片素材路径从配置文件读取，不硬编码
- **图片 UI 支持**：对话框、名字框、按钮、滑块、滚动条等核心控件支持图片背景/前景，支持九宫格拉伸
- **分辨率适配**：以 1920×1080 为基准设计分辨率，按实际窗口尺寸等比缩放
- **增量实施**：按页面逐步改造，每个阶段有独立可验收的产出

### 2.2 明确非目标

- 不实现 Ren'Py screen language 的通用解释器
- 不要求像素级一致——保留美术风格与信息层级即可（与 RFC-002 §2.2 一致）
- 不在此 RFC 中实现 NVL 模式（如需另开 RFC）
- 不做运行时 UI 热重载（开发期可通过重启验证）
- 不做拖拽式 UI 编辑器

---

## 3. 现状分析

### 3.1 当前 UI 实现

| 模块 | 现状 | 问题 |
|------|------|------|
| `egui_screens/helpers.rs` | 3 个硬编码颜色常量 + 1 个通用按钮函数 | 不读取 Theme，不支持图片 |
| `egui_screens/title.rs` | 深色纯色背景 + "Visual Novel Engine" 标题 + 文本按钮 | 无背景图片，无季节切换，英文标签 |
| `egui_screens/ingame.rs` | 半透明底部面板 + 纯文本对话 + 基础选项列表 | 无 textbox.png，无 namebox.png，无快捷菜单 |
| `egui_screens/ingame_menu.rs` | 半透明覆盖层 + 文本按钮列表 | 无背景图片，样式简陋 |
| `egui_screens/settings.rs` | 纯色面板 + egui 原生滑块 | 无自定义滑块样式，缺少显示模式/跳过选项 |
| `egui_screens/save_load.rs` | 纯文本列表 + 文本按钮 | 无缩略图，无网格布局，无分页 |
| `egui_screens/history.rs` | 简单滚动文本列表 | 无分栏布局，样式粗糙 |
| `ui/theme.rs` | 完整的 token 分层体系 | **已有但未被 egui_screens 使用** |
| `ui/skin.rs` | 皮肤配置骨架（icon/button/panel 路径映射） | **已有但完全未实装** |
| `ui/theme_loader.rs` | JSON 主题覆盖加载 | 仅支持 palette 子集 |

### 3.2 ref-project UI 规格（从 `gui.rpy` / `screens.rpy` 提取）

#### 基准参数

| 参数 | 值 |
|------|------|
| 基准分辨率 | 1920 × 1080 |
| 文本字体 | NotoSansSC-Regular.otf |
| 对话字号 | 33px |
| 角色名字号 | 45px |
| 界面文字号 | 33px |
| 标题字号 | 75px |

#### 颜色体系

| 用途 | 颜色 |
|------|------|
| 强调色 | `#ffffff` |
| 空闲态文本 | `#888888` |
| 悬停色 | `#ff9900` |
| 选中色 | `#ffffff` |
| 禁用色 | `#7878787f` |
| 对话文本 | `#000000`（对话框内为黑色文字） |
| 界面文本 | `#ffffff` |
| 选项空闲 | `#cccccc` |
| 选项悬停 | `#ffffff` |

#### 对话框布局

| 参数 | 值 |
|------|------|
| 对话框高度 | 278px |
| 对话框对齐 | 底部 (yalign=1.0) |
| 对话框背景 | `gui/textbox.png` |
| 名字框位置 | (360, 0)，左对齐 |
| 名字框背景 | `gui/namebox.png` |
| 对话文本位置 | (402, 75) |
| 对话文本宽度 | 1116px |

#### 主菜单

| 参数 | 值 |
|------|------|
| 夏篇背景 | `gui/prelude/main_summer_edit.png` |
| 冬篇背景 | `gui/prelude/main_winter_edit.png` |
| 导航按钮位置 | xpos=60，垂直居中 |
| 导航按钮间距 | 6px |
| 冬篇入口 | 仅 `persistent.complete_summer` 后显示 |

#### 游戏菜单

| 参数 | 值 |
|------|------|
| 夏篇背景 | `gui/main_summer.jpg` |
| 冬篇背景 | `gui/main_winter.jpg` |
| 覆盖层 | `gui/overlay/game_menu.png` |
| 导航面板宽度 | 420px |
| 内容区左边距 | 60px |
| 内容区右边距 | 30px |
| 标题标签高度 | 180px |
| 标题字号 | 75px |

#### 快捷菜单

| 参数 | 值 |
|------|------|
| 位置 | 底部居中 (xalign=0.5, yalign=1.0) |
| 按钮字号 | 21px |
| 按钮 | 回退/历史/快进/自动/保存/快存/快读/设置 |

#### 选项按钮

| 参数 | 值 |
|------|------|
| 宽度 | 1185px |
| 位置 | 水平居中，ypos=405, yanchor=0.5 |
| 间距 | 33px |
| 背景图 | `gui/button/choice_idle_background.png` / `choice_hover_background.png` |
| 文本居中 | xalign=0.5 |

#### 存档界面

| 参数 | 值 |
|------|------|
| 网格 | 3 列 × 2 行 |
| 槽位按钮尺寸 | 414 × 309 px |
| 缩略图尺寸 | 384 × 216 px |
| 槽位间距 | 15px |
| 分页按钮 | < A Q 1-9 > |
| 背景图 | `gui/button/slot_idle_background.png` / `slot_hover_background.png` |

#### 历史界面

| 参数 | 值 |
|------|------|
| 条目高度 | 210px |
| 角色名位置 | xpos=233，右对齐，宽度=233 |
| 对话文本位置 | xpos=255，左对齐，宽度=1110 |

#### 确认弹窗

| 参数 | 值 |
|------|------|
| 覆盖层 | `gui/overlay/confirm.png` |
| 框架背景 | `gui/frame.png` |
| 按钮间距 | 150px |

#### 快进指示器

| 参数 | 值 |
|------|------|
| 位置 | ypos=15 |
| 背景 | `gui/skip.png` |
| 文本 | "正在快进" + 闪烁箭头 |

### 3.3 GUI 素材清单

ref-project 的 `gui/` 目录已包含完整素材集：

```
gui/
├── textbox.png              # 对话框背景
├── namebox.png              # 名字框背景
├── frame.png                # 通用框架
├── main_menu.png            # 主菜单背景
├── game_menu.png            # 游戏菜单背景
├── main_summer.jpg          # 夏篇背景
├── main_winter.jpg          # 冬篇背景
├── skip.png                 # 快进指示器
├── notify.png               # 通知框
├── nvl.png                  # NVL 模式背景
├── window_icon.png          # 窗口图标
├── bar/                     # 进度条素材
├── button/                  # 按钮状态素材
│   ├── choice_idle/hover_background.png
│   ├── slot_idle/hover_background.png
│   ├── quick_idle/hover_background.png
│   ├── idle/hover_background.png
│   ├── check_foreground/selected_foreground.png
│   └── radio_foreground/selected_foreground.png
├── overlay/                 # 覆盖层
│   ├── main_menu.png
│   ├── game_menu.png
│   └── confirm.png
├── prelude/                 # 标题画面
│   ├── main_summer_edit.png
│   └── main_winter_edit.png
├── scrollbar/               # 滚动条素材
└── slider/                  # 滑块素材
```

### 3.4 差距总结

| 维度 | 差距等级 | 说明 |
|------|---------|------|
| 对话框 | **大** | 无图片背景、无名字框、布局参数硬编码 |
| 标题画面 | **大** | 无背景图、无季节切换、英文标签 |
| 快捷菜单 | **大** | 完全缺失（游戏中无底部快捷按钮栏） |
| 游戏菜单框架 | **大** | 无背景图、无左侧导航面板布局 |
| 选项按钮 | **中** | 有基本功能但无图片背景样式 |
| 存读档 | **大** | 无缩略图、无网格布局、无分页 |
| 设置 | **中** | 有基本功能但缺少显示模式/跳过选项/自定义滑块 |
| 历史 | **中** | 有基本功能但无分栏布局 |
| 确认弹窗 | **大** | 完全缺失 |
| 快进/通知指示器 | **中** | Toast 有基本实现，快进指示缺失 |
| Theme 集成 | **中** | Token 体系已有但未被页面使用 |
| 分辨率适配 | **大** | 无缩放机制，各页面假设固定尺寸 |

---

## 4. 方案设计

### 4.1 架构概览

```
┌─────────────────────────────────────────────────┐
│                 UiConfig (JSON)                  │
│  布局参数 + 颜色 + 字号 + 素材路径 + 间距        │
└────────────────────┬────────────────────────────┘
                     │ 启动时加载
                     ▼
┌─────────────────────────────────────────────────┐
│              UiLayoutConfig (Rust)               │
│  DialogueLayout / TitleLayout / MenuLayout /     │
│  SaveLoadLayout / SettingsLayout / ...           │
│  + ScaleContext (基准分辨率 → 实际分辨率缩放)     │
└────────────────────┬────────────────────────────┘
                     │ 注入
                     ▼
┌─────────────────────────────────────────────────┐
│              UiAssetCache (Rust)                 │
│  通过 ResourceManager 加载 GUI 图片素材          │
│  缓存为 egui TextureHandle                       │
│  支持九宫格 (NinePatch) 拉伸渲染                 │
└────────────────────┬────────────────────────────┘
                     │ 注入
                     ▼
┌─────────────────────────────────────────────────┐
│           egui_screens/* (改造后)                 │
│  各页面读取 UiLayoutConfig + UiAssetCache        │
│  而非硬编码值                                    │
│  使用 ScaleContext 将基准坐标转为实际坐标        │
└─────────────────────────────────────────────────┘
```

### 4.2 核心类型

#### UiLayoutConfig

对应 Ren'Py `gui.rpy` 中的配置变量。以 JSON 格式存储，启动时一次性加载。

```rust
/// UI 布局配置，对应一套完整的 UI 视觉定义
pub struct UiLayoutConfig {
    pub base_resolution: (u32, u32),  // 基准分辨率 (1920, 1080)
    pub fonts: FontConfig,
    pub colors: ColorConfig,
    pub dialogue: DialogueLayoutConfig,
    pub title: TitleLayoutConfig,
    pub quick_menu: QuickMenuConfig,
    pub game_menu: GameMenuConfig,
    pub choice: ChoiceConfig,
    pub save_load: SaveLoadConfig,
    pub history: HistoryConfig,
    pub settings: SettingsConfig,
    pub confirm: ConfirmConfig,
    pub skip_indicator: SkipIndicatorConfig,
    pub notify: NotifyConfig,
    pub assets: UiAssetPaths,  // 所有 GUI 素材的路径映射
}
```

每个子 config 携带该页面/组件的全部布局参数（位置、尺寸、边距、字号等），其默认值直接来自 ref-project 的 `gui.rpy`。

#### ScaleContext

```rust
/// 分辨率缩放上下文
pub struct ScaleContext {
    base_width: f32,    // 1920.0
    base_height: f32,   // 1080.0
    actual_width: f32,
    actual_height: f32,
    scale_x: f32,       // actual / base
    scale_y: f32,
}

impl ScaleContext {
    /// 将基准坐标/尺寸缩放为实际值
    pub fn x(&self, base: f32) -> f32;
    pub fn y(&self, base: f32) -> f32;
    pub fn size(&self, base: f32) -> f32; // 取 min(scale_x, scale_y) 保持比例
}
```

#### UiAssetCache

```rust
/// GUI 图片素材缓存
pub struct UiAssetCache {
    textures: HashMap<String, egui::TextureHandle>,
}

impl UiAssetCache {
    /// 从 UiAssetPaths + ResourceManager 加载所有 GUI 素材
    pub fn load(paths: &UiAssetPaths, resource_manager: &ResourceManager,
                ctx: &egui::Context) -> Self;

    /// 获取已加载的纹理
    pub fn get(&self, key: &str) -> Option<&egui::TextureHandle>;
}
```

#### NinePatch 渲染

对于需要拉伸的 UI 元素（对话框、按钮背景、框架等），实现九宫格渲染：

```rust
pub struct NinePatch {
    texture: egui::TextureHandle,
    borders: Borders,  // left, top, right, bottom 不拉伸区域
}

impl NinePatch {
    /// 在指定矩形区域内渲染九宫格图片
    pub fn paint(&self, ui: &mut egui::Ui, rect: egui::Rect);
}
```

### 4.3 页面改造设计

#### 4.3.1 对话框 (InGame)

**现状**：`egui::TopBottomPanel::bottom` + 硬编码颜色/尺寸

**目标**：
- 使用 `textbox.png` 作为对话框背景（图片覆盖全底部区域）
- 使用 `namebox.png` 作为名字框背景
- 对话文本位置/宽度/字号从 `DialogueLayoutConfig` 读取
- 名字位置/字号从 config 读取
- 快捷菜单栏（回退/历史/快进/自动/保存/快存/快读/设置）叠加在对话框区域

**实现策略**：使用 egui `Area` + `Image` 绘制背景图片，在其上叠加文本 widget。不使用 `TopBottomPanel`（因其无法直接使用图片背景），改为在固定位置的 `Area` 中自行布局。

#### 4.3.2 标题画面 (Title)

**现状**：深色纯色背景 + 英文标题和按钮

**目标**：
- 全屏背景图片（季节切换：夏篇 `main_summer_edit.png` / 冬篇 `main_winter_edit.png`）
- 左侧导航按钮（中文：开始游戏/冬篇/读取游戏/设置/退出）
- 冬篇入口仅 `complete_summer` 后显示

#### 4.3.3 快捷菜单 (Quick Menu) —— 新增

**现状**：完全缺失

**目标**：
- 对话框区域内底部水平排列的按钮栏
- 按钮：回退(暂不支持)/历史/快进/自动/保存/快存/快读/设置
- 按钮使用 `quick_idle_background.png` / `quick_hover_background.png`
- 字号 21px

**说明**："回退"功能需要 runtime 支持 rollback（当前未实现），此按钮先渲染但禁用。

#### 4.3.4 游戏菜单框架 (Game Menu)

**现状**：各子页面独立使用 `CentralPanel` + `panel_frame()`

**目标**：
- 共享的游戏菜单框架（背景图 + 左侧导航面板 + 右侧内容区 + 标题标签）
- 背景图片季节切换
- 覆盖层 `overlay/game_menu.png`
- 左侧 420px 导航面板（历史/保存/读取/设置/标题界面/退出）
- 右侧内容区可选 viewport 滚动
- 返回按钮在左侧底部

#### 4.3.5 选项按钮 (Choice)

**现状**：基本选项列表，无图片背景

**目标**：
- 居中 1185px 宽按钮
- 使用 `choice_idle_background.png` / `choice_hover_background.png`
- 文本居中，间距 33px

#### 4.3.6 存读档 (SaveLoad)

**现状**：纯文本列表

**目标**：
- 3×2 网格布局
- 每个槽位 414×309px，含截图缩略图 (384×216px)
- 时间/存档名显示
- 分页导航（< A Q 1-9 >）
- 槽位按钮使用 `slot_idle/hover_background.png`

**前置**：需要实现存档截图功能（保存时截取当前画面缩略图）。

#### 4.3.7 设置 (Settings)

**现状**：基本滑块，缺少部分选项

**目标**：
- 显示模式选项（窗口/全屏）
- 跳过选项（未读文本/选项后继续/忽略转场）
- 滑块：文字速度/自动前进时间/音乐音量/音效音量
- 自定义滑块样式（`slider/` 素材）
- 全部静音按钮

#### 4.3.8 历史 (History)

**现状**：简单滚动文本

**目标**：
- 固定高度条目 (210px)
- 左列角色名（右对齐，233px 宽）+ 右列对话文本（左对齐，1110px 宽）
- 角色名颜色跟随角色定义

#### 4.3.9 确认弹窗 (Confirm) —— 新增

**现状**：完全缺失

**目标**：
- 模态覆盖层 `overlay/confirm.png`
- 居中框架 `frame.png`（九宫格拉伸）
- 提示文本 + 确定/取消按钮（间距 150px）
- 用于退出确认、覆盖存档确认、返回标题确认等

#### 4.3.10 快进指示器 (Skip Indicator) —— 新增

**现状**：无可视指示

**目标**：
- 左上角 `skip.png` 背景框
- "正在快进" + 闪烁箭头动画

### 4.4 配置文件格式

配置文件为 JSON，放置于 `assets/ui/layout.json`。缺失时使用内置默认值（等价于 ref-project 参数）。

```json
{
  "base_resolution": [1920, 1080],
  "fonts": {
    "text_font": "NotoSansSC-Regular.otf",
    "text_size": 33,
    "name_text_size": 45,
    "interface_text_size": 33,
    "title_text_size": 75
  },
  "colors": {
    "accent": "#ffffff",
    "idle": "#888888",
    "hover": "#ff9900",
    "selected": "#ffffff",
    "insensitive": "#7878787f",
    "text": "#000000",
    "interface_text": "#ffffff"
  },
  "dialogue": {
    "textbox_height": 278,
    "textbox_yalign": 1.0,
    "name_xpos": 360,
    "name_ypos": 0,
    "dialogue_xpos": 402,
    "dialogue_ypos": 75,
    "dialogue_width": 1116
  },
  "assets": {
    "textbox": "gui/textbox.png",
    "namebox": "gui/namebox.png",
    "title_bg_summer": "gui/prelude/main_summer_edit.png",
    "title_bg_winter": "gui/prelude/main_winter_edit.png",
    "menu_bg_summer": "gui/main_summer.jpg",
    "menu_bg_winter": "gui/main_winter.jpg",
    "overlay_game_menu": "gui/overlay/game_menu.png",
    "overlay_confirm": "gui/overlay/confirm.png",
    "frame": "gui/frame.png",
    "skip": "gui/skip.png",
    "notify": "gui/notify.png"
  }
}
```

### 4.5 素材复用策略

ref-project 的 `gui/` 目录素材直接复制到 `assets/gui/`，通过 `UiAssetPaths` 映射引用。这些素材通过 `ResourceManager` 加载（与背景/立绘使用相同的资源加载路径），在 egui 侧创建 `TextureHandle` 缓存。

### 4.6 与现有系统的关系

| 现有模块 | 关系 |
|---------|------|
| `ui/theme.rs` ThemeTokens | **保留并扩展**——颜色 token 合并到 `UiLayoutConfig.colors`，Theme 仍可作为不含图片的 fallback |
| `ui/skin.rs` UiSkinConfig | **废弃**——被 `UiLayoutConfig.assets` 替代，功能更完整 |
| `ui/theme_loader.rs` | **扩展**——改为加载完整 `UiLayoutConfig` |
| `egui_screens/helpers.rs` | **改造**——硬编码常量替换为从 `UiLayoutConfig` 读取 |
| `backend/egui_integration.rs` | **扩展**——支持额外的 font 加载（NotoSansSC） |
| `resources/` ResourceManager | **复用**——GUI 素材走统一资源路径 |

---

## 5. 风险

| 风险 | 等级 | 缓解 |
|------|------|------|
| egui 对图片背景支持有限（无内置九宫格） | 中 | 自行实现 NinePatch 渲染（egui `Painter` API 支持自定义绘制） |
| egui 布局模型与 Ren'Py 绝对定位模型差异大 | 中 | 对话框/菜单等关键组件使用 `Area`（绝对定位）而非 egui 布局容器 |
| 中文字体加载与 egui 文本渲染性能 | 低 | egui 已支持 CJK 字体，仅需在启动时加载字体文件 |
| GUI 素材文件较大影响加载时间 | 低 | 启动时批量加载 + 按需加载非关键素材 |
| 存档截图功能需要 GPU 回读 | 中 | 使用 wgpu 的 `read_buffer` 在 save 时截取帧缓冲（可后续阶段实现） |

---

## 6. 分阶段计划

### Phase 1：基础设施（UiLayoutConfig + ScaleContext + 素材加载） ✓

- [x] 定义 `UiLayoutConfig` 类型体系 + JSON 反序列化 → `host/src/ui/layout.rs`（17 个子结构，~1000 行）
- [x] 实现 `ScaleContext`（基准→实际分辨率缩放）→ 含 `x()`/`y()`/`uniform()`/`rect()`/`vec2()`
- [x] 实现 `UiAssetCache`（通过 ResourceSource 加载 GUI 图片 → egui TextureHandle）→ `host/src/ui/asset_cache.rs`
- [x] 实现 `NinePatch` 九宫格渲染 → `host/src/ui/nine_patch.rs`（`Borders` + 9 块 UV 切分）
- [x] 复制 ref-project `gui/` 素材到 `assets/gui/`（24 个核心素材文件，不含 `phone/` 子目录）
- [x] 在 `AppState.ui` 中集成 `layout: UiLayoutConfig` + `asset_cache: Option<UiAssetCache>`
- [x] `ResourceManager` 新增 `source()` 方法暴露底层 `ResourceSource`
- [x] 7 个单元测试覆盖 ScaleContext、HexColor 解析、JSON partial override、默认值校验
- [x] 验收：编译通过，237 个测试全部通过

### Phase 2：对话框 + 快捷菜单 ✓

- [x] 改造 `ingame.rs`：textbox.png 背景 + namebox.png 名字框（NinePatch）+ 数据驱动布局
- [x] 新增快捷菜单栏（历史/快进/自动/保存/快存/快读/设置，7 个按钮）
- [x] 选项按钮使用 `choice_idle/hover_background.png`（NinePatch）+ 居中文本 + 1185px 宽度
- [x] `EguiAction` 新增 `QuickSave`/`QuickLoad`/`ToggleSkip`/`ToggleAuto` 变体
- [x] 验收：编译通过

### Phase 3：标题画面 + 游戏菜单框架 ✓

- [x] 改造 `title.rs`：全屏背景图（main_summer.jpg）+ overlay + 左侧中文导航按钮
- [x] 抽取 `game_menu.rs`：通用框架（背景 + overlay + 左导航 + 右内容区 + 标题标签 + 返回按钮）
- [x] 改造 `ingame_menu.rs`：半透明覆盖 + 居中中文按钮列表
- [x] 验收：编译通过

### Phase 4：确认弹窗 ✓

- [x] 新增 `confirm.rs`：`ConfirmDialog` 数据结构 + 模态覆盖层 + NinePatch frame + 确定/取消按钮
- [x] `HostApp` 新增 `pending_confirm` 状态 + ShowConfirm 拦截逻辑
- [x] `EguiAction` 新增 `ShowConfirm { message, on_confirm }` 变体
- [x] 验收：编译通过

### Phase 5：存读档 + 设置 + 历史 ✓

- [x] 改造 `save_load.rs`：网格布局（N×cols）+ slot_idle/hover NinePatch + 缩略图占位 + 中文标签
- [x] 改造 `settings.rs`：中文标签 + 数据驱动字号/间距
- [x] 改造 `history.rs`：双列布局（角色名右对齐 + 对话文本左对齐）
- [x] 验收：编译通过

### Phase 6：指示器 + 收尾 ✓

- [x] 新增 `skip_indicator.rs`：左上角快进动画提示（NinePatch skip.png + 动态箭头）
- [x] Toast overlay Y 偏移调整为 68px（对齐 `notify_ypos`）
- [x] 清空 `helpers.rs`（硬编码常量 `DARK_BG`/`PANEL_BG`/`GOLD`/`dark_frame`/`panel_frame`/`menu_btn` 全部移除）
- [x] 更新模块摘要文档（`ui.md` + `egui_screens.md`）
- [x] RFC-010 状态标记为 Accepted
- [x] 验收：全部 237 + 273 = 510 个测试通过，编译仅 3 个预期 warning

### 后续工作（不在本 RFC 首期范围内）

已完成的架构为以下后续工作奠定基础：

| 项目 | 说明 | 优先级 | 状态 |
|------|------|--------|------|
| 季节切换逻辑 | `title.rs` 根据 `persistent_store.complete_summer` 选择 summer/winter 背景 | 高 | 待做 |
| ~~确认弹窗触发整合~~ | Exit/ReturnToTitle/SaveToSlot(非空)/DeleteSlot 触发 ShowConfirm | 高 | ✓ 已完成 |
| ~~`game_menu_frame` 采用~~ | save_load/settings/history 使用 game_menu_frame 包裹 | 中 | ✓ 已完成 |
| ~~存档删除按钮~~ | save_load 网格中每个非空槽位添加 DeleteSlot 按钮 | 中 | ✓ 已完成 |
| 分页导航 | save_load 底部分页按钮栏 (< 1-9 >) | 中 | 待做 |
| 冬篇入口 | 标题画面仅 `complete_summer` 后显示冬篇按钮 | 中 | 待做 |
| 存档截图 | 保存时截取当前画面缩略图填入 slot（需 GPU 帧回读） | 低 | 待做 |
| ~~`skin.rs` 清理~~ | 删除 `skin.rs`、`theme.rs`、`theme_loader.rs`、`helpers.rs`，移除 `UiContext` 中的 `skin`/`theme` 字段 | 低 | ✓ 已完成 |
| ~~Theme 兼容字段清理~~ | `Theme` 整体移除（颜色/尺寸全由 `UiLayoutConfig` 管理） | 低 | ✓ 已完成 |
| notify.png 背景 | Toast 使用 `notify.png` NinePatch 背景替代纯色 | 低 | 待做 |
| 自定义滑块样式 | 使用 `slider/` 素材替代 egui 原生滑块 | 低 | 待做 |
| NVL 模式 UI | 另开 RFC | -- | |

---

## 7. 实施记录

### 7.1 新增文件

| 文件 | 行数 | 说明 |
|------|------|------|
| `host/src/ui/layout.rs` | ~1000 | `UiLayoutConfig` 完整类型体系 + `ScaleContext` + `HexColor` + 手写 `Default` |
| `host/src/ui/asset_cache.rs` | ~90 | `UiAssetCache`：ResourceSource → image 解码 → egui TextureHandle |
| `host/src/ui/nine_patch.rs` | ~151 | `NinePatch` + `Borders`：9 块 UV 切分 + 独立绘制 |
| `host/src/egui_screens/confirm.rs` | ~157 | `ConfirmDialog` + 模态覆盖层渲染 |
| `host/src/egui_screens/game_menu.rs` | ~159 | 通用游戏菜单框架（左导航 + 右内容区） |
| `host/src/egui_screens/skip_indicator.rs` | ~64 | 快进指示器 + 动态箭头动画 |

### 7.2 改造文件

| 文件 | 变更要点 |
|------|---------|
| `host/src/ui/mod.rs` | 导出新模块；`UiContext::new` / `set_screen_size` 接受 `&UiLayoutConfig` |
| `host/src/app/mod.rs` | `UiSystems` 新增 `layout` + `asset_cache` 字段 |
| `host/src/resources/mod.rs` | `ResourceManager::source()` 方法暴露底层 `ResourceSource` |
| `host/src/host_app.rs` | 初始化 `UiAssetCache`；传递 layout/asset_cache/scale 给各 screen；`pending_confirm` 状态 |
| `host/src/egui_actions.rs` | 新增 `QuickSave`/`QuickLoad`/`ToggleSkip`/`ToggleAuto`/`ShowConfirm` 变体 |
| `host/src/egui_screens/ingame.rs` | 全面重写：NinePatch textbox/namebox/choice，快捷菜单栏 |
| `host/src/egui_screens/title.rs` | 全面重写：全屏背景 + overlay + 中文导航按钮 |
| `host/src/egui_screens/ingame_menu.rs` | 重写为数据驱动布局 + 中文标签 |
| `host/src/egui_screens/save_load.rs` | 重写为网格布局 + NinePatch slot 背景 |
| `host/src/egui_screens/settings.rs` | 中文标签 + 数据驱动字号/间距 |
| `host/src/egui_screens/history.rs` | 双列布局 + 数据驱动配置 |
| `host/src/egui_screens/toast.rs` | Y 偏移使用 `notify_ypos` |
| `host/src/egui_screens/helpers.rs` | 清空（旧常量/函数已废弃） |

### 7.3 与设计方案的偏差

| 设计 | 实际 | 原因 |
|------|------|------|
| `UiAssetCache::load` 通过 `ResourceManager` | 通过 `ResourceManager::source()` 获取底层 `ResourceSource` | `ResourceManager::read_bytes` 返回 `GpuTexture`，GUI 需要原始字节做 image 解码 |
| 选项按钮在 Phase 4 改造 | 随 Phase 2 对话框一并改造 | 选项是 `ingame.rs` 的一部分，拆分不自然 |
| `ingame_menu.rs` 使用 `game_menu_frame` | 保持独立半透明覆盖层 + 居中按钮 | 游戏暂停菜单和游戏菜单（设置/存档等）的交互范式不同；暂停菜单不需要左导航 |
| `game_menu_frame` 被 save_load/settings/history 采用 | ✓ 后续收尾阶段已采用 | 三个子页面均通过 `build_game_menu_frame` 包裹，共享左导航 |
| 分页导航 (< A Q 1-9 >) | 未实现 | 基本网格布局已可用，分页作为后续增强 |
| 存档缩略图 | 占位区域已预留，灰色背景填充 | 截图功能需 GPU 帧回读，另行实现 |
| 自定义滑块样式（`slider/` 素材） | 使用 egui 原生滑块 + 数据驱动尺寸 | 自定义滑块需较多 egui Painter 工作，优先保证功能完整 |
| `skin.rs` 废弃删除 | ✓ 后续收尾阶段已删除 | `skin.rs`、`theme.rs`、`theme_loader.rs`、`helpers.rs` 均已移除，`UiConfig` 配置段同步删除 |
| UI 定制文档 | 未撰写 | 系统已支持 JSON override，文档作为后续工作 |

### 7.4 已知限制

1. **季节切换未接入**：标题/菜单页面固定使用夏篇背景，未读取 persistent store 判断切换
2. ~~**确认弹窗未触发**~~ → ✓ 后续收尾已接入：退出/返回标题/覆盖存档/删除存档均经 `ShowConfirm` 确认
3. **快捷菜单"回退"缺失**：runtime 暂不支持 rollback，该按钮未添加
4. **字体未加载 NotoSansSC**：仍使用 egui 默认字体，CJK 字符依赖 egui 内置 fallback
5. ~~**3 个编译 warning**~~ → ✓ 后续收尾已消除：`DeleteSlot` 已有 UI 触发、`ShowConfirm` 已接入、`game_menu_frame` 已被采用

## 8. 验收标准

| # | 标准 | 状态 | 备注 |
|---|------|------|------|
| 1 | 所有核心页面使用图片素材 + 配置化布局，无硬编码颜色/尺寸常量 | ✓ | `helpers.rs` 硬编码已清空 |
| 2 | 标题画面呈现 ref-project 的季节背景 + 导航布局 | ◐ | 夏篇背景 ✓，季节切换待接入 persistent store |
| 3 | 对话框使用 `textbox.png` / `namebox.png`，文本位置与 ref-project 一致 | ✓ | NinePatch 渲染 |
| 4 | 快捷菜单可用且位于对话框区域 | ✓ | 7 个功能按钮 |
| 5 | 游戏菜单各子页面视觉风格统一且接近 ref-project | ✓ | 各页面通过 `game_menu_frame` 统一包裹，共享左导航 + 右内容区布局 |
| 6 | 选项按钮使用图片背景且居中显示 | ✓ | NinePatch choice 背景 |
| 7 | 确认弹窗功能完整（退出/覆盖存档/返回标题） | ✓ | 退出/返回标题/覆盖存档/删除存档均经 `ShowConfirm` 确认 |
| 8 | 无 `layout.json` 时 fallback 到内置默认值 | ✓ | `Default` 手写实现 |
| 9 | 窗口缩放时 UI 元素保持正确比例 | ✓ | `ScaleContext` 统一缩放 |

---

## 9. 相关 RFC

| RFC | 标题 | 关系 |
|-----|------|------|
| RFC-002 | ref-project 重制体验等价计划 | 本 RFC 是 P2-4 "UI 风格持续贴近原作"的具体实施方案 |
| RFC-004 | 扩展 API 与 Mod 化效果管理 | UI 定制化与 Mod 化的设计理念一致 |
| RFC-007 | 渲染后端迁移 | egui 是本 RFC 的 UI 框架基础 |
| RFC-008 | 渲染后端 Trait 抽象 | GUI 素材加载复用 TextureFactory |
