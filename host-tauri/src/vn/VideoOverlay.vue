<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from "vue";
import type { CutsceneState } from "../types/render-state";
import { useAssets } from "../composables/useAssets";

const { assetUrl } = useAssets();

const props = defineProps<{
  cutscene: CutsceneState;
}>();

const videoSrc = computed(() => assetUrl(props.cutscene.video_path));

const emit = defineEmits<{
  finished: [];
}>();

const videoRef = ref<HTMLVideoElement | null>(null);

function endCutscene() {
  emit("finished");
}

function onEnded() {
  endCutscene();
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape" || e.key === "Enter" || e.key === " ") {
    e.preventDefault();
    endCutscene();
  }
}

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
  videoRef.value?.play().catch(() => {});
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});
</script>

<template>
  <div class="video-overlay" @click="endCutscene">
    <video
      ref="videoRef"
      :src="videoSrc"
      autoplay
      class="cutscene-video"
      @ended="onEnded"
    />
    <div class="skip-hint">点击或按键跳过</div>
  </div>
</template>

<style scoped>
.video-overlay {
  position: absolute;
  inset: 0;
  z-index: 1000;
  background: #000;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.cutscene-video {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.skip-hint {
  position: absolute;
  bottom: 2rem;
  right: 2rem;
  color: rgba(255, 255, 255, 0.5);
  font-size: 0.85rem;
  font-family: var(--vn-font-body);
  pointer-events: none;
}
</style>
