<script setup lang="ts">
import { computed } from "vue";
import { useScreens } from "../composables/useScreens";

const emit = defineEmits<{
  action: [action: string];
}>();

const { screens, isButtonVisible, actionId } = useScreens();

const buttons = computed(() => {
  if (!screens.value) return fallbackButtons;
  return screens.value.quick_menu.buttons.filter(isButtonVisible);
});

const fallbackButtons = [
  { label: "历史", action: "navigate_history" as string | { start_at_label: string } },
  { label: "快进", action: "toggle_skip" as string | { start_at_label: string } },
  { label: "自动", action: "toggle_auto" as string | { start_at_label: string } },
  { label: "保存", action: "open_save" as string | { start_at_label: string } },
  { label: "快存", action: "quick_save" as string | { start_at_label: string } },
  { label: "快读", action: "quick_load" as string | { start_at_label: string } },
  { label: "设置", action: "navigate_settings" as string | { start_at_label: string } },
];
</script>

<template>
  <div class="quick-menu">
    <button
      v-for="(btn, i) in buttons"
      :key="i"
      class="qm-btn"
      @click.stop="emit('action', actionId(btn.action))"
    >
      {{ btn.label }}
    </button>
  </div>
</template>

<style scoped>
.quick-menu {
  position: absolute;
  bottom: 1vh;
  left: 50%;
  transform: translateX(-50%);
  z-index: 101;
  display: flex;
  gap: 2px;
}

.qm-btn {
  padding: 4px 14px;
  background: rgba(0, 0, 0, 0.3);
  border: none;
  border-radius: 3px;
  color: var(--vn-color-idle, #888);
  font-family: var(--vn-font-body);
  font-size: clamp(11px, 0.75vw, 14px);
  cursor: pointer;
  transition: color 0.2s;
  white-space: nowrap;
}

.qm-btn:hover {
  color: var(--vn-color-hover, #ff9900);
}
</style>
