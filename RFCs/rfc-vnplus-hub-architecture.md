# RFC: VN+ Hub 架构愿景

## 元信息

- 编号：RFC-024
- 状态：Proposed（v2：Tauri 路线）
- 作者：claude-4.6-opus
- 日期：2026-03-24（v1）；2026-03-25（v2 重设计）
- 影响范围：`host`（整体架构迁移）、`vn-runtime`（接口层面不变）
- 相关 RFC：RFC-022（UI Mode Plugin System）、RFC-004（扩展 API）、RFC-025 ~ RFC-028（子 RFC）

---

## 背景

Ring Engine 立项时定位为纯视觉小说引擎，核心架构假设为"世界是叙事驱动的"：脚本线性推进，Runtime 产出 Command，Host 执行 Command 渲染画面。这个假设对纯 VN 作品是精确的。

团队技术目标已演化：

- 已有纯 VN 重制计划（ref-project），需要当前引擎叙事能力
- 新立项作品包含 RPG-VN、卡牌 VN、战棋 VN、历史模拟 VN 等品类
- 共同特征：**仍在 VN 品类内部**，叙事是不可或缺的核心，玩法段落与对话交替推进
- 团队用 Godot 开发了一款 VS-like RPG（独立项目），积累了 Godot 经验
- 当前通过 WebView + HTTP Bridge（RFC-021/023）接入 web 小游戏承接非 VN 玩法

当前架构的张力：

1. VN 之外的玩法（卡牌对战、战棋、资源经营）有**独立的状态机和交互循环**，不是叙事的附属品
2. `game_mode`（WebView）是一个务实的逃逸舱，但在架构上是"例外通道"而非一等公民
3. 团队的玩法程序员技术栈是 Godot/GDScript，不熟悉 Rust，也不熟悉 web 开发框架
4. 项目优势是 vibe coding，大范围重构成本低；但单人维护，心智复杂度有上限
5. egui 在 VN 核心体验（富文本排版、精致 UI 样式、对话框设计）上表现力受限

### v1 → v2 的演进

v1 选择"原生 Rust（winit+wgpu+egui）一条路走到底"。经评估后发现：

- **VN 的核心体验是文本和 UI**，而这恰好是 Web 技术的绝对强项
- **egui 做精致 VN 界面很吃力**：富文本排版、CSS 级别的样式控制、响应式布局，egui 都需要大量手工代码
- **Hub 架构的多模态切换**在 WebView 中天然由前端路由实现，不需要额外的 trait 抽象
- **玩法脚本层**不需要嵌入 Lua——WebView 本身就是完整的脚本运行时
- **game_mode 的 HTTP Bridge hack** 在 Tauri 下自然消失——玩法组件直接是前端的一部分
- **Web 前端是 AI 编码能力最强的领域**，vibe coding 体验最好

v2 将技术路线从纯原生 Rust 调整为 **Tauri（Rust 后端 + Vue WebView 前端）**。

---

## 目标与非目标

### 目标

- 明确 Ring Engine 的定位演化方向：**VN+ 引擎**（面向叙事核心、玩法嵌入的 VN 品类作品）
- 提出 Hub 架构愿景：Host 从"VN 宿主"演化为"多模态容器"，VN 叙事是主模态之一
- **确定 Tauri 为 Host 层的目标技术栈**
- 定义架构演化的分阶段实施路径，确保每阶段独立可交付
- 为子 RFC（RFC-025 ~ RFC-028）提供统一上下文

### 非目标

- **不做通用游戏引擎**——不引入物理、碰撞、3D、骨骼动画等通用引擎能力
- **不预设玩法类型**——不提前为卡牌/战棋/模拟等特定品类建模，等具体项目约束
- **不立即废弃当前 wgpu host**——新旧 host 共存过渡，ref-project 在旧 host 上继续推进

---

## 核心判断

### 为什么不做通用游戏引擎

（不变，与 v1 相同）

1. **竞品差距不可追**：单人维护的 Rust 引擎在通用能力上无法追上 Godot/Bevy/Unity
2. **心智复杂度乘法增长**：通用引擎的每个系统（ECS/物理/碰撞/寻路/粒子）互相耦合
3. **比较优势在叙事**：vn-runtime 的显式状态、确定性执行、Command 驱动是差异化价值
4. **VN 品类的玩法复杂度有结构性上限**：2D、回合制/半回合制、状态机驱动，不需要通用引擎能力

### 为什么从原生 Rust 转向 Tauri

v1 排除 WebView 的理由是"团队无 web 开发经验"。这个判断暗含前提：**开发工作主要由人类完成**。但本项目以 vibe coding 为主，AI 编码在 Web 前端领域的能力最强（训练数据最丰富、生态最成熟）。重新评估：

| 方案 | 评估（v2） |
|------|-----------|
| Bevy | 0.x API 频繁 breaking，AI 内建知识过时严重 |
| Godot 嵌入 | gdext 仍 pre-1.0，与 vibe coding 适配差 |
| Unity | 版权/政策风险 |
| 纯原生 Rust (winit+wgpu+egui) | 引擎核心可行，但 **egui 在 VN 核心体验（文本+UI）上表现力不足**；Hub 多模态调度需要额外抽象 |
| **Tauri (Rust + Vue WebView)** | ✅ vn-runtime 不变，核心逻辑留在 Rust；✅ Web 天然强于文本排版和 UI；✅ 前端路由即模态切换；✅ WebView 即玩法脚本运行时，无需嵌入 Lua；✅ vibe coding 最优领域 |

关键前提更新：VN 品类的玩法需求本质是**纯数据 + 状态机 + 2D 渲染 + UI**。其中 Rust 负责数据和状态机（最擅长的），WebView 负责 2D 渲染和 UI（最擅长的）。

### 技术栈选定

| 层 | 技术 | 理由 |
|----|------|------|
| 核心逻辑 | Rust（vn-runtime、共享服务、状态管理） | 不变，已验证 |
| 桌面壳 | Tauri 2.x | 成熟、稳定、轻量，Rust 原生 |
| 前端框架 | Vue 3（Composition API） | AI 训练数据丰富；中文文档最好；vibe coding 准确率高 |
| 构建工具 | Vite | Vue 官方推荐，HMR 毫秒级 |
| 包管理 | pnpm | 硬链接缓存，解决 Windows node_modules 性能问题 |
| CSS 方案 | 纯 CSS / CSS Modules | 保持精简，不引入 Tailwind 等额外依赖 |
| 2D 特效 | CSS transitions + Canvas/WebGL（按需） | 覆盖 VN 所有特效需求 |

### 工具链策略："先做游戏，再做工具"

（方向不变，形态随 Tauri 调整）

| 层级 | 形态 | 应用 |
|------|------|------|
| L0 | 文本配置 + 热重载 | 卡牌定义、角色数值、技能树、经济参数 |
| L1 | 文本 DSL + 实时预览 | 叙事脚本（已有） |
| L2 | Web 组件式领域工具 | 脚本预览编辑器（Tauri 迁移后重新设计，基于 Monaco/CodeMirror） |
| L3 | 通用场景编辑器 | **不需要** |

---

## 方案设计

### 目标架构

```
Ring Engine (VN+) — Tauri 版
│
├── vn-runtime/（不变）
│   └── 叙事核心：脚本 → AST → Executor → Command
│
├── host-tauri/src-tauri/（Rust 后端）
│   ├── vn-runtime 集成（直接依赖）
│   ├── 共享服务层
│   │   ├── AudioManager（Rust 侧 rodio 播放）
│   │   ├── ResourceManager（asset: 协议暴露资源）
│   │   ├── SaveManager
│   │   ├── InputManager（消费前端 IPC 输入）
│   │   └── Config
│   ├── 状态管理（RenderState、CommandExecutor）
│   ├── Tauri IPC 命令层（tick、click、save、load...）
│   └── 模态调度（Rust 侧状态机）
│
├── host-tauri/frontend/（Vue WebView 前端）
│   ├── vn/（VN 渲染组件）
│   │   ├── BackgroundLayer.vue
│   │   ├── CharacterLayer.vue
│   │   ├── DialogueBox.vue
│   │   ├── ChoicePanel.vue
│   │   └── TransitionOverlay.vue
│   ├── screens/（UI 页面）
│   │   ├── TitleScreen.vue
│   │   ├── SaveLoadScreen.vue
│   │   ├── SettingsScreen.vue
│   │   ├── HistoryScreen.vue
│   │   └── ...
│   └── modes/（玩法模态，按需添加）
│       └── [future] CardBattle.vue / StrategyMap.vue / ...
│
└── host/（当前 wgpu host，保留，过渡期共存）
```

### Rust 后端 ↔ WebView 前端 IPC 协议

```
Rust 后端                              Vue 前端
────────                              ────────

AppState                               
├── vn-runtime                         
├── command_executor                   
├── RenderState ──── Tauri event ──►  VN 渲染组件
│   (serialize as JSON)                ├── 背景（<img> + CSS transition）
├── AudioManager                       ├── 立绘（<img> + CSS transform）
├── ResourceManager                    ├── 对话框（HTML rich text）
├── SaveManager                        ├── 选项（按钮列表）
│                                      └── 过渡遮罩（CSS/Canvas）
│   Tauri commands:                    
│   - tick()                           UI 页面
│   - click() / choose(idx)            ├── 标题 / 菜单（Vue 组件）
│   - save(slot) / load(slot)   ◄──── ├── 存档 / 读档
│   - get_render_state()               ├── 设置
│   - list_saves()                     └── 历史记录
│                                      
│   asset: 协议                        玩法模态
│   - 图片/音频资源直达前端      ──►   └── 直接作为 Vue 组件运行
```

### 叙事 ↔ 玩法交互协议

模态切换通过 vn-runtime 的 `Command::RequestUI` 触发，结果通过 `RuntimeInput::UIResult` 回传。Tauri 版下，切换即前端路由导航：

```
叙事流 ──► call_mode "card_battle" with { deck, enemy, ... }
              │
              ├─ Rust 后端暂停 vn-runtime tick
              │   └─ 通过 Tauri event 通知前端切换模态
              │
              ├─ 前端路由切换到 CardBattle.vue
              │   ├─ 通过 IPC 读取 variables（角色状态/道具）
              │   ├─ JS/TS 驱动玩法交互循环
              │   ├─ 渲染在 WebView 内完成
              │   └─ 完成后通过 IPC 回传结果 { winner, turns, ... }
              │
              ├─ Rust 后端恢复 vn-runtime tick
              │   └─ 结果写回 RuntimeState.variables
              │
              └─ 叙事流根据变量分支继续
```

### 团队协作模型（v2）

```
你（引擎）              玩法程序               编剧
    │                    │                    │
  Rust: 核心逻辑         JS/TS: 玩法 UI+规则   .rks: 叙事脚本
  Tauri 后端             Vue 组件               对话/演出/mode 调用
  共享服务 + IPC          数据配置（TOML/JSON）   
    │                    │                    │
    └────────────────────┴────────────────────┘
                         │
                    全文本 git 仓库
```

v2 相比 v1 的改进：玩法程序不需要学习 Lua（一门小众嵌入语言），而是用 JS/TS（Web 最主流的语言）编写玩法 UI 和逻辑。WebView 本身即脚本运行时，无需嵌入额外语言。

---

## 迁移可行性评估

### 可直接复用的模块（~60%）

| 模块 | 改动量 | 说明 |
|------|--------|------|
| vn-runtime（整个 crate） | 零 | 纯逻辑，无 IO 依赖 |
| command_executor | 零 | 纯状态翻译 |
| renderer/effects | 零 | 纯数据映射 |
| renderer/animation | 零 | 纯数学插值 |
| renderer/scene_transition | 极小 | 状态机不变，视觉执行移到前端 |
| save_manager | 零 | 纯文件 IO |
| audio | 极小 | 状态管理不变，Rust 侧 rodio 播放保留 |
| resources（逻辑层） | 小 | LogicalPath、缓存策略不变 |

### 需要适配的模块（~15%）

| 模块 | 改动内容 |
|------|---------|
| renderer (build_draw_commands) | 产出可序列化 JSON 渲染指令（替代 wgpu DrawCommand） |
| resources (TextureContext) | 通过 Tauri `asset:` 协议暴露资源路径（替代 GPU 纹理创建） |
| input | 消费前端 IPC 输入事件（替代 winit 事件） |
| app (AppState) | 去掉 wgpu 依赖，主循环改为 Tauri command 驱动 |

### 完全替换的模块（~25%）

| 模块 | 说明 |
|------|------|
| backend (winit+wgpu+egui) | 替换为 Tauri app 壳 |
| host_app (winit 生命周期) | 替换为 Tauri Builder + setup |
| ui (egui 基础设施) | 替换为 Vue 组件 |
| egui_screens | 所有页面用 Vue 组件重写 |
| game_mode (HTTP Bridge) | **删除**——前端路由直接承载玩法模态 |

### 特效迁移映射

| 当前效果 | wgpu 实现 | Web 方案 | 难度 |
|---------|----------|---------|------|
| Dissolve（交叉淡化） | alpha blending | CSS `opacity` + `transition`，两层 `<img>` 叠加 | 简单 |
| Fade / FadeWhite（遮罩过渡） | 遮罩 alpha 动画 | `<div>` + `opacity` 动画 | 简单 |
| Rule（图片遮罩过渡） | 自定义 dissolve shader | Canvas 2D 或 WebGL fragment shader | 中等 |
| Move（立绘位移） | 动画系统插值 | CSS `transform` + `transition` | 简单 |
| Shake（震屏） | 位移振荡 | CSS `@keyframes` + `transform` | 简单 |
| Blur（模糊） | shader 高斯模糊 | CSS `filter: blur()` + `transition` | 极简单 |
| Dim（变暗） | 半透明遮罩 | CSS `filter: brightness()` | 极简单 |

Rule 过渡是唯一需要 Canvas/WebGL 的效果，可用 PixiJS filter 或裸 WebGL shader 实现。

---

## 分阶段实施路径

### Phase 0：当前（纯 VN 开发 + 迁移准备）

- **状态**：正在进行
- 继续纯 VN 开发和 ref-project 重制（在当前 wgpu host 上）
- RFC-028 Phase A（Snapshot + 回退）已完成，Phase B/C 冻结（待迁移后在新架构上重新设计）

### Phase 1：Tauri 最小 Host + VN 核心渲染

- **触发条件**：ref-project 达到稳定里程碑，有时间窗口做架构迁移
- 搭建 `host-tauri` crate（Cargo workspace member）
- Rust 后端：接入 vn-runtime + CommandExecutor + 共享服务，暴露 Tauri IPC 命令
- Vue 前端：实现 VN 核心渲染（背景 + 立绘 + 对话框 + 选项 + 过渡效果）
- 验收：一个完整 VN 场景可在 Tauri host 中从头到尾跑通
- 当前 wgpu `host` 保留，两个 host 共存

### Phase 2：UI 页面迁移 + game_mode 统一

- **触发条件**：Phase 1 核心渲染稳定
- 迁移所有 UI 页面到 Vue（标题、菜单、存档/读档、设置、历史）
- game_mode（HTTP Bridge + WebView）退役——小游戏直接作为 Vue 组件接入
- 存档/读档/设置等功能走 Tauri IPC
- 验收：Tauri host 功能完全对齐当前 wgpu host

### Phase 3：玩法模态扩展

- **触发条件**：Phase 2 完成 + 第一个非 VN 项目从企划进入预生产
- 玩法 mode 直接作为 Vue 组件开发（JS/TS 编写逻辑 + 渲染）
- 通过 Tauri IPC 访问共享服务（音频、资源、存档、变量读写）
- 用第一个具体玩法（如卡牌）验证开发体验
- 不需要嵌入 Lua——WebView 即脚本运行时

### 独立线：脚本预览编辑器（待重新设计）

- RFC-028 Phase A（Snapshot + Backspace 回退）已在 wgpu host 上实现，迁移到 Tauri host 时可复用逻辑
- Phase B（可点击历史）和 Phase C（F5 热重载）冻结
- Tauri 迁移完成后重新设计：可基于 Monaco/CodeMirror 构建更强的 Web 预览编辑器

---

## 构建与开发体验

### 项目结构

```
Ring-rs/ (Cargo workspace)
├── Cargo.toml                    # workspace root
├── vn-runtime/                   # 不变
├── host/                         # 当前 wgpu host（保留，过渡期共存）
├── host-tauri/
│   ├── src-tauri/
│   │   ├── Cargo.toml            # workspace member，依赖 vn-runtime + tauri
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands.rs       # Tauri IPC 命令
│   │   │   ├── state.rs          # AppState（复用核心逻辑）
│   │   │   └── bridge.rs         # RenderState → JSON 序列化
│   │   └── tauri.conf.json
│   └── frontend/
│       ├── package.json          # pnpm 管理
│       ├── vite.config.ts
│       └── src/
│           ├── App.vue
│           ├── vn/               # VN 渲染组件
│           └── screens/          # UI 页面
├── tools/
└── ...
```

### 构建时间

| 场景 | 当前 host | Tauri host |
|------|----------|-----------|
| 冷编译 | 较快 | 略慢（+Tauri 依赖树，一次性成本） |
| 增量编译（改 Rust） | 快 | 同样快（共享 target/） |
| 增量"编译"（改前端） | N/A（改 egui = Rust 重编译） | **极快**（Vite HMR 毫秒级，不触发 Rust 编译） |

日常开发中大量工作在前端侧（UI 页面、样式调整），**完全不触发 Rust 编译**。对比当前改 egui 代码需要等 Rust 增量编译，前端开发体验显著提升。

### node_modules 缓解

- 使用 **pnpm**（硬链接 + 全局缓存），磁盘占用减少 50-70%
- VN 前端依赖极精简（vue + @tauri-apps/api + vite + typescript），不会膨胀
- 将项目目录加入 Windows Defender 排除列表，避免实时扫描拖慢构建

---

## 子 RFC 影响评估

| RFC | v1 定位 | v2（Tauri）下的变化 |
|-----|---------|-------------------|
| RFC-025（共享服务层提取） | Rust trait 重构 | 方向不变，但服务需通过 Tauri IPC 暴露给前端。已 Accepted 的部分继续有效 |
| RFC-026（统一 Game Mode 框架） | Rust `GameMode` trait + Dispatcher | **根本性简化**：前端路由替代 Rust trait 调度；GameMode trait 不再需要；待重写 |
| RFC-027（玩法脚本层集成） | 嵌入 Lua | **根本性简化**：WebView 即脚本运行时，玩法逻辑用 JS/TS 编写；待重写 |
| RFC-028（脚本预览编辑器） | Phase A/B/C 渐进实现 | Phase A 已完成（Accepted）；Phase B/C 冻结，待 Tauri 迁移后基于 Web 技术重新设计 |

---

## 复杂度控制纪律

1. **新旧 Host 共存**：当前 wgpu host 继续服务 ref-project，不强制切换。Tauri host 达到功能对齐后再考虑废弃旧 host
2. **一次只做一个 mode**：不并行开发多种玩法 mode，每个 mode 自包含
3. **先配置后工具**：每种 mode 首版用文本配置 + 热重载，痛点驱动再做可视化
4. **前端依赖极简主义**：不引入不必要的 npm 包。VN 渲染用 HTML/CSS，仅 Rule 过渡按需引入 Canvas/WebGL
5. **mode 之间禁止耦合**：卡牌 mode 不应调用战棋 mode 的逻辑

---

## 关于方向 B（叙事中间件）的补充

vn-runtime 当前已是纯逻辑、无 IO 依赖。未来如 Godot RPG 项目需要嵌入叙事能力，可通过 GDExtension 写 thin host，工作量有限（vn-runtime API 表面积小：`Parser::parse` + `VNRuntime::tick` + `RuntimeInput`）。

Hub 架构不阻碍此方向。当需求明确时再单独立 RFC。

---

## 风险

| 风险 | 缓解 |
|------|------|
| Tauri 迁移工程量大，影响 ref-project 进度 | 新旧 host 共存，ref-project 不受迁移阻塞 |
| Web 前端 AI 编码虽强但仍有错误 | Vue 3 是训练数据最丰富的框架之一；VN 前端复杂度低 |
| Rule 过渡等 shader 级效果在 WebView 中实现难度不确定 | 可先用 CSS 近似效果上线，再用 Canvas/WebGL 精确实现 |
| IPC 延迟影响交互流畅度 | VN 交互频率低（点击推进），IPC 延迟不可感知；性能敏感的动画在前端 requestAnimationFrame 内完成 |
| 团队成员不熟悉 Web 开发 | vibe coding 覆盖大部分开发；Vue 中文文档完善 |
