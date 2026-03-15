# UI 定制指南

本文档面向游戏开发者，说明如何通过配置文件和素材替换来自定义引擎 UI 外观。

## 概述

引擎使用数据驱动的 UI 系统。所有布局参数、颜色、字号和图片素材路径均从 `assets/ui/layout.json` 读取。未提供 `layout.json` 时，自动使用内置默认值（对应 1920×1080 基准分辨率）。

你可以只覆盖需要修改的字段——未指定的字段会使用默认值。

> **行为定制**：若需要自定义按钮列表、动作映射、可见性条件或背景切换逻辑，请参阅 [UI 行为定制指南](screens_customization.md)。

## 配置文件

### 文件位置

```
assets/ui/layout.json
```

### 最小示例

```json
{
  "colors": {
    "hover": "#ff6600"
  },
  "dialogue": {
    "textbox_height": 300
  }
}
```

这只会修改悬停色和对话框高度，其余所有参数使用默认值。

### 完整结构

```json
{
  "base_width": 1920.0,
  "base_height": 1080.0,

  "fonts": {
    "text_size": 33.0,
    "name_text_size": 45.0,
    "interface_text_size": 33.0,
    "label_text_size": 36.0,
    "notify_text_size": 24.0,
    "title_text_size": 75.0
  },

  "colors": {
    "accent": "#ffffff",
    "idle": "#888888",
    "hover": "#ff9900",
    "selected": "#ffffff",
    "insensitive": "#7878787f",
    "text": "#000000",
    "interface_text": "#ffffff",
    "choice_idle": "#cccccc",
    "choice_hover": "#ffffff"
  },

  "dialogue": {
    "textbox_height": 278.0,
    "textbox_yalign": 1.0,
    "name_xpos": 360.0,
    "name_ypos": 0.0,
    "dialogue_xpos": 402.0,
    "dialogue_ypos": 75.0,
    "dialogue_width": 1116.0,
    "namebox_borders": [30.0, 20.0, 30.0, 20.0]
  },

  "choice": {
    "button_width": 1185.0,
    "ypos": 405.0,
    "spacing": 33.0,
    "button_borders": [25.0, 15.0, 25.0, 15.0]
  },

  "quick_menu": {
    "text_size": 21.0,
    "spacing": 6.0,
    "button_borders": [8.0, 6.0, 8.0, 6.0]
  },

  "title": {
    "navigation_xpos": 60.0,
    "navigation_spacing": 6.0
  },

  "game_menu": {
    "nav_width": 420.0,
    "navigation_spacing": 6.0
  },

  "save_load": {
    "cols": 3,
    "slot_width": 414.0,
    "slot_height": 309.0,
    "slot_spacing": 15.0,
    "thumbnail_width": 384.0,
    "thumbnail_height": 216.0
  },

  "history": {
    "entry_height": 210.0,
    "name_xpos": 233.0,
    "name_width": 233.0,
    "text_xpos": 255.0,
    "text_width": 1110.0
  },

  "settings": {
    "pref_spacing": 12.0
  },

  "confirm": {
    "button_spacing": 150.0,
    "frame_borders": [30.0, 30.0, 30.0, 30.0]
  },

  "skip_indicator": {
    "ypos": 15.0,
    "borders": [15.0, 10.0, 15.0, 10.0]
  },

  "notify": {
    "ypos": 68.0,
    "frame_borders": [15.0, 10.0, 15.0, 10.0]
  },

  "assets": {
    "textbox": "gui/textbox.png",
    "namebox": "gui/namebox.png",
    "frame": "gui/frame.png",
    "main_menu_overlay": "gui/overlay/main_menu.png",
    "game_menu_overlay": "gui/overlay/game_menu.png",
    "confirm_overlay": "gui/overlay/confirm.png",
    "skip": "gui/skip.png",
    "notify": "gui/notify.png",
    "main_summer": "gui/main_summer.jpg",
    "main_winter": "gui/main_winter.jpg",
    "game_menu_bg": "gui/game_menu.png",
    "button_idle": "gui/button/idle_background.png",
    "button_hover": "gui/button/hover_background.png",
    "choice_idle": "gui/button/choice_idle_background.png",
    "choice_hover": "gui/button/choice_hover_background.png",
    "slot_idle": "gui/button/slot_idle_background.png",
    "slot_hover": "gui/button/slot_hover_background.png",
    "quick_idle": "gui/button/quick_idle_background.png",
    "quick_hover": "gui/button/quick_hover_background.png",
    "slider_idle_bar": "gui/slider/horizontal_idle_bar.png",
    "slider_hover_bar": "gui/slider/horizontal_hover_bar.png",
    "slider_idle_thumb": "gui/slider/horizontal_idle_thumb.png",
    "slider_hover_thumb": "gui/slider/horizontal_hover_thumb.png"
  }
}
```

## 分辨率缩放

所有像素值基于 `base_width` × `base_height`（默认 1920×1080）。运行时通过 `ScaleContext` 自动等比缩放到实际窗口尺寸。你不需要针对不同分辨率准备多套配置。

## 素材替换

### 目录结构

素材默认位于 `assets/gui/`：

```
assets/gui/
├── textbox.png              # 对话框背景
├── namebox.png              # 名字框背景
├── frame.png                # 通用框架（确认弹窗）
├── main_summer.jpg          # 标题/菜单背景（夏篇）
├── main_winter.jpg          # 标题/菜单背景（冬篇）
├── game_menu.png            # 游戏菜单背景
├── skip.png                 # 快进指示器
├── notify.png               # Toast 通知背景
├── button/
│   ├── idle_background.png
│   ├── hover_background.png
│   ├── choice_idle_background.png
│   ├── choice_hover_background.png
│   ├── slot_idle_background.png
│   ├── slot_hover_background.png
│   ├── quick_idle_background.png
│   └── quick_hover_background.png
├── overlay/
│   ├── main_menu.png
│   ├── game_menu.png
│   └── confirm.png
└── slider/
    ├── horizontal_idle_bar.png
    ├── horizontal_hover_bar.png
    ├── horizontal_idle_thumb.png
    └── horizontal_hover_thumb.png
```

### 替换方法

1. 将替换素材放入 `assets/gui/` 对应路径（同名覆盖）
2. 或在 `layout.json` 的 `assets` 段指向新路径：

```json
{
  "assets": {
    "textbox": "my_custom_gui/textbox.png"
  }
}
```

### NinePatch 拉伸

以下控件使用九宫格（NinePatch）拉伸：

| 控件 | 素材 | borders 配置 |
|------|------|-------------|
| 对话框 (textbox) | `textbox.png` | 默认自动 |
| 名字框 (namebox) | `namebox.png` | `dialogue.namebox_borders` |
| 选项按钮 | `choice_idle/hover_background.png` | `choice.button_borders` |
| 存档槽位 | `slot_idle/hover_background.png` | 固定 15px |
| 快捷菜单按钮 | `quick_idle/hover_background.png` | `quick_menu.button_borders` |
| 确认框架 | `frame.png` | `confirm.frame_borders` |
| 快进指示器 | `skip.png` | `skip_indicator.borders` |
| Toast 通知 | `notify.png` | `notify.frame_borders` |
| 滑块轨道 | `slider/horizontal_*_bar.png` | 固定 6px |

**borders 格式**：`[left, top, right, bottom]`（像素值）。四角保持原始尺寸，边条和中心区域拉伸填充。

如果你的素材圆角大小不同，调整对应的 borders 值使圆角不被拉伸变形。

## 颜色格式

颜色使用 hex 格式：

- `#RRGGBB` — 不透明色
- `#RRGGBBAA` — 含 alpha 通道

示例：`"#ff9900"` 橙色，`"#7878787f"` 半透明灰。

## 字体

字体通过 `config.json` 的 `default_font` 字段配置（而非 `layout.json`）：

```json
{
  "default_font": "fonts/NotoSansSC-Regular.otf"
}
```

字体文件放在 `assets/` 目录下对应路径。引擎支持 TTF 和 OTF 格式。

## Fallback 行为

- 素材文件不存在 → 对应控件使用纯色 fallback 渲染
- `layout.json` 不存在 → 使用全部默认值
- `layout.json` 字段缺失 → 仅缺失字段使用默认值
- 字体文件不存在 → 使用 egui 内置 fallback 字体（CJK 字符可能显示为豆腐块）
