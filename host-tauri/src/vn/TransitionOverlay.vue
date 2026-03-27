<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { useAssets } from "../composables/useAssets";
import type { SceneTransition, SceneTransitionKind } from "../types/render-state";

const { assetUrl } = useAssets();

const props = defineProps<{
  sceneTransition: Readonly<SceneTransition> | null;
}>();

const visible = computed(() => {
  const st = props.sceneTransition;
  return st != null && st.phase !== "Completed";
});

function isRule(
  kind: SceneTransitionKind,
): kind is { Rule: { mask_path: string; reversed: boolean } } {
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

const overlayStyle = computed(() => {
  const base: Record<string, string | number> = {
    opacity: targetOpacity.value,
    transition:
      transitionDuration.value > 0 ? `opacity ${transitionDuration.value}s linear` : "none",
  };

  if (ruleConfig.value) {
    // TODO: ruleConfig.value.reversed 已从后端传递但前端未消费。
    // 旧 Host 的 shader 通过反转遮罩亮度值 (1.0 - mask) 实现反向过渡，
    // 当前 CSS mask-image + 整层不透明度方案无法直接表达此语义。
    // 待 Rule 过渡整体升级为 Canvas/WebGL 时一并实现。
    const url = assetUrl(ruleConfig.value.mask_path);
    base.backgroundColor = "black";
    base.maskImage = `url(${url})`;
    base.maskSize = "cover";
    base.maskRepeat = "no-repeat";
    base.webkitMaskImage = `url(${url})`;
    base.webkitMaskSize = "cover";
    base.webkitMaskRepeat = "no-repeat";
  } else {
    base.backgroundColor = bgColor.value;
  }

  return base;
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
