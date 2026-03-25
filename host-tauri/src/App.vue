<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import ConfirmDialog from "./components/ConfirmDialog.vue";
import SkipAutoIndicator from "./components/SkipAutoIndicator.vue";
import type Toast from "./components/Toast.vue";
import { useAssets } from "./composables/useAssets";
import { useAudio } from "./composables/useAudio";
import { useConfirmDialog } from "./composables/useConfirmDialog";
import { useEngine } from "./composables/useEngine";
import { useNavigation } from "./composables/useNavigation";
import HistoryScreen from "./screens/HistoryScreen.vue";
import InGameMenu from "./screens/InGameMenu.vue";
import SaveLoadScreen from "./screens/SaveLoadScreen.vue";
import SettingsScreen from "./screens/SettingsScreen.vue";
import TitleScreen from "./screens/TitleScreen.vue";
import type { SaveInfo } from "./types/render-state";
import VNScene from "./vn/VNScene.vue";

const {
  renderState,
  playbackMode,
  startGame,
  handleClick,
  handleChoose,
  saveGame,
  loadGame,
  continueGame,
  listSaves,
  getConfig,
  returnToTitle,
  setPlaybackMode,
  backspace,
  frontendConnected,
  finishCutscene,
  quitGame,
} = useEngine();

const audioState = computed(() => renderState.value?.audio);
const { dispose: disposeAudio } = useAudio(audioState);

const { currentScreen, navigateTo, goBack, resetToTitle, resetToIngame } = useNavigation();

const { init: initAssets } = useAssets();

const showInGameMenu = ref(false);
const toast = ref<InstanceType<typeof Toast> | null>(null);
const saves = ref<SaveInfo[]>([]);

const {
  visible: confirmVisible,
  title: confirmTitle,
  message: confirmMessage,
  ask: askConfirm,
  confirm: onConfirm,
  cancel: onCancelConfirm,
} = useConfirmDialog();

async function refreshSaves() {
  saves.value = await listSaves();
}

async function onNewGame() {
  try {
    const config = await getConfig();
    const scriptPath = config?.start_script_path || "scripts/main.md";
    await startGame(scriptPath);
    resetToIngame();
  } catch {
    toast.value?.show("启动失败", "error");
  }
}

async function onContinue() {
  try {
    await continueGame();
    resetToIngame();
  } catch {
    toast.value?.show("没有可继续的存档", "info");
  }
}

async function onNavigateLoad() {
  await refreshSaves();
  navigateTo("load");
}

async function onNavigateSave() {
  await refreshSaves();
  navigateTo("save");
}

async function onSave(slot: number) {
  try {
    await saveGame(slot);
    toast.value?.show("保存成功", "success");
    await refreshSaves();
  } catch {
    toast.value?.show("保存失败", "error");
  }
}

async function onLoad(slot: number) {
  try {
    await loadGame(slot);
    resetToIngame();
    showInGameMenu.value = false;
    toast.value?.show("读取成功", "success");
  } catch {
    toast.value?.show("读取失败", "error");
  }
}

async function onReturnToTitle() {
  const confirmed = await askConfirm("返回标题", "确定要返回标题画面吗？未保存的进度将丢失。");
  if (!confirmed) return;

  showInGameMenu.value = false;
  try {
    await returnToTitle();
  } catch {
    /* best-effort */
  }
  resetToTitle();
}

function onSceneClick() {
  if (showInGameMenu.value) return;
  handleClick();
}

function onRightClick() {
  if (currentScreen.value === "ingame") {
    showInGameMenu.value = !showInGameMenu.value;
  }
}

function onKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape" && currentScreen.value === "ingame") {
    showInGameMenu.value = !showInGameMenu.value;
    return;
  }
  if (currentScreen.value !== "ingame" || showInGameMenu.value) return;

  if (e.key === "Control") {
    setPlaybackMode("Skip");
    return;
  }
  if (e.key === "a" || e.key === "A") {
    setPlaybackMode(playbackMode.value === "Auto" ? "Normal" : "Auto");
    return;
  }
  if (e.key === "Backspace") {
    backspace();
    return;
  }
  if (e.key === " " || e.key === "Enter") {
    e.preventDefault();
    onSceneClick();
    return;
  }
}

function onKeyUp(e: KeyboardEvent) {
  if (e.key === "Control" && playbackMode.value === "Skip") {
    setPlaybackMode("Normal");
  }
}

onMounted(async () => {
  await frontendConnected();
  await initAssets();
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("keyup", onKeyUp);
  disposeAudio();
});
</script>

<template>
  <div id="game-container" @contextmenu.prevent="onRightClick">
    <!-- Title Screen -->
    <TitleScreen
      v-if="currentScreen === 'title'"
      @new-game="onNewGame"
      @continue="onContinue"
      @load="onNavigateLoad"
      @settings="navigateTo('settings')"
      @quit="quitGame"
    />

    <!-- In-Game -->
    <div
      v-else-if="currentScreen === 'ingame'"
      class="ingame-wrapper"
      @click="onSceneClick"
    >
      <VNScene
        v-if="renderState"
        :render-state="renderState"
        @choose="handleChoose"
        @cutscene-finished="finishCutscene"
      />
      <div v-else class="loading">加载中...</div>

      <InGameMenu
        v-if="showInGameMenu"
        @resume="showInGameMenu = false"
        @save="showInGameMenu = false; onNavigateSave()"
        @load="showInGameMenu = false; onNavigateLoad()"
        @history="showInGameMenu = false; navigateTo('history')"
        @settings="showInGameMenu = false; navigateTo('settings')"
        @title="onReturnToTitle"
      />
    </div>

    <!-- Save/Load -->
    <SaveLoadScreen
      v-else-if="currentScreen === 'save'"
      mode="save"
      :saves="saves"
      @back="goBack"
      @save="onSave"
    />
    <SaveLoadScreen
      v-else-if="currentScreen === 'load'"
      mode="load"
      :saves="saves"
      @back="goBack"
      @load="onLoad"
    />

    <!-- Settings -->
    <SettingsScreen
      v-if="currentScreen === 'settings'"
      @back="goBack"
    />

    <!-- History -->
    <HistoryScreen
      v-if="currentScreen === 'history'"
      @back="goBack"
    />

    <!-- Playback mode indicator -->
    <SkipAutoIndicator
      v-if="currentScreen === 'ingame'"
      :mode="playbackMode"
    />

    <!-- Global overlays -->
    <Toast ref="toast" />
    <ConfirmDialog
      v-if="confirmVisible"
      :title="confirmTitle"
      :message="confirmMessage"
      @confirm="onConfirm"
      @cancel="onCancelConfirm"
    />
  </div>
</template>

<style>
:root {
  --vn-font-body: "Noto Sans SC", "Microsoft YaHei", sans-serif;
  --vn-font-display: "Noto Serif SC", "SimSun", serif;
  --vn-bg-primary: #1a1a2e;
  --vn-bg-overlay: rgba(0, 0, 0, 0.85);
  --vn-text-primary: #e0e0e0;
  --vn-accent: rgba(100, 140, 255, 0.6);
}

html,
body {
  margin: 0;
  padding: 0;
  width: 100%;
  height: 100%;
  overflow: hidden;
  background: #000;
}

#game-container {
  width: 100vw;
  height: 100vh;
  position: relative;
  cursor: default;
  user-select: none;
}

.ingame-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
}

.loading {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: #888;
  font-size: 1.5em;
  font-family: var(--vn-font-body);
}
</style>
