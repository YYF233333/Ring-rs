# UI 行为定制指南

本文档说明如何通过 `screens.json` 自定义各界面的按钮列表、动作映射、可见性条件和背景切换逻辑。

关于 UI 样式定制（布局、颜色、字号、素材），请参阅 [UI 定制指南](ui-customization.md)。

## 概述

引擎使用声明式 JSON 配置定义四个可定制界面的行为：

- **title**：标题页（背景、overlay、按钮列表）
- **ingame_menu**：游戏内暂停菜单（按钮列表）
- **quick_menu**：快捷菜单（按钮列表）
- **game_menu**：游戏菜单框架（背景、overlay、导航按钮、返回按钮）

**配置文件 `screens.json` 必须存在且包含所有字段**，缺失文件或字段将导致启动报错。仓库自带完整的默认配置文件，直接修改即可。

## 配置文件

### 文件位置

```
assets/ui/screens.json
```

### 修改示例

要自定义标题页按钮，直接编辑 `assets/ui/screens.json` 中的 `title.buttons` 字段。注意所有四个界面（`title`、`ingame_menu`、`quick_menu`、`game_menu`）都必须在文件中定义。

## 动作词汇表

每个按钮的 `action` 字段可以是以下字符串之一，或一个对象。

### 字符串动作

| 动作 ID | 说明 |
|---------|------|
| `start_game` | 开始新游戏 |
| `continue_game` | 从最近存档继续 |
| `open_load` | 打开读取界面 |
| `open_save` | 打开保存界面 |
| `navigate_settings` | 进入设置（压栈） |
| `navigate_history` | 进入历史（压栈） |
| `replace_settings` | 切换到设置（同级替换，用于游戏菜单） |
| `replace_history` | 切换到历史（同级替换，用于游戏菜单） |
| `quick_save` | 快速保存 |
| `quick_load` | 快速读取 |
| `toggle_skip` | 切换快进模式 |
| `toggle_auto` | 切换自动模式 |
| `go_back` | 返回上一页 |
| `return_to_title` | 返回标题画面 |
| `return_to_game` | 返回游戏 |
| `exit` | 退出游戏 |

### 对象动作

跳转到脚本中的指定标签开始新游戏：

```json
{ "action": { "start_at_label": "Winter" } }
```

### navigate vs replace

- `navigate_settings` / `navigate_history`：压入导航栈，适用于标题页和暂停菜单
- `replace_settings` / `replace_history`：同级替换当前页面，适用于游戏菜单的左侧导航

## 条件系统

`visible`（按钮可见性）和 `when`（条件背景）字段支持以下条件语法：

| 语法 | 含义 |
|------|------|
| 字段缺失 | 始终可见 |
| `"true"` | 始终可见 |
| `"$has_continue"` | 存在可继续的存档 |
| `"$persistent.KEY"` | 持久化变量 KEY 的值为 true |
| `"!$persistent.KEY"` | 持久化变量 KEY 的值不为 true |

## 确认弹窗

按钮定义中的 `confirm` 字段指定确认弹窗的文案。点击该按钮时先弹出确认对话框，用户确认后才执行动作。

```json
{ "label": "退出", "action": "exit", "confirm": "确定退出游戏？" }
```

## 条件背景

`title` 和 `game_menu` 支持条件化背景，按顺序匹配第一个满足条件的资源。最后一项通常不写 `when`，作为兜底。

```json
{
  "background": [
    { "when": "$persistent.complete_summer", "asset": "main_winter" },
    { "asset": "main_summer" }
  ]
}
```

## 完整示例

以下是与引擎默认行为等价的完整配置：

```json
{
  "title": {
    "background": [
      { "when": "$persistent.complete_summer", "asset": "main_winter" },
      { "asset": "main_summer" }
    ],
    "overlay": "main_menu_overlay",
    "buttons": [
      { "label": "开始游戏", "action": "start_game" },
      { "label": "冬篇", "action": { "start_at_label": "Winter" }, "visible": "$persistent.complete_summer" },
      { "label": "继续游戏", "action": "continue_game", "visible": "$has_continue" },
      { "label": "读取游戏", "action": "open_load" },
      { "label": "设置", "action": "navigate_settings" },
      { "label": "退出", "action": "exit", "confirm": "确定退出游戏？" }
    ]
  },
  "ingame_menu": {
    "buttons": [
      { "label": "继续", "action": "go_back" },
      { "label": "保存", "action": "open_save" },
      { "label": "读取", "action": "open_load" },
      { "label": "设置", "action": "navigate_settings" },
      { "label": "历史", "action": "navigate_history" },
      { "label": "返回标题", "action": "return_to_title", "confirm": "确定返回标题画面？" },
      { "label": "退出", "action": "exit", "confirm": "确定退出游戏？" }
    ]
  },
  "quick_menu": {
    "buttons": [
      { "label": "历史", "action": "navigate_history" },
      { "label": "快进", "action": "toggle_skip" },
      { "label": "自动", "action": "toggle_auto" },
      { "label": "保存", "action": "open_save" },
      { "label": "快存", "action": "quick_save" },
      { "label": "快读", "action": "quick_load" },
      { "label": "设置", "action": "navigate_settings" }
    ]
  },
  "game_menu": {
    "background": [
      { "when": "$persistent.complete_summer", "asset": "main_winter" },
      { "asset": "game_menu_bg" }
    ],
    "overlay": "game_menu_overlay",
    "nav_buttons": [
      { "label": "历史", "action": "replace_history" },
      { "label": "保存", "action": "open_save" },
      { "label": "读取", "action": "open_load" },
      { "label": "设置", "action": "replace_settings" },
      { "label": "返回标题", "action": "return_to_title", "confirm": "确定返回标题画面？" },
      { "label": "退出", "action": "exit", "confirm": "确定退出游戏？" }
    ],
    "return_button": { "label": "返回", "action": "return_to_game" }
  }
}
```

## 向后兼容

- `screens.json` 缺失 → 启动报错（须存在且字段完整，与概述一致）
- 字段缺失 → 对应界面使用默认值
- 未知 action ID → 启动时 warn 并跳过该按钮
- 条件求值失败 → 视为 false（按钮不显示）
