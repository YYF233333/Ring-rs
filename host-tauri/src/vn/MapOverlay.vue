<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { useAssets } from "../composables/useAssets";
import { createLogger } from "../composables/useLogger";
import type { UiModeRequest } from "../types/render-state";

const log = createLogger("map-overlay");

interface MapLocation {
  id: string;
  label: string;
  mask_color?: string;
  x: number;
  y: number;
  enabled: boolean;
  condition?: string;
}

interface MapDefinition {
  title: string;
  background?: string;
  hit_mask?: string;
  locations: MapLocation[];
}

const props = defineProps<{
  request: UiModeRequest;
}>();

const emit = defineEmits<{
  complete: [value: string];
  cancel: [];
}>();

const { assetUrl } = useAssets();

const mapDef = ref<MapDefinition | null>(null);
const hoveredId = ref<string | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

async function loadMapDefinition() {
  const mapId = props.request.params.map_id as string | undefined;
  if (!mapId) {
    error.value = "缺少 map_id 参数";
    loading.value = false;
    return;
  }

  try {
    const url = assetUrl(`maps/${mapId}.json`);
    if (!url) {
      error.value = `无法解析地图路径: maps/${mapId}.json`;
      loading.value = false;
      return;
    }
    const resp = await fetch(url);
    if (!resp.ok) {
      error.value = `地图文件加载失败: ${resp.status}`;
      loading.value = false;
      return;
    }
    mapDef.value = await resp.json();
    log.info(`地图 "${mapId}" 加载成功，${mapDef.value?.locations.length} 个位置`);
  } catch (e) {
    error.value = `地图加载错误: ${e}`;
    log.error("地图加载失败", e);
  } finally {
    loading.value = false;
  }
}

function selectLocation(loc: MapLocation) {
  if (!loc.enabled) return;
  log.info(`选择位置: ${loc.id}`);
  emit("complete", loc.id);
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("cancel");
  }
}

onMounted(() => {
  loadMapDefinition();
  window.addEventListener("keydown", onKeyDown, { capture: true });
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeyDown, { capture: true });
});
</script>

<template>
  <div class="map-overlay" @click.stop>
    <div v-if="loading" class="map-loading">加载中...</div>
    <div v-else-if="error" class="map-error">{{ error }}</div>
    <template v-else-if="mapDef">
      <img
        v-if="mapDef.background"
        :src="assetUrl(mapDef.background)"
        class="map-background"
        alt="地图背景"
      />
      <div v-else class="map-background map-background--fallback" />

      <div class="map-title">{{ mapDef.title }}</div>

      <button
        v-for="loc in mapDef.locations"
        :key="loc.id"
        class="map-location"
        :class="{
          'map-location--disabled': !loc.enabled,
          'map-location--hovered': hoveredId === loc.id,
        }"
        :style="{
          left: `${(loc.x / 1920) * 100}%`,
          top: `${(loc.y / 1080) * 100}%`,
        }"
        :disabled="!loc.enabled"
        @click="selectLocation(loc)"
        @mouseenter="hoveredId = loc.id"
        @mouseleave="hoveredId = null"
      >
        {{ loc.label }}
      </button>

      <button class="map-cancel" @click="emit('cancel')">返回</button>
    </template>
  </div>
</template>

<style scoped>
.map-overlay {
  position: absolute;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: center;
  justify-content: center;
}

.map-loading,
.map-error {
  color: #e0e0e0;
  font-size: 1.5em;
  font-family: var(--vn-font-body);
}

.map-error {
  color: #ff6b6b;
}

.map-background {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.map-background--fallback {
  background: rgba(0, 0, 0, 0.85);
}

.map-title {
  position: absolute;
  top: 3%;
  left: 50%;
  transform: translateX(-50%);
  color: #fff;
  font-size: clamp(1.2rem, 3vw, 2.2rem);
  font-family: var(--vn-font-display, var(--vn-font-body));
  text-shadow: 0 2px 8px rgba(0, 0, 0, 0.6);
  z-index: 1;
  pointer-events: none;
}

.map-location {
  position: absolute;
  transform: translate(-50%, -50%);
  z-index: 2;
  min-width: clamp(100px, 10vw, 200px);
  padding: 0.5em 1.2em;
  border: 1.5px solid rgba(100, 149, 237, 0.8);
  border-radius: 8px;
  background: rgba(40, 40, 60, 0.85);
  color: #ccc;
  font-size: clamp(0.85rem, 1.5vw, 1.2rem);
  font-family: var(--vn-font-body);
  cursor: pointer;
  transition: all 0.2s var(--vn-ease-ui);
  text-align: center;
}

.map-location:hover:not(:disabled),
.map-location--hovered:not(:disabled) {
  background: rgba(80, 80, 120, 0.9);
  color: #fff;
  border-color: rgba(130, 170, 255, 1);
  box-shadow: 0 0 12px rgba(100, 149, 237, 0.3);
}

.map-location--disabled {
  background: rgba(60, 60, 60, 0.7);
  color: #666;
  border-color: rgba(80, 80, 80, 0.5);
  cursor: not-allowed;
}

.map-cancel {
  position: absolute;
  bottom: 3%;
  right: 3%;
  z-index: 2;
  padding: 0.5em 1.5em;
  border: 1px solid rgba(200, 200, 200, 0.3);
  border-radius: 6px;
  background: rgba(40, 40, 40, 0.8);
  color: #aaa;
  font-size: clamp(0.8rem, 1.2vw, 1rem);
  font-family: var(--vn-font-body);
  cursor: pointer;
  transition: all 0.2s var(--vn-ease-ui);
}

.map-cancel:hover {
  background: rgba(60, 60, 60, 0.9);
  color: #fff;
}
</style>
