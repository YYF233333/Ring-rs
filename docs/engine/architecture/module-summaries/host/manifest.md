# host/manifest 摘要

## Purpose

`manifest` 提供立绘布局元数据：组配置、锚点、预缩放、站位预设与路径到角色组的映射规则，供渲染布局查询。

## PublicSurface

- 模块入口：`host/src/manifest/mod.rs`
- 核心类型：`Manifest`、`GroupConfig`、`PositionPreset`、`ManifestWarning`
- 关键接口：`load`、`load_from_bytes`、`load_and_validate`、`validate`、`get_group_config`、`get_preset`

## KeyFlow

1. 从文件或字节加载 `Manifest`。
2. `validate()` 收集锚点、缩放、预设范围与未知组引用等警告。
3. 渲染路径按 sprite 路径查询组配置，再按站位名查询预设。

## Dependencies

- 依赖 `serde`/`serde_json` 解析元数据
- 被 `renderer` 和 `app` 初始化流程消费

## Invariants

- 锚点与预设坐标应位于规范范围，缩放必须为正。
- 立绘路径优先走显式映射，缺失时再走路径推断与默认回退。

## FailureModes

- manifest 结构错误导致解析失败。
- 配置越界或 group 缺失导致布局异常或警告泛滥。

## WhenToReadSource

- 需要扩展立绘布局字段或新增推断规则时。
- 需要排查角色站位/缩放与预期不一致时。

## RelatedDocs

- [host 总览](../host.md)
- [manifest_guide](../../../../authoring/manifest.md)
- [renderer 摘要](renderer.md)

## LastVerified

2026-03-24

## Owner

GPT-5.4