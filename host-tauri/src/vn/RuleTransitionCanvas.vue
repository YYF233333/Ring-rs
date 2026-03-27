<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { useAssets } from "../composables/useAssets";
import { createLogger } from "../composables/useLogger";
import type { SceneTransitionPhaseState } from "../types/render-state";

const log = createLogger("rule-transition");
const { assetUrl } = useAssets();

const props = defineProps<{
  maskPath: string;
  reversed: boolean;
  ramp: number;
  duration: number;
  phase: SceneTransitionPhaseState;
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);

let animationId = 0;
let phaseStartTime = 0;
let maskReady = false;

// Pre-processed mask values (0..1), with reversed already applied
let maskValues: Float32Array | null = null;
let outputData: ImageData | null = null;

const CW = 960;
const CH = 540;

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = src;
  });
}

function easeInOutQuad(t: number): number {
  return t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2;
}

function smoothstep(edge0: number, edge1: number, x: number): number {
  const t = Math.max(0, Math.min(1, (x - edge0) / (edge1 - edge0)));
  return t * t * (3 - 2 * t);
}

async function initMask() {
  const url = assetUrl(props.maskPath);
  if (!url) return;

  try {
    const img = await loadImage(url);
    const offscreen = document.createElement("canvas");
    offscreen.width = CW;
    offscreen.height = CH;
    const ctx = offscreen.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(img, 0, 0, CW, CH);
    const raw = ctx.getImageData(0, 0, CW, CH).data;

    const count = CW * CH;
    maskValues = new Float32Array(count);
    const rev = props.reversed;
    for (let i = 0; i < count; i++) {
      const v = raw[i * 4] / 255;
      maskValues[i] = rev ? 1.0 - v : v;
    }

    outputData = new ImageData(CW, CH);
    log.debug(`mask loaded (${img.naturalWidth}x${img.naturalHeight} → ${CW}x${CH})`);
  } catch (e) {
    log.error("failed to load mask", e);
  }
}

function renderFrame(progress: number) {
  if (!canvasRef.value || !maskValues || !outputData) return;
  const ctx = canvasRef.value.getContext("2d");
  if (!ctx) return;

  const out = outputData.data;
  const mask = maskValues;
  const ramp = props.ramp;
  const len = mask.length;

  for (let i = 0; i < len; i++) {
    const mv = mask[i];
    const coverage = smoothstep(mv - ramp, mv + ramp, progress);
    const idx = i * 4;
    out[idx] = 0;
    out[idx + 1] = 0;
    out[idx + 2] = 0;
    out[idx + 3] = (coverage * 255) | 0;
  }

  ctx.putImageData(outputData, 0, 0);
}

function startPhaseAnimation() {
  cancelAnimationFrame(animationId);

  if (props.phase === "Completed") return;
  if (props.phase === "Hold") {
    renderFrame(1.0);
    return;
  }

  phaseStartTime = performance.now();
  const fadingIn = props.phase === "FadeIn";

  function tick() {
    const elapsed = (performance.now() - phaseStartTime) / 1000;
    const rawT = Math.min(1, elapsed / Math.max(0.001, props.duration));
    const easedT = easeInOutQuad(rawT);
    const progress = fadingIn ? easedT : 1 - easedT;

    renderFrame(progress);

    if (rawT < 1) {
      animationId = requestAnimationFrame(tick);
    }
  }

  animationId = requestAnimationFrame(tick);
}

watch(
  () => props.phase,
  () => {
    if (maskReady) startPhaseAnimation();
  },
);

onMounted(async () => {
  await initMask();
  maskReady = true;
  startPhaseAnimation();
});

onUnmounted(() => {
  cancelAnimationFrame(animationId);
});
</script>

<template>
  <canvas ref="canvasRef" :width="CW" :height="CH" class="rule-canvas" />
</template>

<style scoped>
.rule-canvas {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  z-index: 100;
  pointer-events: none;
}
</style>
