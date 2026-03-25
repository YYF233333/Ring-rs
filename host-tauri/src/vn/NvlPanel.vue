<script setup lang="ts">
import { computed } from "vue";
import type { NvlEntry } from "../types/render-state";

const props = defineProps<{
  entries: readonly NvlEntry[];
  uiVisible: boolean;
}>();

const displayEntries = computed(() =>
  props.entries.map((e) => ({
    speaker: e.speaker,
    text: e.content.slice(0, e.visible_chars),
    isComplete: e.is_complete,
  })),
);
</script>

<template>
  <Transition name="nvl-fade">
    <div v-if="entries.length > 0 && uiVisible" class="nvl-panel">
      <div class="nvl-scroll">
        <div
          v-for="(entry, i) in displayEntries"
          :key="i"
          class="nvl-entry"
        >
          <span v-if="entry.speaker" class="nvl-speaker">{{ entry.speaker }}：</span>
          <span class="nvl-text">{{ entry.text }}</span>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.nvl-panel {
  position: absolute;
  inset: 0;
  z-index: 100;
  background: rgba(0, 0, 0, 0.75);
  display: flex;
  flex-direction: column;
  padding: 8vh 12vw;
  overflow: hidden;
}

.nvl-scroll {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: clamp(8px, 1.5vh, 18px);
}

.nvl-scroll::-webkit-scrollbar {
  width: 3px;
}
.nvl-scroll::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}

.nvl-entry {
  font-family: var(--vn-font-body);
  font-size: clamp(14px, 1.5vw, 22px);
  line-height: 1.8;
  color: var(--vn-color-ui-text, #e0e0e0);
}

.nvl-speaker {
  color: var(--vn-color-hover, #f0c040);
  font-weight: 500;
}

.nvl-text {
  white-space: pre-wrap;
  word-break: break-word;
}

.nvl-fade-enter-active,
.nvl-fade-leave-active {
  transition: opacity 0.3s ease;
}

.nvl-fade-enter-from,
.nvl-fade-leave-to {
  opacity: 0;
}
</style>
