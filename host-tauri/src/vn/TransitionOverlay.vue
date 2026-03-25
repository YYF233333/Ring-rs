<script setup lang="ts">
import { computed } from "vue";
import type { SceneTransition } from "../types/render-state";

const props = defineProps<{
  sceneTransition: Readonly<SceneTransition> | null;
}>();

const overlayStyle = computed(() => {
  const st = props.sceneTransition;
  if (!st) return null;

  const isWhite =
    typeof st.transition_type === "string"
      ? st.transition_type === "FadeWhite"
      : false;
  const bgColor = isWhite ? "rgba(255,255,255," : "rgba(0,0,0,";

  return {
    backgroundColor: `${bgColor}${st.mask_alpha})`,
    transition: `background-color ${st.duration}s linear`,
  };
});

const visible = computed(() => {
  const st = props.sceneTransition;
  return st != null && st.phase !== "Completed";
});
</script>

<template>
  <div v-if="visible" class="transition-overlay" :style="overlayStyle" />
</template>

<style scoped>
.transition-overlay {
  position: absolute;
  inset: 0;
  z-index: 100;
  pointer-events: none;
}
</style>
