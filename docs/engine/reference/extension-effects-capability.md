# 效果扩展：Capability 与回退策略

本文档描述 host 效果系统的 **capability 路由** 与 **回退（fallback）策略**，便于维护扩展与排查“效果未生效”问题。实现位置：`host/src/app/command_handlers/effect_applier.rs`、`host/src/renderer/effects/request.rs`、`host/src/extensions/builtin_effects.rs`。

---

## 一、Capability ID 一览

内建扩展注册的 capability 与效果类型、目标的对应关系如下。

| Capability ID | 效果类型 / 目标 | 说明 |
|---------------|------------------|------|
| `effect.dissolve` | 立绘显示/隐藏、背景过渡（Dissolve） | Alpha 交叉淡化 |
| `effect.fade` | 场景过渡（changeScene）Fade / FadeWhite | 黑屏或白屏遮罩过渡 |
| `effect.rule_mask` | 场景过渡（changeScene）Rule | 图片遮罩过渡 |
| `effect.move` | 立绘位置移动（CharacterMove） | 位移动画 |
| `effect.scene.shake` | 场景效果名含 shake/bounce | 镜头震动 |
| `effect.scene.blur` | 场景效果名含 blur/flashback | 模糊/闪回 |
| `effect.scene.dim` | 场景效果名含 dim | 暗角/变暗 |
| `effect.scene.title_card` | 标题字卡（TitleCard） | 字卡淡入淡出 |

**路由规则**（`EffectRequest::new` 内 `infer_capability_id`）：

- **Dissolve**（任意 target）→ `effect.dissolve`
- **Fade / FadeWhite** 且 target 为 **SceneTransition** → `effect.fade`
- **Rule** 且 target 为 **SceneTransition** → `effect.rule_mask`
- **Move** 且 target 为 **CharacterMove** → `effect.move`
- target 为 **SceneEffect** → `effect.scene.{category}`，`category` 由效果名推断：shake / blur / dim / generic
- target 为 **TitleCard** → `effect.scene.title_card`
- 其余组合 → `effect.{kind_name}`（如 `effect.none`、`effect.dissolve` 等）

---

## 二、EffectTarget 与首选 Capability

| EffectTarget | 常用 EffectKind | 首选 Capability |
|--------------|------------------|-----------------|
| CharacterShow | Dissolve（或脚本指定） | 由 effect.kind 推断，多为 `effect.dissolve` |
| CharacterHide | Dissolve | `effect.dissolve` |
| CharacterMove | Move | `effect.move` |
| BackgroundTransition | Dissolve | `effect.dissolve` |
| SceneTransition | Fade / FadeWhite / Rule / None 等 | `effect.fade` / `effect.rule_mask` / 或 `effect.{kind_name}` |
| SceneEffect | SceneEffect | `effect.scene.shake` / `effect.scene.blur` / `effect.scene.dim` 等 |
| TitleCard | （由命令决定） | `effect.scene.title_card` |

---

## 三、回退策略（Fallback）

当 registry 对某次请求返回 **MissingCapability** 或 **Failed** 时，`effect_applier` 会尝试 **capability 级回退**：根据 `(target, effect.kind)` 构造一条新的 `EffectRequest`，改用更基础或通用的 capability 再分发一次。回退表与代码中 `build_fallback_request` 一致。

### 3.1 有回退的请求

| 请求 Target | 请求 EffectKind | 回退 Capability | 回退时 EffectKind |
|-------------|----------------|-----------------|-------------------|
| CharacterShow | 任意 | `effect.dissolve` | 保持原 effect |
| CharacterHide | 任意 | `effect.dissolve` | 保持原 effect |
| BackgroundTransition | 任意 | `effect.dissolve` | 保持原 effect |
| CharacterMove | 任意 | `effect.move` | 保持原 effect |
| SceneTransition | Rule | `effect.rule_mask` | 保持原 effect |
| SceneTransition | Fade / FadeWhite | `effect.fade` | 保持原 effect |
| SceneTransition | **其他**（含 None、未知） | `effect.fade` | **强制改为 Fade** |

说明：

- 立绘/背景的“任意效果”统一回退到 **dissolve**，保证至少有一个淡入淡出。
- 场景过渡的“未知/无效果”回退到 **fade**，并强制使用 Fade 类型，保证至少有一个黑屏过渡。

### 3.2 无回退的请求

以下目标 **不** 提供 capability 回退，缺失或失败时该次效果请求会被**放弃**（仅打日志）：

- **SceneEffect**（shake/blur/dim 等）
- **TitleCard**

原因：场景效果与字卡没有更简化的通用替代，缺少对应 capability 时跳过更安全。

### 3.3 回退时的防循环

若计算出的回退请求与原请求的 `(capability_id, effect.kind)` 完全一致，则**不再分发**，避免重复调用同一 capability 导致循环或重复日志。

---

## 四、内建扩展与注册

内建效果在启动时通过 `extensions::build_builtin_registry` 注册，每个扩展声明自己处理的 capability ID：

- `builtin.effect.dissolve` → `effect.dissolve`
- `builtin.effect.fade` → `effect.fade`
- `builtin.effect.rule_mask` → `effect.rule_mask`
- `builtin.effect.move` → `effect.move`
- `builtin.effect.scene` → `effect.scene.shake` / `effect.scene.blur` / `effect.scene.dim` / `effect.scene.title_card`

第三方扩展可注册新的 capability ID，或覆盖内建 ID（冲突时注册会报错）。回退逻辑只使用当前 registry 中存在的 capability，不会假设内建一定存在。

---

## 五、排查建议

- **效果完全不生效**：确认该 target + kind 对应的 capability 已由内建或扩展注册；若为 SceneEffect/TitleCard，确认无回退、需提供对应 capability。
- **效果被“降级”**：查看日志中是否出现 “未找到 capability 处理器，尝试 capability 级回退” 或 “回退执行成功”，对应上表可确认实际使用的回退 capability。
- **修改回退表**：仅需改 `host/src/app/command_handlers/effect_applier.rs` 中的 `build_fallback_request`，并同步更新本文档“回退策略”一节。
