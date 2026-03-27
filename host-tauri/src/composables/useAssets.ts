import { ref } from "vue";
import { resolveAssetSrc } from "./useBackend";
import { createLogger } from "./useLogger";

const log = createLogger("assets");

const ready = ref(false);

async function init() {
  ready.value = true;
  log.info("assets ready (ring-asset protocol)");
}

/** 规范化路径：解析 `..` 和 `.` 段，统一为正斜杠 */
function normalizePath(p: string): string {
  const parts = p.replace(/\\/g, "/").split("/");
  const resolved: string[] = [];
  for (const seg of parts) {
    if (seg === ".." && resolved.length > 0) {
      resolved.pop();
    } else if (seg !== "." && seg !== "") {
      resolved.push(seg);
    }
  }
  return resolved.join("/");
}

function assetUrl(logicalPath: string | null | undefined): string | undefined {
  if (!logicalPath) return undefined;
  const normalized = normalizePath(logicalPath);
  const url = resolveAssetSrc(normalized);
  log.debug(`assetUrl: ${logicalPath} → ${normalized} → ${url}`);
  return url;
}

export function useAssets() {
  return { init, assetUrl, ready };
}
