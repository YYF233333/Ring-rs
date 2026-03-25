import { readonly, ref } from "vue";
import type { UiAssets, UiAssetsAndColors, UiColors } from "../types/theme";
import { useAssets } from "./useAssets";
import { callBackend } from "./useBackend";
import { createLogger } from "./useLogger";

const log = createLogger("theme");

const assets = ref<UiAssets | null>(null);
const colors = ref<UiColors | null>(null);
const ready = ref(false);

const CSS_VAR_MAP: Record<keyof UiColors, string> = {
  accent: "--vn-color-accent",
  idle: "--vn-color-idle",
  hover: "--vn-color-hover",
  selected: "--vn-color-selected",
  insensitive: "--vn-color-insensitive",
  text: "--vn-color-text",
  interface_text: "--vn-color-ui-text",
};

function applyCssColors(c: UiColors) {
  const root = document.documentElement;
  for (const [key, varName] of Object.entries(CSS_VAR_MAP)) {
    const val = c[key];
    if (val) root.style.setProperty(varName, val);
  }
}

async function tryLoadCustomTheme() {
  const { assetUrl } = useAssets();
  const themeUrl = assetUrl("gui/theme.css");
  if (!themeUrl) return;
  try {
    const resp = await fetch(themeUrl, { method: "HEAD" });
    if (!resp.ok) return;
    const link = document.createElement("link");
    link.rel = "stylesheet";
    link.href = themeUrl;
    document.head.appendChild(link);
    log.info("custom theme.css loaded");
  } catch {
    // theme.css is optional
  }
}

export function useTheme() {
  async function init() {
    try {
      const data = await callBackend<UiAssetsAndColors>("get_ui_assets");
      assets.value = data.assets;
      colors.value = data.colors;
      applyCssColors(data.colors);
      ready.value = true;
      log.info("theme initialized");
      await tryLoadCustomTheme();
    } catch (e) {
      log.warn("theme config unavailable", e);
      ready.value = true;
    }
  }

  function asset(key: string): string | undefined {
    if (!assets.value) return undefined;
    const logicalPath = assets.value[key];
    if (!logicalPath) return undefined;
    const { assetUrl } = useAssets();
    return assetUrl(logicalPath);
  }

  return {
    assets: readonly(assets),
    colors: readonly(colors),
    ready: readonly(ready),
    init,
    asset,
  };
}
