# RFC: 玩法脚本层集成

## 元信息

- 编号：RFC-027
- 状态：Proposed
- 作者：claude-4.6-opus
- 日期：2026-03-24
- 影响范围：`host`（新增 scripting 模块）、mode 框架（RFC-026）
- 前置：RFC-026（统一 Game Mode 框架）
- 父 RFC：RFC-024（VN+ Hub 架构愿景）

---

## 背景

Hub 架构（RFC-024）中，原生玩法 mode 的逻辑分为两层：

- **Rust 框架层**：mode 骨架、状态机、渲染、共享服务调用——由引擎维护者编写
- **脚本逻辑层**：具体游戏规则、AI 行为、胜负判定、数值公式——由玩法程序员编写

这种分层的动机是团队分工：

- 引擎维护者熟悉 Rust，负责框架和基础设施
- 玩法程序员来自 Godot 背景，会编码但不会 Rust，是非 VN 内容的**主导者**
- 玩法逻辑需要高频迭代（数值调整、规则变更），热重载是刚需

需要嵌入一种脚本语言，作为 Rust 框架和玩法程序员之间的桥梁。

**实施时机：等待第一个非 VN 项目进入预生产，先做 spike 原型验证，再正式实施。**

---

## 目标与非目标

### 目标

- 选定嵌入脚本语言，定义选型标准和候选评估
- 设计 Rust ↔ 脚本的 API 边界（mode 框架向脚本暴露什么能力）
- 支持热重载：修改脚本文件后无需重新编译即可看到效果
- 玩法程序员不需要 Rust 知识即可编写完整的游戏规则

### 非目标

- 替代 vn-runtime 的叙事脚本——叙事用 `.rks`，玩法用嵌入脚本，两者共存
- 脚本语言的可视化编辑器
- 脚本之间的跨 mode 调用
- 现在就选定语言——本 RFC 提供评估框架，最终选型由 spike 结果决定

---

## 脚本语言选型

### 选型标准

| 标准 | 权重 | 说明 |
|------|------|------|
| GDScript 用户上手成本 | 高 | 玩法程序员的背景是 Godot/GDScript |
| Rust 嵌入成熟度 | 高 | 绑定库质量、文档、社区 |
| 热重载支持 | 高 | 迭代效率的核心需求 |
| 运行时性能 | 中 | VN 品类玩法多为回合制，不需要极致性能 |
| 生态/社区 | 中 | 遇到问题时有参考资料 |
| 二进制体积影响 | 低 | 不是关键约束 |

### 候选评估

#### Lua（初步倾向）

| 维度 | 评估 |
|------|------|
| 上手成本 | **低**。GDScript 和 Lua 同为动态类型命令式脚本，设计哲学接近 |
| Rust 绑定 | **mlua**：成熟、活跃维护、支持 Lua 5.4/LuaJIT、async 友好 |
| 热重载 | **原生支持**。重新 `dofile()` 即可，无需特殊机制 |
| 性能 | 对回合制玩法绰绰有余；如需更高性能可切换 LuaJIT |
| 生态 | **极成熟**。游戏行业标配（Love2D、Defold、WoW、Factorio 等） |
| 风险 | 1-indexed 数组（GDScript 是 0-indexed）；标准库极简需自行补充工具函数 |

#### Rhai

| 维度 | 评估 |
|------|------|
| 上手成本 | **中**。语法偏 Rust，GDScript 用户需适应 |
| Rust 绑定 | **原生 Rust crate**，无 FFI，类型安全边界最好 |
| 热重载 | 支持（重新编译脚本） |
| 性能 | 解释执行，比 Lua 慢，但对回合制够用 |
| 生态 | 小众，参考资料少 |
| 风险 | 社区较小，遇到问题可能无参考 |

#### JavaScript（QuickJS 嵌入）

| 维度 | 评估 |
|------|------|
| 上手成本 | **中高**。GDScript 用户需学 JS 范式 |
| Rust 绑定 | rquickjs：可用但不如 mlua 成熟 |
| 热重载 | 支持 |
| 性能 | 够用 |
| 生态 | 最大，但游戏嵌入场景的参考较少 |
| 风险 | 如果团队未来也走 WebView 路线，可统一语言；否则引入 JS 生态增加复杂度 |

### 初步倾向：Lua

Lua 在"GDScript 用户友好度"和"游戏行业验证"两个高权重维度上领先。最终选型需通过 spike 验证。

---

## API 边界设计（草案）

### Rust 侧暴露给脚本的能力

```lua
-- 共享服务
audio.play_bgm("battle_theme.ogg", { fade_in = 1.0 })
audio.play_sfx("card_play.ogg")
resources.load_image("cards/fireball.png")

-- 游戏状态（由 Rust 框架管理，脚本读写）
state.get("player_hp")
state.set("player_hp", 42)

-- 叙事变量（只读，来自 RuntimeState.variables）
story.get("player_deck")
story.get("difficulty_level")

-- UI 构建（通过绑定 egui 或自定义 UI DSL）
ui.button("出牌", { x = 100, y = 200 })
ui.label("HP: " .. state.get("player_hp"))

-- 完成模态，回传结果
mode.complete({ winner = "player", turns = 5 })
```

### 脚本侧结构

```lua
-- modes/card_battle/main.lua

function on_activate(params)
    -- 初始化游戏状态
    local deck = params.deck
    local enemy = params.enemy
    state.set("player_hp", 100)
    state.set("enemy_hp", enemy.hp)
    -- 洗牌、发牌等
end

function on_update(dt)
    -- 每帧逻辑（回合制可能大部分帧无操作）
    if state.get("enemy_hp") <= 0 then
        mode.complete({ winner = "player" })
    end
end

function on_render()
    -- UI 渲染（或由 Rust 框架根据状态自动渲染）
end

function on_card_played(card_id)
    -- 具体出牌逻辑
end
```

### 热重载流程

1. 文件监听器检测到 `.lua` 文件变更
2. 重新执行 `dofile("main.lua")`
3. 保留 `state` 数据，重新绑定函数
4. 下一帧自动使用新逻辑

---

## 实施路径

### Spike 阶段（1 周，第一个非 VN 项目预生产时）

1. 集成 `mlua`（或 Rhai）到 host
2. 实现最小 API 绑定（state get/set + mode.complete）
3. 让玩法程序员用脚本写一个最简化的玩法原型（如简化版卡牌出牌）
4. 评估开发体验：上手难度、调试能力、热重载流畅度
5. 根据 spike 结果确认选型或调整方向

### 正式实施（spike 验证通过后）

1. 完善 API 绑定（音频、资源、UI）
2. 建立脚本项目结构规范（目录布局、入口文件、配置）
3. 错误处理与诊断（脚本运行时错误 → 引擎日志/toast）
4. 编写玩法程序员文档（API 参考、示例、从 GDScript 迁移指南）

---

## 风险

| 风险 | 缓解 |
|------|------|
| 脚本语言选错 | spike 阶段验证，选型不锁死 |
| API 边界设计不合理 | 从最小 API 开始，根据实际需求增长 |
| Rust ↔ 脚本的数据传递性能问题 | 回合制玩法调用频率低，不太可能成为瓶颈 |
| 玩法程序员不适应 Lua | GDScript → Lua 的迁移成本低；如实在不适应可考虑 Rhai 或 JS |
| 调试体验不如 Godot 编辑器 | 提供 print 调试 + 热重载快速迭代；后续可加 REPL |

---

## 验收标准

- [ ] 脚本语言选型确定（spike 验证通过）
- [ ] mlua（或选定方案）集成到 host，编译通过
- [ ] 最小 API 绑定：state get/set、audio play、mode.complete
- [ ] 热重载：修改脚本文件后无需重启即可生效
- [ ] 至少一个完整玩法原型通过脚本实现
- [ ] 玩法程序员反馈开发体验可接受
- [ ] 错误处理：脚本运行时错误不导致引擎崩溃
- [ ] API 参考文档
- [ ] `cargo check-all` 通过
