# 小游戏集成指南

> 本文档说明如何在 VN 脚本中集成 HTML5 小游戏。

## 快速开始

### 1. 创建小游戏目录

```
assets/games/
└── my_game/
    ├── index.html    # 入口文件
    ├── game.js       # 游戏逻辑
    └── assets/       # 游戏资源
```

### 2. 在脚本中调用

```markdown
callGame "my_game" as $result
if $result == "win"
  太好了，你赢了！
else
  没关系，下次再试。
endif
```

### 3. 实现 index.html

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body { margin: 0; background: #000; color: #fff; }
        canvas { display: block; }
    </style>
</head>
<body>
    <canvas id="game"></canvas>
    <script src="game.js"></script>
</body>
</html>
```

### 4. 使用 Engine API

```javascript
// game.js

// 播放音效（通过引擎音频系统）
window.ipc.postMessage(JSON.stringify({
    type: "playSound",
    name: "hit.mp3"
}));

// 读取游戏变量
window.ipc.postMessage(JSON.stringify({
    type: "getState",
    key: "player_level"
}));

// 游戏结束时通知引擎
function gameOver(score) {
    window.ipc.postMessage(JSON.stringify({
        type: "onComplete",
        result: score > 100 ? "win" : "lose"
    }));
}
```

## 脚本语法

### callGame

```markdown
callGame "game_id" as $result_var
callGame "game_id" as $result_var (param1: value1, param2: "str")
```

- `game_id`：对应 `assets/games/{game_id}/` 目录
- `$result_var`：存储游戏结果的变量名
- 参数列表可选，传递给小游戏的初始化参数

### showMap

```markdown
showMap "world_map" as $destination
```

- `map_id`：对应 `assets/maps/{map_id}.json` 文件
- 显示地图界面，等待用户选择位置
- 选择结果存入 `$destination`

## JS Bridge 协议

### Engine → Game 请求

| type | 字段 | 说明 |
|------|------|------|
| `playSound` | `name` | 播放音效 |
| `playBGM` | `name`, `loop` | 播放背景音乐 |
| `getState` | `key` | 读取脚本变量 |
| `setState` | `key`, `value` | 写入脚本变量 |
| `getAssetUrl` | `path` | 获取资源 URL |
| `log` | `level`, `message` | 诊断日志 |
| `onComplete` | `result` | 游戏结束，回传结果 |

### Engine 响应格式

```json
{"success": true, "data": 42}
{"success": false, "error": "variable not found"}
```

## 地图数据格式

`assets/maps/{map_id}.json`：

```json
{
    "title": "世界地图",
    "background": "maps/world_bg.png",
    "locations": [
        {
            "id": "beach",
            "label": "海边",
            "x": 300,
            "y": 500,
            "enabled": true
        },
        {
            "id": "mountain",
            "label": "山顶",
            "x": 800,
            "y": 300,
            "enabled": true,
            "condition": "unlocked_mountain"
        }
    ]
}
```

- `x`, `y`：基准 1920x1080 分辨率的坐标
- `condition`：可选，变量名为 true 时位置可用

## 降级策略

**GUI 模式 WebView 创建失败时**：立即返回空字符串结果，脚本可通过 `$result == ""` 判断。

**Headless 模式（无窗口测试）**：
- WebView 不可用，`callGame` 跳过启动
- 游戏结果由录制文件（replay）中的 `UIResult` 事件提供，确保分支路径与录制时一致
- 录制系统自动捕获所有 `UIResult` 事件（callGame / showMap 等），无需额外配置

## 文本模式

### textMode

```markdown
textMode nvl    # 切换到 NVL 模式（全屏文本累积）
textMode adv    # 切换回 ADV 模式（底部对话框）
```

NVL 模式下：
- 对话文本在半透明全屏背景上逐行累积
- `textBoxClear` 清空已累积的文本
- 切换回 ADV 模式自动清空累积文本
