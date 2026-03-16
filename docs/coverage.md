# 覆盖率（Coverage）口径与本地运行指南

本项目暂不强制 CI，以**本地可复现**为主。覆盖率统一使用 **workspace 口径**（排除工具 crate 与平台胶水代码），用于趋势观察与回归信号。

---

## 工具选择（Windows 优先）

### 推荐：`cargo llvm-cov`

在 Windows 上更稳定、生态更匹配（基于 LLVM 工具链）。

#### 安装

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

#### 生成报告

- `cargo cov`（等价于 `cargo run -p xtask -- cov`）
- HTML 报告：`target/llvm-cov/html/index.html`

---

## 排除策略

### 排除原则

CLAUDE.md 定义了"不值得测试"的代码类型：

- **derive trait**：`Default`/`Clone`/`Debug` 等编译器生成代码
- **serde 序列化**：序列化正确性是 serde 的责任
- **thiserror Display**：`#[error("...")]` 生成的 Display impl
- **简单 getter/setter**：无分支逻辑的单行方法
- **三方框架行为**：egui UI 构建、wgpu 渲染管线、rodio 播放

这些代码计入覆盖率只会稀释信号，不提供回归价值。

### crate 级排除（`--exclude`）

| 排除 crate | 理由 |
|------------|------|
| `xtask` | 开发工具，不是产品代码 |
| `asset-packer` | 打包工具，不是产品代码 |

### 文件级排除（`--ignore-filename-regex`）

规则定义在 `tools/xtask/src/main.rs` 的 `COV_IGNORE` 常量中。

| 排除文件/目录 | 理由 |
|---------------|------|
| `*/src/lib.rs` | 模块声明与重导出，无可执行逻辑 |
| `vn-runtime/src/error.rs` | thiserror derive，无手写逻辑 |
| `host/src/main.rs` | 平台入口，依赖 winit EventLoop |
| `host/src/host_app.rs` | winit ApplicationHandler 事件循环胶水 |
| `host/src/egui_actions.rs` | winit ActiveEventLoop 依赖 |
| `host/src/egui_screens/*` | egui UI 构建逻辑（三方框架行为） |
| `host/src/backend/*`（除 `math.rs`） | GPU 渲染管线，需真实 wgpu 设备 |
| `host/src/audio/*` | rodio 硬件依赖，含设备初始化与播放控制 |
| `host/src/video/*` | FFmpeg 子进程 + rodio 解码，完全硬件依赖 |
| `host/src/app/draw.rs` | 渲染管线薄包装，仅转发调用 |
| `host/src/app/(bootstrap\|init\|mod\|state\|save\|script_loader\|engine_services).rs` | App 层集成胶水，依赖完整 AppState |
| `host/src/app/update/*` | 每帧更新分发，依赖完整 AppState |
| `host/src/app/command_handlers/audio.rs` | 音频命令处理，依赖硬件 |
| `host/src/ui/(asset_cache\|image_slider\|mod\|nine_patch).rs` | egui UI 组件 |
| `host/src/extensions/manifest.rs` | 极小 serde struct |
