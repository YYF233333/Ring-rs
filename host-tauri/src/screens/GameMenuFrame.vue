<script setup lang="ts">
import { computed } from "vue";
import { useScreens } from "../composables/useScreens";
import { useTheme } from "../composables/useTheme";

const emit = defineEmits<{
  action: [action: string];
}>();

const props = defineProps<{
  activeNav?: string;
}>();

const { screens, isButtonVisible, resolveConditionalAsset, actionId } = useScreens();
const { asset } = useTheme();

const menuDef = computed(() => screens.value?.game_menu);

const backgroundUrl = computed(() => {
  if (!menuDef.value) return undefined;
  const key = resolveConditionalAsset(menuDef.value.background);
  return key ? asset(key) : undefined;
});

const overlayUrl = computed(() => {
  if (!menuDef.value) return undefined;
  return asset(menuDef.value.overlay);
});

const navButtons = computed(() => {
  if (!menuDef.value) return fallbackNav;
  return menuDef.value.nav_buttons.filter(isButtonVisible);
});

const returnButton = computed(() => {
  return menuDef.value?.return_button ?? { label: "返回", action: "return_to_game" };
});

const fallbackNav = [
  { label: "历史", action: "replace_history" as string | { start_at_label: string } },
  { label: "保存", action: "open_save" as string | { start_at_label: string } },
  { label: "读取", action: "open_load" as string | { start_at_label: string } },
  { label: "设置", action: "replace_settings" as string | { start_at_label: string } },
];

function isActive(btn: (typeof fallbackNav)[number]): boolean {
  const id = actionId(btn.action);
  return id === props.activeNav;
}
</script>

<template>
  <div class="game-menu">
    <img
      v-if="backgroundUrl"
      class="gm-bg"
      :src="backgroundUrl"
      alt=""
    />
    <div v-else class="gm-bg gm-bg--fallback" />

    <img
      v-if="overlayUrl"
      class="gm-overlay"
      :src="overlayUrl"
      alt=""
    />

    <div class="gm-layout">
      <nav class="gm-sidebar">
        <div class="gm-nav-list">
          <button
            v-for="(btn, i) in navButtons"
            :key="i"
            class="gm-nav-btn"
            :class="{ active: isActive(btn) }"
            @click="emit('action', actionId(btn.action))"
          >
            {{ btn.label }}
          </button>
        </div>
        <button
          class="gm-return-btn"
          @click="emit('action', actionId(returnButton.action))"
        >
          {{ returnButton.label }}
        </button>
      </nav>

      <main class="gm-content">
        <slot />
      </main>
    </div>
  </div>
</template>

<style scoped>
.game-menu {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.gm-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  z-index: 0;
}

.gm-bg--fallback {
  background: linear-gradient(160deg, #0d0d1a 0%, #1a1a2e 50%, #16213e 100%);
}

.gm-overlay {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  z-index: 1;
  pointer-events: none;
}

.gm-layout {
  position: relative;
  z-index: 2;
  width: 100%;
  height: 100%;
  display: flex;
}

.gm-sidebar {
  width: clamp(200px, 22vw, 380px);
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  padding: 6vh 2vw;
  box-sizing: border-box;
}

.gm-nav-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.gm-nav-btn {
  width: 100%;
  padding: 10px 16px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--vn-color-idle, #888);
  font-family: var(--vn-font-body);
  font-size: clamp(13px, 1.1vw, 18px);
  letter-spacing: 2px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.gm-nav-btn:hover {
  color: var(--vn-color-hover, #ff9900);
  background: rgba(255, 255, 255, 0.06);
}

.gm-nav-btn.active {
  color: var(--vn-color-selected, #fff);
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.1);
}

.gm-return-btn {
  padding: 10px 16px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  color: var(--vn-color-idle, #888);
  font-family: var(--vn-font-body);
  font-size: clamp(12px, 1vw, 16px);
  letter-spacing: 2px;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.gm-return-btn:hover {
  color: var(--vn-color-hover, #ff9900);
  background: rgba(255, 255, 255, 0.08);
}

.gm-content {
  flex: 1;
  height: 100%;
  overflow: hidden;
  padding: 5vh 3vw;
  box-sizing: border-box;
}
</style>
