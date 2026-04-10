# RFC: 双向 UI-Script 通信协议

## 元信息

- 编号：RFC-020
- 状态：Accepted
- 作者：claude-4.6-opus
- 日期：2026-03-21
- 相关范围：`vn-runtime`（input / state / command / script / runtime）、`host`（input / command_executor）
- 前置：无

---

## 背景

当前 Runtime/Host 通信遵循严格的单向模式：

- **Runtime → Host**：通过 `Command` 枚举（显示文本、切换背景、播放音频等）
- **Host → Runtime**：通过 `RuntimeInput` 枚举，仅三种变体：
  - `Click`：解除 `WaitForClick`
  - `ChoiceSelected { index }`：解除 `WaitForChoice`，触发标签跳转
  - `Signal { id }`：解除 `WaitForSignal`，无数据负载

`choice` 机制已演示了完整的双向通信闭环：

```
脚本 choice → Command::PresentChoices → WaitForChoice
            → 用户选择 → RuntimeInput::ChoiceSelected → 跳转到目标标签
```

但此闭环有两个局限：

1. **结果类型固定**：只能传递选项索引（`usize`），不能传递结构化数据
2. **流程控制固定**：结果只能用于标签跳转，不能用于变量赋值或条件分支

后续多个功能需要 UI 交互结果回流到脚本：

| 功能 | UI 交互 | 预期结果 |
|------|---------|---------|
| 地图选择 | 用户点选地图位置 | 位置名称（String） |
| NVL 模式切换 | 模式切换指令 | 无需结果（单向命令） |
| 小游戏 (Phase 3) | 游戏结束 | 分数/状态数据 |

需要将 choice 的双向模式泛化为通用协议。

---

## 目标与非目标

### 目标

- 新增通用的"请求 UI 交互并等待结构化结果"协议
- 结果可存入脚本变量，用于条件分支和后续逻辑
- 不影响现有 `Click` / `ChoiceSelected` / `Signal` 机制
- 提供 `requestUI` 脚本语法供脚本作者使用
- 完整的单元测试覆盖往返通信链路

### 非目标

- 具体 UI 模式的实现（NVL、地图等推迟到 Phase 2）
- Host 侧具体 UI 渲染逻辑（Phase 1 仅搭建协议骨架）
- VarValue 类型扩展（如 Map/Array，按需在后续阶段添加）
- WebView 小游戏集成（Phase 3，RFC-021）

---

## 方案设计

### 方案对比

#### 方案 A：扩展 Signal

给现有 `Signal` 加 `data` 字段：

```rust
RuntimeInput::Signal {
    id: SignalId,
    data: Option<HashMap<String, VarValue>>,
}
```

- 优点：改动量最小，复用现有类型
- 缺点：语义污染——`Signal` 本意是"事件通知"（场景切换完毕、视频播放结束），加上数据后变成"通知 + 响应"双重角色。`WaitForSignal` 也被复用于两种完全不同的场景（效果等待 vs UI 结果等待），违反项目"用 enum 编码状态"的哲学

#### 方案 B：新增 UIResult（采纳）

新增专用变体，职责分离：

```rust
RuntimeInput::UIResult {
    key: String,
    value: VarValue,
}

WaitingReason::WaitForUIResult {
    key: String,
    result_var: String,
}
```

- 优点：类型精确，Signal 保持"通知"语义，UIResult 承担"结构化响应"语义；`WaitForUIResult` 自包含所有恢复所需信息（key 用于匹配，result_var 用于存储）
- 缺点：多两个枚举变体

**选择理由**：项目哲学明确"类型精度不是过度工程"、"用 enum 编码状态"。Signal 和 UIResult 的语义本质不同，分离后各自职责清晰，长期维护成本更低。

### 协议设计

#### 请求路径（Script → Runtime → Host）

```
ScriptNode::RequestUI { mode, result_var, params }
    ↓ Executor
Command::RequestUI { key, mode, params }  +  WaitForUIResult { key, result_var }
    ↓ Host CommandExecutor
Host 展示对应 UI 模式，等待用户交互
```

#### 响应路径（Host → Runtime → Script）

```
用户交互完成
    ↓ Host
RuntimeInput::UIResult { key, value }
    ↓ Runtime handle_input
匹配 WaitForUIResult.key == UIResult.key
    ↓
state.set_var(result_var, value)
state.clear_wait()
    ↓
脚本继续执行，可通过 $result_var 访问结果
```

### 新增类型定义

#### RuntimeInput 新增变体

```rust
// vn-runtime/src/input.rs
pub enum RuntimeInput {
    Click,
    ChoiceSelected { index: usize },
    Signal { id: SignalId },

    /// UI 交互结果（解除 `WaitForUIResult`）
    ///
    /// Host 完成 UI 交互后，将结果回传 Runtime。
    /// `key` 必须与对应的 `WaitForUIResult.key` 匹配。
    UIResult {
        /// 请求标识符（与 WaitForUIResult 配对）
        key: String,
        /// 交互结果值
        value: VarValue,
    },
}
```

#### WaitingReason 新增变体

```rust
// vn-runtime/src/state.rs
pub enum WaitingReason {
    None,
    WaitForClick,
    WaitForChoice { choice_count: usize },
    WaitForTime(Duration),
    WaitForSignal(SignalId),

    /// 等待 UI 交互结果
    ///
    /// Runtime 发出 `Command::RequestUI` 后进入此状态。
    /// 收到匹配的 `RuntimeInput::UIResult` 后，将 `value` 存入
    /// `result_var` 指定的脚本变量，然后清除等待。
    WaitForUIResult {
        /// 请求标识符（与 UIResult.key 配对）
        key: String,
        /// 结果存储的目标变量名
        result_var: String,
    },
}
```

#### Command 新增变体

```rust
// vn-runtime/src/command/mod.rs
pub enum Command {
    // ... existing variants ...

    /// 请求 Host 展示自定义 UI 并等待用户交互
    ///
    /// Host 收到后应根据 `mode` 展示对应 UI，用户完成交互后
    /// 通过 `RuntimeInput::UIResult { key, value }` 回传结果。
    RequestUI {
        /// 请求标识符（用于匹配响应）
        key: String,
        /// UI 模式标识（Host 据此选择展示哪种 UI）
        mode: String,
        /// 模式特定参数
        params: HashMap<String, VarValue>,
    },
}
```

#### ScriptNode 新增变体

```rust
// vn-runtime/src/script/ast/mod.rs
pub enum ScriptNode {
    // ... existing variants ...

    /// 请求 UI 交互
    ///
    /// 对应 `requestUI "mode" as $var (params)` 语法
    RequestUI {
        /// UI 模式标识
        mode: String,
        /// 结果存储变量名（不含 $ 前缀）
        result_var: String,
        /// 模式特定参数
        params: Vec<(String, Expr)>,
    },
}
```

### 脚本语法

```markdown
requestUI "mode_name" as $result_var
requestUI "mode_name" as $result_var (param1: value1, param2: "string_value")
```

示例：

```markdown
requestUI "show_map" as $destination (map_id: "world")
if $destination == "beach"
  goto **beach_chapter**
elseif $destination == "mountain"
  goto **mountain_chapter**
endif
```

语法规则：
- `requestUI` 为关键字
- 模式名用双引号包裹
- `as $var` 指定结果存储变量（必选）
- 参数列表可选，用圆括号包裹，格式与 `sceneEffect` 参数一致

### handle_input 处理逻辑

```rust
// vn-runtime/src/runtime/engine/mod.rs handle_input()
(WaitingReason::WaitForUIResult { key: expected_key, result_var },
 RuntimeInput::UIResult { key, value }) => {
    if key == *expected_key {
        self.state.set_var(result_var.clone(), value);
        self.state.clear_wait();
    }
    // key 不匹配时静默忽略（与 Signal 行为一致）
    Ok(())
}
```

### Executor 处理逻辑

```rust
// vn-runtime/src/runtime/executor/mod.rs
ScriptNode::RequestUI { mode, result_var, params } => {
    let key = mode.clone(); // key 复用 mode 名称
    let evaluated_params = /* 求值 params 中的表达式 */;
    Ok(ExecuteResult::with_wait(
        vec![Command::RequestUI {
            key: key.clone(),
            mode: mode.clone(),
            params: evaluated_params,
        }],
        WaitingReason::WaitForUIResult {
            key,
            result_var: result_var.clone(),
        },
    ))
}
```

---

## 影响范围

| 模块 | 改动 | 风险 |
|------|------|------|
| `vn-runtime/src/input.rs` | 新增 `UIResult` 变体 + 构造器 | 低：纯新增 |
| `vn-runtime/src/state.rs` | 新增 `WaitForUIResult` 变体 + 构造器 | 低：纯新增，需注意序列化兼容 |
| `vn-runtime/src/command/mod.rs` | 新增 `RequestUI` 变体，新增 `use` 导入 | 低：纯新增，Host 侧 match 需补分支 |
| `vn-runtime/src/script/ast/mod.rs` | 新增 `RequestUI` AST 节点 | 低：纯新增 |
| `vn-runtime/src/script/parser/phase2/` | 新增 `requestUI` 解析 | 中：需要正确解析参数列表 |
| `vn-runtime/src/runtime/executor/mod.rs` | 新增 `RequestUI` 执行分支 | 低：模式与现有 SceneEffect 类似 |
| `vn-runtime/src/runtime/engine/mod.rs` | `handle_input` 新增 UIResult 分支 | 中：需正确处理变量写入 |
| `host/src/command_executor/` | 新增 `RequestUI` 命令处理 | 低：Phase 1 仅日志 + 标记 |
| `host/src/input/mod.rs` | `update()` 处理 `WaitForUIResult` | 低：Phase 1 暂不采集 |

---

## 迁移计划

本 RFC 为纯新增功能，无破坏性变更：

1. 所有新增变体为 enum 扩展，现有匹配分支不受影响
2. `WaitForUIResult` 序列化兼容：新字段有默认语义，旧存档不包含此状态
3. Host 侧 `Command` match 需补 `RequestUI` 分支（编译器会强制提醒）
4. 现有 `Signal` / `Choice` / `Click` 路径完全不变

---

## 验收标准

- [ ] `RuntimeInput::UIResult` 变体定义完整，含构造器和序列化测试
- [ ] `WaitingReason::WaitForUIResult` 变体定义完整，含构造器和序列化测试
- [ ] `Command::RequestUI` 变体定义完整
- [ ] `ScriptNode::RequestUI` AST 节点定义完整
- [ ] Parser 正确解析 `requestUI "mode" as $var` 和 `requestUI "mode" as $var (params)` 语法
- [ ] Parser round-trip 测试通过
- [ ] Executor 将 RequestUI 节点转换为正确的 Command + WaitForUIResult
- [ ] `handle_input` 正确处理 UIResult：key 匹配时写入变量并清除等待
- [ ] `handle_input` 正确处理 UIResult：key 不匹配时忽略
- [ ] 现有 Click / ChoiceSelected / Signal 测试全部通过
- [ ] Host 侧 Command match 编译通过（补 RequestUI 分支）
- [ ] `cargo check-all` 通过
- [ ] 脚本语法规范文档更新
- [ ] 模块摘要文档更新
