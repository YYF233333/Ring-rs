# RFC: 玩法脚本层集成

## 元信息

- 编号：RFC-027
- 状态：Proposed（需 spike 验证 Dioxus JS interop）
- 作者：claude-4.6-opus
- 日期：2026-03-24（v1 Lua）；2026-04-14（v2 WebView JS 注入）
- 影响范围：`host-dioxus/`（新增 GameModeHost 组件 + JS bridge）
- 前置：无（RFC-026 已 Superseded，GameModeHost 直接作为 Dioxus 组件实现）
- 父 RFC：RFC-024（VN+ Hub 架构愿景 v3）

---

## 背景

Hub 架构（RFC-024 v3）中，Ring Engine 分为两层运行时：

- **VN 核心**：Dioxus RSX 组件，与 vn-runtime 同进程，类型安全，零 IPC
- **玩法模态**：非 VN 的游戏玩法段落（卡牌、战棋、资源经营等），由脚本驱动叙事中断

玩法模态的设计约束：

1. **玩法程序员熟悉 GDScript/JS，不熟悉 Rust**——入口语言不能是 Rust
2. **高频迭代**——数值调整、规则变更需要热重载，不能每次改动等 Rust 编译
3. **UI 多样性高**——卡牌界面、战棋地图、模拟面板各不相同，需要强表现力
4. **AI 辅助是主力**——vibe coding 场景下，语言的 AI 编码能力是核心约束

### v1 → v2 的演进

v1 选择嵌入 Lua（mlua），理由是 GDScript 用户友好度和游戏行业验证。

v2 改为 WebView JS 注入。原因：

- Dioxus Desktop 底层即 WebView（wry/tao），**JS 运行时已经存在**，无需引入额外依赖
- 玩法 mode 的 UI 可以**直接用 HTML/CSS 渲染**，表现力远超任何嵌入式脚本的 UI 绑定层
- **Web 是 AI 编码能力最强的领域**（训练数据最丰富、生态最成熟）
- Lua 的 UI 渲染需要额外绑定层（绑到 Dioxus 或 egui），成本高且表现力受限

---

## 目标与非目标

### 目标

- 玩法 mode 用 JS/TS + HTML/CSS 编写，通过 Dioxus WebView 注入运行
- 定义 Rust ↔ JS bridge API（变量读写、音频、资源、mode 生命周期）
- 支持热重载：修改 JS/HTML/CSS 后无需重编译 Rust 即可看到效果
- 玩法程序员不需要 Rust 知识即可编写完整的游戏玩法

### 非目标

- 替代 vn-runtime 的叙事脚本——叙事用 `.rks`，玩法用 JS，两者共存
- 在 VN 核心组件中使用 JS——JS 仅限 GameModeHost 容器内
- 引入 npm/node_modules 到主项目——玩法 mode 的 JS 自包含，不依赖构建工具
- 现在就实施——本 RFC 在 spike 验证 Dioxus JS interop 后正式启动

---

## 技术方案：WebView JS 注入

### 为什么是 JS 而非 Lua

| 维度 | Lua 嵌入（v1） | WebView JS 注入（v2） |
|------|---------------|---------------------|
| 运行时 | mlua（额外依赖） | WebView 内置（零额外依赖） |
| UI 渲染 | 需绑定到 Dioxus/egui | **原生 HTML/CSS/Canvas** |
| 表现力 | 受限于绑定层 | **完整 Web 能力** |
| AI 编码 | 中等 | **最强** |
| 热重载 | `dofile()` 重载 | 重新注入 JS/HTML |
| 上手成本 | GDScript→Lua 低 | GDScript→JS 低 |
| 调试工具 | print + 自建 REPL | **浏览器 DevTools**（已有） |

关键优势：玩法 mode 的 UI **直接就是 Web 页面**。卡牌界面用 CSS Grid，战棋地图用 Canvas，经营面板用 Flexbox——这些都是 Web 前端的绝对强项，不需要任何额外绑定层。

### Dioxus JS Interop 机制

Dioxus 0.7 Desktop 提供的 WebView 交互能力：

| 方向 | 机制 | 用途 |
|------|------|------|
| Rust → JS | `eval()` / `document.eval()` | 注入 mode HTML/JS/CSS、调用 JS 函数 |
| JS → Rust | Custom event + `use_eval` 回调 | mode 完成通知、状态变更请求 |
| 资源访问 | 资源 URL（ResourceManager 提供路径） | 图片/音频/配置文件 |

> **待 spike 验证**：`eval()` 的双向通信稳定性、HTML 子树注入可行性、与 RSX 渲染的隔离性。

### 玩法 Mode 文件结构

```
assets/modes/card-battle/
├── index.html       # Mode 入口页面
├── main.js          # 玩法逻辑
├── style.css        # 玩法 UI 样式
└── manifest.json    # 元信息（mode_id, version, entry）
```

Mode 自包含，不依赖 npm、不依赖构建工具。可以用 vanilla JS，也可以用编辑器内置的 TS 类型检查（通过 JSDoc 注释）。

### Rust ↔ JS Bridge API

GameModeHost 在注入 mode 时，向 WebView 全局注入 `ring` bridge 对象：

```javascript
// ── 游戏状态（mode 自己的状态，Rust 持久化） ──
ring.state.get("player_hp")           // → number | string | null
ring.state.set("player_hp", 42)

// ── 叙事变量（来自 RuntimeState.variables，只读） ──
ring.story.get("player_deck")         // → value
ring.story.get("difficulty_level")

// ── 音频 ──
ring.audio.playBgm("battle_theme.ogg", { fadeIn: 1.0 })
ring.audio.playSfx("card_play.ogg")
ring.audio.stopBgm({ fadeOut: 0.5 })

// ── 资源 ──
ring.assets.url("cards/fireball.png") // → 可用于 <img src="..."> 的 URL

// ── Mode 生命周期 ──
ring.mode.complete({ winner: "player", turns: 5 })  // 结束 mode，回传结果
ring.mode.cancel()                                    // 取消 mode（异常退出）
```

Rust 侧 `GameModeHost` 拦截 bridge 调用，转发到对应的共享服务（AudioManager、ResourceManager、RuntimeState.variables）。

### 热重载流程

1. 文件监听器（仅 debug build）检测到 `assets/modes/` 下文件变更
2. GameModeHost 重新注入 mode 的 HTML/JS/CSS
3. 保留 `ring.state` 数据（Rust 侧持有），JS 侧重新加载逻辑
4. 下一帧自动使用新代码

---

## GameModeHost 组件设计

```rust
/// 玩法模态容器
///
/// 进入 mode 时占据全屏（隐藏 VN 渲染层），
/// 注入 mode 的 JS/HTML/CSS 到 WebView 容器中。
#[component]
fn GameModeHost(
    mode_id: String,
    params: HashMap<String, VarValue>,
    on_complete: EventHandler<GameModeResult>,
) -> Element {
    // 1. 从 ResourceManager 加载 mode 的 manifest + 入口文件
    // 2. 注入 ring bridge 对象到 WebView 全局
    // 3. 注入 mode 的 HTML/JS/CSS
    // 4. 监听 ring.mode.complete/cancel 事件
    // 5. 收到完成事件后调用 on_complete
    todo!()
}
```

VN 叙事与 GameModeHost 的关系：**VN 是默认态，GameModeHost 是中断态**。`Command::RequestUI` 触发切换，mode 完成后 `RuntimeInput::UIResult` 回传结果。

---

## 实施路径

### Spike 阶段（第一个非 VN 项目预生产时触发）

1. **Dioxus JS interop PoC**：
   - 测试 `eval()` 双向通信（Rust→JS 调用、JS→Rust 事件）
   - 测试在 WebView 中注入完整 HTML 子树
   - 测试 JS 注入与 Dioxus RSX 渲染的隔离性（互不干扰）
   - 测试 mode 退出后 JS 上下文清理
2. **最小玩法原型**：
   - 用 vanilla JS + HTML/CSS 实现一个最简化的卡牌出牌界面
   - 验证 `ring.state` 读写、`ring.mode.complete()` 回调
   - 玩法程序员试用，评估开发体验

### Phase 1：GameModeHost 框架（spike 通过后）

1. 实现 `GameModeHost` Dioxus 组件
2. 实现 `ring` bridge 对象（state/story/audio/assets/mode）
3. 实现 mode manifest 加载与入口注入
4. 热重载支持（debug build）
5. 错误处理：JS 运行时错误 → Rust 日志/toast，不崩溃引擎

### Phase 2：首个生产 mode

1. 用第一个具体玩法项目验证完整开发流程
2. 根据实际需求迭代 bridge API
3. 编写玩法程序员文档（API 参考、示例、项目结构规范）

---

## 风险

| 风险 | 缓解 |
|------|------|
| Dioxus JS interop 不够稳定 | spike 阶段验证；最坏情况回退到 Lua 嵌入（v1 方案仍可用） |
| JS 注入干扰 Dioxus RSX 渲染 | GameModeHost 激活时隐藏 VN 层；容器隔离 |
| mode 退出后 JS 上下文泄漏 | 销毁+重建容器策略；泄漏检测 |
| bridge API 设计不合理 | 从最小 API 开始，根据首个 mode 实际需求增长 |
| 玩法程序员不适应 JS | JS 是 Web 最主流语言；AI 辅助最强；GDScript→JS 门槛低 |
| 无具体玩法项目需求 | 明确触发条件：第一个非 VN 项目进入预生产前仅做 spike |

---

## 验收标准

- [ ] Dioxus JS interop spike 通过（eval 双向通信、HTML 注入、隔离性验证）
- [ ] `GameModeHost` Dioxus 组件实现，支持 mode 注入与生命周期管理
- [ ] `ring` bridge API：state get/set、story get、audio play、assets url、mode complete/cancel
- [ ] 热重载：修改 mode 的 JS/HTML/CSS 后无需重启即可生效
- [ ] 至少一个完整玩法原型通过 JS 实现
- [ ] 玩法程序员反馈开发体验可接受
- [ ] JS 运行时错误不导致引擎崩溃
- [ ] mode 退出后资源正确清理
- [ ] `cargo check-all` 通过
