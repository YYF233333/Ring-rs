<script setup lang="ts">
import { computed } from "vue";
import { useAssets } from "../composables/useAssets";
import type { BackgroundTransition } from "../types/render-state";

const { assetUrl } = useAssets();

const props = defineProps<{
  backgroundPath: string | null;
  backgroundTransition: Readonly<BackgroundTransition> | null;
}>();

const backgroundUrl = computed(() => assetUrl(props.backgroundPath));

const oldBackgroundUrl = computed(() => assetUrl(props.backgroundTransition?.old_background));

const isDissolving = computed(() => props.backgroundTransition != null);

const dissolveDuration = computed(() => {
  return props.backgroundTransition?.duration ?? 0.3;
});
</script>

<template>
  <div class="background-layer">
    <!-- Old background (fades out during dissolve) -->
    <img
      v-if="isDissolving && oldBackgroundUrl"
      :src="oldBackgroundUrl"
      class="background-image background-old"
      :style="{
        transition: `opacity ${dissolveDuration}s ease-in-out`,
        opacity: 0,
      }"
      alt=""
    />

    <!-- Current background -->
    <img
      v-if="backgroundUrl"
      :key="backgroundUrl"
      :src="backgroundUrl"
      class="background-image"
      :style="{
        transition: isDissolving
          ? `opacity ${dissolveDuration}s ease-in-out`
          : 'none',
      }"
      alt=""
    />
  </div>
</template>

<style scoped>
.background-layer {
  position: absolute;
  inset: 0;
  z-index: 0;
  overflow: hidden;
}

.background-image {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.background-old {
  z-index: 0;
}
</style>
