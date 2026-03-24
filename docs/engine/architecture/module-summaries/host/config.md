# host/config 摘要

## Purpose

`config` 定义 Host 启动配置模型，负责加载、保存与校验。运行时不提供“缺字段自动补默认值”的回退；配置缺失、字段缺失或字段名拼错都直接报错。

## PublicSurface

- 模块入口：`host/src/config/mod.rs`
- 核心类型：`AppConfig`、`WindowConfig`、`DebugConfig`、`AudioConfig`、`ResourceConfig`、`AssetSourceType`
- 关键接口：`AppConfig::load`、`save`、`validate`
- 错误类型：`ConfigError`

## KeyFlow

1. 启动时读取并反序列化 `config.json`。
2. `validate()` 按资源来源检查 `assets_root` / `zip_path`、入口脚本与音量范围。
3. 其他初始化路径消费已校验的 `AppConfig`。

## Dependencies

- 依赖 `serde`/`serde_json` 完成配置序列化
- 被 `app` 初始化流程和多个子系统消费

## Invariants

- 配置文件必须存在且所有字段完整（无代码内默认值回退）。
- 所有配置结构体使用 `#[serde(deny_unknown_fields)]` 拒绝未知字段。
- `start_script_path` 必须有效，作为运行入口约束。
- `impl Default` 存在，但正式加载路径不依赖它补齐缺失字段。
- VN-specific 的运行时设置（text_speed、auto_delay）不在 AppConfig 中，而在 UserSettings（app_mode）中（参见 RFC-025 字段归属文档化）。

## FailureModes

- 配置文件缺失 → `ConfigError::LoadFailed`
- 配置文件格式错误或字段缺失 → `ConfigError::LoadFailed`（含 serde 错误信息，指出缺失字段）
- 配置文件含未知字段 → `ConfigError::LoadFailed`（deny_unknown_fields）
- 资源路径或 zip_path 不存在 → `ConfigError::ValidationFailed`

## WhenToReadSource

- 需要新增配置项时（同时需更新 `config.json` 默认文件）。
- 需要排查启动配置校验失败时。

## RelatedDocs

- [host 总览](../host.md)
- [config_guide](../../../../authoring/config.md)
- [resources 摘要](resources.md)
- [RFC-013: 配置默认值外部化](../../../../../RFCs/Accepted/rfc-config-externalization.md)

## LastVerified

2026-03-24

## Owner

claude-4.6-opus