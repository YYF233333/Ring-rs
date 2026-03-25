# Tauri 调试文档

## Agent 调试流程

### 1. 启动开发服务器

在终端运行：

```bash
cd host-tauri; pnpm tauri dev
```

等待终端出现 `Debug HTTP server: http://127.0.0.1:9528` 和 Vite 的 `Local: http://localhost:5173/` 即启动完成。

### 2. 查看日志

前后端日志统一输出到 stdout（运行 `pnpm tauri dev` 的终端）。

**前端日志**格式（通过 `useLogger` → `log_frontend` IPC 转发到 Rust tracing）：

```
INFO frontend{module=engine}: startGame called with script_path="scripts/main.vns"
WARN frontend{module=settings}: 设置项不存在: foo
```

识别要点：`target: "frontend"`、大括号内的 `module=xxx` 标识来源组件。

**后端日志**是标准 tracing 格式：

```
INFO ring_engine_frontend: 项目根目录 root="F:\\Code\\Ring-rs"
INFO ring_engine_frontend: Debug HTTP server: http://127.0.0.1:9528
```

### 3. 浏览器调试（Headless 模式）

通过 browser MCP 在外部浏览器中进行可视化调试。**必须使用 Headless 模式**以避免 Tauri 窗口和浏览器双客户端竞争状态。

**启动方式：**

```powershell
cd host-tauri; $env:RING_HEADLESS="1"; pnpm tauri dev
```

设置 `RING_HEADLESS` 环境变量后，Tauri 窗口会自动隐藏，仅保留 Rust 后端和 Debug HTTP Server。外部浏览器成为唯一的游戏客户端。

**调试操作：**

1. 打开 `http://localhost:5173` — 前端自动检测非 Tauri 环境，回退到 HTTP API
2. 可执行的操作：
   - 截图查看渲染结果
   - 点击按钮、输入文本
   - 查看 DOM 结构
3. 在浏览器控制台获取状态快照：

```javascript
await fetch('http://localhost:9528/api/debug_snapshot', { method: 'POST' }).then(r => r.json())
```

> **警告**：不设置 `RING_HEADLESS` 直接在浏览器打开 `localhost:5173` 会导致 Tauri 窗口和浏览器各自运行独立的游戏循环，竞争同一个 `AppStateInner`，产生状态不同步、打字机效果碎裂、游戏双倍速推进等问题。

### 4. 状态快照

`debug_snapshot` 命令返回引擎内部状态的 JSON 概览，用于快速判断问题所在。

**通过 curl 调用：**

```bash
curl http://localhost:9528/api/debug_snapshot -X POST
```

**返回字段说明：**

| 字段 | 类型 | 说明 |
|------|------|------|
| `has_runtime` | bool | 是否已初始化 VN Runtime |
| `render_state` | object | 当前完整渲染状态（对话、立绘、背景等） |
| `playback_mode` | string | 播放模式（`Normal` / `Auto` / `Skip`） |
| `history_count` | number | 历史记录条数 |
| `has_audio` | bool | AudioManager 是否可用 |
| `current_bgm` | string? | 当前播放的 BGM 路径（null 表示无 BGM） |
| `user_settings` | object | 用户设置（音量、文字速度等） |

### 5. 常用诊断场景

| 症状 | 诊断步骤 |
|------|---------|
| 画面空白 | 检查 `has_runtime` 是否为 true；查看终端有无脚本解析错误 |
| 对话不推进 | 检查 `playback_mode`；通过 `render_state` 查看当前指令 |
| 没有声音 | 检查 `has_audio`；查看 `current_bgm` 和终端音频相关日志 |
| 资源加载失败 | 浏览器访问 `http://localhost:9528/assets/{path}` 验证资源是否存在 |

---

## 人类 Bug 反馈流程

### 报告 Bug 时请提供

1. **复现步骤**：在哪个脚本、执行了什么操作（如 "加载 `scripts/chapter1.vns` 后点击三次，对话框消失"）
2. **终端日志**：复制终端中的相关错误输出。前端日志会标记 `frontend`，后端日志是普通 tracing 格式
3. **浏览器 Console**：按 F12 打开 DevTools → Console，截取或复制错误信息
4. **可选截图**：如果是视觉问题（布局错位、颜色异常等），附上截图

### Agent 处理 Bug 报告的流程

1. **读终端日志** — 通过读取终端文件定位错误信息和调用栈
2. **获取状态快照** — 如需深入了解引擎内部状态：
   ```bash
   curl http://localhost:9528/api/debug_snapshot -X POST
   ```
3. **浏览器验证**（如 browser MCP 可用）— 打开 `http://localhost:5173` 复现问题并观察 DOM/网络请求
4. **定位源码** — 根据日志和状态信息，定位到具体模块：
   - 前端渲染问题 → `host-tauri/src/` 下的 Vue 组件
   - 后端逻辑问题 → `host-tauri/src-tauri/src/state.rs`（业务逻辑）
   - 脚本解析问题 → `vn-runtime/` 的 parser 模块
   - 命令执行问题 → `host-tauri/src-tauri/src/command_executor.rs`
5. **修复 → 测试** — 修改代码后运行 `cargo check` 和相关测试验证
