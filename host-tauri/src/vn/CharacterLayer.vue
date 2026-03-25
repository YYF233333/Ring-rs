<script setup lang="ts">
import { useAssets } from "../composables/useAssets";
import type { CharacterSprite } from "../types/render-state";

const { assetUrl } = useAssets();

defineProps<{
  characters: Readonly<Record<string, CharacterSprite>>;
}>();

const positionMap: Record<string, string> = {
  Left: "15%",
  NearLeft: "30%",
  Center: "50%",
  NearRight: "70%",
  Right: "85%",
};

function getCharacterStyle(char: Readonly<CharacterSprite>): Record<string, string | number> {
  const x = positionMap[char.position] || "50%";
  const td = char.transition_duration ?? 0;
  const transition = td > 0
    ? `left ${td}s ease-in-out, opacity ${td}s ease-in-out, transform ${td}s ease-in-out`
    : "none";
  return {
    position: "absolute",
    bottom: "0",
    left: x,
    transform: `translateX(-50%) translate(${char.offset_x}px, ${char.offset_y}px) scale(${char.scale_x}, ${char.scale_y})`,
    opacity: char.alpha,
    transition,
    zIndex: char.z_order,
    maxHeight: "100%",
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
}

.character-sprite {
  image-rendering: auto;
  user-select: none;
}
</style>
