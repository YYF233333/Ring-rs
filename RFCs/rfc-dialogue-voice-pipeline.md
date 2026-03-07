# RFC: 对话语音标注与自动播放管线

## 元信息

- 编号：RFC-001
- 状态：Proposed
- 作者：Ring-rs 开发组
- 日期：2026-03-07
- 相关范围：`docs/script_syntax_spec.md`、`vn-runtime`、`host`、`tools/xtask`

---

## 1. 背景

当前脚本语音仍依赖手写音频命令，内容制作负担较高，且缺失资源不易提前发现。  
需要建立“台词携带 voice_id，Host 自动播放”的统一语音管线。

---

## 2. 目标与非目标

### 2.1 目标

- 对话行可附带 `voice_id`，不要求显式播放语音命令。
- Runtime 保持声明式输出，Host 自动解析并播放语音。
- 缺失语音资源不影响运行（运行时降级），但能被工具链提前诊断。
- Skip/Auto 下语音行为可预测。

### 2.2 非目标

- 不在本 RFC 引入新的“语音播放脚本命令”。
- 不要求一次性覆盖所有历史脚本语音资产。

---

## 3. 提案

### 3.1 脚本标注

- 对话行尾支持 `voice_id` 标注：
  - `角色："台词" [#v:foo_001]`

### 3.2 Parser/AST

- 对话节点新增：`voice_id: Option<String>`。

### 3.3 Runtime -> Host 协议

- 扩展 `Command::ShowText`，携带 `voice_id: Option<String>`。
- Runtime 仍只产出声明式 `Command`，不管理音频通道细节。

### 3.4 Host 播放语义

- 执行 `ShowText` 时按约定路径查找语音并播放到 Voice 专用通道。
- 默认路径：`voices/{voice_id}.ogg`（可按顺序尝试 `ogg/mp3/wav/flac`）。
- 找不到资源：静默跳过（可输出诊断，不中断运行）。
- 推进到下一句/Skip：停止当前 voice。
- Auto：是否等待 voice 播放完成后推进，作为可配置策略。

### 3.5 工具链

- 扩展 `cargo script-check` 或新增 `cargo voice-index`，输出：
  - 台词 -> voice_id -> 解析路径 -> 资源存在性
  - 缺失项清单

---

## 4. DoD（验收）

- voice 缺失不影响运行；voice 存在时可稳定随对话播放且不叠音。
- Skip/推进行为一致（默认 stop 当前 voice）。
- Auto 行为可配置且符合预期。
- 工具链可产出可定位的缺失诊断。

---

## 5. 实施阶段

1. 语法与 AST 扩展
2. `Command::ShowText` 协议扩展
3. Host Voice 通道执行策略
4. 工具链检查与缺失报告
5. 回归与样例脚本覆盖
