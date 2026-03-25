<script setup lang="ts">
import { computed } from "vue";
import type { RenderState } from "../types/render-state";
import BackgroundLayer from "./BackgroundLayer.vue";
import ChapterMark from "./ChapterMark.vue";
import CharacterLayer from "./CharacterLayer.vue";
import ChoicePanel from "./ChoicePanel.vue";
import DialogueBox from "./DialogueBox.vue";
import NvlPanel from "./NvlPanel.vue";
import QuickMenu from "./QuickMenu.vue";
import TitleCard from "./TitleCard.vue";
import TransitionOverlay from "./TransitionOverlay.vue";
import VideoOverlay from "./VideoOverlay.vue";

const props = defineProps<{
  renderState: RenderState;
}>();

const emit = defineEmits<{
  choose: [index: number];
  "cutscene-finished": [];
  "quick-action": [action: string];
}>();

const sceneLayerStyle = computed(() => {
  const e = props.renderState.scene_effect;
  const transforms: string[] = [];
  const filters: string[] = [];

  if (e.shake_offset_x !== 0 || e.shake_offset_y !== 0) {
    transforms.push(`translate(${e.shake_offset_x}px, ${e.shake_offset_y}px)`);
  }
  if (e.blur_amount > 0) {
    filters.push(`blur(${e.blur_amount * 4}px)`);
  }
  if (e.dim_level > 0) {
    filters.push(`brightness(${1 - e.dim_level * 0.7})`);
  }

  const style: Record<string, string> = {};
  if (transforms.length > 0) style.transform = transforms.join(" ");
  if (filters.length > 0) style.filter = filters.join(" ");
  return style;
});
</script>

<template>
  <div class="vn-scene">
    <div class="vn-scene-layer" :style="sceneLayerStyle">
    <BackgroundLayer
      :background-path="renderState.current_background"
      :background-transition="renderState.background_transition"
    />

    <CharacterLayer :characters="renderState.visible_characters" />
    </div>

    <template v-if="renderState.text_mode === 'NVL'">
      <NvlPanel
        :entries="renderState.nvl_entries"
        :ui-visible="renderState.ui_visible"
      />
    </template>
    <template v-else>
      <DialogueBox
        :dialogue="renderState.dialogue"
        :ui-visible="renderState.ui_visible"
      />

      <QuickMenu
        v-if="renderState.ui_visible && renderState.dialogue"
        @action="(a) => emit('quick-action', a)"
      />
    </template>

    <ChoicePanel
      :choices="renderState.choices"
      @choose="(idx) => emit('choose', idx)"
    />

    <ChapterMark :chapter-mark="renderState.chapter_mark" />

    <TitleCard :title-card="renderState.title_card" />

    <TransitionOverlay :scene-transition="renderState.scene_transition" />

    <VideoOverlay
      v-if="renderState.cutscene"
      :cutscene="renderState.cutscene"
      @finished="emit('cutscene-finished')"
    />
  </div>
</template>

<style scoped>
.vn-scene {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden;
  font-family: var(--vn-font-body, "Noto Sans SC", "Microsoft YaHei", sans-serif);
}

.vn-scene-layer {
  position: absolute;
  inset: 0;
  will-change: transform, filter;
}
</style>
