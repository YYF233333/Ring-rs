# RFC: VN+ Hub 架构愿景

## 元信息

- 编号：RFC-024
- 状态：Proposed
- 作者：claude-4.6-opus
- 日期：2026-03-24
- 影响范围：`host`（整体架构）、`vn-runtime`（接口层面）
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

---

## 目标与非目标

### 目标

- 明确 Ring Engine 的定位演化方向：**VN+ 引擎**（面向叙事核心、玩法嵌入的 VN 品类作品）
- 提出 Hub 架构愿景：Host 从"VN 宿主"演化为"多模态容器"，VN 叙事是主模态之一
- 定义架构演化的分阶段实施路径，确保每阶段独立可交付
- 为子 RFC（RFC-025 ~ RFC-028）提供统一上下文

### 非目标

- **不做通用游戏引擎**——不引入物理、碰撞、3D、骨骼动画等通用引擎能力
- **不立即实施**——本 RFC 是愿景与路线图，具体实施由子 RFC 各自推进
- **不预设玩法类型**——不提前为卡牌/战棋/模拟等特定品类建模，等具体项目约束
- **不替代 WebView 通道**——WebView 作为 fallback 模态持续可用

---

## 核心判断

### 为什么不做通用游戏引擎

1. **竞品差距不可追**：单人维护的 Rust 引擎在通用能力上无法追上 Godot/Bevy/Unity
2. **心智复杂度乘法增长**：通用引擎的每个系统（ECS/物理/碰撞/寻路/粒子）互相耦合
3. **比较优势在叙事**：vn-runtime 的显式状态、确定性执行、Command 驱动是差异化价值
4. **VN 品类的玩法复杂度有结构性上限**：2D、回合制/半回合制、状态机驱动，不需要通用引擎能力

### 为什么选原生 Rust 而非外部框架

| 方案 | 致命问题 |
|------|----------|
| WebView | 团队无 web 开发经验，vibe coding 缓解但不解决 |
| Godot 嵌入 | bug 多，与 vibe coding 适配差 |
| Unity | 版权/政策风险 |
| Bevy | 0.x API 频繁 breaking |
| **原生 Rust** | 已验证的 vibe coding 流程，单一技术栈，完美集成 vn-runtime |

关键前提：VN 品类的玩法需求（卡牌/战棋/模拟）本质是**纯数据 + 状态机 + 2D 渲染 + UI**，现有技术栈（wgpu + egui + Rust）已具备全部基础能力。

### 工具链策略："先做游戏，再做工具"

VN+ 品类的工具需求集中在 L0-L2 层级，不需要通用场景编辑器（L3）：

| 层级 | 形态 | 应用 |
|------|------|------|
| L0 | 文本配置 + 热重载 | 卡牌定义、角色数值、技能树、经济参数 |
| L1 | 文本 DSL + 实时预览 | 叙事脚本（已有），脚本预览编辑器（RFC-028） |
| L2 | 领域特化简易编辑器 | 战棋地图编辑器（egui 工具窗口，按需开发） |
| L3 | 通用场景编辑器 | **不需要** |

每种 mode 的第一版用文本配置 + 热重载。只在明确痛点出现时才做可视化工具。

---

## 方案设计

### 目标架构

```
Ring Engine (VN+)
│
├── vn-runtime（不变）
│   └── 叙事核心：脚本 → AST → Executor → Command
│
├── 共享服务层（RFC-025）
│   ├── AudioManager
│   ├── ResourceManager
│   ├── SaveManager
│   ├── InputManager
│   ├── Config
│   └── SpriteRenderer（2D 渲染能力）
│
├── host/hub（变薄，模态调度 + 窗口/GPU）
│   ├── winit + wgpu + egui 后端
│   ├── GameMode trait 调度器（RFC-026）
│   └── 模态间切换协议
│
└── modes/（可插拔模态）
    ├── vn_mode/（当前主体渲染管线）
    ├── webview_mode/（当前 game_mode，保留）
    └── [future] native_game_mode/（Rust 框架 + 脚本逻辑，RFC-026/027）
```

### 叙事 ↔ 玩法交互协议

模态切换通过 vn-runtime 的 `Command::RequestUI` 触发，结果通过 `RuntimeInput::UIResult` 回传：

```
叙事流 ──► call_mode "card_battle" with { deck, enemy, ... }
              │
              ├─ Hub 暂停 vn_mode，激活 card_battle_mode
              │   ├─ 读取 RuntimeState.variables（角色状态/道具）
              │   ├─ 运行玩法交互循环（自己的 update/render）
              │   └─ 完成后产出结果 { winner, turns, hp_remaining }
              │
              ├─ Hub 关闭 card_battle_mode，恢复 vn_mode
              │   └─ 结果写回 RuntimeState.variables
              │
              └─ 叙事流根据变量分支继续
```

### 团队协作模型（模式 γ）

```
你（引擎）           玩法程序（Godot 背景）       编剧
    │                    │                       │
  Rust: mode 框架       Lua: 玩法规则脚本          .rks: 叙事脚本
  定义 API 边界         出牌逻辑/AI/胜负判定        对话/演出/mode 调用
  共享服务              数据配置（TOML）             
    │                    │                       │
    └────────────────────┴───────────────────────┘
                         │
                    全文本 git 仓库
```

玩法程序通过嵌入脚本语言（RFC-027）编写游戏逻辑，不需要 Rust 知识。

---

## 分阶段实施路径

### Phase 0：当前（纯 VN 开发，积累体感）

- **状态**：正在进行
- 继续纯 VN 开发和 ref-project 重制
- 在日常开发中注意共享服务的解耦意识
- 观察 `UiModeHandler` 和 `game_mode` 的使用模式

### Phase 1：共享服务层提取（RFC-025）

- **触发条件**：可立即开始，作为日常重构的一部分
- 将 AudioManager、ResourceManager、SaveManager 的接口从 VN 假设中解耦
- 存档格式预留扩展点

### Phase 2：统一 Game Mode 框架（RFC-026）

- **触发条件**：第一个非 VN 项目从企划进入预生产
- 统一 `UiModeHandler` 和 `game_mode` 为 `GameMode` trait
- VN 渲染逻辑封装为 vn_mode

### Phase 3：玩法脚本层集成（RFC-027）

- **触发条件**：Phase 2 完成 + 第一个具体玩法需求确定
- 嵌入脚本语言（Lua 为候选），定义 mode API
- 用具体玩法（如卡牌）验证 Rust 框架 + 脚本逻辑的开发体验

### 独立线：脚本预览编辑器（RFC-028）

- **触发条件**：可立即开始，与上述 Phase 正交
- Artemis 式脚本预览，服务纯 VN 叙事开发

---

## 复杂度控制纪律

1. **一次只做一个 mode**：不并行开发多种玩法 mode，每个 mode 自包含
2. **先配置后工具**：每种 mode 首版用文本配置 + 热重载，痛点驱动再做可视化
3. **共享服务抽象不过度**：只抽取已证实被多个 mode 需要的能力，不预设
4. **mode 之间禁止耦合**：卡牌 mode 不应调用战棋 mode 的逻辑

---

## 关于方向 B（叙事中间件）的补充

vn-runtime 当前已是纯逻辑、无 IO 依赖。未来如 Godot RPG 项目需要嵌入叙事能力，可通过 GDExtension 写 thin host，工作量有限（vn-runtime API 表面积小：`Parser::parse` + `VNRuntime::tick` + `RuntimeInput`）。

Hub 架构不阻碍此方向。当需求明确时再单独立 RFC。


