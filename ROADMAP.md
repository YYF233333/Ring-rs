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
   - ✅ 过渡效果实现（dissolve/fade/fadewhite/rule(ImageDissolve)）
   - ✅ 音频系统（rodio，支持 MP3/WAV/FLAC/OGG）

---

## 开发历程总结（浓缩版）

> 目标：避免把 ROADMAP 写成“开发日志”。这里仅保留里程碑结论，细节进入对应阶段归档。

### 里程碑摘要（阶段 1-14）
- **基础架构**：主循环/渲染/输入/资源管理跑通，Runtime/Host 分离落地
- **渲染与输入**：背景/立绘/对话/选择/章节标记 + 打字机效果 + 输入防抖
- **演出与音频**：dissolve/fade/fadewhite/rule(ImageDissolve) + rodio 音频（BGM/SFX/淡入淡出/切换）
- **质量与路径治理**：端到端脚本验证 + 统一 `std::path` 规范化解决 `../` 资源键不一致
- **脚本语法补齐**：音频/控制流（goto） + 立绘布局元数据系统
- **架构性改动**：存档/读档 + 历史记录 + 配置治理
- **结构性完善**：配置落地 + 存档/历史完善 + 资源治理

---

## 阶段 11-14：脚本语法扩展 + 架构完善 + 质量提升 ✅ 已完成

> **成果总结**：补齐脚本语法（音频/控制流）、建立立绘布局元数据系统、实现存档/历史/配置架构、完善测试和文档。

**核心成果**：
- ✅ **脚本语法**：音频指令（SFX/BGM/stopBGM）、控制流（goto）、路径规范化
- ✅ **立绘布局系统**：manifest.json 元数据（anchor/pre_scale/presets），避免硬编码适配
- ✅ **架构性改动**：存档/读档系统、历史记录、配置治理（AppConfig + config.json）
- ✅ **质量提升**：测试覆盖率提升、文档完善（manifest/save 格式说明）

**关键文件**：
- `vn-runtime/src/script/{ast.rs,parser.rs}`、`vn-runtime/src/runtime/executor.rs`
- `host/src/manifest/mod.rs`、`host/src/renderer/mod.rs`
- `vn-runtime/src/{save.rs,history.rs}`、`host/src/{save_manager/mod.rs,config/mod.rs}`
- `docs/{manifest_guide.md,save_format.md}`

---

## 阶段 15-17：演出系统 + 玩家 UI + 体验打磨 ✅ 已完成

> **成果总结**：完成了演出系统重构（changeBG/changeScene 职责分离、过渡命名参数、Rule/ImageDissolve）、玩家 UI 系统（Title/菜单/存读档/设置/历史）、以及体验打磨（Continue 存档、入口配置化、文档整理）。

**核心成果**：
- ✅ **演出系统**：`changeBG`（简单切换）vs `changeScene`（复合演出）职责分离；过渡支持命名参数；Rule/ImageDissolve 两段式实现
- ✅ **玩家 UI**：`AppMode` + `NavigationStack` 状态机；完整的 UI 组件库（Theme/Button/Panel/List/Modal/Toast）；Title/菜单/存读档/设置/历史界面
- ✅ **体验打磨**：Continue 专用存档；SaveLoad 1-99 槽位 + 完整元信息；入口脚本配置化；文档同步更新

**关键文件**：
- `host/src/renderer/transition.rs`、`host/src/renderer/image_dissolve.rs`
- `host/src/app_mode.rs`、`host/src/ui/*`、`host/src/screens/*`
- `host/src/save_manager/mod.rs`、`host/src/config/mod.rs`

---

## 阶段 18：Scale-up 资源动态加载 + 资源打包/发布 ✅ 已完成

> **成果总结**：实现了完整的资源动态加载系统，支持按需加载、LRU 缓存驱逐、显存预算控制；支持从文件系统或 ZIP 包加载资源；提供资源打包工具用于发布。详见 `docs/resource_management.md`。

**核心功能**：
- ✅ **ResourceSource 抽象层**：统一文件系统和 ZIP 包的资源访问接口
- ✅ **TextureCache + LRU 驱逐**：默认 256MB 显存预算，自动驱逐最久未使用的纹理
- ✅ **按需加载**：启动不再预加载所有资源，运行时按需加载并缓存
- ✅ **ZipSource + 打包工具**：支持将资源打包为 ZIP 文件，发布时无需散落资源目录
- ✅ **Debug Overlay**：按 F1 显示缓存统计（命中率、占用、驱逐次数等）

**关键文件**：
- `host/src/resources/`：ResourceManager、ResourceSource、TextureCache
- `tools/asset-packer/`：资源打包工具
- `docs/resource_management.md`：用户使用指南

---

## 下一步开发方向

### 阶段 19：脚本语法扩展（变量系统 + 条件分支）🟦 计划中

> **主题**：扩展脚本语言，支持变量、条件分支、循环等编程特性，使脚本更灵活。

**目标**：
- **变量系统**：支持数字、字符串、布尔类型；变量作用域（全局/局部）；变量持久化到存档
- **条件分支**：`if/elseif/else` 语法，支持变量比较和逻辑运算
- **循环**：`while` 循环，支持条件控制
- **表达式求值**：支持算术、比较、逻辑运算

**核心设计**：
- `RuntimeState` 扩展：添加 `variables: HashMap<String, Value>` 字段
- AST 扩展：新增 `If`、`While`、`SetVariable` 节点
- 表达式解析器：支持变量引用和运算（`$var_name` 语法）
- 向后兼容：现有脚本无需修改即可运行

**关键文件**：
- `vn-runtime/src/script/ast.rs`：扩展 AST 节点
- `vn-runtime/src/script/parser.rs`：表达式解析
- `vn-runtime/src/runtime/engine.rs`：变量作用域管理
- `vn-runtime/src/state.rs`：RuntimeState 扩展

**验收标准**：
- 支持变量声明、赋值、引用
- 支持 `if/elseif/else` 条件分支
- 支持 `while` 循环
- 变量随存档持久化
- 现有脚本无需修改即可运行

### 阶段 20：演出效果增强 🟦 计划中

> **主题**：增强演出效果，支持立绘动画、对话框动画、更丰富的过渡效果。

**目标**：
- **立绘动画**：淡入/淡出、移动、缩放动画
- **对话框动画**：显示/隐藏动画、样式切换
- **过渡效果扩展**：更多内置过渡效果（wipe、slide 等）
- **动画系统**：统一的时间轴和缓动函数（ease-in/out 等）

**关键文件**：
- `host/src/renderer/animation.rs`：动画系统
- `host/src/renderer/character.rs`：立绘动画
- `host/src/renderer/dialogue.rs`：对话框动画

### 阶段 21：性能优化 + 跨平台支持 🟦 计划中

> **主题**：性能优化（异步加载、多线程）和跨平台支持（Linux、macOS）。

**目标**：
- **异步资源加载**：后台线程加载资源，避免阻塞主线程
- **多线程渲染**：纹理解码在后台线程完成
- **跨平台支持**：Linux、macOS 构建和测试
- **性能分析**：集成性能分析工具

### 阶段 22：编辑器工具 🟦 计划中

> **主题**：开发可视化脚本编辑器，提升开发效率。

**目标**：
- **脚本编辑器**：语法高亮、自动补全、实时预览
- **资源管理器**：可视化资源浏览和管理
- **调试工具**：断点、变量监视、执行流程可视化

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
