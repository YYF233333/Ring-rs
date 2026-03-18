# 测试审计清单（2026-03）

> 目标：为“高价值 / 低价值 / undecided”分层提供统一基线，后续新增测试默认先对照本清单归类。

## 分类口径

- `high_value`：验证不变量、状态机转换、错误分类、边界条件、对外契约、跨模块链路、恢复/兼容语义。
- `low_value`：主要复读实现或提高机械覆盖率，例如构造器默认值、getter/setter、`Display`、纯 `serde` roundtrip、简单路径/字符串格式化。
- `undecided`：命中了真实业务路径，但断言还偏浅，或 helper/冒烟测试暂时难以快速定级。

## 范围总览

| crate / 文件 | 测试数 | 结论 |
| --- | ---: | --- |
| `vn-runtime/src/script/parser/tests.rs` | 100 | 高价值密度最高，但夹杂 helper 级低价值与灰区样本 |
| `vn-runtime/src/runtime/executor/tests.rs` | 53 | 以高价值语义/契约测试为主 |
| `vn-runtime/src/runtime/engine/tests.rs` | 28 | 以高价值状态机/恢复测试为主 |
| `vn-runtime/src/diagnostic/tests/` | 20 | 高低价值混杂，适合分层 |
| `vn-runtime/src/script/expr/tests.rs` | 14 | 高价值为主，少量 Display 低价值 |
| `vn-runtime/src/command/tests.rs` | 13 | 灰区与低价值为主，少量契约测试可上调 |
| `vn-runtime/src/script/ast/tests.rs` | 8 | 以灰区 helper/容器语义为主 |
| `vn-runtime/src/history.rs` 内联测试 | 8 | 混合，容量/顺序较高价值，时间戳/清空偏低 |
| `vn-runtime/src/save.rs` 内联测试 | 7 | 兼容性高价值，Display/字符串偏低 |
| `vn-runtime/src/state.rs` 内联测试 | 6 | `persistent` 隔离高价值，其余多为低/灰 |
| `vn-runtime/src/input.rs` 内联测试 | 2 | 低价值 |
| `host/src/command_executor/tests.rs` | 46 | 高价值密度高，Runtime -> Host 契约核心 |
| `host/src/renderer/render_state/tests.rs` | 34 | 高低混杂，状态机和 inline effect 很值钱 |
| `host/src/input/tests.rs` | 28 | 输入契约与防抖高价值，getter/default 偏低 |
| `host/src/manifest/tests.rs` | 29 | 高低混杂，schema/默认值比例较高 |
| `host/src/save_manager/tests.rs` | 22 | 错误恢复和多文件副作用高价值，格式化偏低 |
| `host/src/resources/tests.rs` | 21 | 资源抽象/错误恢复高价值，创建类测试偏低 |
| `host/src/config/tests.rs` | 20 | 配置 schema 与严格校验高价值，默认值偏低 |
| `host/src/renderer/headless_tests/` | 18 | 组合绘制链路→high_value，几何/枚举/getter→low_value；已分层为 `headless_tests/mod.rs` + `high_value.rs` + `low_value.rs` |
| `host/src/renderer/effects/resolver.rs` 内联测试 | 18 | 契约与回退逻辑高价值 |
| `host/src/renderer/transition.rs` 内联测试 | 14 | 状态机与 skip 行为高价值，debug/默认值偏低 |
| `host/src/renderer/scene_transition/tests.rs` | 13 | 高价值状态机测试 |
| `host/src/app/app_mode/tests.rs` | 13 | 导航栈高价值，辅助函数与页码枚举偏低/灰 |
| `host/src/resources/source.rs` 内联测试 | 13 | 资源来源抽象高价值 |
| `host/src/resources/path.rs` 内联测试 | 12 | 路径规范化与解析高价值 |
| `host/src/ui/layout/tests.rs` | 10 | 整体偏低/灰 |
| `host/src/resources/cache.rs` 内联测试 | 10 | 缓存淘汰/统计偏灰到高 |
| `host/src/extensions/tests.rs` | 10+ | 兼容性/能力校验高价值 |
| `host/src/ui/screen_defs/tests.rs` | 10+ | 条件求值与资源回退高价值，schema/serde 偏灰 |
| `host/src/renderer/character_animation.rs` 内联测试 | 8 | 偏灰 |
| `host/src/renderer/animation/transform.rs` 内联测试 | 13 | 低/灰为主 |
| `host/src/renderer/animation/system.rs` 内联测试 | 13 | 状态机与注册/回收高价值 |
| `host/src/renderer/animation/core.rs` 内联测试 | 5 | 灰区偏高 |
| `host/src/renderer/background_transition.rs` 内联测试 | 4 | 低/灰 |
| `host/src/renderer/animation/easing.rs` 内联测试 | 4 | 低价值 |
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

### `vn-runtime/src/runtime/engine/tests.rs`

`high_value`
- `test_call_script_and_return_flow`
- `test_call_script_auto_return_on_child_eof`
- `test_wait_for_signal_clears_only_on_expected_id`
- `test_state_mismatch_error`
- `test_invalid_choice_index_error`
- `test_choice_selected_label_not_found_error`
- `test_wait_for_time_click_interrupts`
- `test_wait_for_time_ignores_non_click_input`

`undecided`
- `test_runtime_history_recording`
- `test_record_history_for_background_and_bgm`
- `test_record_history_for_extend_text`
- `test_record_history_for_chapter_mark`
- `test_record_history_for_stop_bgm`
- `test_runtime_state_restore`

`low_value`
- `test_runtime_creation`
- `test_runtime_find_label_delegates_to_current_script`
- `test_runtime_is_finished`
- `test_runtime_restore_ctor`
- `test_runtime_restore_history`

### `vn-runtime/src/runtime/executor/tests.rs`

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

`undecided`
- `test_execute_dialogue`
- `test_execute_choice`
- `test_execute_goto`
- `test_execute_show_character`
- `test_execute_show_character_updates_state`
- `test_execute_change_scene_without_transition`
- `test_execute_change_scene_non_mask_named_arg_preserved`
- `test_execute_extend_text`
- `test_execute_extend_text_no_wait`
- `test_execute_dialogue_no_wait`
- `test_execute_conditional_true_branch`
- `test_execute_conditional_else_branch`
- `test_execute_conditional_elseif`
- `test_execute_conditional_with_wait_in_branch`

`low_value`
- `test_execute_label_no_command`
- `test_execute_textbox_hide`
- `test_execute_textbox_show`
- `test_execute_textbox_clear`
- `test_execute_full_restart_emits_command`
- `test_execute_bgm_duck`
- `test_execute_bgm_unduck`
- `test_execute_stop_bgm`

### `vn-runtime/src/diagnostic/tests.rs`

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

`undecided`
- `test_extract_resource_references_in_conditional`
- `test_get_defined_labels`
- `test_diagnostic_result_filter`
- `test_diagnostic_result_merge`
- `test_diagnostic_result_warn_count_and_non_empty`

`low_value`
- `test_diagnostic_display`
- `test_diagnostic_display_without_line_and_detail`
- `test_diagnostic_display_info_level`
- `test_resource_type_display`
- `test_extract_resource_references_show_without_path`

### `vn-runtime/src/command/tests.rs`

`high_value`
- `test_transition_get_arg_fallback`
- `test_transition_get_duration_and_reversed_wrong_type_returns_none`
- `test_transition_get_duration_and_reversed`
- `test_position_parse_supports_all_variants`

`undecided`
- `test_transition_get_named`
- `test_transition_get_positional`
- `test_transition_is_all_named_false_for_positional_args`
- `test_position_from_str`
- `test_transition_with_named_args`

`low_value`
- `test_transition_simple`
- `test_transition_with_args`
- `test_command_serialization`
- `test_transition_serialization_with_named_args`

### `host/src/command_executor/tests.rs`

`high_value`
- `test_execute_show_character_diff_and_move_uses_diff_then_move`
- `test_change_scene_rule_produces_scene_transition`
- `test_explicit_duration_overrides_default_for_all_targets`
- `test_execute_show_character_reposition_with_move_triggers_animation`
- `test_execute_show_background_with_transition`
- `test_batch_error_stops_execution`
- `test_title_card_sets_render_state_and_effect`

`undecided`
- `test_execute_show_text`
- `test_execute_present_choices`
- `test_execute_show_character`
- `test_execute_show_character_reposition_with_dissolve_is_teleport`
- `test_hide_nonexistent_character_no_panic`
- `test_extend_text`

`low_value`
- `test_executor_creation`
- `test_full_restart_is_noop`

### `host/src/resources/tests.rs`

`high_value`
- `test_failed_texture_cache_can_suppress_retries`
- `test_headless_load_texture_full_flow`
- `test_headless_load_texture_cache_hit`
- `test_headless_load_texture_missing_returns_error`
- `test_load_failed_texture_suppresses_retry`
- `test_materialize_to_fs_zip_source`

`undecided`
- `test_headless_no_texture_context_returns_error`
- `test_preload_textures_stops_on_first_error`
- `test_preload_textures_success`
- `test_read_text_missing_returns_error`
- `test_resource_exists`
- `test_list_files_delegates_to_source`

`low_value`
- `test_image_crate_can_decode_webp`
- `test_resource_manager_creation`
- `test_logical_path_as_cache_key`
- `test_texture_cache_stats_after_load`

## 长尾规则

- `NavigationStack`、`SceneTransition`、`RenderState` inline effect、`ResourceManager` 失败抑制、`SaveManager` 损坏输入恢复，应默认视为 `high_value` 候选。
- `Display`、路径文件名格式、默认值、简单 `new()`、纯 roundtrip serialization，默认视为 `low_value`。
- 冒烟式集成测试、helper 边界测试、schema/serde 测试，如未精确约束副作用或错误分类，先落 `undecided`。
