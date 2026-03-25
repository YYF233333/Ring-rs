<script setup lang="ts">
import { computed } from "vue";
import type { DialogueState } from "../types/render-state";

const props = defineProps<{
  dialogue: DialogueState | null;
  uiVisible: boolean;
}>();

const visibleText = computed(() => {
  if (!props.dialogue) return "";
  return props.dialogue.content.slice(0, props.dialogue.visible_chars);
});

const showIndicator = computed(() => {
  return props.dialogue?.is_complete ?? false;
});
</script>

<template>
  <Transition name="dialogue-fade">
    <div v-if="dialogue && uiVisible" class="dialogue-box">
      <div v-if="dialogue.speaker" class="speaker-label">
        {{ dialogue.speaker }}
      </div>
      <div class="dialogue-content">
        <span class="dialogue-text">{{ visibleText }}</span>
        <span v-if="showIndicator" class="advance-indicator">▼</span>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.dialogue-box {
  --dialogue-bg: rgba(10, 10, 20, 0.85);
  --dialogue-radius: 0.6vw;
  --speaker-color: #f0c040;
  --text-color: #eaeaea;
  --font-size: clamp(14px, 1.6vw, 22px);

  position: absolute;
  bottom: 3vh;
  left: 10%;
  width: 80%;
  padding: 1.8vh 2.2vw;
  background: var(--dialogue-bg);
  border-radius: var(--dialogue-radius);
  backdrop-filter: blur(8px);
  box-shadow: 0 0 20px rgba(0, 0, 0, 0.5);
  z-index: 100;
}

.speaker-label {
  position: absolute;
  top: -2.4vh;
  left: 1.5vw;
  padding: 0.3vh 1vw;
  background: rgba(10, 10, 20, 0.9);
  border-radius: var(--dialogue-radius) var(--dialogue-radius) 0 0;
  color: var(--speaker-color);
  font-size: calc(var(--font-size) * 0.9);
  font-weight: 600;
  letter-spacing: 0.05em;
}

.dialogue-content {
  color: var(--text-color);
  font-size: var(--font-size);
  line-height: 1.7;
  min-height: 3.4em;
}

.dialogue-text {
  white-space: pre-wrap;
  word-break: break-word;
}

.advance-indicator {
  display: inline-block;
  margin-left: 0.5em;
  color: var(--speaker-color);
  animation: blink 0.8s ease-in-out infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.2; }
}

.dialogue-fade-enter-active,
.dialogue-fade-leave-active {
  transition: opacity 0.25s ease;
}

.dialogue-fade-enter-from,
.dialogue-fade-leave-to {
  opacity: 0;
}
</style>
