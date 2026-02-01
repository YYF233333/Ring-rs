# 存档格式说明

Ring-rs 的存档系统使用 JSON 格式，支持版本兼容性检测。

## 文件位置

```
saves/
├── slot_001.json
├── slot_002.json
└── ...
```

## 存档结构

```json
{
  "version": {
    "major": 1,
    "minor": 0
  },
  "metadata": {
    "slot": 1,
    "timestamp": "1738400000",
    "chapter_title": "第一章",
    "play_time_secs": 3600
  },
  "runtime_state": {
    "position": {
      "script_id": "test_comprehensive",
      "node_index": 15
    },
    "variables": {},
    "waiting": "WaitForClick",
    "visible_characters": {},
    "current_background": "backgrounds/scene.jpg"
  },
  "audio": {
    "current_bgm": "bgm/Signal.mp3",
    "bgm_looping": true
  },
  "render": {
    "background": "backgrounds/scene.jpg",
    "characters": [
      {
        "alias": "beifeng",
        "texture_path": "characters/北风.png",
        "position": "Center"
      }
    ]
  },
  "history": {
    "events": [...],
    "max_events": 1000
  }
}
```

## 字段说明

### version

存档格式版本号。

- `major`：主版本号，不兼容的格式变更时增加
- `minor`：次版本号，向后兼容的新字段时增加

**兼容性规则**：major 必须相同才能读取。

### metadata

存档元数据，用于 UI 显示。

| 字段 | 类型 | 说明 |
|------|------|------|
| `slot` | number | 存档槽位号 (1-99) |
| `timestamp` | string | 保存时间（Unix 时间戳） |
| `chapter_title` | string? | 当前章节标题 |
| `play_time_secs` | number | 游戏时长（秒） |

### runtime_state

Runtime 的核心状态。

| 字段 | 类型 | 说明 |
|------|------|------|
| `position.script_id` | string | 当前脚本 ID |
| `position.node_index` | number | 当前节点索引 |
| `variables` | object | 脚本变量 |
| `waiting` | string | 等待状态 |
| `visible_characters` | object | 当前显示的角色 |
| `current_background` | string? | 当前背景路径 |

### audio

音频状态，用于恢复 BGM。

| 字段 | 类型 | 说明 |
|------|------|------|
| `current_bgm` | string? | 当前 BGM 路径 |
| `bgm_looping` | boolean | BGM 是否循环 |

### render

渲染快照，用于恢复视觉状态。

| 字段 | 类型 | 说明 |
|------|------|------|
| `background` | string? | 背景路径 |
| `characters` | array | 可见角色列表 |

### history

历史记录，用于支持回看功能。

| 字段 | 类型 | 说明 |
|------|------|------|
| `events` | array | 历史事件列表 |
| `max_events` | number | 最大事件数 |

## 历史事件类型

```json
// 对话事件
{
  "Dialogue": {
    "speaker": "角色名",
    "content": "对话内容",
    "timestamp": 1738400000
  }
}

// 章节标记
{
  "ChapterMark": {
    "title": "第一章",
    "timestamp": 1738400000
  }
}

// 选择事件
{
  "ChoiceMade": {
    "options": ["选项A", "选项B"],
    "selected_index": 0,
    "timestamp": 1738400000
  }
}

// 背景切换
{
  "BackgroundChange": {
    "path": "backgrounds/scene.jpg",
    "timestamp": 1738400000
  }
}

// BGM 切换
{
  "BgmChange": {
    "path": "bgm/music.mp3",  // 或 null 表示停止
    "timestamp": 1738400000
  }
}
```

## 版本迁移

当 major 版本不兼容时：

1. 读取存档会返回 `IncompatibleVersion` 错误
2. 错误信息包含存档版本和当前版本
3. 可以选择删除旧存档或手动迁移

## 使用示例

### 快捷键操作

- **F5**：快速保存到槽位 1
- **F9**：快速读取槽位 1

### 手动管理

存档文件是纯文本 JSON，可以：

- 直接编辑修改状态
- 备份/恢复
- 在不同设备间传输

## 安全性

- 存档不含敏感信息
- 不执行任何代码
- 校验失败时不会崩溃，仅报错
