# 覆盖率（Coverage）口径与本地运行指南

本项目暂不强制 CI，以**本地可复现**为主。覆盖率的目标是：把“能跑”提升为“可度量”，并优先把 `vn-runtime` 覆盖率做到接近 100%。

---

## 口径（我们统计什么）

- **主口径（Primary）**：`vn-runtime` 覆盖率
  - 统计类型：以 **line coverage** 为主（必要时补充 branch coverage）
  - 原因：`vn-runtime` 不依赖窗口/音频/渲染设备，最适合做到高覆盖率，并且对逻辑回归最敏感
- **次口径（Secondary）**：workspace 覆盖率（趋势观察）
  - 说明：`host` 受 macroquad/图形设备/平台差异影响较大，不追求“接近 100%”，更关注关键链路的 headless 测试与“可测试边界”

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

## exclude 策略（为什么 workspace 覆盖率会“很低”）

workspace 覆盖率很容易被以下内容稀释：

- `tools/*`（工具 crate）
- `host` 的平台入口/渲染薄壳（受环境影响，且很多逻辑并不适合做细粒度覆盖率驱动）

因此我们在 `cov-workspace` 中排除了：
- `xtask`
- `asset-packer`

> 未来如果需要更精细的 exclude（例如忽略 `host/src/main.rs` 入口、或某些平台适配代码），再在阶段 21.1 中逐步补齐口径配置。

