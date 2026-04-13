# Dioxus 0.7 Desktop 知识参考

host-dioxus 使用 Dioxus 0.7 Desktop。Claude 基线知识覆盖到 Dioxus 0.4.x，以下是 0.5+ 的关键变更。

## Component 语法 (0.5+)

| 旧 (0.4) | 新 (0.5+) |
|-----------|------------|
| `fn App(cx: Scope) -> Element` | `fn App() -> Element` |
| `cx.use_hook()` | `use_signal()`, `use_memo()` |
| `use_state`, `use_ref` | `Signal<T>` (always Copy) |
| 每平台单独 launcher | `dioxus::launch(App)` |

## Signals

| 操作 | 语法 |
|------|------|
| 创建 | `let count = use_signal(\|\| 0);` |
| 读取（订阅） | `count.read()` 或 `count()` |
| 读取（不订阅） | `count.peek()` |
| 写入 | `count.write()`, `count.set(5)`, `count += 1` |
| 全局 | `static THEME: GlobalSignal<T> = GlobalSignal::new(\|\| ...)` |
| 映射 | `user.map(\|u\| &u.name)` |

### Stores (0.7+)

```rust
#[derive(Store)]
struct AppState {
    users: BTreeMap<String, User>,
}

#[component]
fn UserList(state: Store<AppState>) -> Element {
    let users = state.users();
    rsx! {
        for (id, user) in users.iter() {
            UserRow { key: "{id}", user }  // 仅变更项重渲染
        }
    }
}
```

## Hooks 速查

| Hook | 用途 |
|------|------|
| `use_signal(\|\| val)` | 可变响应式状态 |
| `use_memo(move \|\| expr)` | 依赖变更时重计算 |
| `use_effect(move \|\| { ... })` | 渲染后副作用 |
| `use_resource(move \|\| async { ... })` | 异步数据（可 restart/cancel） |
| `use_coroutine(\|rx\| async { ... })` | 长生命周期异步任务 + channel |
| `use_callback(move \|\| { ... })` | 记忆化事件处理器 |
| `use_future(move \|\| async { ... })` | 挂载时执行一次 |
| `use_context_provider(\|\| val)` | 提供 Context |
| `use_context::<T>()` | 消费 Context |

## RSX 模式

### 条件渲染

```rust
rsx! {
    if show { Header {} }
    if logged_in { Dashboard {} } else { LoginForm {} }
    match status {
        Status::Loading => rsx! { Spinner {} },
        Status::Ready(data) => rsx! { Content { data } },
    }
    div { class: if active { "active" }, "Content" }
}
```

### 列表

```rust
for item in items.iter() {
    li { key: "{item.id}", "{item.name}" }
}
```

### Props

```rust
#[component]
fn Button(
    label: String,
    #[props(default)] disabled: bool,
    #[props(default = "primary")] variant: &'static str,
    #[props(optional)] icon: Option<String>,
) -> Element { ... }
```

- `children: Element` 参数接收子元素，用 `{children}` 渲染
- `#[props(into)]` 接受 `Into<T>` 转换
- `#[props(extends = GlobalAttributes)]` 继承 HTML 属性

### 属性合并 (0.5+)

```rust
div { class: "base", class: if enabled { "active" } }
// 渲染: class="base active"
```

### 事件

```rust
button { onclick: move |_| count += 1, "Click" }
input { value: "{text}", oninput: move |e| text.set(e.value()) }
// 0.6+: e.prevent_default() 直接调用
```

### Element 引用

```rust
div {
    onmounted: move |data| el.set(Some(data.data())),
}
// el.get_client_rect().await, el.scroll_to(...).await, el.set_focus().await
```

### Element 即 Result (0.6+)

```rust
fn Profile(id: u32) -> Element {
    let user = get_user(id)?;  // 传播到 ErrorBoundary
    rsx! { "{user.name}" }
}
```

### Suspense (0.6+)

```rust
SuspenseBoundary {
    fallback: |_| rsx! { "Loading..." },
    AsyncChild {}
}
fn AsyncChild() -> Element {
    let data = use_resource(fetch_data).suspend()?;
    rsx! { "{data}" }
}
```

### Assets (0.6+)

```rust
const LOGO: Asset = asset!("/assets/logo.png");
const STYLES: Asset = asset!("/app.css", CssAssetOptions::new().minify(true));
rsx! { img { src: LOGO } }
```

### CSS Modules (0.7.3+)

```rust
css_module!(Styles = "/styles.module.css", AssetOptions::css_module());
rsx! { div { class: Styles::container } }  // 类型安全，编译期检查
```

## Desktop 配置

```rust
use dioxus::desktop::{Config, WindowBuilder, LogicalSize};

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("My App")
                        .with_inner_size(LogicalSize::new(800, 600))
                )
                .with_disable_context_menu(true)
                .with_background_color((255, 255, 255, 255))
                .with_devtools(cfg!(debug_assertions))
        )
        .launch(App);
}
```

### with_on_window_ready (0.7.1+)

窗口创建前回调，用于 WGPU overlay 等：

```rust
Config::new().with_on_window_ready(|window| { /* ... */ })
```

## CLI

| 命令 | 用途 |
|------|------|
| `dx serve` | 开发服务器 + 热重载 |
| `dx build --release` | 生产构建 |
| `dx bundle` | 打包分发 |
| `dx check` | RSX lint |
| `dx fmt` | RSX 格式化 |

## 热重载边界

**即时重载：** 字面值、文本、格式化段、属性顺序、模板结构

**需要重编译：** Rust 逻辑、组件结构、控制流条件、struct 字段变更

## Reactivity 注意事项

**Memo 在属性中不能正确订阅：**
```rust
// BAD: 不会更新
let style = use_memo(move || format!("color: {}", color()));
rsx! { div { style: style } }

// GOOD: 直接在 RSX 中读 signal
rsx! { div { style: format!("color: {}", color()) } }
```

**用独立 CSS 属性而非 style 字符串：**
```rust
rsx! {
    p {
        font_weight: if bold() { "bold" } else { "normal" },
        text_align: "{align}",
    }
}
```

**thread_local 在热重载时重置——用 Signal/Store 代替。**