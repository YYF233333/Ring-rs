<script setup lang="ts">
import { computed, ref } from "vue";
import { useTheme } from "../composables/useTheme";
import type { ChoicesState } from "../types/render-state";

defineProps<{
  choices: ChoicesState | null;
}>();

const emit = defineEmits<{
  choose: [index: number];
}>();

const { asset } = useTheme();
const choiceIdleUrl = computed(() => asset("choice_idle"));
const choiceHoverUrl = computed(() => asset("choice_hover"));
const hoveredIdx = ref<number | null>(null);
</script>

<template>
  <Transition name="choice-fade">
    <div v-if="choices" class="choice-overlay">
      <div class="choice-list">
        <button
          v-for="(item, idx) in choices.choices"
          :key="idx"
          class="choice-button"
          :class="{ hovered: hoveredIdx === idx || choices.hovered_index === idx }"
          :style="choiceIdleUrl ? {
            backgroundImage: (hoveredIdx === idx || choices.hovered_index === idx)
              ? `url(${choiceHoverUrl || choiceIdleUrl})`
              : `url(${choiceIdleUrl})`,
            backgroundSize: '100% 100%',
          } : undefined"
          @mouseenter="hoveredIdx = idx"
          @mouseleave="hoveredIdx = null"
          @click.stop="emit('choose', idx)"
        >
          {{ item.text }}
        </button>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.choice-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.35);
  z-index: 200;
}

.choice-list {
  display: flex;
  flex-direction: column;
  gap: clamp(6px, 1.2vh, 16px);
  width: clamp(400px, 62vw, 1000px);
}

.choice-button {
  padding: clamp(8px, 1.4vh, 18px) 2.5vw;
  background: rgba(20, 20, 40, 0.85);
  border: 1px solid rgba(200, 200, 220, 0.12);
  border-radius: 4px;
  color: var(--vn-color-ui-text, #ddd);
  font-family: var(--vn-font-body);
  font-size: clamp(14px, 1.4vw, 20px);
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: center;
  backdrop-filter: blur(4px);
}

.choice-button:hover,
.choice-button.hovered {
  color: var(--vn-color-hover, #fff);
  border-color: var(--vn-color-hover, rgba(255, 153, 0, 0.3));
}

/* Fallback hover effect when no choice images */
.choice-button:not([style]):hover,
.choice-button:not([style]).hovered {
  background: rgba(60, 60, 100, 0.9);
  transform: scale(1.01);
}

.choice-fade-enter-active,
.choice-fade-leave-active {
  transition: opacity 0.3s ease;
}

.choice-fade-enter-from,
.choice-fade-leave-to {
  opacity: 0;
}
</style>
