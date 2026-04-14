# 测试审计清单（2026-03-20）

> **注意**：本文档是 2026-03-20 的历史快照。其中 `host/` 部分记录的是已退役的 legacy host crate（winit/wgpu/egui 架构）的 534 个测试，不适用于当前的 `host-dioxus`（Dioxus 0.7 Desktop）。`vn-runtime` 部分仍然有效。

> 目标：为”高价值 / 低价值 / undecided”分层提供统一基线，后续新增测试默认先对照本清单归类。

## 分类口径

- `high_value`：验证不变量、状态机转换、错误分类、边界条件、对外契约、跨模块链路、恢复/兼容语义。
- `low_value`：主要复读实现或提高机械覆盖率，例如构造器默认值、getter/setter、`Display`、纯 `serde` roundtrip、简单路径/字符串格式化。
- `undecided`：命中了真实业务路径，但断言还偏浅，或 helper/冒烟测试暂时难以快速定级。

## 范围总览

**当前合计**：`vn-runtime` **273** 个测试，`host` **534** 个测试，共 **807** 个。

**分层结论（Phase 0–4）**

- **已完成** `high_value` / `low_value` 分层；所有 `undecided` 已分流完毕（167 → 31 `high_value` + 136 `low_value`）；各 `tests/mod.rs` 仅保留 `mod` 与共享 helper，**无** `#[test]`。
- **曾删除的 `low_value.rs`（11，因清空机械用例）**：`script/ast`、`script/expr`、`command`、`diagnostic`、`runtime/engine`、`manifest`、`save_manager`、`command_executor`、`resources`、`ui/layout`、`renderer/scene_transition`。其中 **`runtime/engine`** 与 **`command_executor`** 在 Phase 2 为承接下沉用例**重建**了 `low_value.rs`。
- **测试数变化**：原合计 815，净减 **8**（删除无价值用例后现为 **807**）。

| crate / 文件 | 测试数 | 结论 |
| --- | ---: | --- |
| `vn-runtime/src/script/parser/tests/` | 120 | 已分层为 `high_value.rs` + `low_value.rs`；`mod.rs` 仅 helper，无 `#[test]` |
| `vn-runtime/src/script/parser/` 内联（`expr_parser.rs`、`inline_tags.rs`） | 12 | 与上表分列以便与历史口径对齐；仍为 parser 相关回归 |
| `vn-runtime/src/runtime/executor/tests/` | 51 | 已分层；`mod.rs` 无 `#[test]`；executor `low_value` 经 Phase 4 加强了 `ExecuteResult` 四字段断言 |
| `vn-runtime/src/runtime/engine/tests/` | 26 | 已分层；`low_value.rs` 为 Phase 2 重建（承接 undecided 下沉）；原文件曾随 11 文件清理删除 |
| `vn-runtime/src/diagnostic/tests/` | 15 | **仅** `high_value.rs`；原 `low_value` 机械用例已删除 |
| `vn-runtime/src/script/expr/tests/` | 13 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `vn-runtime/src/command/tests/` | 7 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `vn-runtime/src/script/ast/tests/` | 6 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `vn-runtime/src/history.rs` 内联测试 | 8 | Phase 4：时间戳相关改为 before/after 夹逼 |
| `vn-runtime/src/save.rs` 内联测试 | 7 | 兼容性高价值，Display/字符串偏低 |
| `vn-runtime/src/state.rs` 内联测试 | 6 | `persistent` 隔离高价值，其余多为低/灰 |
| `vn-runtime/src/input.rs` 内联测试 | 2 | 低价值 |
| `host/src/command_executor/tests/` | 46 | 已分层；`low_value.rs` 为 Phase 2 重建；`mod.rs` 无 `#[test]`；原 `low_value.rs` 曾随清理删除 |
| `host/src/renderer/render_state/tests/` | 35 | 已分层；`mod.rs` 无 `#[test]` |
| `host/src/input/tests/` | 28 | 已分层；输入契约与防抖高价值，getter/default 偏低 |
| `host/src/manifest/tests/` | 29 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `host/src/save_manager/tests/` | 24 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `host/src/resources/tests/` | 20 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `host/src/config/tests/` | 20 | 配置 schema 与严格校验高价值，默认值偏低 |
| `host/src/renderer/headless_tests/` | 16 | 组合绘制链路→high_value，几何/枚举/getter→low_value；`mod.rs` 无 `#[test]` |
| `host/src/renderer/effects/resolver.rs` 内联测试 | 18 | 契约与回退逻辑高价值 |
| `host/src/renderer/transition.rs` 内联测试 | 14 | 状态机与 skip 行为高价值，debug/默认值偏低 |
| `host/src/renderer/scene_transition/tests/` | 18 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `host/src/app/app_mode/tests/` | 21 | 已分层；导航栈高价值 |
| `host/src/resources/source.rs` 内联测试 | 13 | 资源来源抽象高价值 |
| `host/src/resources/path.rs` 内联测试 | 13 | 路径规范化与解析高价值 |
| `host/src/ui/layout/tests/` | 8 | **仅** `high_value.rs`；`low_value.rs` 已删除 |
| `host/src/resources/cache.rs` 内联测试 | 13 | 缓存淘汰/统计偏灰到高 |
| `host/src/extensions/tests/` | 33 | 已分层；兼容性/能力校验高价值 |
| `host/src/ui/screen_defs/tests/` | 13 | 已分层；条件求值与资源回退高价值 |
| `host/src/renderer/character_animation.rs` 内联测试 | 8 | 偏灰 |
| `host/src/renderer/animation/transform.rs` 内联测试 | 13 | 低/灰为主 |
| `host/src/renderer/animation/system.rs` 内联测试 | 13 | 状态机与注册/回收高价值 |
| `host/src/renderer/animation/core.rs` 内联测试 | 5 | 灰区偏高 |
| `host/src/renderer/background_transition.rs` 内联测试 | 4 | 低/灰 |
| `host/src/renderer/animation/easing.rs` 内联测试 | 17 | Phase 4：elastic/bounce 等改为验证 overshoot 与非单调性 |
| `host/src/renderer/animation/traits.rs` 内联测试 | 4 | 低价值 |
| `host/src/audio/mod.rs` 内联测试 | 3 | `duck/unduck` 高于音量默认值 |

## mutants 佐证

- 本仓库现有 `mutants.out` 结果显示 `missed` 高度集中在 `vn-runtime/src/script/parser/*`，尤其是 `phase2.rs`、`helpers.rs`、`inline_tags.rs`。
- 次高风险点是 `vn-runtime/src/runtime/engine/mod.rs::handle_script_control()` 的跨脚本恢复分支。
- `vn-runtime/src/script/expr/mod.rs::values_equal()` 仅有一个 `EPSILON` 边界 miss，更接近可接受噪音。
- 当前 mutation 结果没有显示 `state.rs`、`command/mod.rs`、`diagnostic/mod.rs` 存在集中 missed；这些模块的高价值测试更多体现在契约回归，而不是 mutation 薄弱点。
- `host` 推广策略采用“白名单思路”：先只纳入已有高价值测试、且不依赖真实窗口 / GPU / rodio / FFmpeg / egui 事件循环的纯逻辑模块。
- `host` 当前适合纳入 mutants 的首批模块：`command_executor`、`resources`、`save_manager`、`config`、`manifest`、`input`、`extensions`，以及 `renderer/render_state`、`renderer/effects/resolver`、`renderer/scene_transition`。
- `host` 当前不适合默认纳入 mutants 的区域：`backend/*`、`audio/*`、`video/*`、`egui_screens/*`、`app/update/*`、`app/command_handlers/*`，以及噪音较高的 UI / 渲染辅助模块。

## 热点文件的函数级样本

以下仅抽样；**原 `undecided` 类目已撤销**，表中曾列于 undecided 的用例均已分流至 `high_value.rs` 或 `low_value.rs`（标注「已分流」）。各小节末尾保留 **`undecided`（已分流完毕）** 标题，表示该类目无残留列表。

### `vn-runtime/src/runtime/engine/tests/`

`high_value`
- `test_runtime_tick_dialogue`
- `test_runtime_script_end`
- `test_runtime_with_goto`
- `test_runtime_with_choice`
- `test_runtime_restore_ctor`
- `test_runtime_is_finished`
- `test_call_script_and_return_flow`
- `test_call_script_auto_return_on_child_eof`
- `test_wait_for_signal_clears_only_on_expected_id`
- `test_state_mismatch_error`
- `test_invalid_choice_index_error`
- `test_choice_selected_label_not_found_error`
- `test_wait_for_time_click_interrupts`
- `test_wait_for_time_ignores_non_click_input`
- `test_runtime_history_recording`（已分流）
- `test_runtime_state_restore`（已分流）
- `test_record_history_for_extend_text`（已分流）

`low_value`
- `test_record_history_for_background_and_bgm`（已分流）
- `test_record_history_for_chapter_mark`（已分流）
- `test_record_history_for_stop_bgm`（已分流）

`undecided`（已分流完毕）

### `vn-runtime/src/runtime/executor/tests/`

`high_value`
- `test_execute_show_character_without_path_uses_existing_binding`
- `test_execute_show_character_without_path_errors_when_not_bound`
- `test_execute_change_scene_resolves_mask_path`
- `test_execute_conditional_with_multiple_body_nodes`
- `test_execute_conditional_with_goto`
- `test_execute_call_script_control_flow`
- `test_execute_return_from_script_control_flow`
- `test_execute_wait`
- `test_execute_set_var_persistent_prefix_routes_to_persistent_variables`
- `test_execute_scene_effect_with_duration_waits`
- `test_execute_title_card_waits_for_signal`
- `test_execute_cutscene_waits_for_signal`
- `test_execute_cutscene_resolves_path`
- `test_execute_label_no_command`（已分流）
- `test_execute_stop_bgm`（已分流）
- `test_execute_full_restart_emits_command`（已分流）
- `test_execute_dialogue`（已分流）
- `test_execute_choice`（已分流）
- `test_execute_goto`（已分流）
- `test_execute_show_character`（已分流）
- `test_execute_hide_character_updates_state`（已分流）
- `test_execute_conditional_true_branch`（已分流）
- `test_execute_conditional_else_branch`（已分流）

`low_value`
- `test_execute_textbox_hide`
- `test_execute_textbox_show`
- `test_execute_textbox_clear`
- `test_execute_bgm_duck`
- `test_execute_bgm_unduck`
- `test_execute_change_scene_without_transition`（已分流）
- `test_execute_change_scene_non_mask_named_arg_preserved`（已分流）
- `test_execute_extend_text`（已分流）
- `test_execute_extend_text_no_wait`（已分流）
- `test_execute_dialogue_no_wait`（已分流）
- `test_execute_conditional_elseif`（已分流）
- `test_execute_conditional_with_wait_in_branch`（已分流）

`undecided`（已分流完毕）

### `vn-runtime/src/diagnostic/tests/`

`high_value`
- `test_analyze_script_undefined_label`
- `test_analyze_script_valid_labels`
- `test_analyze_script_choice_targets`
- `test_analyze_conditional_jump_targets`
- `test_analyze_script_with_line_numbers`
- `test_script_source_map`
- `test_extract_resource_references`
- `test_extract_resource_references_change_scene`
- `test_extract_resource_references_includes_cutscene`
- `test_get_jump_targets`
- `test_get_defined_labels`（已分流）
- `test_extract_resource_references_in_conditional`（已分流）
- `test_extract_resource_references_show_without_path`
- `test_diagnostic_result_filter`（已分流）
- `test_diagnostic_result_merge`（已分流）

`undecided`（已分流完毕）

### `vn-runtime/src/command/tests/`

`high_value`
- `test_transition_get_arg_fallback`
- `test_transition_get_duration_and_reversed_wrong_type_returns_none`
- `test_transition_get_duration_and_reversed`
- `test_position_parse_supports_all_variants`
- `test_position_from_str`（已分流；与上同类合并后仍保留契约价值）
- `test_command_serialization`（已分流）
- `test_transition_serialization_with_named_args`（已分流）

`undecided`（已分流完毕）

### `host/src/command_executor/tests/`

`high_value`
- `test_execute_show_character_diff_and_move_uses_diff_then_move`
- `test_change_scene_rule_produces_scene_transition`
- `test_explicit_duration_overrides_default_for_all_targets`
- `test_execute_show_character_reposition_with_move_triggers_animation`
- `test_execute_show_background_with_transition`
- `test_batch_execution_last_wait_wins`（Phase 4 自 `test_batch_error_stops_execution` 更名）
- `test_full_restart_is_noop`（已分流）
- `test_execute_show_text`（已分流）
- `test_execute_present_choices`（已分流）
- `test_execute_show_character`（已分流）

`low_value`
- `test_execute_show_character_reposition_with_dissolve_is_teleport`（已分流）
- `test_hide_nonexistent_character_no_panic`（已分流）
- `test_extend_text`（已分流）
- `test_title_card_sets_render_state_and_effect`（已分流）

`undecided`（已分流完毕）

### `host/src/resources/tests/`

`high_value`（本模块已无独立 `low_value.rs`，下列含原 undecided 分流结果）

- `test_failed_texture_cache_can_suppress_retries`
- `test_headless_load_texture_full_flow`
- `test_headless_load_texture_cache_hit`
- `test_headless_load_texture_missing_returns_error`
- `test_load_failed_texture_suppresses_retry`
- `test_materialize_to_fs_zip_source`
- `test_headless_no_texture_context_returns_error`（已分流）
- `test_preload_textures_stops_on_first_error`（已分流）
- `test_preload_textures_success`（已分流）
- `test_read_text_missing_returns_error`（已分流）
- `test_resource_exists`（已分流）
- `test_list_files_delegates_to_source`（已分流）
- `test_image_crate_can_decode_webp`（已分流）
- `test_logical_path_as_cache_key`（已分流）
- `test_texture_cache_stats_after_load`（已分流）

`undecided`（已分流完毕）

## 长尾规则

- `NavigationStack`、`SceneTransition`、`RenderState` inline effect、`ResourceManager` 失败抑制、`SaveManager` 损坏输入恢复，应默认视为 `high_value` 候选。
- `Display`、路径文件名格式、默认值、简单 `new()`、纯 roundtrip serialization，默认视为 `low_value`。
- 冒烟式集成测试、helper 边界测试、schema/serde 测试，如未精确约束副作用或错误分类，可先落 `undecided` 再收敛（当前仓库 undecided 队列已空）。

## 执行记录（2026-03-20 测试质量提升）

### Phase 0+1：归类修正 + 删除无价值测试

- 约 71 个被错误归类为 low_value 的测试移入 high_value
- 约 42 个纯机械覆盖率测试被删除（Display/getter/构造器/Default/委托标准库）
- 11 个 low_value.rs 文件因清空而被删除（`script/ast`、`script/expr`、`command`、`diagnostic`、`runtime/engine`、`manifest`、`save_manager`、`command_executor`、`resources`、`ui/layout`、`renderer/scene_transition`；其中 engine / command_executor 的 `low_value.rs` 在 Phase 2 为承接下沉用例而重建）

### Phase 2：undecided 分流

- 167 个 undecided 测试全部分流（31 → high_value，136 → low_value）
- 所有 tests/mod.rs 不再有 `#[test]` 函数

### Phase 3：编排层测试基建 RFC

- 创建 `RFCs/rfc-app-layer-test-infra.md`（Draft）

### Phase 4：断言深度提升

- executor low_value 5 个测试补充四字段断言
- history 时间戳改为 before/after 夹逼
- command_executor `test_batch_error_stops_execution` 重命名
- easing elastic/bounce 断言提升

### 最终测试计数

- vn-runtime：273 passed
- host：534 passed
- 总计：807 passed（原 815，净减 8）
