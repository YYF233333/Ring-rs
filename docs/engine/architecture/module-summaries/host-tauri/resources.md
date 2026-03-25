# host-tauri/resources

> LastVerified: 2026-03-25
> Owner: Claude

## 职责

资源路径规范化与文件系统读取——提供 `LogicalPath` 类型约束和 `ResourceManager` 统一资源访问。

## 关键类型/结构

| 类型 | 说明 |
|------|------|
| `LogicalPath` | newtype 包装的规范化逻辑路径（相对于 assets_root，`/` 分隔，无 `assets/` 前缀） |
| `ResourceManager` | 简化版资源管理器（仅文件系统模式，持有 `base_path`） |
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
| `new(base_path)` | 创建资源管理器 |
| `read_text(path)` | 读取文本资源 |
| `read_bytes(path)` | 读取二进制资源 |
| `resolve_fs_path(path)` | 返回文件系统绝对路径 |
| `resource_exists(path)` | 检查资源是否存在 |
| `base_path()` | 获取 assets 根目录 |

## 数据流

```
脚本中的资源路径 (如 "images/bg01.png")
  │
  ▼
LogicalPath::new() → 规范化（统一分隔符、解析 ..、去除 assets/ 前缀）
  │
  ▼
ResourceManager::resolve() → base_path.join(logical_path)
  │
  ▼
std::fs::read / read_to_string → Result<T, ResourceError>
```

### normalize_logical_path 处理规则

1. `\` → `/`（统一分隔符）
2. 去除 `./` 前缀
3. 解析 `..`（弹出上级）和 `.`（忽略）
4. 去除 `assets/` 前缀
5. 空段忽略

## 关键不变量

- `ResourceManager` 仅支持文件系统模式（简化版，无 ZIP 支持）
- 所有资源路径必须通过 `LogicalPath` 类型约束，禁止裸字符串调用
- `resolve()` 是内部方法，外部通过 `read_text`/`read_bytes`/`resolve_fs_path` 访问
- 路径规范化是幂等的

## 与其他模块的关系

| 模块 | 关系 |
|------|------|
| `state.rs` | 被持有：`AppStateInner.resource_manager`，用于脚本加载和音频字节读取 |
| `audio.rs` | 使用：`normalize_logical_path` 规范化音频路径 |
| `manifest.rs` | 使用：`normalize_logical_path` 规范化立绘路径 |
| `commands.rs` | 使用：`get_assets_root` 返回 base_path |
| `lib.rs` | 创建：setup 中根据配置创建 ResourceManager |
| 前端 `useAssets.ts` | 间接：通过 `get_assets_root` IPC 获取路径后拼接 |

## 附录：Manifest

`manifest.rs` 提供立绘元数据管理：

| 类型 | 说明 |
|------|------|
| `Manifest` | 顶层清单：characters + presets + defaults |
| `GroupConfig` | 立绘组配置（anchor + pre_scale） |
| `PositionPreset` | 站位预设（x, y, scale），内置 9 个默认预设 |
| `CharactersConfig` | 角色配置：groups 映射 + sprites 映射 |

`get_group_config(sprite_path)` 查找顺序：显式映射 → 路径推导（父目录名/文件名前缀） → 默认配置。
