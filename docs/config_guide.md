# `config.json` 配置说明

本文解释仓库根目录 `config.json` 中各项配置的含义、默认值与使用场景。

> 配置读取与校验实现：`host/src/config/mod.rs`

## 快速示例

### 开发模式（从文件系统加载资源）

```json
{
  "name": "My VN",
  "assets_root": "assets",
  "start_script_path": "scripts/main.md",
  "asset_source": "fs",
  "manifest_path": "manifest.json",
  "default_font": "fonts/simhei.ttf",
  "window": { "width": 1280, "height": 720, "title": "My VN", "fullscreen": false },
  "debug": { "script_check": true, "log_level": "info" },
  "audio": { "master_volume": 1.0, "bgm_volume": 0.8, "sfx_volume": 1.0, "muted": false },
  "resources": { "texture_cache_size_mb": 256 }
}
```

### 发布模式（从 ZIP 加载资源）

```json
{
  "name": "My VN",
  "assets_root": "assets",
  "start_script_path": "scripts/main.md",
  "asset_source": "zip",
  "zip_path": "game.zip",
  "window": { "width": 1280, "height": 720, "title": "My VN", "fullscreen": false }
}
```

## 顶层字段

### `name`（可选）

- **用途**：游戏名称。
- **影响**：
  - 窗口标题通常由 `window.title` 决定（更直接）。
  - 打包发行版时（`cargo pack release ...`），会用 `name` 来给可执行文件命名（并清理不适合作为文件名的字符）。
- **默认值**：如果缺失，打包工具会使用 `"Ring"`（见 `tools/asset-packer`）。

### `assets_root`

- **用途**：资源根目录（开发模式 `asset_source = "fs"` 时用于从文件系统读取资源）。
- **类型**：字符串路径（实现里是 `PathBuf`）。
- **默认值**：`"assets"`
- **注意**：脚本、图片、音频等资源最终都会被解析为 **相对于 `assets_root`** 的路径（见 [resource_management.md](resource_management.md)）。

### `saves_dir`

- **用途**：存档目录（Continue 与槽位存档都会写在这里）。
- **默认值**：`"saves"`
- **参考**：[save_format.md](save_format.md)

### `manifest_path`

- **用途**：立绘布局配置（manifest）文件路径。
- **默认值**：`"manifest.json"`
- **路径规则**：相对于 `assets_root`  
  - 例如默认值对应 `assets/manifest.json`
- **参考**：[manifest_guide.md](manifest_guide.md)

### `default_font`

- **用途**：默认字体文件路径（用于 UI/文本渲染，支持中文）。
- **默认值**：`"fonts/simhei.ttf"`
- **路径规则**：相对于 `assets_root`

### `start_script_path`（必填）

- **用途**：入口脚本路径。
- **必填**：是。未配置会导致配置校验失败（并在运行时阻止继续）。
- **路径规则**：相对于 `assets_root`  
  - 例如：`"scripts/main.md"` 对应 `assets/scripts/main.md`

### `asset_source`

- **用途**：资源来源模式。
- **可选值**：
  - `"fs"`：从文件系统读取（开发模式，默认）
  - `"zip"`：从 ZIP 读取（发布模式）
- **默认值**：`"fs"`
- **参考**：[resource_management.md](resource_management.md)

### `zip_path`（仅 ZIP 模式必填）

- **用途**：资源 ZIP 文件路径。
- **要求**：当 `asset_source = "zip"` 时必须配置，否则校验失败。
- **路径规则**：当前实现按“普通路径”检查是否存在（通常与你的 exe 同目录）。

## `window` 窗口配置

### `window.width` / `window.height`

- **用途**：窗口分辨率。
- **默认值**：`1920x1080`

### `window.title`

- **用途**：窗口标题。
- **默认值**：`"Ring VN Engine"`

### `window.fullscreen`

- **用途**：是否全屏。
- **默认值**：`false`

## `debug` 调试配置

### `debug.script_check`

- **用途**：Host 启动时是否自动运行脚本静态检查（语法 / label / 资源引用）。
- **默认值**：
  - debug build（`cargo run`）默认 `true`
  - release build 默认 `false`
- **注意**：检查结果**只输出诊断，不阻塞启动**（需要“阻塞式检查”请使用 `cargo script-check`）。

### `debug.log_level`

- **用途**：日志等级。
- **允许值**：`trace` / `debug` / `info` / `warn` / `error` / `off`（不区分大小写）
- **来源**：
  - `debug.log_level`
  - 默认 `info`

## `audio` 音频配置

### `audio.master_volume` / `audio.bgm_volume` / `audio.sfx_volume`

- **用途**：音量（范围 0.0 ~ 1.0）。
- **默认值**：
  - `master_volume`: `1.0`
  - `bgm_volume`: `0.8`
  - `sfx_volume`: `1.0`
- **校验规则**：超出 0.0~1.0 会导致配置校验失败。

### `audio.muted`

- **用途**：是否静音。
- **默认值**：`false`

## `resources` 资源缓存配置

### `resources.texture_cache_size_mb`

- **用途**：纹理缓存大小（MB），用于控制显存占用（LRU 缓存）。
- **默认值**：`256`
- **调参建议**：
  - 资源体量大、目标设备显存充足：可以适当增大
  - 发布前可按 [resource_management.md](resource_management.md) 的 Debug Overlay（F1）观察命中率/驱逐次数再调整

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

## 常见问题

### Q：为什么 `assets_root` 只写 `"assets"`，脚本里的 `<img src>` 却能用 `../backgrounds/...`？

脚本里的 `<img src>` / `<audio src>` 推荐写 **相对于脚本文件** 的路径，便于 Typora 预览；引擎解析时会基于脚本目录做 base path 计算，并最终归一化成相对于 `assets_root` 的资源路径。详见：[script_syntax_spec.md](script_syntax_spec.md) 与 [resource_management.md](resource_management.md)。

### Q：发布模式下 `assets_root` 还生效吗？

会影响“逻辑上的资源根”。但在 ZIP 模式下资源实际从 `zip_path` 读取。发布建议使用 `cargo pack release ...` 生成 `dist/`，其中 `config.json` 会被自动改成 ZIP 模式并与 `game.zip` 放在同目录。

