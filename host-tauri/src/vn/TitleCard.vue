<script setup lang="ts">
import { computed } from "vue";
import type { TitleCardState } from "../types/render-state";

const props = defineProps<{
  titleCard: TitleCardState | null;
}>();

const alpha = computed(() => {
  if (!props.titleCard) return 0;
  const { elapsed, duration } = props.titleCard;
  if (duration <= 0) return 1;

  const fadeTime = duration * 0.2;
  if (elapsed < fadeTime) return elapsed / fadeTime;
  if (elapsed > duration - fadeTime) return (duration - elapsed) / fadeTime;
  return 1;
});
</script>

<template>
  <div v-if="titleCard" class="title-card" :style="{ opacity: alpha }">
    <span class="title-text">{{ titleCard.text }}</span>
  </div>
</template>

<style scoped>
.title-card {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #000;
  z-index: 500;
}

.title-text {
  color: #f0f0f0;
  font-size: clamp(24px, 4vw, 56px);
  font-weight: 300;
  letter-spacing: 0.15em;
  text-align: center;
  max-width: 80%;
  line-height: 1.5;
}
</style>
