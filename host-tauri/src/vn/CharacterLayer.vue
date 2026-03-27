<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useAssets } from "../composables/useAssets";
import type { CharacterSprite } from "../types/render-state";

const DESIGN_WIDTH = 1920;
const DESIGN_HEIGHT = 1080;

const { assetUrl } = useAssets();

defineProps<{
  characters: Readonly<Record<string, CharacterSprite>>;
}>();

const vpWidth = ref(window.innerWidth);
const vpHeight = ref(window.innerHeight);

function onResize() {
  vpWidth.value = window.innerWidth;
  vpHeight.value = window.innerHeight;
}

onMounted(() => window.addEventListener("resize", onResize));
onBeforeUnmount(() => window.removeEventListener("resize", onResize));

const baseScale = computed(() =>
  Math.min(vpWidth.value / DESIGN_WIDTH, vpHeight.value / DESIGN_HEIGHT),
);

function getCharacterStyle(char: Readonly<CharacterSprite>): Record<string, string | number> {
  const td = char.transition_duration ?? 0;
  const transition =
    td > 0
      ? `left ${td}s var(--vn-ease-character), top ${td}s var(--vn-ease-character), opacity ${td}s var(--vn-ease-character), transform ${td}s var(--vn-ease-character)`
      : "none";

  const anchorXPct = char.anchor_x * 100;
  const anchorYPct = char.anchor_y * 100;
  const s = baseScale.value;
  const scaleX = s * char.render_scale * char.scale_x;
  const scaleY = s * char.render_scale * char.scale_y;

  return {
    position: "absolute",
    left: `${char.pos_x * 100}%`,
    top: `${char.pos_y * 100}%`,
    transformOrigin: `${anchorXPct}% ${anchorYPct}%`,
    transform: `translate(${-anchorXPct}%, ${-anchorYPct}%) translate(${char.offset_x}px, ${char.offset_y}px) scale(${scaleX}, ${scaleY})`,
    opacity: char.target_alpha,
    transition,
    zIndex: char.z_order,
  };
}
</script>

<template>
  <div class="character-layer">
    <img
      v-for="(char, alias) in characters"
      :key="alias"
      :src="assetUrl(char.texture_path)"
      :style="getCharacterStyle(char)"
      class="character-sprite"
      draggable="false"
    />
  </div>
</template>

<style scoped>
.character-layer {
  position: absolute;
  inset: 0;
  z-index: 1;
  pointer-events: none;
  overflow: hidden;
}

.character-sprite {
  image-rendering: auto;
  user-select: none;
}
</style>
