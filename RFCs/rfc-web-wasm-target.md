# RFC: Web (WASM) 发布目标

## 元信息

- 编号: RFC-037
- 状态: Proposed
- 作者: Yufeng Ying
- 日期: 2026-04-14
- 范围: host-dioxus, vn-runtime（仅审计）, tools/xtask
- 前置: RFC-033 (Dioxus 宿主迁移) — 已 Accepted

---

## 背景

当前 Ring Engine 仅支持 Desktop 平台（Windows/macOS/Linux）。重制完成后计划发布到 GitHub Pages，需要 Web (WASM) 作为第二发布目标。

**现状审计结果（2026-04-14）：**

- **vn-runtime**: 已完全 WASM 兼容（零 `std::time::SystemTime`、零 IO、零平台依赖）
- **host-dioxus**: ~45% 就绪。音频（Web Audio API）、UI 组件、事件处理天然兼容；资源加载、存档系统、debug server 需要替换
- **Phase 0 已完成**: feature flags (`desktop`/`web`)、`now_secs()` cfg 分支、debug_server cfg 门控

**关键阻塞项（均在 host-dioxus）：**

| 子系统 | 阻塞原因 | 当前实现 |
|--------|----------|---------|
| 资源加载 | `std::fs` 不可用 | `FsSource` / `ZipSource` 基于文件系统 |
| 存档 | `std::fs` 不可用 | `SaveManager` 基于文件系统 |
| 自定义协议 | `ring-asset://` 仅 Desktop wry | 通过 wry 自定义协议加载资源 |
| 应用入口 | `dioxus::desktop::Config` | Desktop 窗口配置 |
| 配置加载 | `std::fs::read_to_string` | 文件系统读取 `config.json` |

---

## 目标与非目标

### 目标

1. host-dioxus 可编译为 `wasm32-unknown-unknown` 目标，在现代浏览器中运行
2. GitHub Pages 可部署：`dx build --platform web` → 静态文件 → 直接托管
3. 功能对等：对话、演出、分支、音频、存档/读档均可用
4. Desktop 零回退：所有现有 Desktop 功能和测试不受影响

### 非目标

- **移动端 (iOS/Android)**: 不在本 RFC 范围，Dioxus mobile 尚不成熟
- **SSR / Fullstack**: 视觉小说无需服务端渲染
- **离线 PWA**: 首版不做 Service Worker 离线缓存
- **Web 版 debug server**: Desktop 专属，Web 通过浏览器 DevTools 调试
- **Web 版自定义协议**: 不复刻 `ring-asset://`，直接用 HTTP 相对路径

---

## 设计

### 1. 资源加载：`FetchSource`

新增 `ResourceSource` 实现，通过 Fetch API 从静态服务器加载资源。

```
Desktop:  LogicalPath → FsSource (std::fs::read)
Web:      LogicalPath → FetchSource (fetch → Response.arrayBuffer)
```

**方案选型：**

| 方案 | 优点 | 缺点 |
|------|------|------|
| A. 逐文件 fetch | 实现简单，按需加载 | 大量小文件时 HTTP 开销高 |
| B. ZIP 整包 fetch + 内存解压 | 单次下载，复用 `ZipSource` 逻辑 | 首次加载大，无法增量 |
| C. 混合：关键资源打包，其余按需 fetch | 平衡首屏和总体积 | 实现复杂度最高 |

**推荐 B（ZIP 整包 fetch + 内存解压）** 作为首版。理由：
- 游戏运行过程中不能容忍网络波动导致素材加载失败
- 单次下载后全部资源在内存中，运行时零网络依赖
- 复用现有 `ZipSource` 解压逻辑，改动最小
- 配合 HTTP 压缩（gzip/brotli）减小传输体积

**实现要点：**
- 启动时 `fetch` 下载资源 ZIP（`/assets.zip`）→ `Response.arrayBuffer()`
- 将 `ArrayBuffer` 交给 `zip` crate 内存解压（`ZipArchive<Cursor<Vec<u8>>>`）
- 复用现有 `ZipSource` 的 `ResourceSource` 实现，仅数据来源从文件改为内存
- GH Pages 部署时用 `asset-packer` 生成 ZIP，同时配置 HTTP 压缩
- 加载界面显示下载进度（ZIP 大小已知，可算百分比）

### 2. 存档系统：`WebSaveManager`

Desktop `SaveManager` 使用 `std::fs`，Web 端需要浏览器存储替代。

| 方案 | 容量 | 特点 |
|------|------|------|
| localStorage | ~5-10MB | 同步 API，简单，字符串 KV |
| IndexedDB | ~50MB+ | 异步 API，结构化，支持 Blob |

**推荐 localStorage** 作为首版。理由：
- 存档 JSON 通常 < 100KB，远低于容量上限
- API 简单（`setItem`/`getItem`），通过 `web-sys` 或 `document::eval` 调用
- 缩略图可用 base64 编码存入（或首版省略缩略图）

**接口设计：**
- 统一 trait `SaveBackend`，Desktop 和 Web 各实现一版
- 或直接用 `cfg` 在 `SaveManager` 内部分叉（代码量小时更实用）

### 3. 应用入口：Desktop / Web 分叉

```rust
// main.rs
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // 现有 Desktop 入口：tracing_subscriber, Config, WindowBuilder...
}

#[cfg(target_arch = "wasm32")]
fn main() {
    tracing_wasm::set_as_global_default();  // 或 console_log
    dioxus::launch(App);
}
```

`App()` 组件本身在两个目标上共享，差异仅在初始化和平台服务。

### 4. 自定义协议替代

Desktop 通过 `ring-asset://` 协议在 WebView 中加载图片/音频。Web 端无需此机制——直接使用 HTTP 相对路径：

```
Desktop RSX:  img { src: "ring-asset://backgrounds/school.png" }
Web RSX:      img { src: "/assets/backgrounds/school.png" }
```

**实现：** 资源 URL 生成函数按 cfg 返回不同前缀。

### 5. 配置加载

Desktop 从文件系统读 `config.json`。Web 端选项：

- **A. fetch 加载**：`GET /assets/config.json`，与资源系统复用
- **B. 编译时嵌入**：`include_str!("../../assets/config.json")`
- **推荐 A**：允许部署后修改配置而无需重新编译

### 6. 无需改动的子系统

| 子系统 | 原因 |
|--------|------|
| 音频 (`audio_bridge.rs`) | 已使用 Web Audio API via JS |
| 键盘输入 | 已使用 `document::eval()` JS 注入 |
| UI 组件 (screens/, components/) | 纯 Dioxus RSX，平台无关 |
| vn-runtime | 纯逻辑，已审计通过 |

---

## 影响

| 模块 | 改动类型 | 风险 |
|------|----------|------|
| `host-dioxus/Cargo.toml` | feature flags（已完成） | 低 |
| `host-dioxus/src/main.rs` | 入口分叉 + 协议替代 | 中 |
| `host-dioxus/src/resources.rs` | 新增 `FetchSource` | 中 |
| `host-dioxus/src/save_manager.rs` | Web 存储适配 | 中 |
| `host-dioxus/src/config.rs` | 配置加载适配 | 低 |
| `host-dioxus/src/init.rs` | 初始化流程适配 | 中 |
| `tools/xtask` | 可选：添加 `cargo web-build` alias | 低 |
| `vn-runtime` | 无改动（已就绪） | 无 |

---

## 迁移计划

### Phase 0 — 基础设施 ✅ 已完成

- [x] vn-runtime 移除 `SystemTime::now()`，时间戳由 Host 注入
- [x] `now_secs()` cfg 分支（SystemTime / js_sys）
- [x] Feature flags: `desktop`（默认）/ `web`
- [x] Debug server cfg 门控

### Phase 1 — 最小可运行

- [ ] Desktop/Web 入口分叉（`fn main()` cfg 分支）
- [ ] 资源 URL 前缀统一（`ring-asset://` vs `/assets/`）
- [ ] `config.json` 加载适配
- [ ] `find_project_root()` Web 端 stub
- [ ] 验证：`dx build --platform web` 编译通过

### Phase 2 — 资源系统

- [ ] 启动时 fetch 下载资源 ZIP，内存解压为 `ZipSource`
- [ ] `ZipSource` 适配 `Cursor<Vec<u8>>`（从文件句柄改为内存 buffer）
- [ ] 加载界面显示下载进度
- [ ] 验证：Web 端可加载并显示第一个场景

### Phase 3 — 持久化

- [ ] `SaveManager` Web 实现（localStorage）
- [ ] 存档/读档/继续 功能可用
- [ ] 缩略图处理（base64 或省略）
- [ ] 验证：Web 端可存档、刷新页面后继续

### Phase 4 — 收尾与部署

- [ ] tracing 日志适配（`tracing-wasm` 或 `console_log`）
- [ ] GH Pages 部署流水线（GH Actions）
- [ ] 跨浏览器基础测试（Chrome, Firefox, Safari）
- [ ] 验证：部署到 GH Pages 并完整跑通一个场景

### 向后兼容

- Desktop 功能不受影响：`default = ["desktop"]` 确保默认行为不变
- 存档格式不变：`SaveData` JSON 结构相同，仅存储后端不同
- 资源目录结构不变：Web 端直接部署 `assets/` 目录

---

## 验收标准

- [ ] `cargo check-all` 全通过（Desktop 目标）
- [ ] `dx build --platform web` 编译成功
- [ ] Web 端可加载脚本、显示对话、播放音频、执行分支
- [ ] Web 端存档/读档可用（localStorage）
- [ ] Desktop 全部现有测试仍通过
- [ ] 成功部署到 GH Pages 并可访问
