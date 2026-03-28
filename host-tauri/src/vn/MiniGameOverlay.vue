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

const gameId = props.request.params.game_id as string | undefined;
const iframeSrc = gameId ? assetUrl(`games/${gameId}/index.html`) : undefined;

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

  const assetBase = assetUrl(`games/${gameId}`) ?? "";
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
  window.addEventListener("message", onMessage);
});

onUnmounted(() => {
  window.removeEventListener("message", onMessage);
});
</script>

<template>
  <div class="minigame-overlay" @click.stop>
    <div v-if="!iframeSrc" class="minigame-error">
      缺少 game_id 参数
    </div>
    <template v-else>
      <div v-if="loading" class="minigame-loading">加载中...</div>
      <iframe
        ref="iframeRef"
        :src="iframeSrc"
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
