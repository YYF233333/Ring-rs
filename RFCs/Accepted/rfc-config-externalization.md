# RFC-013: 配置默认值外部化

## 元信息

- **编号**：RFC-013
- **状态**：Accepted
- **作者**：Ring-rs 开发组
- **日期**：2026-03-16
- **相关范围**：`host/src/config/`、`host/src/ui/layout/`、`host/src/ui/screen_defs/`、`tools/asset-packer/`、`config.json`、`assets/ui/layout.json`、`assets/ui/screens.json`、`docs/config_guide.md`、`docs/ui_customization.md`、`docs/screens_customization.md`

---

## 实现状态（更新于 2026-03-16）

| 项 | 状态 |
|----|------|
| `config.json` 必填、`load` 返回 `Result`、缺失/解析失败即 crash | ✅ 已实现 |
| `ui/layout.json` 必填、`load` 返回 `Result`、缺失/解析失败即 crash | ✅ 已实现 |
| `ui/screens.json` 必填、`load` 返回 `Result`、缺失/解析失败即 crash | ✅ 已实现 |
| 移除 `config` / `layout` 的 `#[serde(default)]` | ✅ 已实现 |
| `screens.json` 字段 | ⚠️ 部分保留：`ButtonDef.visible`、`ButtonDef.confirm`、`ConditionalAsset.when` 仍为 `#[serde(default)]`（缺失即 `None`），便于书写可选可见性/确认文案/条件 |
| 主要结构体添加 `#[serde(deny_unknown_fields)]` | ✅ 已实现（config / layout / screen_defs） |
| 仓库自带完整默认 JSON（config、layout、screens） | ✅ 已实现 |
| `asset-packer` release 时改写 `config.json`（ZIP 模式 + `script_check: false`、`log_file: "game.log"`） | ✅ 已实现 |
| `impl Default` 保留供测试、运行时加载路径不调用 | ✅ 已实现 |

调用方：`main.rs` 与 `app/mod.rs` 对三个 `load` 均使用 `.unwrap_or_else(|e| panic!("{}", e))`，缺失或解析失败会立即退出并打印错误信息。

## 背景

当前 Host 的配置系统（`config.json`、`ui/layout.json`、`ui/screens.json`）采用"缺失即回退默认值"策略：配置文件可以不存在，JSON 字段可以缺失，均静默回退到代码内硬编码的默认值。

这导致三个可维护性问题：

1. **默认值散落**：120+ 个字段的默认值分布在 `config/mod.rs`（13 个 `default_xxx()` 函数）、`ui/layout/mod.rs`（`mod defaults` 约 250 行）、`ui/screen_defs/mod.rs`（`mod defaults` 约 225 行），查找和修改需要阅读源码。
2. **静默回退掩盖错误**：配置文件缺失、字段拼写错误、格式不对均不报错，开发者不知道配置是否生效。
3. **新功能不可见**：新增配置项后，开发者不知道有这个选项，因为默认值让一切"正常工作"。

## 目标

- 将三个开发者配置文件（`config.json`、`ui/layout.json`、`ui/screens.json`）的所有默认值迁移到外部 JSON 文件
- 配置文件缺失或字段缺失时立即 crash 并给出清晰错误信息
- 从 serde 结构体上移除所有 `#[serde(default)]` / `#[serde(default = "...")]` 注解
- 移除 `config/mod.rs` 的默认值函数和 `layout/mod.rs`、`screen_defs/mod.rs` 的 `mod defaults` 块
- 仓库自带完整的默认配置文件，开箱即用

## 非目标

- 不改动 `user_settings.json`（玩家运行时生成的偏好设置）
- 不改动 `persistent.json`（运行时脚本状态）
- 不改动 `manifest.json`（内容配置，空游戏可无角色，保持可选）
- 不引入配置版本号或自动迁移机制

## 方案

### 覆盖范围

| 配置文件 | 当前策略 | 改后策略 |
|----------|---------|---------|
| `config.json` | 缺失/解析失败 → `Default::default()` | 缺失/解析失败 → crash |
| `ui/layout.json` | 缺失/解析失败 → `Default::default()` | 缺失/解析失败 → crash |
| `ui/screens.json` | 缺失/解析失败 → `Default::default()` | 缺失/解析失败 → crash |
| `manifest.json` | 不变 | 不变 |
| `user_settings.json` | 不变 | 不变 |
| `persistent.json` | 不变 | 不变 |

### 字段级策略

移除所有 `#[serde(default)]` / `#[serde(default = "...")]`。JSON 缺少任何字段即反序列化失败。

例外（保留 `Option` 语义但仍需显式写 `null`）：
- `config.json` 的 `name`：不影响启动，纯展示/打包用
- `config.json` 的 `zip_path`：仅 zip 模式需要

添加 `#[serde(deny_unknown_fields)]` 到主要配置结构体，防止字段拼写错误被忽略。

### debug/release 差异处理

统一为一个 `config.json`，默认配置为开发模式（`script_check: true`，`log_file: null`）。`asset-packer` 打包 release 时自动修改为发布配置（`script_check: false`，`log_file: "game.log"`）。

### 加载函数签名变更

```rust
// Before
impl AppConfig {
    pub fn load(path: &str) -> Self { /* 失败返回 Default */ }
}

// After
impl AppConfig {
    pub fn load(path: &str) -> Result<Self, ConfigError> { /* 失败返回 Err */ }
}
```

`UiLayoutConfig::load` 和 `ScreenDefinitions::load` 同理。调用方在错误时打印清晰信息后 panic。

### 仓库默认文件

利用现有 `Default::default()` 实现生成初始 JSON 文件，放入仓库：
- 根目录 `config.json`（补全所有字段）
- `assets/ui/layout.json`（完整布局配置）
- `assets/ui/screens.json`（完整界面行为定义）

### 测试策略

`impl Default` 保留在代码中供测试使用，运行时加载路径不再调用。

## 风险

- **前向兼容代价**：新增字段 = 破坏性变更，旧 JSON 文件 break。在本项目上下文中可接受（单人开发，显式报错优于静默回退）。
- **layout.json 体量大**：80+ 字段的完整 JSON 约 200+ 行，但这是显式化的目的。
- **asset-packer 适配**：release 打包需要知道哪些 debug 字段要改写。

## 迁移计划（已完成）

1. ~~生成三个完整默认 JSON 文件~~ ✅
2. ~~改造三个 `load` 函数为 `Result` 返回 + 调用方 crash 处理~~ ✅
3. ~~移除所有 `serde(default)` 注解和 `mod defaults` 代码块~~ ✅（screens 仅保留 3 处可选字段的 `serde(default)`）
4. ~~适配 `asset-packer` release 打包的 debug 字段改写~~ ✅
5. ~~更新文档（`config_guide.md`、UI 定制指南、模块摘要）~~ ✅

## 验收标准

- `cargo check-all` 通过
- 三个 JSON 文件存在时行为与改造前完全一致
- 删除任一 JSON 文件时启动 crash 并给出明确错误信息
- JSON 中缺少任何必填字段时 crash 并给出缺失字段名
- JSON 中存在未知字段时 crash 并给出字段名
- `asset-packer` release 打包正确改写 debug 字段
