<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useAssets } from "../composables/useAssets";
import { createLogger } from "../composables/useLogger";
import type { UiModeRequest } from "../types/render-state";

const log = createLogger("minigame");

const props = defineProps<{
  request: UiModeRequest;
}>();

const emit = defineEmits<{
  complete: [value: unknown];
}>();

const { assetUrl } = useAssets();
const iframeRef = ref<HTMLIFrameElement | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const iframeDocument = ref<string | null>(null);

const gameId = props.request.params.game_id as string | undefined;
const gameIndexUrl = gameId ? assetUrl(`games/${gameId}/index.html`) : undefined;
const gameBaseUrl = gameId ? assetUrl(`games/${gameId}`) : undefined;

function injectBaseHref(html: string, baseHref: string): string {
  const baseTag = `<base href="${baseHref.endsWith("/") ? baseHref : `${baseHref}/`}">`;
  if (/<base\b[^>]*>/i.test(html)) {
    return html.replace(/<base\b[^>]*>/i, baseTag);
  }
  if (/<head[^>]*>/i.test(html)) {
    return html.replace(/<head([^>]*)>/i, `<head$1>${baseTag}`);
  }
  return `${baseTag}${html}`;
}

async function loadGameDocument() {
  if (!gameIndexUrl || !gameBaseUrl) {
    error.value = "缺少 game_id 参数";
    loading.value = false;
    return;
  }

  loading.value = true;
  error.value = null;
  iframeDocument.value = null;

  try {
    const resp = await fetch(gameIndexUrl);
    if (!resp.ok) {
      error.value = `小游戏页面加载失败: ${resp.status}`;
      loading.value = false;
      return;
    }

    const html = await resp.text();
    iframeDocument.value = injectBaseHref(html, gameBaseUrl);
  } catch (e) {
    error.value = `小游戏页面加载错误: ${e}`;
    loading.value = false;
    log.error("小游戏页面加载失败", e);
  }
}

function onMessage(e: MessageEvent) {
  if (e.source !== iframeRef.value?.contentWindow) return;

  const data = e.data;
  if (!data || typeof data !== "object") return;

  if (data.type === "engine:complete") {
    log.info(`小游戏完成: ${JSON.stringify(data.result)}`);
    emit("complete", data.result ?? "");
  } else if (data.type === "engine:log") {
    log.debug(`[minigame] ${data.message}`);
  }
}

function onIframeLoad() {
  loading.value = false;
  if (!iframeRef.value?.contentWindow) return;

  const assetBase = gameBaseUrl ?? "";
  const plainParams = JSON.parse(JSON.stringify(props.request.params));

  iframeRef.value.contentWindow.postMessage(
    {
      type: "engine:init",
      info: {
        game_id: gameId,
        params: plainParams,
      },
      assetBase,
    },
    "*",
  );
  log.info(`小游戏 init 消息已发送: game_id=${gameId}`);
}

onMounted(() => {
  void loadGameDocument();
  window.addEventListener("message", onMessage);
});

onUnmounted(() => {
  window.removeEventListener("message", onMessage);
});
</script>

<template>
  <div class="minigame-overlay" @click.stop>
    <div v-if="error" class="minigame-error">
      {{ error }}
    </div>
    <template v-else>
      <div v-if="loading" class="minigame-loading">加载中...</div>
      <iframe
        v-if="iframeDocument"
        ref="iframeRef"
        :srcdoc="iframeDocument"
        class="minigame-iframe"
        sandbox="allow-scripts allow-same-origin"
        @load="onIframeLoad"
      />
    </template>
  </div>
</template>

<style scoped>
.minigame-overlay {
  position: absolute;
  inset: 0;
  z-index: 300;
  background: #000;
}

.minigame-iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.minigame-loading {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
  font-size: 1.5em;
  font-family: var(--vn-font-body);
  z-index: 1;
}

.minigame-error {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: #ff6b6b;
  font-size: 1.5em;
  font-family: var(--vn-font-body);
}
</style>
