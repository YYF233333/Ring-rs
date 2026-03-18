# host/resources 摘要

## Purpose

`resources` 负责 Host 资源访问：统一路径解析、纹理缓存、音频与文本读取、文件来源抽象（文件系统/ZIP）。

## PublicSurface

- 模块入口：`host/src/resources/mod.rs`
- 核心类型：`ResourceManager`、`TextureCache`、`ResourceError`、`ResourceSource`、`LogicalPath`
- 关键子模块：`cache`、`path`、`source`、`error`
- `FsSource`/`ZipSource` 为 `pub(crate)` 可见性，外部只通过 `ResourceSource` trait 交互

## KeyFlow

1. 资源路径通过 `LogicalPath::new()` 构造（自动规范化，去除 `assets/` 前缀、统一 `/` 分隔符）。
2. 纹理加载先查 LRU 缓存（键为 `LogicalPath.as_str()`），未命中则从 `ResourceSource` 读取并解码缓存。
3. 音频/文本/字节资源通过 `ResourceManager` 统一接口读取。
4. 需要真实文件系统路径的场景（如 FFmpeg）使用 `materialize_to_fs()`。
5. 可选资源使用 `read_text_optional()` 避免 NotFound 错误。
6. `FsSource`/`ZipSource` 只在 `init::create_resource_manager` 内部构造；需 source 时从 `ResourceManager::source()` 获取。

## LogicalPath

`LogicalPath` 是规范化资源路径的 newtype 包装，编译期防止与文件系统 `PathBuf` 混用：
- 只能通过 `LogicalPath::new(raw)` 构造（内部调用 `normalize_logical_path`）
- 不变量：相对于 assets_root、`/` 分隔符、已解析 `..` 和 `.`
- 所有 `ResourceManager` 和 `ResourceSource` API 使用 `&LogicalPath`

## Dependencies

- 依赖 `rendering_types::{Texture, TextureContext}` 完成纹理创建（不直接依赖 backend/wgpu）
- 依赖 `image` crate 完成图片解码
- 被 `renderer`、`app`、`audio`、`manifest`、脚本加载路径广泛调用

## Invariants

- `LogicalPath` 保证路径规范化不变量，缓存键跨平台一致。
- 资源来源抽象不泄露具体存储介质细节给调用方。
- `FsSource`/`ZipSource` 不对外公开，所有资源访问走 `ResourceManager`。

## FailureModes

- 路径解析错误导致资源找不到。
- 缓存预算不足引发频繁抖动或加载开销上升。
- 字节解码失败导致纹理/文本加载失败。

## WhenToReadSource

- 需要排查资源路径兼容性（相对路径、跨平台路径）时。
- 需要优化缓存策略或新增资源来源类型时。
- 需要添加新的 `ResourceManager` 公共 API 时。

## RelatedDocs

- [host 总览](../host.md)
- [resource_management](../../resource_management.md)
- [manifest 摘要](manifest.md)

## LastVerified

2026-03-18

## Owner

Ring-rs 维护者
