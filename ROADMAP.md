# Visual Novel Engine 开发路线图

> 本文档定义了项目的具体执行计划，遵循 PLAN.md 中的架构约束。

---

## 项目当前状态

### ✅ 已完成模块

1. **vn-runtime 核心运行时**
   - ✅ 脚本解析器（Parser）：覆盖当前已实现语法，50+ 测试用例
   - ✅ AST 定义：完整的脚本节点类型
   - ✅ Command 定义：Runtime → Host 通信协议
   - ✅ RuntimeInput 定义：Host → Runtime 输入模型
   - ✅ WaitingReason 定义：显式等待状态模型
   - ✅ RuntimeState 定义：可序列化的运行时状态
   - ✅ Engine（VNRuntime）：核心执行引擎
   - ✅ Executor：AST 节点到 Command 的转换
   - ✅ 错误处理：完整的错误类型和错误信息

2. **host 适配层（macroquad）**
   - ✅ 窗口与主循环
   - ✅ 资源管理系统（PNG/JPEG 支持）
   - ✅ 渲染系统（背景/角色/对话框/选择分支/章节标题）
   - ✅ 输入处理（键盘/鼠标，防抖）
   - ✅ Command 执行器
   - ✅ Runtime 集成
   - ✅ 过渡效果实现（dissolve/fade/fadewhite）
   - ✅ 音频系统（rodio，支持 MP3/WAV/FLAC/OGG）

---

## 开发历程总结

### 阶段 1-2：基础框架 + 资源管理
- 窗口/主循环/调试信息
- `ResourceManager` 负责图片与音频资源加载、缓存与路径解析
- 支持 PNG 和 JPEG 图片格式

### 阶段 3：渲染系统
- 背景、角色、对话框、选择分支 UI 渲染
- 中文字体加载（simhei.ttf）
- 打字机效果

### 阶段 4：输入处理
- `InputManager` 统一采集鼠标/键盘输入
- WaitingReason 驱动的输入分发与防抖

### 阶段 5：Command 执行器
- 完整 Command 分发与 RenderState 更新
- 过渡效果与音频命令的执行管线

### 阶段 6：Runtime 集成
- Script 模式接入 `VNRuntime` 与 `Parser`
- Demo/Command/Script 三模式切换

### 阶段 7：过渡效果
- dissolve/fade/fadewhite 过渡支持
- 背景切换时自动应用过渡

### 阶段 8：音频系统
- `AudioManager` 采用 `rodio`，支持 MP3/WAV/FLAC/OGG
- BGM 循环/淡入淡出/切换，SFX 播放与音量控制

### 阶段 9：UI 完善
- 选择分支支持鼠标悬停高亮与点击选择
- 章节标题移至左上角，避免遮挡内容

### 阶段 10：测试与优化 ✅
- 创建完整功能测试脚本（test_comprehensive.md）
- 端到端功能验证：背景/角色/对话/选择分支/章节标题
- 修复 JPEG 图片加载问题（添加 image crate 支持）
- 修复选择分支后选项不消失的 bug
- 修复脚本资源路径：素材路径**相对于脚本文件**解析（便于 Typora 预览）
- 修复路径 edge case：统一使用 `std::path` 规范化路径，解决 `../` 导致的纹理缓存键不一致/背景不显示问题
- 文档更新

---

## 阶段 11：脚本语法补齐 + 立绘资源/布局元数据系统 ✅ 已完成

> 主题：**脚本语法补齐（音频/控制流）** + **立绘资源/布局元数据系统**

### 11.1 脚本语法补齐：音频/控制流 ✅

- **已完成**
  - `<audio src="..."></audio>`：SFX 播放一次
  - `<audio src="..."></audio> loop`：BGM 循环播放
  - `stopBGM`：停止当前 BGM（带淡出）
  - `goto **label**`：无条件跳转
  - 资源路径解析规则调整：素材路径相对于脚本文件，且路径已规范化（`std::path`）
  - BGM 切换自带交叉淡化效果（1秒）
  - 错误处理：文件不存在/格式不支持时打印错误但不崩溃
  - ~~音量/静音~~：由玩家在设置中控制，脚本层不实现
- **验收标准** ✅
  - 脚本可完整覆盖：播放 BGM → 切换 BGM（自动交叉淡化）→ 播放 SFX → stopBGM

### 11.2 资源管理（立绘）：anchor + pre_scale + preset 布局系统 ✅

- **背景**：立绘尺寸/构图不统一，无法用单一缩放规则保证"站位稳定/构图一致"。
- **核心想法**：为**每组立绘**提供可配置的 `anchor`（重心/对齐点）与 `pre_scale`（预处理缩放），再叠加**全局 preset**（点位 scale/偏移），使不同立绘在屏幕上呈现一致的相对效果。

- **已完成**
  - `assets/manifest.json` 资源清单：
    - `characters.groups`: 立绘组配置（anchor + pre_scale）
    - `characters.sprites`: 立绘路径到组的显式映射
    - `presets`: 九宫格站位预设（x, y, scale）
    - `defaults`: 默认配置（未配置立绘的兜底）
  - `host/src/manifest/mod.rs` 模块：
    - Manifest 数据结构定义（serde 序列化/反序列化）
    - `get_group_config()`: 查找顺序 = 显式映射 → 路径推导 → 默认配置
    - `infer_group_id()`: 兜底规则（目录名 / 文件名前缀）
    - 4 个单元测试覆盖核心场景
  - `Renderer::render_characters()` 重写：
    - 基于 anchor + pre_scale + preset 计算位置和缩放
    - 删除旧的硬编码 `position_to_screen_coords()` / `scale_character_size()`

- **验收标准** ✅
  - 元数据缺失时有合理默认值（不崩溃，可用）
  - 新立绘组只需编辑 manifest.json，无需改代码

---

## 阶段 12：架构性改动 ✅ 已完成

> 主题：**先做会影响项目结构的部分**

### 12.1 存档/读档系统 ✅

- **实现内容**
  - `vn-runtime/src/save.rs`：定义 `SaveData`、`SaveVersion`、`SaveMetadata`、`AudioState`、`RenderSnapshot`
  - 版本兼容策略：major 版本必须一致，minor 可不同
  - `host/src/save_manager/mod.rs`：存档文件布局 `saves/slot_XXX.json`，读写 API
  - 快捷键：F5 保存，F9 读取
  - `VNRuntime::restore_state()` 支持状态恢复

### 12.2 历史记录数据模型 ✅

- **实现内容**
  - `vn-runtime/src/history.rs`：定义 `HistoryEvent`（Dialogue/ChapterMark/ChoiceMade/Jump/BackgroundChange/BgmChange）
  - `History` 容器，支持最大事件数限制、序列化
  - `VNRuntime` 在 tick 时自动记录历史
  - 历史数据随存档持久化

### 12.3 资源与配置治理 ✅

- **实现内容**
  - `host/src/config/mod.rs`：`AppConfig`（assets_root、saves_dir、window、debug、audio 配置）
  - `config.json` 配置文件，支持默认值和校验
  - `Manifest::validate()` 校验方法，检测无效锚点/预缩放/预设位置/引用不存在的组
  - `ManifestWarning` 警告类型，友好的错误提示

---

## 阶段 13：测试覆盖率 + 文档水平提升 ✅ 已完成

> 主题：**在大改动完成后集中提升质量**

### 13.1 测试覆盖率提升 ✅

- **完成内容**
  - parser：新增 goto/audio/stopBGM/相对路径测试（+14 个测试）
  - engine：新增历史记录、状态恢复、goto 跳转、选择分支测试（+5 个测试）
  - 总测试数：101 个（host 24 + vn-runtime 77）

### 13.2 文档质量提升 ✅

- **完成内容**
  - README：更新项目结构、快捷键说明、功能列表
  - `docs/manifest_guide.md`：manifest 配置完整指南
  - `docs/save_format.md`：存档格式与版本兼容说明

---

## 后续开发方向

> 后续方向以“先结构治理、再 UI/演出增强、最后性能与工具链”为原则推进。

1. **结构性完善（优先）**
   - **配置落地**：把 host 中的硬编码路径/窗口参数统一切换为读取 `config.json`（assets_root / saves_dir / window / audio 等）
   - **存档系统完善**：多 slot 管理、UI 面板入口、存档元信息（截图/章节/时长）完善、版本迁移策略（major 变更的迁移工具/提示）
   - **历史系统完善**：把 ChoiceMade/Jump 等事件在 Runtime 输入处理阶段补齐记录（目前主要记录来自 Command）
   - **资源治理**：统一资源 key 规范（归一化路径），补齐 manifest schema 文档与校验覆盖（含 presets/anchor/pre_scale 的边界）

2. **玩家 UI / 体验增强（第二优先）**
   - **历史回看 UI**：面板展示历史对话、点击跳回（可选）、与存档对齐
   - **存档 UI**：slot 列表、预览、删除、重命名
   - **设置 UI**：音量/静音/字体/分辨率等（注意：音量属于玩家控制，不进脚本）
   - **对话框样式增强**：阴影、描边、自定义背景皮肤
   - **`rule` 遮罩过渡**：演出增强（非结构性）

3. **工程化 / 性能 / 工具链（持续）**
   - **性能**：资源预加载与按需释放、纹理缓存策略、渲染性能优化
   - **工程化**：CI（fmt/clippy/test）、发布构建脚本、崩溃日志与诊断开关
   - **工具链**：资源打包工具、脚本 linter/formatter、manifest 可视化调参工具（可选）

---

## 开发原则

1. **遵循 PLAN.md 约束**
   - Runtime 与 Host 严格分离
   - Command 驱动模式
   - 显式状态管理

2. **测试驱动开发**
   - 每个模块都要有单元测试
   - 关键功能要有集成测试
   - 修复 bug 后补充回归测试

3. **渐进式开发**
   - 先实现核心功能，再完善细节
   - 每个阶段都要有可运行的版本
   - 及时集成和测试

4. **代码质量**
   - 清晰的模块划分
   - 完善的文档注释
   - 遵循 Rust 最佳实践

---

> **注意**：本路线图是动态文档，会根据实际开发进度和需求变化进行调整。
