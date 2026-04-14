# `config.json` 配置说明

本文解释仓库根目录 `config.json` 中各项配置的含义与使用场景。

> **重要**：`config.json` 必须存在且包含所有字段，缺失文件或字段将导致启动报错。
> 仓库自带完整的默认配置文件，直接修改即可。
>
> 配置读取与校验实现：`host-dioxus/src/config.rs`

## 快速示例

### 开发模式（从文件系统加载资源）

```json
{
  "name": "My VN",
  "assets_root": "assets",
  "saves_dir": "saves",
  "manifest_path": "manifest.json",
  "default_font": "fonts/simhei.ttf",
  "start_script_path": "scripts/main.md",
  "asset_source": "fs",
  "zip_path": null,
  "window": { "width": 1280, "height": 720, "title": "My VN", "fullscreen": false },
  "debug": { "script_check": true, "log_level": "info", "log_file": null },
  "audio": { "master_volume": 1.0, "bgm_volume": 0.8, "sfx_volume": 1.0, "muted": false },
  "resources": { "texture_cache_size_mb": 256 }
}
```

### 发布模式（从 ZIP 加载资源）

```json
{
  "name": "My VN",
  "assets_root": "assets",
  "saves_dir": "saves",
  "manifest_path": "manifest.json",
  "default_font": "fonts/simhei.ttf",
  "start_script_path": "scripts/main.md",
  "asset_source": "zip",
  "zip_path": "game.zip",
  "window": { "width": 1280, "height": 720, "title": "My VN", "fullscreen": false },
  "debug": { "script_check": false, "log_level": null, "log_file": "game.log" },
  "audio": { "master_volume": 1.0, "bgm_volume": 0.8, "sfx_volume": 1.0, "muted": false },
  "resources": { "texture_cache_size_mb": 256 }
}
```

## 顶层字段

### `name`（必填，可写 `null`）

- **用途**：游戏名称。
- **影响**：
  - 窗口标题通常由 `window.title` 决定（更直接）。
  - 打包发行版时（`cargo pack release ...`），会用 `name` 来给 Tauri 可执行文件命名（并清理不适合作为文件名的字符）。
- **参考值**：默认配置中为 `"Ring VN Engine"`；写 `null` 时打包工具使用 `"Ring"`。

### `assets_root`

- **用途**：资源根目录（开发模式 `asset_source = "fs"` 时用于从文件系统读取资源）。
- **类型**：字符串路径（实现里是 `PathBuf`）。
- **参考值**：`"assets"`
- **注意**：脚本、图片、音频等资源最终都会被解析为 **相对于 `assets_root`** 的路径（见 [资源系统与打包](resources.md)）。

### `saves_dir`

- **用途**：存档目录（Continue 与槽位存档都会写在这里）。
- **参考值**：`"saves"`
- **参考**：[save format](../engine/reference/save-format.md)

### `manifest_path`

- **用途**：立绘布局配置（manifest）文件路径。
- **参考值**：`"manifest.json"`
- **路径规则**：相对于 `assets_root`  
  - 例如默认配置中对应 `assets/manifest.json`
- **参考**：[manifest 指南](manifest.md)

### `default_font`

- **用途**：默认字体文件路径（用于 UI/文本渲染，支持中文）。
- **参考值**：`"fonts/simhei.ttf"`
- **路径规则**：相对于 `assets_root`

### `start_script_path`

- **用途**：入口脚本路径。
- **校验**：不能为空，否则 `validate()` 报错。
- **路径规则**：相对于 `assets_root`  
  - 例如：`"scripts/main.md"` 对应 `assets/scripts/main.md`

### `asset_source`

- **用途**：资源来源模式。
- **可选值**：
  - `"fs"`：从文件系统读取（开发模式）
  - `"zip"`：从 ZIP 读取（发布模式）
- **参考值**：`"fs"`
- **参考**：[资源系统与打包](resources.md)

### `zip_path`（必填，Fs 模式写 `null`）

- **用途**：资源 ZIP 文件路径。
- **要求**：当 `asset_source = "zip"` 时必须为有效路径，否则校验失败。Fs 模式下写 `null`。
- **路径规则**：当前实现按“普通路径”检查是否存在（通常与你的 exe 同目录）。

## `window` 窗口配置

### `window.width` / `window.height`

- **用途**：窗口分辨率。
- **参考值**：`1920x1080`

### `window.title`

- **用途**：窗口标题。
- **参考值**：`"Ring VN Engine"`

### `window.fullscreen`

- **用途**：是否全屏。
- **参考值**：`false`

## `debug` 调试配置

### `debug.script_check`

- **用途**：Host 启动时是否自动运行脚本静态检查（语法 / label / 资源引用）。
- **参考值**：开发配置 `true`，release 打包时 `asset-packer` 自动改为 `false`。
- **注意**：检查结果**只输出诊断，不阻塞启动**（需要“阻塞式检查”请使用 `cargo script-check`）。

### `debug.log_level`

- **用途**：日志等级。
- **允许值**：`trace` / `debug` / `info` / `warn` / `error` / `off`（不区分大小写），写 `null` 时使用 `info`
- **参考值**：`"info"`

### `debug.log_file`

- **用途**：日志输出文件路径。设置后日志写入文件而非标准输出。
- **类型**：字符串或 `null`
- **参考值**：开发配置 `null`（输出到控制台），release 打包时 `asset-packer` 自动改为 `"game.log"`。
- **注意**：
  - release 构建会自动隐藏 Windows 控制台窗口（`windows_subsystem = "windows"`），此时建议写入文件。
  - 显式设为 `null` 可强制输出到标准输出（release 构建下日志将丢失，因为无控制台窗口）。
  - 文件在每次启动时会被覆盖（不追加）。
  - 如果文件创建失败，自动回退到标准输出。

## `audio` 音频配置

### `audio.master_volume` / `audio.bgm_volume` / `audio.sfx_volume`

- **用途**：音量（范围 0.0 ~ 1.0）。
- **参考值**：
  - `master_volume`: `1.0`
  - `bgm_volume`: `0.8`
  - `sfx_volume`: `1.0`
- **校验规则**：超出 0.0~1.0 会导致配置校验失败。

### `audio.muted`

- **用途**：是否静音。
- **参考值**：`false`

## `resources` 资源缓存配置

### `resources.texture_cache_size_mb`

- **用途**：纹理缓存大小（MB），用于控制显存占用（FIFO 缓存）。
- **参考值**：`256`
- **调参建议**：
  - 资源体量大、目标设备显存充足：可以适当增大
  - 发布前可通过日志中的缓存统计观察命中率/驱逐次数再调整（参见 [资源系统与打包](resources.md)）

## 配置校验（会检查什么）

运行时会做基本校验（`AppConfig::validate()`）：

- **入口脚本必须配置**：`start_script_path` 不能为空
- `asset_source = "fs"`：
  - `assets_root` 必须存在
  - `assets_root/start_script_path` 必须存在
- `asset_source = "zip"`：
  - 必须配置 `zip_path`
  - `zip_path` 指向的文件必须存在
- 音量字段必须在 0.0~1.0

## 字段归属分类

> RFC-025：为 Hub 多模态架构准备，标注各配置字段是全局共享还是 VN 模态专属。

### 全局配置（任何 mode 共用）

以下字段属于通用宿主/应用层配置，不绑定 VN 语义：

- **AppConfig**：`name`、`assets_root`、`saves_dir`、`asset_source`、`zip_path`
- **WindowConfig**（全部）：`width`、`height`、`title`、`fullscreen`
- **AudioConfig**（全部）：`master_volume`、`bgm_volume`、`sfx_volume`、`muted`
- **ResourceConfig**（全部）：`texture_cache_size_mb`
- **DebugConfig**（全部）：`script_check`、`log_level`、`log_file`

### VN 工程约定（当前仅 VN 使用）

以下字段与 VN 工程结构强相关，但作为「入口/布局配置」模式本身是通用的：

- **AppConfig**：`start_script_path`（入口脚本）、`manifest_path`（立绘布局）、`default_font`（默认字体）

### 运行时用户设置（VN 模态专属）

以下字段位于 `UserSettings`（`host-dioxus/src/state.rs`），不在 `config.json` 中：

- `text_speed`：文字速度（每秒字符数）
- `auto_delay`：自动播放延迟（秒）

这些字段仅在 VN 文字演出中有意义。将来引入新 mode 时，各 mode 可定义自己的 mode-specific 用户设置。

---

## 常见问题

### Q：为什么 `assets_root` 只写 `"assets"`，脚本里的 `<img src>` 却能用 `../backgrounds/...`？

脚本里的 `<img src>` / `<audio src>` 推荐写 **相对于脚本文件** 的路径，便于 Typora 预览；引擎解析时会基于脚本目录做 base path 计算，并最终归一化成相对于 `assets_root` 的资源路径。详见：[脚本语法规范](script-syntax.md) 与 [资源系统与打包](resources.md)。

### Q：发布模式下 `assets_root` 还生效吗？

会影响“逻辑上的资源根”。但在 ZIP 模式下资源实际从 `zip_path` 读取。发布建议使用 `cargo pack release ...` 生成 `dist/`，其中 `config.json` 会被自动改成 ZIP 模式并与 `game.zip` 放在同目录。

