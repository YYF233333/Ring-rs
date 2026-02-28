# host/manifest 摘要

## Purpose

`manifest` 管理立绘元数据：角色分组、锚点、预缩放、站位预设与路径映射，为渲染布局提供规则来源。

## PublicSurface

- 模块入口：`host/src/manifest/mod.rs`
- 核心类型：`Manifest`、`GroupConfig`、`PositionPreset`、`ManifestWarning`
- 关键接口：`load`、`load_from_bytes`、`validate`、`get_group_config`、`get_preset`

## KeyFlow

1. 从文件或字节读取并反序列化 manifest。
2. `validate` 校验锚点/缩放/预设与组引用关系。
3. 渲染路径按立绘路径查询组配置与站位预设，计算最终显示参数。

## Dependencies

- 依赖 `serde`/`serde_json` 解析元数据
- 被 `renderer` 和 `app` 初始化流程消费

## Invariants

- 锚点与预设坐标应位于规范范围，缩放必须为正。
- 立绘路径映射应可追溯到有效 group，缺失时有可预测回退策略。

## FailureModes

- manifest 结构错误导致解析失败。
- 配置越界或 group 缺失导致布局异常或警告泛滥。

## WhenToReadSource

- 需要扩展立绘布局字段或新增推断规则时。
- 需要排查角色站位/缩放与预期不一致时。

## RelatedDocs

- [host 总览](../host.md)
- [manifest_guide](../../manifest_guide.md)
- [renderer 摘要](renderer.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
