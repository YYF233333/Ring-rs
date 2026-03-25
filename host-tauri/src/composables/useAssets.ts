import { ref } from "vue";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

let assetsRoot: string | null = null;
const ready = ref(false);

async function init() {
  if (assetsRoot !== null) return;
  assetsRoot = await invoke<string>("get_assets_root");
  console.debug("[useAssets] assetsRoot =", assetsRoot);
  ready.value = true;
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
  if (!logicalPath || assetsRoot === null) return undefined;
  const normalized = normalizePath(logicalPath);
  const sep = assetsRoot.endsWith("/") || assetsRoot.endsWith("\\") ? "" : "/";
  const fullPath = `${assetsRoot}${sep}${normalized}`;
  const url = convertFileSrc(fullPath);
  console.debug("[useAssets] assetUrl:", logicalPath, "→", normalized, "→", url);
  return url;
}

export function useAssets() {
  return { init, assetUrl, ready };
}
