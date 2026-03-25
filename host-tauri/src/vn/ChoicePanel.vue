<script setup lang="ts">
import type { ChoicesState } from "../types/render-state";

defineProps<{
  choices: ChoicesState | null;
}>();

const emit = defineEmits<{
  choose: [index: number];
}>();
</script>

<template>
  <Transition name="choice-fade">
    <div v-if="choices" class="choice-overlay">
      <div class="choice-list">
        <button
          v-for="(item, idx) in choices.choices"
          :key="idx"
          class="choice-button"
          :class="{ hovered: choices.hovered_index === idx }"
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
  background: rgba(0, 0, 0, 0.4);
  z-index: 200;
}

.choice-list {
  display: flex;
  flex-direction: column;
  gap: 1.2vh;
  max-width: 60%;
  min-width: 30%;
}

.choice-button {
  padding: 1.4vh 2.5vw;
  background: rgba(20, 20, 40, 0.85);
  border: 1px solid rgba(200, 200, 220, 0.15);
  border-radius: 0.4vw;
  color: #ddd;
  font-size: clamp(14px, 1.5vw, 20px);
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: center;
  backdrop-filter: blur(6px);
}

.choice-button:hover,
.choice-button.hovered {
  background: rgba(60, 60, 100, 0.9);
  border-color: rgba(200, 200, 255, 0.35);
  color: #fff;
  transform: scale(1.02);
  box-shadow: 0 0 12px rgba(100, 100, 200, 0.3);
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
