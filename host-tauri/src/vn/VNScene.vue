<script setup lang="ts">
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

defineProps<{
  renderState: RenderState;
}>();

const emit = defineEmits<{
  choose: [index: number];
  "cutscene-finished": [];
  "quick-action": [action: string];
}>();
</script>

<template>
  <div class="vn-scene">
    <BackgroundLayer
      :background-path="renderState.current_background"
      :background-transition="renderState.background_transition"
    />

    <CharacterLayer :characters="renderState.visible_characters" />

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
</style>
