# Tauri 开发指南

面向新接触本项目 Tauri 宿主的开发者和 Agent。

## 环境准备

| 依赖 | 最低版本 | 备注 |
|------|---------|------|
| Node.js | >= 18 | 推荐用 `nvm` 管理 |
| pnpm | 最新 | `corepack enable && corepack prepare pnpm@latest --activate` |
| Rust toolchain | stable | 通过 [rustup](https://rustup.rs/) 安装 |
| Tauri CLI | — | 已声明在 `devDependencies`，通过 `pnpm tauri` 调用，无需全局安装 |

首次克隆后在 `host-tauri/` 下执行 `pnpm install` 安装前端依赖。

## 项目结构

```
host-tauri/
├── src/                     # Vue 3 前端（TypeScript）
│   ├── composables/         #   Composition API 封装（useEngine, useBackend…）
│   ├── components/          #   通用 UI 组件
│   ├── screens/             #   系统画面（标题、存档、设置…）
│   ├── vn/                  #   VN 渲染组件（对话框、立绘、背景…）
│   ├── types/               #   TypeScript 类型定义（与 Rust 同步）
│   ├── App.vue              #   根组件
│   └── main.ts              #   入口
├── src-tauri/               # Rust 后端
│   ├── src/
│   │   ├── lib.rs           #   应用启动 & invoke_handler 注册
│   │   ├── commands.rs      #   IPC 命令薄代理
│   │   ├── state.rs         #   核心业务逻辑（AppStateInner）
│   │   ├── render_state.rs  #   RenderState 定义（前端渲染数据）
│   │   ├── command_executor.rs  #   VN Command 执行器
│   │   ├── debug_server.rs  #   Debug HTTP 服务器（仅 debug build）
│   │   ├── audio.rs         #   音频管理
│   │   ├── resources.rs     #   资源管理
│   │   └── ...
│   └── Cargo.toml           #   后端依赖
├── package.json             # 前端依赖 & scripts
├── vite.config.ts           # Vite 构建配置
├── tsconfig.json            # TypeScript 配置
└── index.html               # SPA 入口 HTML
```

## 开发流程

### 启动

```bash
cd host-tauri; pnpm tauri dev
```

该命令同时启动三个进程：
1. **Vite dev server**（默认 `localhost:5173`）— 前端热更新
2. **Rust 后端编译 & 运行** — Cargo build + 启动原生窗口
3. **Tauri WebView 窗口** — 加载 Vite dev server 页面

### 热更新

| 修改内容 | 行为 |
|---------|------|
| `.vue` / `.ts` 文件 | Vite HMR 自动热更新，无需重启 |
| Rust 源码 | Tauri CLI 检测变更 → 自动重新编译 → 重启后端窗口 |

### 类型检查

```bash
cd host-tauri; npx vue-tsc --noEmit
```

## IPC 约定

前端与后端通过 IPC 通信。所有后端调用必须经过 `callBackend()`：

```typescript
import { callBackend } from "@/composables/useBackend";

const state = await callBackend<RenderState>("tick", { dt: 0.016 });
```

`callBackend` 自动选择通道：
- **Tauri WebView 内** → `@tauri-apps/api/core` 的 `invoke()`
- **普通浏览器** → HTTP POST 到 `http://localhost:9528/api/{command}`（回退到 Debug Server）

### 新增 IPC 命令

需要修改 **3 个文件**：

1. **`src-tauri/src/commands.rs`** — 添加 `#[command]` 函数（薄代理：lock → 调方法 → 返回）：
   ```rust
   #[command]
   pub fn my_command(state: State<AppState>, arg1: String) -> Result<MyResult, String> {
       let inner = state.inner.lock().map_err(|e| e.to_string())?;
       inner.do_something(&arg1).map_err(|e| e.to_string())
   }
   ```

2. **`src-tauri/src/lib.rs`** — 在 `invoke_handler` 宏中注册：
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... 已有命令 ...
       commands::my_command,
   ])
   ```

3. **`src-tauri/src/debug_server.rs`** — 在 `dispatch` 函数的 match 中添加分支，使浏览器调试可用：
   ```rust
   "my_command" => {
       let arg1 = args["arg1"].as_str().unwrap_or_default().to_string();
       let inner = state.lock().map_err(|e| e.to_string())?;
       let result = inner.do_something(&arg1).map_err(|e| e.to_string())?;
       Ok(serde_json::to_value(result).unwrap())
   }
   ```

> **注意**：`commands.rs` 只做薄代理，业务逻辑实现在 `state.rs` 的 `AppStateInner` 方法中。

## Debug HTTP Server

debug build 下（`cfg(debug_assertions)`），应用启动时自动在独立线程启动 HTTP 服务器：

```
http://127.0.0.1:9528
```

这使得前端可以在普通浏览器中运行并与后端通信，方便使用浏览器 DevTools 调试。

### 使用方式

1. 正常启动 `pnpm tauri dev`
2. 在浏览器打开 `http://localhost:5173`
3. 前端自动检测到非 Tauri 环境，`callBackend` 回退到 HTTP API

### 端点

| 路径 | 方法 | 说明 |
|------|------|------|
| `/api/{command}` | POST | 命令端点，与 IPC 命令一一对应 |
| `/assets/{path}` | GET | 静态资源代理（映射到 `assets_root`） |

## 常见问题

### 端口 5173 被占用

Vite 会自动递增端口（5174, 5175…），但 Tauri WebView 可能仍连接旧端口。解决：关闭占用进程后重启 `pnpm tauri dev`。

### 资源路径找不到

- 确认 `assets/` 目录存在于仓库根目录
- 后端通过向上查找 `assets` 目录来定位项目根，CWD 需在仓库目录树内
- 可在 `config.json` 中配置 `assets_root` 指向自定义路径

### Rust 编译错误但前端正常

Vite dev server 和 Rust 编译是独立进程。Rust 编译失败时前端仍可访问 `localhost:5173`，但后端不可用。查看终端中的 Cargo 编译错误修复后会自动重启。

### TypeScript 类型与 Rust 不同步

`src/types/render-state.ts` 必须与 `src-tauri/src/render_state.rs` 手工保持同步。字段名使用 `snake_case`（与 Rust serde 序列化一致）。修改 `RenderState` 后务必同步更新两侧定义。
