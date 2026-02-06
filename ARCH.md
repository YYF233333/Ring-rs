# Ring Engine 架构设计

> 本文档定义项目架构约束

## 总体架构原则（硬约束）

- Runtime 与 Host 分离
   - **`vn-runtime`**：纯逻辑核心（脚本解析/执行、状态管理、等待建模、产出 `Command`）
   - **`host`**：IO/渲染/音频/输入/资源宿主（执行 `Command` 产生画面/音频/UI）
   - Runtime **禁止**：引擎 API（macroquad）、IO、真实时间依赖
   - Host **禁止**：脚本逻辑；直接修改 Runtime 内部状态

- 显式状态、确定性执行
   - 所有运行状态必须**显式建模**且可序列化（支持存档/读档）
   - 不允许隐式全局状态
   - 不依赖真实时间推进逻辑（时间等待由 Host 负责）

- 命令驱动（Command-based）
   - Runtime **只产出** `Command`
   - Host **只执行** `Command`
   - Runtime 不直接渲染/播放音频/等待输入

---

## VN Runtime 核心模型（必须遵守）

- `RuntimeState`（唯一可变状态）
   - 脚本执行位置（`ScriptPosition`）
   - 脚本变量（variables）
   - 当前等待状态（`WaitingReason`）
   - 以及其他可恢复的显式状态（如已显示角色/背景等）

   要求：**可序列化**、可测试；禁止隐式状态。

- `WaitingReason`（显式等待模型）

   允许的等待原因（示例口径）：

   ```text
   None
   WaitForClick
   WaitForChoice { choice_count }
   WaitForTime(Duration)
   WaitForSignal(SignalId)
   ```

   禁止使用隐式 await/sleep 来推进脚本。

- 执行模型（tick）
   - Runtime 通过 `tick(input) -> (Vec<Command>, WaitingReason)` 驱动
   - 若处于等待：仅处理输入尝试解除等待
   - 若不等待：持续推进脚本直到再次阻塞或结束

- `RuntimeInput`（Host → Runtime）

   典型输入：

   ```text
   Click
   ChoiceSelected(index)
   Signal(signal_id)
   ```

   说明：`WaitForTime` 由 Host 处理（Host 等待指定时长再调用 tick）。

---

## Command 模型（Runtime → Host）

- `Command` 是 Runtime 与 Host 的**唯一通信方式**
- 要求：**声明式**、不包含引擎类型、不产生副作用
