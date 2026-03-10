# host/resources 摘要

## Purpose

`resources` 负责 Host 资源访问：统一路径解析、纹理缓存、音频与文本读取、文件来源抽象（文件系统/ZIP）。

## PublicSurface

- 模块入口：`host/src/resources/mod.rs`
- 核心类型：`ResourceManager`、`TextureCache`、`ResourceError`、`ResourceSource`
- 关键子模块：`cache`、`path`、`source`、`error`

## KeyFlow

1. 资源路径先通过 `resolve_path/normalize_*` 归一化。
2. 纹理加载先查 LRU 缓存，未命中则从 `ResourceSource` 读取并解码缓存。
3. 音频/文本/字节资源通过统一来源接口读取。
4. 每帧可通过 pin/unpin 机制降低关键纹理被驱逐风险。

## Dependencies

- 依赖 `rendering_types::{Texture, TextureContext}` 完成纹理创建（不直接依赖 backend/wgpu）
- 依赖 `image` crate 完成图片解码
- 被 `renderer`、`app`、`audio`、`manifest`、脚本加载路径广泛调用

## Invariants

- 逻辑路径到规范路径映射保持稳定，缓存键统一。
- 资源来源抽象不泄露具体存储介质细节给调用方。

## FailureModes

- 路径解析错误导致资源找不到。
- 缓存预算不足引发频繁抖动或加载开销上升。
- 字节解码失败导致纹理/文本加载失败。

## WhenToReadSource

- 需要排查资源路径兼容性（相对路径、跨平台路径）时。
- 需要优化缓存策略或新增资源来源类型时。

## RelatedDocs

- [host 总览](../host.md)
- [resource_management](../../resource_management.md)
- [manifest 摘要](manifest.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
