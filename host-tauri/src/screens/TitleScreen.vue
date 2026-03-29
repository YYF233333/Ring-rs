<script setup lang="ts">
import { computed } from "vue";
import { useScreens } from "../composables/useScreens";
import { useTheme } from "../composables/useTheme";

const emit = defineEmits<{
  action: [action: string];
}>();

const { screens, isButtonVisible, resolveConditionalAsset, actionId } = useScreens();
const { asset } = useTheme();

const titleDef = computed(() => screens.value?.title);

const backgroundUrl = computed(() => {
  if (!titleDef.value) return undefined;
  const key = resolveConditionalAsset(titleDef.value.background);
  return key ? asset(key) : undefined;
});

const overlayUrl = computed(() => {
  if (!titleDef.value) return undefined;
  return asset(titleDef.value.overlay);
});

const visibleButtons = computed(() => {
  if (!titleDef.value) return fallbackButtons;
  return titleDef.value.buttons.filter(isButtonVisible);
});

const fallbackButtons = [
  { label: "开始游戏", action: "start_game" as string | { start_at_label: string } },
  { label: "继续游戏", action: "continue_game" as string | { start_at_label: string } },
  { label: "读取存档", action: "open_load" as string | { start_at_label: string } },
  { label: "设置", action: "navigate_settings" as string | { start_at_label: string } },
  { label: "退出", action: "exit" as string | { start_at_label: string } },
];

function onButtonClick(btn: (typeof fallbackButtons)[number]) {
  emit("action", actionId(btn.action));
}
</script>

<template>
  <div class="title-screen">
    <img
      v-if="backgroundUrl"
      class="title-bg"
      :src="backgroundUrl"
      alt=""
    />
    <div v-else class="title-bg title-bg--fallback" />

    <img
      v-if="overlayUrl"
      class="title-overlay"
      :src="overlayUrl"
      alt=""
    />

    <div class="title-content">
      <nav class="menu-list">
        <button
          v-for="(btn, i) in visibleButtons"
          :key="i"
          class="menu-btn"
          @click="onButtonClick(btn)"
        >
          {{ btn.label }}
        </button>
      </nav>
    </div>

    <div class="version-tag">v0.1.0</div>
  </div>
</template>

<style scoped>
.title-screen {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.title-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  z-index: 0;
}

.title-bg--fallback {
  background: linear-gradient(160deg, #0d0d1a 0%, #1a1a2e 50%, #16213e 100%);
}

.title-overlay {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  z-index: 1;
  pointer-events: none;
}

.title-content {
  position: relative;
  z-index: 2;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  padding-left: 5vw;
}

.menu-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.menu-btn {
  width: clamp(180px, 14vw, 260px);
  padding: 10px 0;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  color: var(--vn-color-ui-text, #c0c0c0);
  font-family: var(--vn-font-body);
  font-size: clamp(13px, 1.1vw, 18px);
  letter-spacing: 2px;
  cursor: pointer;
  transition: all 0.25s ease;
  text-align: center;
  backdrop-filter: blur(4px);
}

.menu-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  border-color: var(--vn-color-hover, rgba(255, 153, 0, 0.4));
  color: var(--vn-color-hover, #ff9900);
  transform: translateX(4px);
}

.menu-btn:active {
  transform: translateX(2px);
  background: rgba(255, 255, 255, 0.18);
}

.version-tag {
  position: absolute;
  bottom: 16px;
  right: 20px;
  z-index: 2;
  font-family: var(--vn-font-body);
  font-size: 11px;
  color: rgba(255, 255, 255, 0.2);
}
</style>
