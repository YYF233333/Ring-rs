<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { callBackend } from "./composables/useBackend";
import { useEngine } from "./composables/useEngine";
import { useNavigation } from "./composables/useNavigation";
import { useAssets } from "./composables/useAssets";
import type { SaveInfo, PlaybackMode } from "./types/render-state";
import VNScene from "./vn/VNScene.vue";
import TitleScreen from "./screens/TitleScreen.vue";
import SaveLoadScreen from "./screens/SaveLoadScreen.vue";
import SettingsScreen from "./screens/SettingsScreen.vue";
import HistoryScreen from "./screens/HistoryScreen.vue";
import InGameMenu from "./screens/InGameMenu.vue";
import Toast from "./components/Toast.vue";
import ConfirmDialog from "./components/ConfirmDialog.vue";
import SkipAutoIndicator from "./components/SkipAutoIndicator.vue";

const {
  renderState,
  startGame,
  handleClick,
  handleChoose,
  stop,
  saveGame,
  loadGame,
  listSaves,
} = useEngine();

const { currentScreen, navigateTo, goBack, resetToTitle, resetToIngame } =
  useNavigation();

const { init: initAssets } = useAssets();

const showInGameMenu = ref(false);
const toast = ref<InstanceType<typeof Toast> | null>(null);
const saves = ref<SaveInfo[]>([]);

const confirmVisible = ref(false);
const confirmTitle = ref("");
const confirmMessage = ref("");
let confirmResolve: ((v: boolean) => void) | null = null;

function askConfirm(title: string, message: string): Promise<boolean> {
  confirmTitle.value = title;
  confirmMessage.value = message;
  confirmVisible.value = true;
  return new Promise((resolve) => {
    confirmResolve = resolve;
  });
}

function onConfirm() {
  confirmVisible.value = false;
  confirmResolve?.(true);
}

function onCancelConfirm() {
  confirmVisible.value = false;
  confirmResolve?.(false);
}

async function refreshSaves() {
  saves.value = await listSaves();
}

async function onNewGame() {
  try {
    await startGame("scripts/remake/main.md");
    resetToIngame();
  } catch {
    toast.value?.show("启动失败", "error");
  }
}

async function onContinue() {
  try {
    const state = await callBackend("continue_game");
    if (state) {
      resetToIngame();
    } else {
      toast.value?.show("没有可继续的存档", "info");
    }
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
  const confirmed = await askConfirm(
    "返回标题",
    "确定要返回标题画面吗？未保存的进度将丢失。",
  );
  if (!confirmed) return;

  showInGameMenu.value = false;
  stop();
  try {
    await callBackend("return_to_title");
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

const playbackMode = ref<PlaybackMode>("Normal");

async function setPlaybackMode(mode: PlaybackMode) {
  playbackMode.value = mode;
  try {
    await callBackend("set_playback_mode", { mode: mode.toLowerCase() });
  } catch {
    /* best-effort */
  }
}

async function handleBackspace() {
  try {
    await callBackend("backspace");
  } catch {
    /* no snapshot available */
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
    handleBackspace();
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
  await callBackend("frontend_connected").catch(() => {});
  await initAssets();
  window.addEventListener("keydown", onKeyDown);
  window.addEventListener("keyup", onKeyUp);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
  window.removeEventListener("keyup", onKeyUp);
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
      @quit="() => {}"
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
