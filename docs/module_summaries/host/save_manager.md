# host/save_manager 摘要

## Purpose

`save_manager` 提供 Host 存档文件管理：槽位存档读写、存档列表、Continue 专用存档与展示用元信息提取。

## PublicSurface

- 模块入口：`host/src/save_manager/mod.rs`
- 核心类型：`SaveManager`、`SaveInfo`
- 常量：`MAX_SAVE_SLOTS`
- 关键接口：`save/load/delete/list_saves`、`save_continue/load_continue`、`get_save_info/get_continue_info`、`thumbnail_path/save_thumbnail/load_thumbnail_bytes`、`next_available_slot`、`ensure_dir`

**持久化变量（`PersistentStore`）**：

- 模块入口：`host/src/app/persistent.rs`
- 核心类型：`PersistentStore { variables: HashMap<String, VarValue> }`
- 关键接口：`load(saves_dir)`、`save()`、`merge_from(vars)`、`save_or_log()`
- 文件：`saves/persistent.json`（key 为 bare key，不含 `persistent.` 前缀）

## KeyFlow

1. 启动或保存前通过 `ensure_dir` 确保存档目录存在。
2. 槽位存档按 `slot_XXX.json` 读写 `vn_runtime::SaveData`；槽位可关联缩略图 `thumb_XXX.png`（`save_thumbnail`/`load_thumbnail_bytes`）。
3. Continue 存档使用独立文件 `continue.json` 管理恢复入口。
4. UI 层通过 `SaveInfo`（`get_save_info`/`get_continue_info`）提供展示信息（时间、章节、脚本、游玩时长）。
5. 应用启动时 `PersistentStore::load(saves_dir)` 加载持久变量；执行 `fullRestart` 时 `merge_from + save` 写入磁盘。

## Dependencies

- 依赖 `vn_runtime::{SaveData, SaveError}`
- 被 `app/save` 与相关页面（存档/读档）消费

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
- [save_format](../../save_format.md)
- [ui 摘要](ui.md)（存读档界面与 ScreenDefinitions）

## LastVerified

2026-03-18

## Owner

Ring-rs 维护者
