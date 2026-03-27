<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { SceneTransition, SceneTransitionKind } from "../types/render-state";
import RuleTransitionCanvas from "./RuleTransitionCanvas.vue";

const props = defineProps<{
  sceneTransition: Readonly<SceneTransition> | null;
}>();

const visible = computed(() => {
  const st = props.sceneTransition;
  return st != null && st.phase !== "Completed";
});

function isRule(
  kind: SceneTransitionKind,
): kind is { Rule: { mask_path: string; reversed: boolean; ramp: number } } {
  return typeof kind === "object" && kind !== null && "Rule" in kind;
}

const ruleConfig = computed(() => {
  const st = props.sceneTransition;
  if (!st || !isRule(st.transition_type)) return null;
  return st.transition_type.Rule;
});

const isWhite = computed(() => {
  const st = props.sceneTransition;
  if (!st) return false;
  return typeof st.transition_type === "string" && st.transition_type === "FadeWhite";
});

const bgColor = computed(() => (isWhite.value ? "white" : "black"));

const targetOpacity = ref(0);
const transitionDuration = ref(0);

watch(
  () => props.sceneTransition?.phase,
  (phase) => {
    const st = props.sceneTransition;
    if (!st) return;
    if (isRule(st.transition_type)) return;

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

const fadeOverlayStyle = computed(() => ({
  backgroundColor: bgColor.value,
  opacity: targetOpacity.value,
  transition:
    transitionDuration.value > 0
      ? `opacity ${transitionDuration.value}s var(--vn-ease-scene)`
      : "none",
}));
</script>

<template>
  <template v-if="visible">
    <RuleTransitionCanvas
      v-if="ruleConfig && sceneTransition"
      :mask-path="ruleConfig.mask_path"
      :reversed="ruleConfig.reversed"
      :ramp="ruleConfig.ramp"
      :duration="sceneTransition.duration"
      :phase="sceneTransition.phase"
    />
    <div v-else class="transition-overlay" :style="fadeOverlayStyle" />
  </template>
</template>

<style scoped>
.transition-overlay {
  position: absolute;
  inset: 0;
  z-index: 100;
  pointer-events: none;
}
</style>
