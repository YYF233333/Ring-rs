<script setup lang="ts">
import type { ChapterMarkState } from "../types/render-state";

defineProps<{
  chapterMark: ChapterMarkState | null;
}>();
</script>

<template>
  <div v-if="chapterMark" class="chapter-mark" :style="{ opacity: chapterMark.alpha }">
    <span class="chapter-title">{{ chapterMark.title }}</span>
  </div>
</template>

<style scoped>
.chapter-mark {
  position: absolute;
  top: 4vh;
  left: 3vw;
  z-index: 300;
  transition: opacity 0.4s ease;
}

.chapter-title {
  display: inline-block;
  padding: 0.6vh 1.8vw;
  background: rgba(10, 10, 20, 0.7);
  border-left: 3px solid #f0c040;
  color: #eee;
  font-size: clamp(16px, 2vw, 28px);
  font-weight: 500;
  letter-spacing: 0.08em;
  backdrop-filter: blur(4px);
}
</style>
