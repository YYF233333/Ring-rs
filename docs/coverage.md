# 覆盖率（Coverage）口径与本地运行指南

本项目暂不强制 CI，以**本地可复现**为主。覆盖率的目标是：把"能跑"提升为"可度量"，并优先把 `vn-runtime` 覆盖率做到接近 100%。

---

## 口径（我们统计什么）

- **主口径（Primary）**：`vn-runtime` 覆盖率
  - 统计类型：以 **line coverage** 为主（必要时补充 branch coverage）
  - 原因：`vn-runtime` 不依赖窗口/音频/渲染设备，最适合做到高覆盖率，并且对逻辑回归最敏感
- **次口径（Secondary）**：workspace 覆盖率（趋势观察）
  - 说明：`host` 受图形设备/平台差异影响较大，不追求"接近 100%"，更关注关键链路的 headless 测试与"可测试边界"
  - RFC-008 引入了 `NullTexture` / `NullTextureFactory`，使 `renderer::build_draw_commands`、`ResourceManager::load_texture`、`CommandExecutor` 集成链路可在无 GPU 环境下测试

---

## 工具选择（Windows 优先）

### 推荐：`cargo llvm-cov`（主口径）

在 Windows 上更稳定、生态更匹配（基于 LLVM 工具链）。

#### 安装

```bash
cargo install cargo-llvm-cov
rustup component add llvm-tools-preview
```

#### 生成报告

- **vn-runtime 覆盖率（主口径）**：
  - `cargo cov-runtime`
- **workspace 覆盖率（次口径）**：
  - `cargo cov-workspace`

#### 报告输出位置

- HTML 报告默认输出到：
  - `target/llvm-cov/html/index.html`

---

## exclude 策略

### 排除原则

CLAUDE.md 定义了"不值得测试"的代码类型：

- **derive trait**：`Default`/`Clone`/`Debug` 等编译器生成代码
- **serde 序列化**：序列化正确性是 serde 的责任
- **thiserror Display**：`#[error("...")]` 生成的 Display impl
- **简单 getter/setter**：无分支逻辑的单行方法
- **三方框架行为**：egui UI 构建、wgpu 渲染管线、rodio 播放

这些代码计入覆盖率只会稀释信号，不提供回归价值。

### cov-runtime 排除（`--ignore-filename-regex`）

| 排除文件 | 理由 |
|----------|------|
| `vn-runtime/src/lib.rs` | 纯模块声明与重导出，无可执行逻辑 |
| `vn-runtime/src/error.rs` | 100% thiserror derive，无手写逻辑 |

### cov-workspace 排除

**crate 级排除**（`--exclude`）：

| 排除 crate | 理由 |
|------------|------|
| `xtask` | 开发工具，不是产品代码 |
| `asset-packer` | 打包工具，不是产品代码 |

**文件级排除**（`--ignore-filename-regex`），在 vn-runtime 排除基础上额外排除：

| 排除文件/目录 | 理由 |
|---------------|------|
| `*/lib.rs` | 两个 crate 的模块声明与重导出 |
| `host/src/main.rs` | 平台入口，依赖 winit EventLoop |
| `host/src/host_app.rs` | winit ApplicationHandler 事件循环胶水 |
| `host/src/egui_actions.rs` | winit ActiveEventLoop 依赖 |
| `host/src/egui_screens/*` | egui UI 构建逻辑（三方框架行为） |
| `host/src/backend/*`（除 `math.rs`） | GPU 渲染管线，需真实 wgpu 设备 |
| `host/src/audio/playback.rs` | rodio 硬件依赖的音频播放 |
| `host/src/app/draw.rs` | 渲染管线薄包装，仅转发调用 |

排除规则定义在 `tools/xtask/src/main.rs` 的 `COV_RUNTIME_IGNORE` 和 `COV_WORKSPACE_IGNORE` 常量中。
