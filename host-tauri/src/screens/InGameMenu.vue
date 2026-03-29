<script setup lang="ts">
import { computed } from "vue";
import { useScreens } from "../composables/useScreens";

const emit = defineEmits<{
  action: [action: string];
}>();

const { screens, isButtonVisible, actionId } = useScreens();

const buttons = computed(() => {
  if (!screens.value) return fallbackButtons;
  return screens.value.ingame_menu.buttons.filter(isButtonVisible);
});

const fallbackButtons = [
  { label: "继续", action: "go_back" as string | { start_at_label: string } },
  { label: "保存", action: "open_save" as string | { start_at_label: string } },
  { label: "读取", action: "open_load" as string | { start_at_label: string } },
  { label: "设置", action: "navigate_settings" as string | { start_at_label: string } },
  { label: "历史", action: "navigate_history" as string | { start_at_label: string } },
  { label: "返回标题", action: "return_to_title" as string | { start_at_label: string } },
  { label: "退出", action: "exit" as string | { start_at_label: string } },
];

function onButtonClick(btn: (typeof fallbackButtons)[number]) {
  emit("action", actionId(btn.action));
}
</script>

<template>
  <div class="ingame-menu-overlay" @click.self="emit('action', 'go_back')">
    <nav class="ingame-menu">
      <button
        v-for="(btn, i) in buttons"
        :key="i"
        class="igm-btn"
        @click="onButtonClick(btn)"
      >
        {{ btn.label }}
      </button>
    </nav>
  </div>
</template>

<style scoped>
.ingame-menu-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 500;
  backdrop-filter: blur(6px);
}

.ingame-menu {
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
}

.igm-btn {
  width: clamp(180px, 14vw, 260px);
  padding: 10px 0;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  color: var(--vn-color-ui-text, #c0c0c0);
  font-family: var(--vn-font-body);
  font-size: clamp(13px, 1.1vw, 18px);
  letter-spacing: 2px;
  cursor: pointer;
  transition: all 0.2s ease;
  backdrop-filter: blur(4px);
}

.igm-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: var(--vn-color-hover, rgba(255, 153, 0, 0.4));
  color: var(--vn-color-hover, #ff9900);
}
</style>
