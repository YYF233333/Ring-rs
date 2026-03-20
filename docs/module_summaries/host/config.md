# host/config 摘要

## Purpose

`config` 定义 Host 运行配置模型，负责配置加载、序列化与有效性校验。
所有字段均为必填（`Option` 字段需显式写 `null`），配置文件缺失或字段缺失时直接报错。

## PublicSurface

- 模块入口：`host/src/config/mod.rs`
- 核心类型：`AppConfig`、`WindowConfig`、`DebugConfig`（含 `log_file`、`recording_buffer_size_mb`、`recording_output_dir`）、`AudioConfig`、`ResourceConfig`、`AssetSourceType`（Fs/Zip）
- 关键接口：`AppConfig::load`（返回 `Result`）、`save`、`validate`
- 错误类型：`ConfigError`（含 `LoadFailed`、`SerializationFailed`、`IoError`、`ValidationFailed`）

## KeyFlow

1. 启动时读取 `config.json`，文件缺失或解析失败即报错退出。
2. 运行前调用 `validate` 校验资源来源、入口脚本与音量范围。
3. 其他模块（app/audio/resources）消费配置字段完成初始化。

## Dependencies

- 依赖 `serde`/`serde_json` 完成配置序列化
- 被 `app` 初始化流程和多个子系统消费

## Invariants

- 配置文件必须存在且所有字段完整（无代码内默认值回退）。
- 所有配置结构体使用 `#[serde(deny_unknown_fields)]` 拒绝未知字段。
- `start_script_path` 必须有效，作为运行入口约束。
- `impl Default` 仅供测试使用，运行时加载路径不调用。

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
- [config_guide](../../config_guide.md)
- [resources 摘要](resources.md)
- [RFC-013: 配置默认值外部化](../../../RFCs/Accepted/rfc-config-externalization.md)

## LastVerified

2026-03-19

## Owner

Composer