# host/resources 摘要

## Purpose

`resources` 是 Host 的统一资源入口，负责逻辑路径规范化、纹理缓存、文本/字节读取，以及文件系统与 ZIP 两类来源的抽象切换。

## PublicSurface

- 模块入口：`host/src/resources/mod.rs`
- 核心类型：`ResourceManager`、`TextureCache`、`LogicalPath`、`ResourceSource`、`ResourceError`
- 关键能力：纹理加载/预加载、文本与字节读取、目录列举、`materialize_to_fs()`
- `FsSource` / `ZipSource` 为 `pub(crate)`，对外只暴露 trait 抽象

## KeyFlow

1. 调用方先用 `LogicalPath::new()` 规范化路径。
2. 纹理读取先查 FIFO 缓存，未命中时经 `ResourceSource` 读字节、解码，再用 `TextureContext` 建纹理。
3. 文本/字节/列举接口都经 `ResourceManager` 统一下发到底层来源。
4. 仅在需要真实文件路径的子系统中使用 `materialize_to_fs()`。

## LogicalPath

`LogicalPath` 是规范化资源路径的 newtype 包装，编译期防止与文件系统 `PathBuf` 混用：
- 只能通过 `LogicalPath::new(raw)` 构造（内部调用 `normalize_logical_path`）
- 不变量：相对于 assets_root、`/` 分隔符、已解析 `..` 和 `.`
- 所有 `ResourceManager` 和 `ResourceSource` API 使用 `&LogicalPath`

## Dependencies

- 依赖 `rendering_types::{Texture, TextureContext}` 完成纹理创建（不直接依赖 backend/wgpu）
- 依赖 `image` crate 完成图片解码
- 被 `renderer`、`app`、`audio`、`manifest` 等模块广泛调用

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
- [resource_management](../../../../authoring/resources.md)
- [manifest 摘要](manifest.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4