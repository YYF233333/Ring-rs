# host/save_manager 摘要

## Purpose

`save_manager` 负责 Host 层的存档文件落盘与列举，包括普通槽位、Continue 存档，以及 UI 所需的轻量展示信息与缩略图文件。

## PublicSurface

- 模块入口：`host/src/save_manager/mod.rs`
- 核心类型：`SaveManager`、`SaveInfo`
- 常量：`MAX_SAVE_SLOTS`
- 关键接口：槽位存读删、Continue 存读删、信息提取、缩略图读写、`next_available_slot`、`ensure_dir`

## KeyFlow

1. `ensure_dir()` 保证存档目录存在。
2. 槽位存档读写 `slot_XXX.json`，缩略图读写 `thumb_XXX.png`。
3. Continue 存档单独使用 `continue.json`。
4. UI 通过 `get_save_info()` / `get_continue_info()` 获取展示元信息。

## Dependencies

- 依赖 `vn_runtime::{SaveData, SaveError}`
- 被 `app/save` 与相关页面（存档/读档）消费
- 持久化变量的磁盘读写由 `host/src/app/persistent.rs` 中的 `PersistentStore` 负责，属于应用层配套子系统，而非 `save_manager` 自身 API

## Invariants

- 槽位编号在约定范围内（1..=MAX_SAVE_SLOTS）。
- Continue 存档与普通槽位存档语义分离。

## FailureModes

- 文件系统 IO 失败导致存读档失败。
- 存档 JSON 损坏导致解析失败。

## WhenToReadSource

- 需要调整存档布局、命名或兼容策略时。
- 需要排查 Continue 与普通存档切换行为时。

## RelatedDocs

- [host 总览](../host.md)
- [save_format](../../../reference/save-format.md)
- [ui 摘要](ui.md)（存读档界面与 ScreenDefinitions）

## LastVerified

2026-03-24

## Owner

GPT-5.4