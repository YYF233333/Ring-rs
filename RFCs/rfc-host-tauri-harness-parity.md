# RFC-032: host-tauri Harness 能力对齐

## 元信息

- 编号：RFC-032
- 状态：Active
- 作者：GPT-5.4
- 日期：2026-03-30
- 相关范围：`host-tauri/src-tauri/src/{lib.rs,state.rs,commands.rs,render_state.rs,debug_server.rs,resources.rs,config.rs,manifest.rs,save_manager.rs}`、`host-tauri/src/{App.vue,composables/useEngine.ts,composables/useNavigation.ts,composables/useBackend.ts}`、`host-tauri/package.json`、`docs/host-tauri/*`、`docs/maintenance/host-migration-gap-analysis.md`
- 前置：RFC-029、RFC-031；参考：RFC-018、RFC-019

---

## 背景

`host-tauri` 已经完成了基础宿主闭环：`vn-runtime -> CommandExecutor -> RenderState -> Vue`、手动存读档、缩略图、FS/ZIP 资源访问、基础音频、`RequestUI`、浏览器调试都能工作。

当前迁移的主要问题不再是“有没有这个模块”，而是“宿主 harness 是否足够 authoritative、可重复、可验证”。RFC 提出时的高频 bug 源如下：

1. 页面/菜单切换只存在于前端路由层，后端不知道当前是否应暂停推进，导致非 `ingame` 屏幕下脚本、等待态、播放模式仍可能继续前进。
2. `Continue` 只剩读取能力，缺少维护生命周期，快速回到问题现场的链路不完整。
3. 浏览器调试依赖 `RING_HEADLESS=1` 这一人工约定来避免双客户端竞争，同一会话可能被多个 `requestAnimationFrame -> tick` 循环同时驱动。
4. `config`/`manifest`/资源 debug 路径存在 fail-open 或旁路 `ResourceManager` 的情况，开发态与正式运行态不完全等价。
5. 缺少 deterministic、非交互的 harness 入口，难以形成“复现 -> 诊断 -> 修复 -> 回归验证”的稳定开发迭代链路。

旧 `host` 在这些方面已经形成成熟基线，但本 RFC 不要求 `host-tauri` 逐文件、逐格式复制旧实现，而是要求达成**能力对齐**：让新宿主具备同等级别的 authoritative 编排、快速重入、自动化验证、机读诊断与严格 bootstrap 能力。

### 当前差距的能力视图

| 能力 | 旧 `host` | 当前 `host-tauri` | 问题性质 |
|------|-----------|-------------------|----------|
| authoritative 宿主模式 | 有 | 有（主干已落地） | 剩余主要是前端桥接复杂度与 `active_ui_mode` 非严格 modal |
| `Continue` / 重入 | 完整 | 部分 | 生命周期与 `start_at_label` 已齐，但恢复边界仍缺 |
| deterministic harness | 有 | 部分 | `debug_run_until` 已落地，但仍不是独立 runner |
| 机读诊断产物 | 有 | 部分 | 已有 trace bundle，但无 EventStream / CLI parity |
| strict bootstrap / 资源透明性 | 有 | 有（主干已落地） | strict config/manifest + ResourceManager parity 已补齐 |

## 实施快照（2026-03-30）

- 已完成：后端 authoritative 的 `host_screen` / `client_token` ownership、`playback_mode` 单一真源、`start_game_at_label`、`Continue` 生命周期、strict `config` / `manifest` bootstrap、`ResourceManager` 统一资源路径、`debug_run_until + trace bundle` 基线，以及 `--headless-harness` 薄 CLI wrapper。
- 部分完成：`save/load/continue` 已按旧 host 的粗粒度恢复口径收敛到稳定边界，但仍不会重建存档瞬间的 `dialogue` / `choices` / `active_ui_mode`；自动化侧仍缺 `EventStream` / replay / timeout 等更完整工具链。
- 仍未完成：旧 `host` 级的 automation parity、输入回放链路，以及部分演出等价性（如 skip-all-active-effects、地图 hit-mask、部分转场/scene effect fidelity）。
- 当前 RFC 的重点已从“把 authority 拉回后端”转为“收敛恢复边界与自动化入口”，因此状态应从 `Proposed` 调整为 `Active`。

---

## 目标与非目标

### 目标

- 定义 `host-tauri` 中 authoritative 的宿主 harness 边界，让后端而不是前端页面状态决定“是否推进游戏”。
- 补齐 `save/load/continue/start-at-label` 的重入与恢复链路，使开发者和自动化都能快速回到问题现场。
- 为 `host-tauri` 提供 deterministic、非交互的验证入口，支持 fixed-step 驱动、超时/退出条件和机读诊断产物。
- 恢复 bootstrap 的 fail-fast 语义，确保配置、manifest、资源访问在开发调试与正式运行之间保持一致口径。
- 将浏览器调试、Tauri WebView 调试、未来 CLI/自动化入口统一到同一 harness core 上，避免分裂的驱动协议。

### 非目标

- 不要求 1:1 复刻旧 `host` 的 `EventStream` JSONL 格式、输入录制文件格式或 CLI 参数外观。
- 不在本 RFC 中解决渲染语义细节问题，如 dissolve、scene effect、角色入场淡入、cutscene/BGM 编排等。
- 不在本 RFC 中扩展小游戏 bridge API、地图 hit-mask 或历史页面信息密度。
- 不进行与 harness 无关的前端路由/组件重写；只收敛对宿主 authority 有影响的状态边界。
- 不承诺首期就提供完整的录像回放系统；只要求 deterministic 驱动与可机读产物。

---

## 方案设计

### 1. 引入后端 authoritative 的 HarnessCore

`AppStateInner` 继续作为会话真源，但其职责从“命令执行 + 状态堆积”升级为“宿主 harness core”。实现上引入一个后端 authoritative 的宿主模式枚举（名称可在实现时微调，例如 `HostSessionMode`），至少覆盖以下语义状态：

- `Title`
- `InGame`
- `OverlayPaused`（游戏内菜单、存读档、设置、历史等应阻断推进的系统页）
- `UiModeActive`（`RequestUI` 驱动的 runtime 交互态）
- `Suspended` 或等价的不可推进态

该模式决定：

- `tick` 是否允许推进脚本、等待计时、过渡与播放模式
- 哪些输入可被消费
- 打开系统页时是否强制退出 `Auto/Skip`
- 当前会话是否允许保存 `continue`

前端的 `currentScreen`、`screenStack`、`showInGameMenu` 等状态不再是 authority，只是后端模式的 UI 投影。

### 2. 收敛客户端 ownership 与 tick 协议

当前最大的 harness footgun 是“多个客户端可同时驱动同一个 `AppStateInner`”。本 RFC 要求把单客户端 ownership 从文档约定升级为协议约束。

方案：

- `frontend_connected` 从“简单重置会话”升级为“声明连接并领取 session ownership”的入口。
- 后端为当前活动客户端生成 `client_token` 或等价 lease 标识。
- 所有会推进会话的命令（如 `tick`、点击、选择、保存、返回标题、播放模式切换等）都必须携带该 ownership 标识。
- 后端拒绝非当前 owner 的推进请求，并返回明确错误。
- Debug HTTP、Tauri IPC、未来 CLI wrapper 共享同一 ownership 规则；CLI/headless 作为专用 owner，不与 UI 客户端并存。

这样可以把“必须设置 `RING_HEADLESS=1` 才安全”的文档规则，收敛成工具本身保证的协议不变量。

### 3. 统一 playback 与页面暂停语义

`playback_mode` 必须以后端状态为单一真源，前端不再保有独立 authoritative 副本。

要求：

- `RenderState` 暴露后端 authoritative 的 `playback_mode` 与必要的宿主模式投影。
- 打开 `OverlayPaused` 页面时，后端统一把播放模式归一到 `Normal`。
- 非 `InGame` 态下，`tick` 只能刷新必要的展示/诊断字段，不能推进剧情或等待态。
- `RequestUI`、存读档、设置、历史等页面必须在后端层面显式声明是否暂停，而不是由前端自行约定。

目标不是把所有 UI 逻辑都搬回 Rust，而是确保“暂停不暂停、自动推进不推进、等待是否解锁”由同一处 authority 判断。

### 4. 补齐重入与恢复闭环

开发迭代链路要求可以快速进入问题现场，因此 `host-tauri` 需要统一以下重入原语：

- `start_game(script_path)`
- `start_game_at_label(script_path, label)`
- `continue_game()`
- `load_game(slot)`
- `return_to_title(save_continue: bool)`

其中 `Continue` 生命周期需明确为：

- 从活动游戏态返回标题时，可按策略写入 `continue`
- 开始新游戏时清理旧 `continue`
- 脚本自然结束时不保留过期 `continue`
- `quick save/load` 与 `continue` 不是同一概念，不能继续依赖“隐藏槽位别名”维持语义

恢复流程必须是**副作用受控**的：

- bootstrap 会话与“执行入口首帧”解耦
- 从 save/continue 恢复时，不得先额外跑一次 `run_script_tick()` 再回灌存档
- 恢复域至少覆盖 `runtime`、`render`、`history`、`audio`、`playback`、等待态与必要的宿主模式字段

如继续保留 `Backspace`/snapshot stack，其定位应是“会话内快速恢复能力”，并与上述重入原语共享同一恢复边界。

### 5. 提供 deterministic harness 与机读产物

能力对齐不要求复刻旧 `headless.rs` 的 CLI 形态，但要求提供可重复、可脚本化的驱动接口。为此，本 RFC 引入统一的 harness driver 能力：

- fixed-step `step(dt)` / `tick`
- `run_until(condition, max_steps, timeout)`
- 输入注入（点击、选项、UI 结果、信号）
- 快照导出
- trace 采集

建议优先级：

1. 先在 `debug_server` 上暴露统一的 deterministic driver API，服务浏览器调试、MCP 自动化与 smoke 验证。
2. 再基于同一 core 增加薄 CLI wrapper，避免单独维护一套 headless 逻辑分支。

为了让回归验证可归档，本 RFC 要求输出机读产物包（名称可实现时微调，以下简称 `trace bundle`），至少包含：

- `metadata.json`：脚本入口、驱动模式、step 配置、退出原因
- `trace.jsonl` 或等价机读 trace：关键状态转换与输入/输出摘要
- `final_snapshot.json`：结束态快照

可选附加产物：

- 中间快照
- 缩略图或截图
- 浏览器控制台/后端日志摘录

本 RFC 只要求这些产物**机器可读、可稳定生成、可用于回归比较**，不要求沿用旧 `host` 的文件格式。

### 6. 收紧 bootstrap 与资源 parity

`host-tauri` 当前的 bootstrap 问题不是“能不能启动”，而是“启动失败是否暴露得足够早，调试环境与正式环境是否是同一系统”。

本 RFC 要求：

- `config` 恢复 strict schema：关键结构体 `deny_unknown_fields`，并提供 `validate()`；缺失字段、未知字段、非法路径、非法范围值一律 fail-fast。
- `manifest` 至少要区分“可告警的缺省”和“必须中止的错误”，不能整体静默回退 `with_defaults()`。
- Debug 资源访问不再直接映射文件系统目录，而是统一走 `ResourceManager` / 逻辑路径解析，确保浏览器调试与正式运行都能验证 FS/ZIP 两种来源。
- `start_script_path`、UI 配置、关键资源根路径不允许再在前端各自做静默兜底。

目标是让环境错误尽早暴露，而不是把错误推迟到运行时表现异常。

### 7. 打包开发工作流

除了底层能力，还需要把它们固化成稳定入口。建议在 `host-tauri` 层提供以下标准工作流：

- `dev:tauri`：Tauri WebView 开发
- `dev:browser`：浏览器单客户端调试
- `harness:smoke`：跑一条 deterministic smoke path 并产出 `trace bundle`
- `--headless-harness`：不依赖浏览器/debug server 的薄 CLI harness

这些入口复用同一 harness core，只在驱动层不同。文档只负责说明场景与命令，不再承担维护关键不变量。

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `host-tauri/src-tauri/src/state.rs` | 引入宿主模式 authority、恢复边界、deterministic driver 核心 | 高 |
| `host-tauri/src-tauri/src/render_state.rs` | 增加宿主模式/播放模式的前端投影字段 | 中 |
| `host-tauri/src-tauri/src/commands.rs` | 接线 ownership、重入原语、driver API | 中 |
| `host-tauri/src-tauri/src/debug_server.rs` | 从“镜像 IPC”升级为 deterministic harness 入口之一 | 高 |
| `host-tauri/src-tauri/src/save_manager.rs` | 完整 `continue` 生命周期、恢复一致性 | 中 |
| `host-tauri/src-tauri/src/config.rs` / `manifest.rs` / `lib.rs` | strict bootstrap、fail-fast、启动阶段治理 | 中 |
| `host-tauri/src-tauri/src/resources.rs` | 调试资源路径与正式路径统一 | 中 |
| `host-tauri/src/composables/useEngine.ts` | ownership/token 管理、播放模式真源回灌、前端驱动收口 | 高 |
| `host-tauri/src/App.vue` / `useNavigation.ts` | 页面状态从 authority 改为后端投影 | 中 |
| `docs/host-tauri/*` / `docs/maintenance/*` | 调试文档、差距分析、经验沉淀同步更新 | 低 |

---

## 迁移计划

### Phase 1：authoritative 宿主模式与 ownership

1. 在后端定义宿主模式及 tick policy。
2. 让页面/菜单/`RequestUI` 显式切换宿主模式，而不是只改前端 screen。
3. 给推进类命令加入 ownership 校验，消除双客户端竞争。
4. 把 `playback_mode` 收回后端 authority，前端只消费投影。

### Phase 2：重入与恢复闭环

1. 接通 `start_at_label`。
2. 补齐 `Continue` 生命周期。
3. 拆分 bootstrap 与首帧执行，修复 save/load/continue 的副作用恢复。
4. 视需要统一 snapshot rollback 的恢复边界。

### Phase 3：deterministic harness 与 trace bundle

1. 抽出 fixed-step driver 核心。
2. 先在 `debug_server` 上提供 `step/run_until/snapshot/trace` 能力。
3. 建立 `trace bundle` 规范，并接入一条 smoke 验证路径。
4. 视收益补充 CLI wrapper，但共享同一 driver core。

### Phase 4：strict bootstrap 与资源 parity

1. `config`/`manifest` 收紧为 fail-fast。
2. debug 资源访问改走 `ResourceManager`。
3. 去掉前端和启动链路中的静默默认值兜底。
4. 整理标准开发脚本与调试文档。

---

## 验收标准

- [x] 非 `InGame` 页面或菜单打开时，脚本、等待计时、`Auto/Skip`、场景 signal 不再继续推进。
- [x] 同一会话不能被两个客户端同时驱动；非 owner 的推进请求会被明确拒绝。
- [x] `playback_mode` 以后端为唯一真源，菜单/系统页打开时统一回到 `Normal`。
- [x] `start_game_at_label` 可作为正式开发重入入口使用。
- [x] `Continue` 具备完整生命周期：写入、清理、恢复、失败处理均有明确语义。
- [ ] `save/load/continue` 恢复不会额外执行入口首帧副作用；`runtime/render/history/audio/playback` 能一致恢复。
- [x] 至少存在一条 deterministic 驱动入口，可 fixed-step 运行、限定退出条件并导出机读 `trace bundle`。
- [x] 浏览器调试与正式运行都通过统一资源访问路径验证 FS/ZIP 资源来源，不再存在 debug-only 文件系统旁路。
- [x] `config` 对缺失字段、未知字段、非法路径和非法范围值均 fail-fast；关键启动配置不再静默回退默认值。
- [x] `docs/host-tauri/debugging.md`、`docs/host-tauri/dev-guide.md`、`docs/maintenance/host-migration-gap-analysis.md` 与相关摘要已同步更新。
- [x] `cargo check-all` 通过，并新增覆盖宿主模式暂停、`Continue` 生命周期、strict bootstrap、resource parity 的高价值测试。
