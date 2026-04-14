# RFC: ZIP 资源自定义协议

## 元信息

- 编号：RFC-031
- 状态：Superseded（host-tauri 已归档，由 RFC-033 Dioxus 迁移取代）
- 作者：Claude
- 日期：2026-03-28
- 相关范围：host-tauri（Rust 后端 + Vue 前端）、tools/asset-packer
- 前置：无

---

## 背景

`host-tauri` 的发布文档（`docs/authoring/resources.md`、`getting-started.md`）和打包工具（`cargo pack release`）承诺：发行版目录仅包含 `exe + game.zip + config.json`，不需要解压后的 `assets/` 目录。

但当前实现无法兑现这个承诺。后端脚本/JSON 读取已通过 `ResourceManager` 统一了 FS/ZIP 来源，但前端媒体资源（背景、立绘、音频、视频、Rule mask、主题 CSS）仍依赖 `get_assets_root` 返回的磁盘路径 + Tauri `convertFileSrc()` 生成 `asset://` URL。纯 ZIP 模式下该路径不存在，媒体加载会全部失败。

此外，`manifest.json` 在初始化时直接从磁盘路径读取，绕过了 `ResourceManager`，ZIP 模式下会静默退回默认值。

---

## 目标与非目标

### 目标

- 注册自定义 URI scheme 协议，让前端通过统一 URL 访问资源，后端透明处理 FS/ZIP 来源。
- 让 `cargo pack release` 生成的发行版（无 `assets/` 目录）可以独立运行。
- 修正 `manifest` 加载路径，走 `ResourceManager` 统一来源。
- 修正 `find_project_root()` 的 release 模式假设。

### 非目标

- 不改变脚本/JSON 等后端文本资源的读取方式（已通过 `ResourceManager` 正确工作）。
- 不改变 `debug_server` 的 HTTP 静态文件服务（浏览器调试模式仍走 `/assets/`）。
- 不实现资源缓存/预解压策略（由协议 handler 按请求即时读取）。

---

## 方案设计

### 1. 自定义协议注册

在 `lib.rs` 使用 `tauri::Builder::register_uri_scheme_protocol` 注册名为 `ring-asset` 的协议。

前端请求 URL 格式（跨平台由 Tauri 自动处理）：
- Windows/Android: `http://ring-asset.localhost/<logical-path>`
- macOS/Linux/iOS: `ring-asset://localhost/<logical-path>`

Handler 流程：
1. 从 `request.uri().path()` 提取路径，去除前导 `/`
2. 使用 `LogicalPath::new()` 规范化
3. 通过 `ResourceManager::read_bytes()` 读取（FS 或 ZIP 透明）
4. 按扩展名推断 MIME type
5. 构建 `http::Response` 返回字节

### 2. MIME 推断

在 `resources.rs` 新增 `guess_mime_type(path: &str) -> &'static str`，覆盖常见媒体类型：

| 扩展名 | MIME |
|--------|------|
| `.png` | `image/png` |
| `.jpg` `.jpeg` | `image/jpeg` |
| `.webp` | `image/webp` |
| `.gif` | `image/gif` |
| `.svg` | `image/svg+xml` |
| `.mp3` | `audio/mpeg` |
| `.ogg` | `audio/ogg` |
| `.wav` | `audio/wav` |
| `.mp4` | `video/mp4` |
| `.webm` | `video/webm` |
| `.json` | `application/json` |
| `.css` | `text/css` |
| 其他 | `application/octet-stream` |

### 3. 前端 URL 生成

修改 `useBackend.ts` 和 `useAssets.ts`：

- Tauri 模式下，`assetUrl()` 使用 `convertFileSrc(logicalPath, "ring-asset")` 生成协议 URL。`convertFileSrc` 的第二个参数指定协议名，会自动处理平台差异。
- 浏览器调试模式下，继续走 `http://localhost:9528/assets/...`。
- 不再需要 `get_assets_root` IPC 调用（init 阶段简化）。

### 4. Manifest 加载修正

`lib.rs` 中 manifest 加载改为通过 `ResourceManager::read_text()` 读取，然后用 `serde_json::from_str()` 解析。这样 ZIP 模式下 manifest 也能正常读取。

### 5. 项目根定位修正

`find_project_root()` 改为优先查找 `config.json`（而非 `assets/` 目录），因为 release 产物中始终有 `config.json` 但不一定有 `assets/`。

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host-tauri/src-tauri/src/lib.rs` | 注册协议 handler、修正 manifest 加载、修正项目根定位 | 中 |
| `host-tauri/src-tauri/src/resources.rs` | 新增 `guess_mime_type`、移除 `base_path` 过时注释 | 低 |
| `host-tauri/src/composables/useBackend.ts` | `resolveAssetSrc` 改为协议 URL | 低 |
| `host-tauri/src/composables/useAssets.ts` | 简化 `init`/`assetUrl`，不再依赖 `assetsRoot` | 低 |
| `host-tauri/src-tauri/src/commands.rs` | `get_assets_root` 保留但弱化（仅 debug 用途） | 低 |
| 前端 Vue 组件 | 不变（均通过 `assetUrl()` 消费） | 无 |
| `host-tauri/src/composables/useTheme.ts` | 不变（通过 `assetUrl()` / `fetch` 加载） | 低 |
| `debug_server.rs` | 不变（仅 debug build，仍走 FS 静态服务） | 无 |

---

## 迁移计划

1. 后端注册协议 + 补 MIME 推断（向后兼容，不影响现有 FS 模式）
2. 前端切换 URL 生成方式（FS 模式下协议 handler 仍能从磁盘读取，行为等价）
3. 修正 manifest 加载与项目根定位
4. 验证 release ZIP 独立运行

---

## Rejected Alternatives

**物化到缓存目录（materialize_to_fs）**：启动或按需时将 ZIP 内容解压到临时目录，前端继续通过 `convertFileSrc` 使用文件路径。虽然前端改动更小，但引入了磁盘占用管理、缓存失效、并发解压等复杂度，且本质上仍在回避"前端不应依赖磁盘路径"这个架构问题。自定义协议方案一次性解决根因，长期维护成本更低。

---

## 验收标准

- [ ] `cargo pack release --output-dir dist --zip` 生成的产物（无 `assets/` 目录）可独立运行
- [ ] 背景、立绘、音频（BGM/SFX）、视频、Rule mask、主题 CSS 在 ZIP 模式下均可正常加载
- [ ] FS 模式（开发期）行为不变
- [ ] 浏览器调试模式（`http://localhost:5173`）行为不变
- [ ] `manifest.json` 在 ZIP 模式下正确加载（不退回默认值）
- [ ] `cargo check-all` 通过
- [ ] 相关文档已更新
