<script setup lang="ts">
import type { PlaybackMode } from "../types/render-state";

defineProps<{
  mode: PlaybackMode;
}>();
</script>

<template>
  <Transition name="indicator-fade">
    <div v-if="mode !== 'Normal'" class="playback-indicator">
      {{ mode === "Auto" ? "AUTO" : "SKIP" }}
    </div>
  </Transition>
</template>

<style scoped>
.playback-indicator {
  position: fixed;
  top: 1rem;
  left: 1rem;
  z-index: 900;
  padding: 0.25rem 0.75rem;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  font-family: var(--vn-font-body);
  color: rgba(255, 255, 255, 0.9);
  background: rgba(0, 0, 0, 0.6);
  border: 1px solid rgba(255, 255, 255, 0.15);
  pointer-events: none;
  backdrop-filter: blur(4px);
}

.indicator-fade-enter-active,
.indicator-fade-leave-active {
  transition: opacity 0.2s ease;
}

.indicator-fade-enter-from,
.indicator-fade-leave-to {
  opacity: 0;
}
</style>
