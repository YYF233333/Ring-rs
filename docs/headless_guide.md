# Headless 测试模式使用指南

本文档说明 Ring VN Engine 的 headless 模式用法，覆盖输入录制、无窗口回放和结构化事件流输出。

## 概述

Headless 模式是一条完整的 AI 调试管线：

```
人类游玩（GUI） → F8 导出录制 → headless 快进回放 → 结构化事件流 → AI 分析
```

三个核心子系统：

| 子系统 | RFC | 说明 |
|--------|-----|------|
| 输入录制 | RFC-016 | 后台无感录制输入事件，F8 一键导出 |
| 事件流 | RFC-018 | 在引擎关键边界点产出 JSON Lines 格式的结构化事件 |
| Headless | RFC-019 | 无窗口/无 GPU 环境运行引擎状态机 |

## 输入录制

### 配置

在 `config.json` 的 `debug` 节中设置：

```json
{
  "debug": {
    "recording_buffer_size_mb": 8,
    "recording_output_dir": "recordings"
  }
}
```

- `recording_buffer_size_mb`：缓冲区大小（MB），`0` 禁用录制
- `recording_output_dir`：导出目录

### 使用

1. 正常运行游戏（GUI 模式）
2. 输入事件自动在后台录制到环形缓冲区
3. 按 **F8** 导出当前缓冲区内容到 `recordings/recording_YYYYMMDD_HHMMSS.jsonl`

### 录制文件格式

JSON Lines 格式（`.jsonl`），首行为元数据，后续每行一条输入事件：

```jsonl
{"version":1,"logical_width":1280,"logical_height":720,"engine_version":"0.1.0","recorded_at":"2026-03-19T12:00:00+08:00","duration_ms":5000,"entry_script":"scripts/main.md"}
{"t_ms":0,"event":{"type":"KeyPress","key":"Space"}}
{"t_ms":500,"event":{"type":"MousePress","button":"Left","x":640.0,"y":360.0}}
{"t_ms":1000,"event":{"type":"KeyRelease","key":"Space"}}
```

## Headless 回放

### CLI 参数

```bash
host --headless --replay-input=recordings/recording.jsonl [options]
```

| 参数 | 说明 | 默认值 |
|------|------|--------|
| `--headless` | 启用 headless 模式 | - |
| `--replay-input=<path>` | 输入录制文件路径（headless 必须） | - |
| `--event-stream=<path>` | 事件流输出文件路径 | `events.jsonl` |
| `--exit-on=<condition>` | 退出条件 | `replay-end` |
| `--max-frames=<n>` | 最大帧数限制 | 无限制 |
| `--timeout-sec=<n>` | 超时秒数 | 无限制 |

### 退出条件

- `replay-end`：回放数据耗尽时退出
- `script-finished`：脚本执行完毕时退出

### 运行环境

- **无窗口**：不创建 OS 窗口，不需要显示设备
- **无 GPU**：使用 `NullTextureFactory` 替代 wgpu 后端
- **无音频设备**：`AudioManager` 以 headless 模式运行（状态追踪正常，跳过 rodio I/O）
- **固定帧率**：60 FPS 固定 dt，全速推进（不受真实时间限制）

## 结构化事件流

### 启用

- **Headless 模式**：默认启用，输出到 `--event-stream` 指定路径
- **GUI 模式**：通过 `--event-stream=<path>` CLI 参数启用

### 事件类型

| 事件 | 说明 |
|------|------|
| `ScriptTick` | Runtime tick 完成，包含节点索引、命令数、等待原因 |
| `CommandProduced` | 命令生成，包含变体名和概要 |
| `CommandExecuted` | 命令执行完成，包含结果 |
| `StateChanged` | 状态变更（如 waiting_reason 变化） |
| `InputReceived` | 输入事件产出 RuntimeInput |
| `TransitionUpdate` | 过渡效果完成 |
| `AudioEvent` | 音频命令执行（BGM/SFX） |

### 输出格式

JSON Lines，每行一个事件：

```jsonl
{"ts_ms":0,"event":"ScriptTick","data":{"node_index":0,"commands_count":3,"waiting_reason":"WaitForClick"}}
{"ts_ms":0,"event":"CommandProduced","data":{"variant":"ShowBackground","summary":"..."}}
{"ts_ms":16,"event":"InputReceived","data":{"variant":"Click"}}
```

## AI 调试工作流示例

```bash
# 1. 玩家游玩时遇到 bug，按 F8 导出录制
# 录制文件自动保存到 recordings/ 目录

# 2. 以 headless 模式回放
host --headless --replay-input=recordings/recording_20260319_120000.jsonl --event-stream=debug_events.jsonl

# 3. 将事件流提供给 AI 分析
# AI 可以阅读 debug_events.jsonl 中的结构化事件
# 追踪命令执行链路、状态变更、输入时序，定位 root cause
```
