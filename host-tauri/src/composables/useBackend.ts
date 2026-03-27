import { convertFileSrc, invoke } from "@tauri-apps/api/core";

const DEBUG_API_BASE = "http://localhost:9528";

/** 检测是否在 Tauri WebView 环境中运行（vs 普通浏览器） */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/**
 * 统一后端调用入口。
 * Tauri 模式下走 IPC invoke，浏览器模式下走 HTTP API 回退。
 */
export async function callBackend<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (isTauri()) {
    return invoke<T>(command, args);
  }
  const resp = await fetch(`${DEBUG_API_BASE}/api/${command}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args ?? {}),
  });
  if (!resp.ok) {
    const errText = await resp.text();
    throw new Error(errText);
  }
  const text = await resp.text();
  if (!text) return undefined as T;
  return JSON.parse(text);
}

/**
 * 将逻辑资源路径转换为可加载的 URL。
 *
 * Tauri 模式下使用 `ring-asset` 自定义协议（`convertFileSrc` 自动处理
 * Windows `http://ring-asset.localhost/...` vs macOS/Linux `ring-asset://localhost/...`）。
 * 浏览器调试模式下走 debug HTTP server 的静态文件服务。
 */
export function resolveAssetSrc(logicalPath: string): string {
  if (isTauri()) {
    return convertFileSrc(logicalPath, "ring-asset");
  }
  return `${DEBUG_API_BASE}/assets/${logicalPath}`;
}
