import { callBackend } from "./useBackend";

type LogLevel = "debug" | "info" | "warn" | "error";

const consoleFns: Record<LogLevel, (...args: unknown[]) => void> = {
  debug: console.debug,
  info: console.info,
  warn: console.warn,
  error: console.error,
};

/**
 * 创建一个模块级 logger。
 * 所有级别同时输出到 browser console 和 Rust tracing（通过 IPC/HTTP 转发）。
 */
export function createLogger(module: string) {
  function log(level: LogLevel, message: string, data?: unknown) {
    const dataStr = data !== undefined ? JSON.stringify(data) : null;

    consoleFns[level](`[${module}]`, message, ...(data !== undefined ? [data] : []));

    callBackend("log_frontend", { level, module, message, data: dataStr }).catch(() => {});
  }

  return {
    debug: (msg: string, data?: unknown) => log("debug", msg, data),
    info: (msg: string, data?: unknown) => log("info", msg, data),
    warn: (msg: string, data?: unknown) => log("warn", msg, data),
    error: (msg: string, data?: unknown) => log("error", msg, data),
  };
}
