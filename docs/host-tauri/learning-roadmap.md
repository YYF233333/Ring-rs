# 前端学习路线图

> **目标读者**：Rust 开发者，了解 HTML/CSS/JS 概念但未写过实际代码。
> **学习目标**：能读懂 Agent 的前端改动、做基本 UI 调试、提出 review 意见、理解项目前端架构。
> **不是目标**：成为前端专家。按需查阅，不要求系统学完。

---

## 阶段 1：看懂项目代码（1-2 天）

### 1.1 TypeScript 基础（vs Rust 对照）

TypeScript 是 JavaScript 的超集，加了静态类型。语法与 Rust 有不少对应关系。

| 概念 | Rust | TypeScript |
|------|------|------------|
| 类型标注 | `let s: &str` | `let s: string` |
| 结构体 | `struct Foo { bar: i32 }` | `interface Foo { bar: number }` |
| 枚举 | `enum E { A, B(String) }` | `type E = "A" \| { B: string }` |
| 泛型 | `Vec<T>` | `Array<T>` |
| 不可变 | 默认不可变 | `readonly` / `Readonly<T>` |
| 可空 | `Option<T>` | `T \| null` |
| 模块 | `mod` + `pub` | `export` / `import` |

**对照文件**：`host-tauri/src/types/render-state.ts`

打开这个文件，你会看到一组熟悉的模式：

```typescript
// 类似 Rust struct — 字段名 + 类型
export interface RenderState {
  current_background: string | null;       // Option<String>
  visible_characters: Readonly<Record<string, CharacterSprite>>;  // HashMap<String, CharacterSprite>
  dialogue: Readonly<DialogueState> | null; // Option<DialogueState>
  ui_visible: boolean;
  text_mode: "ADV" | "NVL";               // 类似 enum { ADV, NVL }
  nvl_entries: readonly NvlEntry[];        // Vec<NvlEntry>，readonly ≈ &[NvlEntry]
}
```

```typescript
// union type — 类似 Rust enum（带关联数据的变体）
export type SceneTransitionKind =
  | "Fade"
  | "FadeWhite"
  | { Rule: { mask_path: string; reversed: boolean } };

// 纯字符串 union — 类似 Rust 的 c-like enum
export type PlaybackMode = "Normal" | "Auto" | "Skip";
```

**关键差异**：
- TS 的 `interface` 只描述形状，没有方法实现（方法写在别处）
- TS 的 union type `A | B` 比 Rust enum 松散——没有穷尽检查保证（虽然开 `strict` 模式会改善）
- `Readonly<T>` 只是编译期约束，运行时可以绕过（不像 Rust 的 borrow checker）

> **推荐阅读**：[TypeScript 手册 — 基础](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html)（30 分钟即可覆盖上述内容）

---

### 1.2 Vue 3 SFC 结构

Vue 使用 **Single-File Component (SFC)** 格式：一个 `.vue` 文件包含三个区域。

```
┌─────────────────────────────────┐
│ <script setup lang="ts">        │  ← 逻辑（Rust 里的 impl 块）
│   // 响应式状态、函数、导入     │
│ </script>                       │
├─────────────────────────────────┤
│ <template>                      │  ← 模板（声明式 UI，类似 JSX / 模板引擎）
│   <div>{{ message }}</div>      │
│ </template>                     │
├─────────────────────────────────┤
│ <style scoped>                  │  ← 样式（CSS，scoped 表示只作用于本组件）
│   .container { color: red; }    │
│ </style>                        │
└─────────────────────────────────┘
```

**对照文件**：`host-tauri/src/components/Toast.vue`（项目中最简单的组件）

这个文件展示了完整的 SFC 三段结构：

- `<script setup>` 中定义了 `ToastItem` 接口、`items` 响应式数组、`show()` 方法
- `<template>` 中用 `v-for` 渲染列表、`{{ }}` 插值显示文本
- `<style scoped>` 中的 CSS 只影响本组件的 DOM 元素

`script setup` 是 Vue 3 的语法糖——顶层声明的变量/函数自动可在 `<template>` 中使用，不需要手动 `return`。

> **推荐阅读**：[Vue 3 — SFC 语法定义](https://cn.vuejs.org/api/sfc-spec.html)

---

### 1.3 Vue 响应式系统

Vue 的核心抽象：当数据变化时，UI 自动更新。类比 Rust 概念：

| Vue | 类比 Rust | 说明 |
|-----|-----------|------|
| `ref(value)` | `RefCell<T>` + 自动通知 | 可变容器，读写用 `.value`，模板中自动解包 |
| `computed(() => expr)` | 派生值，自动缓存 | 依赖变化时重新计算，否则返回缓存 |
| `readonly(ref)` | `&T`（共享不可变引用） | 只读视图，防止外部修改 |
| `watch(source, callback)` | — | 副作用监听器，source 变化时执行 callback |

**对照文件**：`host-tauri/src/composables/useEngine.ts`

```typescript
const renderState = ref<RenderState | null>(null);  // 响应式状态
const isRunning = ref(false);

// ... 修改 .value 后，所有使用 renderState 的模板会自动更新
renderState.value = state;

// 返回 readonly 视图，外部只能读不能写（类似暴露 &T）
return {
  renderState: readonly(renderState),
  isRunning: readonly(isRunning),
  // ...
};
```

**`ref` 的 `.value` 规则**：
- 在 `<script>` 中必须用 `.value` 访问：`renderState.value = state`
- 在 `<template>` 中自动解包，直接用变量名：`{{ renderState }}`

> **推荐阅读**：[Vue 3 — 响应式基础](https://cn.vuejs.org/guide/essentials/reactivity-fundamentals.html)

---

### 1.4 Vue 模板语法

模板是声明式的——描述"UI 应该长什么样"，而不是"如何操作 DOM"。

| 语法 | 含义 | Rust 类比 |
|------|------|-----------|
| `{{ expr }}` | 文本插值 | `format!("{}", expr)` |
| `v-if="cond"` / `v-else` | 条件渲染 | `if cond { render_a() } else { render_b() }` |
| `v-for="item in list"` | 列表渲染 | `for item in list { render(item) }` |
| `:attr="expr"` | 动态属性绑定（`v-bind` 缩写） | — |
| `@event="handler"` | 事件绑定（`v-on` 缩写） | — |

**对照文件**：`host-tauri/src/vn/ChoicePanel.vue`

```html
<template>
  <Transition name="choice-fade">
    <!-- v-if：仅当 choices 非 null 时渲染 -->
    <div v-if="choices" class="choice-overlay">
      <div class="choice-list">
        <!-- v-for：遍历选项列表 -->
        <button
          v-for="(item, idx) in choices.choices"
          :key="idx"
          class="choice-button"
          :class="{ hovered: choices.hovered_index === idx }"
          @click.stop="emit('choose', idx)"
        >
          <!-- {{ }} 文本插值 -->
          {{ item.text }}
        </button>
      </div>
    </div>
  </Transition>
</template>
```

这段代码做的事：如果有选项数据，渲染一个遮罩层，遍历每个选项生成按钮，点击时触发 `choose` 事件。

**对照文件**：`host-tauri/src/screens/HistoryScreen.vue`

```html
<!-- v-for + v-if 组合使用 -->
<div v-for="(entry, i) in entries" :key="i" class="history-entry">
  <span v-if="entry.speaker" class="entry-speaker">{{ entry.speaker }}</span>
  <span class="entry-text">{{ entry.text }}</span>
</div>
<div v-if="entries.length === 0" class="history-empty">
  暂无历史记录
</div>
```

> **推荐阅读**：[Vue 3 — 模板语法](https://cn.vuejs.org/guide/essentials/template-syntax.html)

---

### 1.5 Composable 模式

Vue 的 composable 是 `useXxx()` 函数，封装可复用的响应式逻辑。类似 Rust 的模块化，但管理的是响应式状态。

**设计模式**：把状态定义在函数外部（模块级），函数返回状态的 readonly 视图 + 操作方法。多处调用 `useXxx()` 共享同一份状态（单例）。

本项目的 composable 一览：

| Composable | 职责 | 文件 |
|------------|------|------|
| `useEngine` | 游戏生命周期：启动、游戏循环(tick)、点击、选择、存档 | `composables/useEngine.ts` |
| `useAssets` | 资源 URL 解析：将逻辑路径转为可加载的 URL | `composables/useAssets.ts` |
| `useSettings` | 用户设置：音量、文字速度、全屏等 | `composables/useSettings.ts` |
| `useNavigation` | 页面导航：标题画面 ↔ 游戏 ↔ 存档 ↔ 设置 ↔ 历史 | `composables/useNavigation.ts` |
| `useLogger` | 日志系统：统一输出到浏览器控制台和 Rust tracing | `composables/useLogger.ts` |
| `useBackend` | 后端通信：Tauri IPC / HTTP API 统一入口 | `composables/useBackend.ts` |

**典型模式**（以 `useNavigation` 为例）：

```typescript
// 模块级状态 — 所有调用者共享
const currentScreen = ref<Screen>("title");
const screenStack = ref<Screen[]>([]);

// 导出函数 — 返回只读状态 + 操作方法
export function useNavigation() {
  function navigateTo(screen: Screen) {
    screenStack.value.push(currentScreen.value);
    currentScreen.value = screen;
  }
  function goBack() {
    const prev = screenStack.value.pop();
    if (prev) currentScreen.value = prev;
  }
  return {
    currentScreen: readonly(currentScreen),  // 外部只读
    navigateTo,                              // 外部可调用来改状态
    goBack,
  };
}
```

> **推荐阅读**：[Vue 3 — 组合式函数](https://cn.vuejs.org/guide/reusability/composables.html)

---

## 阶段 2：理解工具链（0.5 天）

### 2.1 包管理器（pnpm）

pnpm 是 Node.js 的包管理器，类比 Cargo。

| pnpm | Cargo | 说明 |
|------|-------|------|
| `pnpm install` | `cargo build`（下载依赖） | 安装 `package.json` 中声明的依赖 |
| `package.json` | `Cargo.toml` | 声明项目元数据、依赖、脚本 |
| `pnpm-lock.yaml` | `Cargo.lock` | 锁定依赖精确版本 |
| `node_modules/` | `~/.cargo/registry/` | 依赖存放目录（不提交 git） |
| `pnpm dev` | `cargo run` | 启动开发服务器 |
| `pnpm build` | `cargo build --release` | 生产构建 |

本项目 `package.json` 中的核心依赖：

```
dependencies:
  @tauri-apps/api    — Tauri 前端 SDK（IPC、文件系统等）
  vue                — UI 框架

devDependencies:
  vite               — 构建工具
  typescript          — 类型检查器
  @vitejs/plugin-vue  — Vite 的 Vue 支持插件
  vue-tsc            — Vue 文件的 TypeScript 检查
```

---

### 2.2 构建工具（Vite）

Vite 负责开发时的模块热替换（HMR）和生产构建时的打包优化。

- **开发模式**：`pnpm dev` 启动开发服务器（默认 `localhost:5173`），修改 `.vue` / `.ts` 文件后浏览器自动更新，不丢失组件状态。这就是 HMR。
- **构建模式**：`pnpm build` 将所有源码打包为优化后的静态文件（JS/CSS/HTML），供 Tauri 嵌入。

**配置文件**：`host-tauri/vite.config.ts`

```typescript
export default defineConfig({
  plugins: [vue()],         // 启用 Vue SFC 支持
  server: {
    port: 5173,             // 开发服务器端口
    strictPort: true,       // 端口被占用时报错而非自动换
  },
});
```

> **推荐阅读**：[Vite 官方文档 — 开始](https://cn.vite.dev/guide/)（只需看"为什么选 Vite"一节理解原理即可）

---

### 2.3 TypeScript 编译器

- `npx vue-tsc --noEmit`：类型检查（不产出文件），等价于 `cargo check`
- `tsconfig.json`：编译配置

本项目 `tsconfig.json` 中的关键配置：

```json
{
  "compilerOptions": {
    "strict": true,                    // 严格模式（类似 Rust 默认的严格检查）
    "noUnusedLocals": true,            // 禁止未使用的局部变量
    "noUnusedParameters": true,        // 禁止未使用的参数
    "noFallthroughCasesInSwitch": true // switch 必须有 break
  }
}
```

---

### 2.4 Tauri IPC 模型

本项目的前后端通信架构：

```
┌──────────────────────────────────────────────────────────────────┐
│  前端 (TypeScript / Vue)                                         │
│                                                                  │
│  callBackend("tick", { dt })                                     │
│       │                                                          │
│       ▼                                                          │
│  useBackend.ts  ──→  Tauri: invoke()  ──→  Tauri IPC            │
│                  └→  浏览器: fetch() ──→  HTTP (debug_server)   │
│                                                                  │
├──────────────── JSON 序列化 / 反序列化 ──────────────────────────┤
│                                                                  │
│  后端 (Rust)                                                     │
│                                                                  │
│  #[command]                                                      │
│  pub fn tick(state: State<AppState>, dt: f32)                    │
│      -> Result<RenderState, String>                              │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

**核心规则**：前端所有后端调用**必须**通过 `callBackend()`（`composables/useBackend.ts`），不得直接 `import { invoke }`。

**`callBackend` 的双模式设计**：

```typescript
export async function callBackend<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (isTauri()) {
    return invoke<T>(command, args);       // Tauri WebView: 走 IPC
  }
  // 浏览器调试模式: 走 HTTP API
  const resp = await fetch(`${DEBUG_API_BASE}/api/${command}`, { ... });
  return JSON.parse(text);
}
```

这个设计使得前端可以在普通浏览器中运行（连接 debug HTTP 服务器），方便独立调试 UI，不必每次都启动完整的 Tauri 应用。

**类型自动同步**：
- Rust 侧的 `RenderState` 通过 `#[derive(Serialize)]` 序列化为 JSON
- TS 侧的 `RenderState` 接口手动保持同步（字段名使用 `snake_case` 与 serde 一致）
- 文件：Rust `src-tauri/src/render_state.rs` ↔ TS `src/types/render-state.ts`

---

## 阶段 3：能做基本修改（按需查阅）

### 3.1 CSS 基础

本项目最常用的 CSS 模式：

**Flexbox 布局**（几乎所有组件都在用）：

```css
.container {
  display: flex;           /* 启用 flex 布局 */
  flex-direction: column;  /* 纵向排列（默认 row 横向） */
  align-items: center;     /* 交叉轴居中 */
  justify-content: center; /* 主轴居中 */
  gap: 8px;                /* 子元素间距 */
}
```

**绝对定位**（遮罩层、弹窗常用）：

```css
.overlay {
  position: absolute;  /* 相对于最近的 position 非 static 的祖先定位 */
  inset: 0;            /* top/right/bottom/left 全部设为 0（铺满） */
  z-index: 200;        /* 层叠顺序，数值大的在上面 */
}
```

**本项目的 CSS 特点**：
- 大量使用 `rgba()` 半透明色和 `backdrop-filter: blur()` 实现毛玻璃效果
- 使用 `clamp()` 做响应式字号：`font-size: clamp(14px, 1.5vw, 20px)`
- 使用 CSS 变量：`var(--vn-font-body)` 统一字体
- `vh` / `vw` 单位表示视口百分比（viewport height / width）

> **推荐阅读**：
> - [MDN — Flexbox](https://developer.mozilla.org/zh-CN/docs/Learn/CSS/CSS_layout/Flexbox)
> - [MDN — 定位](https://developer.mozilla.org/zh-CN/docs/Learn/CSS/CSS_layout/Positioning)

---

### 3.2 添加新的 IPC 调用

四步流程（以添加一个假想的 `get_statistics` 命令为例）：

**步骤 1**：Rust — 在 `host-tauri/src-tauri/src/commands.rs` 添加命令函数

```rust
#[command]
pub fn get_statistics(state: State<AppState>) -> Result<Statistics, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    // ...
}
```

**步骤 2**：Rust — 在 `host-tauri/src-tauri/src/lib.rs` 注册命令

在 `invoke_handler` 的命令列表中添加 `get_statistics`。

**步骤 3**：Rust — 在 `host-tauri/src-tauri/src/debug_server.rs` 添加 HTTP dispatch 分支

使浏览器调试模式也能调用该命令。

**步骤 4**：TypeScript — 直接调用

```typescript
const stats = await callBackend<Statistics>("get_statistics");
```

无需额外注册——`callBackend` 根据命令名字符串动态分发。

---

### 3.3 添加新的 Vue 组件

**Props（父 → 子数据）和 Emits（子 → 父事件）**

Props 和 Emits 是 Vue 组件间通信的核心机制。类比 Rust：

- Props ≈ 函数参数（只读输入）
- Emits ≈ 回调函数 / channel sender（向上通知）

**对照文件**：`host-tauri/src/vn/ChoicePanel.vue`

```typescript
// 定义 Props — 父组件传入的数据（只读）
defineProps<{
  choices: ChoicesState | null;
}>();

// 定义 Emits — 本组件可以触发的事件
const emit = defineEmits<{
  choose: [index: number];   // 事件名: 参数类型
}>();
```

父组件中使用：

```html
<!-- :choices 绑定 prop，@choose 监听事件 -->
<ChoicePanel
  :choices="renderState.choices"
  @choose="handleChoose"
/>
```

**创建新组件的步骤**：

1. 创建 `host-tauri/src/components/MyWidget.vue`（或 `vn/` / `screens/` 视场景而定）
2. 编写 `<script setup>` + `<template>` + `<style scoped>` 三段
3. 在父组件中 `import MyWidget from "./components/MyWidget.vue"` 后在 `<template>` 中使用

**本项目的目录规范**：
| 目录 | 放什么 | 示例 |
|------|--------|------|
| `vn/` | VN 渲染组件（游戏画面相关） | `DialogueBox.vue`、`BackgroundLayer.vue` |
| `screens/` | 系统画面（全屏页面） | `TitleScreen.vue`、`SettingsScreen.vue` |
| `components/` | 通用 UI 组件 | `Toast.vue`、`ConfirmDialog.vue` |

---

## 附录：项目前端架构速览

```
host-tauri/src/
├── main.ts                  ← 入口：创建 Vue 应用、挂载根组件
├── App.vue                  ← 根组件：页面路由、全局事件、键盘快捷键
├── types/
│   └── render-state.ts      ← TypeScript 类型定义（镜像 Rust RenderState）
├── composables/             ← 可复用的响应式逻辑
│   ├── useBackend.ts        ← 后端通信（唯一允许 import invoke 的地方）
│   ├── useEngine.ts         ← 游戏循环 + 状态管理
│   ├── useAssets.ts         ← 资源 URL 解析
│   ├── useNavigation.ts     ← 页面导航状态机
│   ├── useSettings.ts       ← 用户设置持久化
│   └── useLogger.ts         ← 统一日志
├── vn/                      ← VN 场景渲染组件
│   ├── VNScene.vue          ← 游戏场景容器
│   ├── BackgroundLayer.vue  ← 背景图层
│   ├── CharacterLayer.vue   ← 角色立绘
│   ├── DialogueBox.vue      ← 对话框
│   ├── ChoicePanel.vue      ← 选项面板
│   └── ...
├── screens/                 ← 系统画面
│   ├── TitleScreen.vue      ← 标题画面
│   ├── SaveLoadScreen.vue   ← 存档/读档
│   ├── SettingsScreen.vue   ← 设置
│   ├── HistoryScreen.vue    ← 历史记录
│   └── InGameMenu.vue       ← 游戏内菜单
└── components/              ← 通用 UI
    ├── Toast.vue            ← 提示消息
    ├── ConfirmDialog.vue    ← 确认对话框
    └── SkipAutoIndicator.vue ← 跳过/自动指示器
```

**核心数据流**：

```
Rust 后端 (vn-runtime)
    │
    │  tick() 每帧返回 RenderState (JSON)
    ▼
useEngine.ts
    │
    │  renderState.value = state  (ref 更新)
    ▼
Vue 响应式系统自动触发 UI 更新
    │
    ▼
VNScene.vue → BackgroundLayer / CharacterLayer / DialogueBox / ...
```

前端是纯**渲染层**——不持有游戏逻辑状态，一切由 Rust 后端驱动。
