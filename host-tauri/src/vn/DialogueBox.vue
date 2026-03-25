<script setup lang="ts">
import { computed } from "vue";
import { useTheme } from "../composables/useTheme";
import type { DialogueState } from "../types/render-state";

const props = defineProps<{
  dialogue: DialogueState | null;
  uiVisible: boolean;
}>();

const { asset } = useTheme();

const textboxUrl = computed(() => asset("textbox"));
const nameboxUrl = computed(() => asset("namebox"));

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
      <img
        v-if="textboxUrl"
        class="textbox-bg"
        :src="textboxUrl"
        alt=""
      />

      <div v-if="dialogue.speaker" class="namebox-wrapper">
        <img
          v-if="nameboxUrl"
          class="namebox-bg"
          :src="nameboxUrl"
          alt=""
        />
        <span class="speaker-label">{{ dialogue.speaker }}</span>
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
  --text-color: var(--vn-color-text, #1a1a1a);
  --speaker-color: var(--vn-color-hover, #f0c040);
  --font-size: clamp(14px, 1.6vw, 22px);

  position: absolute;
  bottom: 0;
  left: 0;
  width: 100%;
  height: clamp(150px, 25vh, 280px);
  z-index: 100;
}

.textbox-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: fill;
  pointer-events: none;
}

/* Fallback when textbox image not available */
.dialogue-box:not(:has(.textbox-bg)) {
  background: rgba(10, 10, 20, 0.85);
  backdrop-filter: blur(8px);
}

.namebox-wrapper {
  position: absolute;
  top: -2px;
  left: 18%;
  transform: translateY(-100%);
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 100px;
  height: 36px;
}

.namebox-bg {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: fill;
}

/* Fallback namebox */
.namebox-wrapper:not(:has(.namebox-bg)) {
  background: rgba(10, 10, 20, 0.9);
  border-radius: 4px 4px 0 0;
  padding: 0 16px;
}

.speaker-label {
  position: relative;
  z-index: 1;
  color: var(--speaker-color);
  font-size: calc(var(--font-size) * 0.85);
  font-weight: 600;
  letter-spacing: 0.05em;
  padding: 0 16px;
  white-space: nowrap;
}

.dialogue-content {
  position: relative;
  z-index: 1;
  color: var(--text-color);
  font-size: var(--font-size);
  line-height: 1.7;
  min-height: 3em;
  padding: 6vh 22% 2vh 22%;
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
