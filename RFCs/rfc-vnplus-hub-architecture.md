# RFC: VN+ Hub 架构愿景

## 元信息

- 编号：RFC-024
- 状态：Active（v3 Dioxus + JS 注入路线）
- 作者：claude-4.6-opus
- 日期：2026-03-24（v1）；2026-03-25（v2 Tauri）；2026-04-14（v3 Dioxus）
- 影响范围：`host-dioxus/`（整体架构）、`vn-runtime/`（接口层面不变）
- 相关 RFC：RFC-027（玩法脚本层）、RFC-033（Dioxus 迁移，Accepted）、RFC-034（旧宿主退役，Accepted）

---

## 背景

Ring Engine 立项时定位为纯视觉小说引擎，核心架构假设为"世界是叙事驱动的"：脚本线性推进，Runtime 产出 Command，Host 执行 Command 渲染画面。这个假设对纯 VN 作品是精确的。

团队技术目标已演化：

- 已有纯 VN 重制计划（ref-project），需要当前引擎叙事能力
- 新立项作品包含 RPG-VN、卡牌 VN、战棋 VN、历史模拟 VN 等品类
- 共同特征：**仍在 VN 品类内部**，叙事是不可或缺的核心，玩法段落与对话交替推进
- 团队用 Godot 开发了一款 VS-like RPG（独立项目），积累了 Godot 经验

### v1 → v2 → v3 的演进

**v1（纯原生 Rust）**：选择 winit+wgpu+egui 一条路走到底。问题：egui 在 VN 核心体验（富文本、精致 UI）上表现力受限；Hub 多模态调度需要额外抽象。

**v2（Tauri）**：将技术路线调整为 Tauri（Rust 后端 + Vue WebView 前端）。动机：Web 技术天然强于文本排版和 UI；WebView 即玩法脚本运行时。

**v3（Dioxus + JS 注入，当前）**：Tauri 开发半个月后（34 个 IPC 命令已适配），暴露三个结构性问题（详见 RFC-033）：

1. **双语言工具链摩擦**：cargo + pnpm + vite + biome + vue-tsc 五套独立工具
2. **IPC 边界成本**：所有前后端交互经 JSON 序列化；Rust struct ↔ TS type 手动同步；34 个薄代理函数纯粹是胶水
3. **调试架构矛盾**：debug server 与 Tauri WebView 形成双客户端竞争

根本原因：**Tauri 的 IPC 隔离对"前端是纯渲染层"的 VN 引擎只有成本没有收益**。但 Tauri 的 WebView 运行时对"玩法模态"仍有价值。

v3 的判断：**用 Dioxus Desktop 消除 VN 核心的 IPC 边界，同时保留底层 WebView 能力用于玩法模态的 JS 注入**。

---

## 目标与非目标

### 目标

- 明确 Ring Engine 的定位演化方向：**VN+ 引擎**（面向叙事核心、玩法嵌入的 VN 品类作品）
- Hub 架构：Host 从"VN 宿主"演化为"多模态容器"，VN 叙事是主模态之一
- **VN 核心在 Dioxus RSX 中渲染（同进程、类型安全、零序列化）**
- **玩法模态通过 WebView JS 注入实现（HTML/CSS/JS，非 Rust 程序员可参与）**
- 为 RFC-027（玩法脚本层）提供架构上下文

### 非目标

- **不做通用游戏引擎**——不引入物理、碰撞、3D、骨骼动画等通用引擎能力
- **不预设玩法类型**——不提前为卡牌/战棋/模拟等特定品类建模
- **不将 VN 核心渲染迁移到 JS**——Dioxus RSX 是 VN 核心的正确选择

---

## 核心判断

### 为什么不做通用游戏引擎

（与 v1/v2 相同，不变）

1. **竞品差距不可追**：单人维护的 Rust 引擎在通用能力上无法追上 Godot/Bevy/Unity
2. **心智复杂度乘法增长**：通用引擎的每个系统互相耦合
3. **比较优势在叙事**：vn-runtime 的显式状态、确定性执行、Command 驱动是差异化价值
4. **VN 品类的玩法复杂度有结构性上限**：2D、回合制/半回合制、状态机驱动

### 为什么是 Dioxus + JS 注入（v3）

v2 排除 Dioxus 的隐含前提是"需要 Web 前端做 VN 渲染"。但 RFC-033 证明 **Dioxus RSX 完全能胜任 VN 渲染**（背景/立绘/对话/过渡/音频全链路已跑通），且消除了 IPC 边界。

同时，玩法模态的需求与 VN 核心不同：

| 维度 | VN 核心 | 玩法模态 |
|------|---------|---------|
| 状态管理 | 引擎控制（RuntimeState） | mode 自管理 |
| 渲染复杂度 | 固定模式（层叠图片+文本） | 多样化（卡牌/战棋/模拟） |
| 迭代频率 | 低（引擎稳定后少动） | 高（数值调整、规则变更） |
| 编写者 | 引擎维护者（Rust） | 玩法程序员（非 Rust） |
| AI 辅助效率 | Rust/Dioxus（中等） | Web（最强） |

**结论：VN 核心用 Dioxus RSX（同进程、类型安全），玩法模态用 WebView JS（表现力强、非 Rust 程序员可参与、AI 编码最优）。**

Dioxus Desktop 底层即 WebView（wry/tao），无需引入额外运行时。JS 注入的 IPC 边界**仅存在于玩法模态**，不影响 VN 核心性能。

### 技术栈选定（v3）

| 层 | 技术 | 理由 |
|----|------|------|
| 核心逻辑 | Rust（vn-runtime） | 不变，已验证 |
| VN 渲染 | Dioxus 0.7 Desktop（RSX） | 同进程、类型安全、无 IPC |
| 玩法模态 | JS/TS + HTML/CSS（WebView 注入） | Web 表现力、AI 编码最优 |
| 构建工具 | cargo + dx | 单工具链（VN 侧）；JS 侧按需 |
| 调试 | debug server HTTP API + MCP | 已验证，无客户端竞争问题 |

### 工具链策略："先做游戏，再做工具"

| 层级 | 形态 | 应用 |
|------|------|------|
| L0 | 文本配置 + 热重载 | 卡牌定义、角色数值、技能树、经济参数 |
| L1 | 文本 DSL + 实时预览 | 叙事脚本（已有） |
| L2 | 领域工具（按需） | 脚本预览、数值编辑 |
| L3 | 通用场景编辑器 | **不需要** |

---

## 方案设计

### 目标架构（v3）

```
Ring Engine (VN+) — Dioxus 版
│
├── vn-runtime/（不变）
│   └── 叙事核心：脚本 → AST → Executor → Command
│
├── host-dioxus/（Dioxus 0.7 Desktop）
│   ├── VN 核心渲染（RSX 组件，同进程，零 IPC）
│   │   ├── BackgroundLayer / CharacterLayer / DialogueBox
│   │   ├── ChoicePanel / TransitionOverlay / SceneEffect
│   │   └── 直接读写 Signal/Store，类型安全
│   │
│   ├── UI 页面（RSX 组件）
│   │   ├── TitleScreen / SaveLoadScreen / SettingsScreen
│   │   ├── HistoryScreen / InGameMenu
│   │   └── 同进程，与 VN 核心共享状态
│   │
│   ├── 玩法模态容器（GameModeHost）
│   │   ├── 进入 mode → 注入 JS/HTML/CSS 到 WebView 容器
│   │   ├── Rust ↔ JS 通过 Dioxus eval/event bridge 通信
│   │   ├── JS 侧通过 bridge 访问：变量、音频、资源 URL
│   │   └── 完成 → JS 回传结果 → Rust 恢复 vn-runtime tick
│   │
│   ├── 共享服务
│   │   ├── AudioManager（同进程）
│   │   ├── ResourceManager（同进程）
│   │   ├── SaveManager
│   │   └── Config
│   │
│   └── Debug Server（HTTP REST API + MCP）
│
└── tools/（xtask / asset-packer / debug-mcp）
```

### VN 核心：同进程渲染

VN 核心渲染由 Dioxus RSX 组件实现，与引擎状态在同一进程内直接交互：

```
vn-runtime                    host-dioxus
────────                      ──────────
tick(input)                   
→ Vec<Command>                CommandExecutor
→ WaitingReason               → 更新 Signal<RenderState>
                              → RSX 组件自动响应变更
                              → 渲染（HTML/CSS via WebView）
```

无序列化、无 IPC、无类型镜像。与 Tauri 方案的根本区别。

### 玩法模态：WebView JS 注入

玩法模态利用 Dioxus Desktop 底层的 WebView 能力：

```
叙事流 ──► call_mode "card_battle" with { deck, enemy, ... }
              │
              ├─ Rust 暂停 vn-runtime tick
              │   └─ GameModeHost 组件激活
              │
              ├─ 注入 mode 的 JS/HTML/CSS 到 WebView 容器
              │   ├─ JS 通过 bridge 对象访问引擎服务
              │   ├─ HTML/CSS 渲染玩法 UI（完整 Web 表现力）
              │   └─ 玩法逻辑在 JS 中运行
              │
              ├─ JS 调用 ring.mode.complete({ winner, turns, ... })
              │   └─ Dioxus event 将结果传回 Rust
              │
              └─ Rust 恢复 vn-runtime tick
                  └─ 结果写回 RuntimeState.variables
```

详细 API 设计见 RFC-027。

### 团队协作模型（v3）

```
引擎维护者              玩法程序员              编剧
    │                    │                    │
  Rust: 核心逻辑         JS/TS: 玩法 UI+规则   .rks: 叙事脚本
  Dioxus RSX: VN 渲染    HTML/CSS: 玩法界面     对话/演出/mode 调用
  共享服务 + bridge       资源配置（JSON）
    │                    │                    │
    └────────────────────┴────────────────────┘
                         │
                    全文本 git 仓库
```

对比 v2（Tauri）：VN 核心不再需要前端开发者。JS 仅用于玩法模态，工具链负担大幅降低。

---

## 当前状态与实施路径

### 已完成

| 阶段 | 内容 | 状态 |
|------|------|------|
| VN 核心迁移 | Dioxus RSX 全渲染链路（RFC-033） | ✅ Accepted |
| 旧宿主退役 | winit/wgpu/egui host 删除（RFC-034） | ✅ Accepted |
| UI 页面 | 标题/存读档/设置/历史/游内菜单 | ✅ 完成 |
| 调试基础设施 | debug server HTTP API + MCP | ✅ 完成 |

### 下一步

| 阶段 | 内容 | 触发条件 |
|------|------|---------|
| GameModeHost PoC | 验证 Dioxus JS interop：eval 双向通信、HTML 注入、隔离性 | 第一个非 VN 项目进入预生产 |
| JS Bridge API | 定义 ring.* 对象、变量读写、音频控制、mode 生命周期 | PoC 通过 |
| 首个玩法 mode | 用具体项目（如卡牌）验证开发体验 | Bridge API 稳定 |

### 独立线：ref-project 重制

ref-project 在 host-dioxus 上继续推进（RFC-002），不受 Hub 玩法扩展影响。

---

## 子 RFC 影响评估

| RFC | 状态 | v3 下的变化 |
|-----|------|-----------|
| RFC-025（共享服务层提取） | Accepted | 方向不变；服务对 VN 核心直接调用，对玩法 mode 通过 JS bridge 暴露 |
| RFC-026（统一 Game Mode 框架） | Superseded | Dioxus 组件模型替代 Rust trait 调度；GameModeHost 组件承担容器职责 |
| RFC-027（玩法脚本层集成） | Proposed | **重写**：从 Lua 嵌入改为 WebView JS 注入（见 RFC-027 v2） |
| RFC-028（脚本预览编辑器） | Accepted | Phase A 已完成；后续可利用 Dioxus 热重载能力 |

---

## 复杂度控制纪律

1. **VN 核心不跨语言**：核心渲染和状态管理全在 Rust/Dioxus 内，不引入 JS 依赖
2. **一次只做一个 mode**：不并行开发多种玩法 mode，每个 mode 自包含
3. **先配置后工具**：每种 mode 首版用文本配置 + 热重载，痛点驱动再做可视化
4. **JS 依赖极简主义**：玩法 mode 的 JS 代码自包含，不引入 npm 包管理
5. **mode 之间禁止耦合**：卡牌 mode 不应调用战棋 mode 的逻辑
6. **GameModeHost 是唯一 JS 入口**：不在 VN 核心组件中使用 eval/JS interop

---

## 方向 B：叙事中间件

vn-runtime 当前已是纯逻辑、无 IO 依赖。未来如 Godot RPG 项目需要嵌入叙事能力，可通过 GDExtension 写 thin host，工作量有限（vn-runtime API 表面积小：`Parser::parse` + `VNRuntime::tick` + `RuntimeInput`）。

Hub 架构不阻碍此方向。当需求明确时再单独立 RFC。

---

## 风险

| 风险 | 缓解 |
|------|------|
| Dioxus JS interop 能力不足（eval 不稳定、无法注入完整 HTML 子树） | spike 阶段验证；最坏情况可回退到嵌入 Lua |
| 玩法 mode 的 JS 干扰 Dioxus RSX 渲染 | GameModeHost 组件隔离容器；mode 激活时隐藏 VN 渲染层 |
| mode 退出后 JS 上下文未完全清理 | 生命周期管理 + 容器销毁重建策略 |
| 玩法程序员不适应 JS | JS 是 Web 最主流语言，AI 辅助生态最好；GDScript→JS 门槛低于 GDScript→Rust |
| 无具体玩法项目时架构空转 | 明确"触发条件"：第一个非 VN 项目进入预生产前，仅做方向预案 |
