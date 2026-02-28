# host/config 摘要

## Purpose

`config` 定义 Host 运行配置模型，负责配置加载、默认值合并、序列化与有效性校验。

## PublicSurface

- 模块入口：`host/src/config/mod.rs`
- 核心类型：`AppConfig`、`WindowConfig`、`DebugConfig`、`AudioConfig`、`ResourceConfig`
- 关键接口：`AppConfig::load`、`save`、`validate`

## KeyFlow

1. 启动时读取 `config.json`，失败则回落默认值。
2. 运行前调用 `validate` 校验资源来源、入口脚本与音量范围。
3. 其他模块（app/audio/resources）消费配置字段完成初始化。

## Dependencies

- 依赖 `serde`/`serde_json` 完成配置序列化
- 被 `app` 初始化流程和多个子系统消费

## Invariants

- 配置优先级固定：命令行 > 配置文件 > 默认值。
- `start_script_path` 必须有效，作为运行入口约束。

## FailureModes

- 配置文件缺失或格式错误导致默认回退。
- 资源路径或 zip_path 不存在导致校验失败。

## WhenToReadSource

- 需要新增配置项或修改默认值策略时。
- 需要排查启动配置校验失败时。

## RelatedDocs

- [host 总览](../host.md)
- [config_guide](../../config_guide.md)
- [resources 摘要](resources.md)

## LastVerified

2026-02-28

## Owner

Ring-rs 维护者
