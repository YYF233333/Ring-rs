<script setup lang="ts">
import { computed, watch, ref } from "vue";
import type { SceneTransition } from "../types/render-state";

const props = defineProps<{
  sceneTransition: Readonly<SceneTransition> | null;
}>();

const visible = computed(() => {
  const st = props.sceneTransition;
  return st != null && st.phase !== "Completed";
});

const isWhite = computed(() => {
  const st = props.sceneTransition;
  if (!st) return false;
  return typeof st.transition_type === "string" && st.transition_type === "FadeWhite";
});

const bgColor = computed(() => isWhite.value ? "white" : "black");

const targetOpacity = ref(0);
const transitionDuration = ref(0);

watch(
  () => props.sceneTransition?.phase,
  (phase) => {
    const st = props.sceneTransition;
    if (!st) return;

    switch (phase) {
      case "FadeIn":
        targetOpacity.value = 0;
        transitionDuration.value = 0;
        requestAnimationFrame(() => {
          targetOpacity.value = 1;
          transitionDuration.value = st.duration;
        });
        break;
      case "Hold":
        targetOpacity.value = 1;
        transitionDuration.value = 0;
        break;
      case "FadeOut":
        targetOpacity.value = 1;
        transitionDuration.value = 0;
        requestAnimationFrame(() => {
          targetOpacity.value = 0;
          transitionDuration.value = st.duration;
        });
        break;
      case "Completed":
        targetOpacity.value = 0;
        transitionDuration.value = 0;
        break;
    }
  },
  { immediate: true },
);

const overlayStyle = computed(() => ({
  backgroundColor: bgColor.value,
  opacity: targetOpacity.value,
  transition: transitionDuration.value > 0
    ? `opacity ${transitionDuration.value}s linear`
    : "none",
}));
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
