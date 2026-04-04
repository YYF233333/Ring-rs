# Host -> host-tauri 迁移差距分析

> **归档声明**：host-tauri 已停止开发，迁移目标改为 host-dioxus（见 RFC-033）。
> 本文档保留为历史记录，记录了 host-tauri 时期的功能对齐分析。

> LastReviewed: 2026-03-30
> 审计方式: 摘要优先 + 关键源码抽样复核
> 范围: `host/`、`host-tauri/`、相关摘要文档与当前实现
> 判定口径: 只把“用户可感知能力 / 宿主维护能力”缺失记为 gap；纯技术栈替代不记为 gap

## 一、结论摘要

- `host-tauri` 已经完成基础宿主闭环：`vn-runtime -> CommandExecutor -> RenderState -> Vue`、槽位存档基础链路、缩略图、FS/ZIP 资源访问、基础音频、`RequestUI`、浏览器调试都能跑通。
- 2026-03-30 补齐后，`host-tauri` 已新增后端 authoritative 的 `host_screen`、`client_token` ownership、`Continue` 生命周期、`start_at_label`、strict `config`/`manifest` bootstrap，以及 `debug_run_until + trace bundle` 这条 deterministic harness 基线。
- 当前迁移的主要问题已从“宿主基础设施缺失”收敛到“剩余语义差异”。高优先级 gap 主要剩下部分渲染/演出语义、存读档/历史页等 UX 细节，以及比旧 `host` 更完整的自动化能力（EventStream / replay parity）。
- 旧文档当前真正偏乐观的已不再是 `config` / `manifest` bootstrap，而是 `save/load/continue` 的恢复边界、演出 skip fidelity，以及若干前端交互细节；这些仍不能写成“高保真对齐”。
- `wgpu`、`egui`、`rodio`、`ffmpeg-sidecar`、`winit` 等旧 host 的平台落地细节不应按一比一迁移追责；应以玩家可感知行为和维护能力为主。

## 二、状态概览

| 领域 | 当前判断 | 主要结论 |
|------|----------|----------|
| 应用编排 / 交互 | 部分对齐 | `host_screen + client_token + Continue + StartAtLabel` 已齐；剩余主要是播放模式、历史页与存读档 UX |
| 渲染 / 效果 / 媒体 | 部分对齐 | 基础画面可用，但转场、角色层级/淡入、scene effect、键盘交互仍有语义缺口 |
| 资源 / 配置 / 存档 | 部分对齐 | FS/ZIP、缩略图、UI 配置桥接、strict config、manifest 校验、debug ZIP parity 已齐；剩余主要是存读档 UX |
| 调试 / 自动化 | 部分对齐 | 浏览器调试、`debug_run_until` 与薄 CLI harness 已成型；剩余主要是 `EventStream` / 输入回放 / 更强自动化 parity |
| 架构扩展性 | 低优先级差距 | capability / plugin 系统未迁，但当前不是玩家可见 blocker |

## 三、已对齐能力

- `Command -> RenderState -> 前端渲染` 主链路已闭环：`host-tauri/src-tauri/src/commands.rs`、`state.rs`、`command_executor.rs`、`render_state.rs` 与 `host-tauri/src/composables/useEngine.ts`、`host-tauri/src/vn/VNScene.vue` 已形成稳定工作流。
- 基础 VN 画面能力已迁移：背景、角色、ADV/NVL 文本、选项、章节标记、标题卡、视频过场在 `host-tauri/src/vn/` 下都有对应组件与后端状态。
- 槽位存档文件读写与列表展示主链路已落地：`host-tauri/src-tauri/src/save_manager.rs`、`commands.rs`、`host-tauri/src/screens/SaveLoadScreen.vue` 已支持槽位存读档与列表展示。
- 缩略图主链路已迁移：`host-tauri/src/composables/useSceneCapture.ts` 在前端合成场景图，`save_game_with_thumbnail` 与 `get_thumbnail` 负责持久化与展示。
- 持久化变量与回退快照已迁移：`host-tauri/src-tauri/src/state.rs` 保留了 `PersistentStore` 与回退快照能力，`Backspace` 回退链路可工作。
- `RequestUI` 基础闭环已迁移：`active_ui_mode`、`submit_ui_result`、`MapOverlay.vue`、`MiniGameOverlay.vue` 可承接 `show_map` 与 `call_game`。
- FS / ZIP 资源访问已迁移：`host-tauri/src-tauri/src/resources.rs` 与 `lib.rs` 中的 `ring-asset` 协议让正式运行时前端对资源来源基本透明。
- UI 配置桥接已迁移：`get_screen_definitions`、`get_ui_assets`、`get_ui_condition_context` 以及 `host-tauri/src/composables/useScreens.ts`、`useTheme.ts` 已接起 `screens.json` / `layout.json`。
- 基础音频能力已迁移：`host-tauri/src-tauri/src/audio.rs` + `host-tauri/src/composables/useAudio.ts` 已支持 BGM、SFX、duck、fade、crossfade 的声明式同步。
- 交互式调试基线已迁移：`host-tauri/src-tauri/src/debug_server.rs` 提供 IPC 镜像、资源服务和 `debug_snapshot`，足以支撑浏览器调试与 Agent 自动化。
- 宿主 authority 已迁移：`host-tauri/src-tauri/src/state.rs`、`render_state.rs`、`commands.rs`、`host-tauri/src/composables/useEngine.ts` / `App.vue` 现在通过 `host_screen + client_token` 收敛前后端状态，非 `InGame` 屏幕不会继续推进剧情。
- `Continue` 生命周期与开发重入原语已补齐：`save_continue` / `delete_continue`、`return_to_title(save_continue)`、`init_game_at_label` 与前端标题动作已接通。
- `bootstrap` 与资源 parity 已补齐：`config.rs` 恢复 `deny_unknown_fields + validate()`，`manifest.rs` 增加 `parse_and_validate()` 与 `ManifestWarning`，`debug_server.rs` 的 `/assets/*` 改为走 `ResourceManager`，浏览器调试可覆盖 FS/ZIP 两种来源。
- `debug_run_until + trace bundle` 已形成 smoke 级自动化基线：`state.rs::debug_run_until()`、`HarnessTraceBundle`、`host-tauri/scripts/harness-smoke.mjs` 与 `src-tauri/src/headless_cli.rs` 已提供浏览器态和 CLI 态两条 fixed-step 入口，但仍缺少旧 `host` 那种 `EventStream` / replay 完整配套。

## 四、剩余 gap 与部分对齐

### 4.1 应用编排与交互

- `save/load/continue` 已按旧 host 口径收敛到稳定恢复边界：当前实现不再把 `Choice / UIResult / Signal / Cutscene` 这类宿主中间态直接写入 slot/continue，而是退回最近 snapshot 边界保存；旧存档中的残留等待态也会保护性归一到 `WaitForClick`。判定：`已按旧 host 对齐（仍非 full-frame save）`。

- `P1 播放模式与演出跳过仍是部分对齐`：菜单/系统页打开时已统一回到 `Normal`，Skip 也会快进 `Time/Signal/Cutscene` 这类 host wait；但它仍不等价于旧 `host` 的 `skip_all_active_effects()`，不会一并收敛背景 dissolve、角色淡入、scene effect、title card、scene transition 这类活跃演出。判定：`部分对齐`。
- `P2 存读档 UX 仍是简化版`：`host-tauri/src/screens/SaveLoadScreen.vue` 固定为 `5` 页 * `6` 槽，只展示 `1..30`；快存/快读在 `host-tauri/src/App.vue` 中被映射到隐藏的 `slot 0`；删除与覆盖仍没有旧 `host` 的确认流程。判定：`部分对齐`。
- `P3 历史信息不完整`：`host-tauri/src-tauri/src/state.rs::HistoryEntry` 只有 `speaker` 和 `text`，`HistoryScreen.vue` 也只显示这两项；旧 `host` 历史里有更完整的章节/事件语义。判定：`部分对齐`。
- `P2 show_map 退化为按钮地图`：`host-tauri/src/vn/MapOverlay.vue` 定义了 `hit_mask` / `mask_color` 字段，但当前实现只是在 `1920x1080` 设计坐标上摆按钮，并不消费命中图。`Esc` 取消仍保留，但复杂热点图并非等价迁移。判定：`部分对齐`。
- `P2 小游戏桥接 API 缩水`：旧 `host` 的小游戏桥接提供音频、状态读写、完成回传等较完整 API；新的 `host-tauri/public/engine-sdk.js` 只有 `getInfo()`、`assetUrl()`、`log()`、`complete()`。当前架构本身可以接受，但 API 面明显缩小。判定：`部分对齐`。

### 4.2 渲染、效果与媒体语义

- `P1 背景 dissolve 不是完整双层交叉淡化`：`host-tauri/src/vn/BackgroundLayer.vue` 在旧背景元素创建时就直接给 `opacity: 0`，新背景也没有显式 `0 -> 1` 的起始态；从代码看更像“瞬时切换 + CSS 包装”，而不是旧 `host` 的真正双层交叉渐变。判定：`部分对齐`。
- `P1 角色层级与入场淡入语义不对齐`：`host-tauri/src-tauri/src/render_state.rs::show_character()` 默认把新角色的 `z_order` 设为 `0`；`host-tauri/src/vn/CharacterLayer.vue` 又直接用 `target_alpha` 而不是当前 `alpha` 渲染。结果是角色能出现，但入场淡入与层级排序不再等价于旧 `host`。判定：`部分对齐`。
- `P1 场景过渡状态机与 Skip 语义被简化`：`host-tauri/src-tauri/src/render_state.rs` 只保留 `FadeIn / Hold / FadeOut / Completed`，没有旧 `host` 的更细 phase 语义；`host-tauri/src/vn/RuleTransitionCanvas.vue` 也把 rule 过渡简化为 `960x540` 的 CPU canvas；更关键的是，当前 Skip 只是清 wait，不等价于旧 `host` 对活跃 transition/effect 的统一 skip-to-end。判定：`部分对齐`。
- `P2 blur / dim scene effect 语义弱化`：`host-tauri/src-tauri/src/state.rs::apply_scene_effect()` 对 `Blur` / `BlurOut` / `Dim` / `DimReset` 直接设置瞬时值并立刻把 `scene_effect_active` 置回 false，duration 基本未被消费；前端 `dim` 也只是亮度滤镜，不是旧 `host` 的黑遮罩。判定：`部分对齐`。
- `P2 选项键盘导航缺失`：`host-tauri/src/vn/ChoicePanel.vue` 只有鼠标 hover / click；`host-tauri/src/App.vue` 的键盘处理也没有旧 `ChoiceNavigator` 对 `ArrowUp/ArrowDown/W/S/Enter/Space` 的完整选择导航语义。判定：`部分对齐`。
- `P3 章节标记时长不完全一致`：功能存在，但 `host-tauri` 的 phase 时长与旧 `host` 并非一比一。判定：`低优先级部分对齐`。

### 4.3 资源、配置、存档与工具链

- `P2 headless automation parity 仍是精简版`：`host-tauri/src-tauri/src/main.rs --headless-harness` 已提供独立 CLI wrapper，可直接初始化 runtime 并导出 trace bundle；但与旧 `host` 相比，仍缺少 replay / timeout / EventStream 等更完整的自动化工具链。判定：`部分对齐`。
- `P2 EventStream 未迁移`：当前有 `debug_server`、`debug_snapshot` 和日志转发，但没有旧 `host/src/event_stream/mod.rs` 那种按时间序列输出的 JSONL 事件流。它可以被视为“有意不迁移的调试能力降级”，不能算“已经等价覆盖”。判定：`不迁移，但需在文档中明确不是等价替代`。
- `不迁移 输入录制/回放`：当前 `host-tauri` 完全没有录制缓冲、导出器、回放器或 CLI 接线。如果团队决定用浏览器自动化、`vn-runtime` 单测和 debug server 替代，这项可以继续明确标为“不迁移”，不应挂在 P1/P2 backlog。判定：`明确不迁移`。

## 五、可视为技术栈替代而非 gap 的项目

- `wgpu` / `TextureFactory` / `DrawCommand` / `SpriteRenderer` / `DissolveRenderer` / `TextureCache` 等 GPU 落地细节不需要在 `host-tauri` 里一比一复制；新宿主的正确边界是 `RenderState -> DOM/CSS/canvas`。
- `egui` 即时模式 UI、`winit` 事件 plumbing、WebView 嵌入与窗口生命周期都属于旧平台实现细节；新宿主改成 `Vue + Tauri IPC` 是合理替代。
- `rodio` sink、`ffmpeg-sidecar`、视频音频解码细节也不需要同构迁移；需要对齐的是 fade、duck、resume 等玩家可感知语义，不是底层 API。
- `RequestUI` 与小游戏承载架构收敛到 `active_ui_mode + iframe + postMessage` 本身可以接受；当前真正的 gap 是 API 面缩水和地图/小游戏语义弱化，而不是“没有继续使用旧的 `UiModeRegistry + BridgeServer` 形态”。
- `ExtensionRegistry` / `CapabilityId` / `EffectExtension` 缺失更像“低优先级架构差距”，不是当前玩家可见迁移 blocker。只有未来要做第三方效果插件时，才需要单独立项。

## 六、对旧版 gap 文档结论的修正

- `config` 已从“基础加载已对齐，严格校验未对齐”升级为“strict schema + validate 已对齐”。
- `manifest` 已从“消费链路已对齐，校验与告警未对齐”升级为“消费链路 + parse_and_validate + 启动期告警已对齐”。
- `save_manager` 相关结论应拆开写：槽位文件读写、缩略图、Continue 生命周期已对齐；slot/continue 现在也已按旧 host 的粗粒度恢复口径收敛，但这仍不等价于 snapshot/backspace 的 full-frame restore。
- `Headless CLI` 已从建议项升级为已落地的薄 wrapper；后续真正的 gap 是 replay / EventStream / timeout 等 automation parity。
- `debug_run_until + trace bundle` 现在已同时具备 debug server 和 CLI 两种入口，但仍应表述为“调试 / smoke 级的部分替代”，不是旧 `headless` runner 的完全等价实现。
- `EventStream 不迁移` 这条结论可以保留，但表述必须收紧为“当前没有等价实现，交互式调试改由 `debug_server + debug_snapshot + 日志` 承担”，不能写成“已经被完整替代”。
- `输入录制/回放不迁移` 这条结论仍成立；当前代码里没有任何迁移骨架，说明它是明确舍弃项而不是未完工项。
- 前端交互层不能只笼统写“部分 UX 细节”：目前主要剩 `SaveLoadScreen` 的分页/确认语义与历史页信息密度，而 `active_ui_mode` modal、fullscreen 应用、cutscene duck/unduck 已经补齐。
- 渲染层不能再把“背景 dissolve / 角色 fade+z-order / 场景过渡 Skip / blur-dim 时序”统称为“已对齐”；这些都属于仍需补齐语义的部分对齐项。

## 七、建议优先级

- `P1`：继续收敛 Skip 对活跃演出的等价语义；若追求旧 `host` 级自动化，再补 replay / EventStream / timeout 等 harness 能力。
- `P2`：完善存读档页的分页/确认/快档语义；恢复地图 hit-mask 模式；补小游戏 bridge 缺失的状态/音频 API。
- `P3 / 按需`：补历史页的章节/事件信息、选项键盘导航、章节标记精确时长，以及可插拔 capability / extension 架构。
