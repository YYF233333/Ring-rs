# host-tauri/resources

> LastVerified: 2026-03-28
> Owner: Claude

## 职责

资源路径规范化与多来源读取——提供 `LogicalPath` 类型约束、`ResourceSource` trait 抽象和 `ResourceManager` 统一入口。后端脚本、JSON 等通过 trait 从文件系统或 ZIP 读取；前端资源（图片/音频/视频）仍通过 Tauri asset protocol 使用文件系统路径（由 `base_path` 解析）。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `LogicalPath` | newtype 包装的规范化逻辑路径（相对于 assets_root，`/` 分隔，无 `assets/` 前缀） |
| `ResourceSource` | 资源来源抽象 trait：`read_text`、`read_bytes`、`exists`（`Send + Sync`） |
| `FsSource` | 文件系统实现：`base_path` + `std::fs` 读写 |
| `ZipSource` | ZIP 文件实现（`cfg(feature = "zip")`）：预建路径索引、按需从归档读取 |
| `ResourceManager` | 统一资源管理器：后端读取通过 `ResourceSource` 代理，持有 `source: Box<dyn ResourceSource>` 与 `base_path: PathBuf` |
| `ResourceError` | 资源错误枚举：LoadFailed / NotFound |

### LogicalPath 不变量

- 相对于 assets_root（不含 `assets/` 前缀）
- 统一使用 `/` 分隔符
- 已解析 `..` 和 `.`

### LogicalPath API

| 方法 | 说明 |
|------|------|
| `new(raw)` | 从原始字符串构造，自动规范化 |
| `as_str()` | 获取内部字符串切片 |
| `file_stem()` | 提取文件名（不含扩展名） |
| `parent_dir()` | 提取父目录路径 |
| `join(relative)` | 拼接子路径并规范化 |
| `to_path_buf()` | 转换为 PathBuf |

### ResourceManager API

| 方法 | 说明 |
|------|------|
| `new(base_path)` | 创建资源管理器（内部使用 `FsSource`） |
| `with_source(source, base_path)` | 使用指定 `ResourceSource` 创建；`base_path` 仍用于 asset protocol |
| `read_text(path)` | 读取文本资源（经 `source`） |
| `read_bytes(path)` | 读取二进制资源（经 `source`） |
| `resolve_fs_path(path)` | 返回逻辑路径对应的文件系统绝对路径（用于 asset 协议） |
| `resource_exists(path)` | 检查资源是否存在（经 `source.exists`） |
| `base_path()` | 获取 assets 根目录（文件系统路径） |

## 数据流

```
脚本中的资源路径 (如 "images/bg01.png")
  │
  ▼
LogicalPath::new() → 规范化（统一分隔符、解析 ..、去除 assets/ 前缀）
  │
  ▼
ResourceManager::read_* / resource_exists
  │
  ▼
ResourceSource trait（FsSource 或 ZipSource）→ 实际读取，而非在 ResourceManager 内直接 std::fs（FS 模式下由 FsSource 内部使用 std::fs）
```

### normalize_logical_path 处理规则

1. `\` → `/`（统一分隔符）
2. 去除 `./` 前缀
3. 解析 `..`（弹出上级）和 `.`（忽略）
4. 去除 `assets/` 前缀
5. 空段忽略

## 关键不变量

- 支持 **FS** 与 **ZIP** 两种后端来源，由构造时注入的 `ResourceSource` 决定。
- 所有资源路径必须通过 `LogicalPath` 类型约束，禁止裸字符串调用 `ResourceManager`。
- `base_path` 始终为文件系统上的 assets 根，供前端 asset protocol 解析 URL；与 ZIP 后端并存，不因 ZIP 而省略。
- 路径规范化是幂等的。

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `state.rs` | 被持有：`AppStateInner.resource_manager`，用于脚本加载等 |
| `audio.rs` | 使用：`normalize_logical_path` 规范化音频路径 |
| `manifest.rs` | 使用：`normalize_logical_path` 规范化立绘路径 |
| `commands.rs` | 使用：`get_assets_root` 返回 base_path |
| `lib.rs` | 创建：`create_resource_manager` 根据 `AppConfig.asset_source` 选择 `ResourceManager::new`（FS）或 `ResourceManager::with_source(Box::new(ZipSource::open(...)), assets_root)`（ZIP，需 `zip` feature） |
| 前端 `useAssets.ts` | 间接：通过 `get_assets_root` IPC 获取路径后拼接 |

## 附录：Manifest

`manifest.rs` 提供立绘元数据管理。`Manifest` 在应用初始化时加载并存入 `Services`；`CommandExecutor` 在处理 `ShowCharacter` 时读取它，将脚本中的 `Position` 解析为写入 `CharacterSprite` 的归一化坐标与缩放。

| 类型 | 说明 |
|------|------|
| `Manifest` | 顶层清单：characters + presets + defaults |
| `GroupConfig` | 立绘组配置（anchor + pre_scale） |
| `PositionPreset` | 站位预设（x, y, scale），内置 9 个默认预设 |
| `CharactersConfig` | 角色配置：groups 映射 + sprites 映射 |

`get_group_config(sprite_path)` 查找顺序：显式映射 → 路径推导（父目录名/文件名前缀） → 默认配置。
