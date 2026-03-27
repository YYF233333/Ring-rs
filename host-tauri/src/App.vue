<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import ConfirmDialog from "./components/ConfirmDialog.vue";
import SkipAutoIndicator from "./components/SkipAutoIndicator.vue";
import Toast from "./components/Toast.vue";
import { useAssets } from "./composables/useAssets";
import { useAudio } from "./composables/useAudio";
import { useConfirmDialog } from "./composables/useConfirmDialog";
import { useEngine } from "./composables/useEngine";
import { useNavigation } from "./composables/useNavigation";
import { useScreens } from "./composables/useScreens";
import { useTheme } from "./composables/useTheme";
import GameMenuFrame from "./screens/GameMenuFrame.vue";
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

const {
  currentScreen,
  navigateTo,
  replaceGameMenuPage,
  goBack,
  resetToTitle,
  resetToIngame,
  isInGameMenu,
} = useNavigation();

const { init: initAssets } = useAssets();
const { init: initScreens, refreshConditions } = useScreens();
const { init: initTheme } = useTheme();

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
  await refreshConditions();
}

async function handleTitleAction(action: string) {
  if (action === "start_game") {
    await onNewGame();
  } else if (action === "continue_game") {
    await onContinue();
  } else if (action === "open_load") {
    await onNavigateLoad();
  } else if (action === "navigate_settings") {
    navigateTo("settings");
  } else if (action === "exit") {
    await quitGame();
  } else if (action.startsWith("start_at_label:")) {
    try {
      const config = await getConfig();
      const scriptPath = config?.start_script_path || "scripts/main.md";
      await startGame(scriptPath);
      resetToIngame();
    } catch {
      toast.value?.show("启动失败", "error");
    }
  }
}

async function handleInGameMenuAction(action: string) {
  showInGameMenu.value = false;
  if (action === "go_back") return;
  if (action === "open_save") {
    await onNavigateSave();
  } else if (action === "open_load") {
    await onNavigateLoad();
  } else if (action === "navigate_history") {
    navigateTo("history");
  } else if (action === "navigate_settings") {
    navigateTo("settings");
  } else if (action === "return_to_title") {
    await onReturnToTitle();
  } else if (action === "exit") {
    await quitGame();
  }
}

async function handleGameMenuAction(action: string) {
  if (action === "replace_history") {
    replaceGameMenuPage("history");
  } else if (action === "open_save") {
    await refreshSaves();
    replaceGameMenuPage("save");
  } else if (action === "open_load") {
    await refreshSaves();
    replaceGameMenuPage("load");
  } else if (action === "replace_settings") {
    replaceGameMenuPage("settings");
  } else if (action === "return_to_title") {
    await onReturnToTitle();
  } else if (action === "return_to_game") {
    goBack();
  } else if (action === "exit") {
    await quitGame();
  }
}

/** 当前 game_menu 框架内活跃的导航 action */
const activeGameMenuNav = computed(() => {
  const s = currentScreen.value;
  if (s === "history") return "replace_history";
  if (s === "save") return "open_save";
  if (s === "load") return "open_load";
  if (s === "settings") return "replace_settings";
  return undefined;
});

async function handleQuickAction(action: string) {
  if (action === "navigate_history") {
    navigateTo("history");
  } else if (action === "toggle_skip") {
    await setPlaybackMode(playbackMode.value === "Skip" ? "Normal" : "Skip");
  } else if (action === "toggle_auto") {
    await setPlaybackMode(playbackMode.value === "Auto" ? "Normal" : "Auto");
  } else if (action === "open_save") {
    await onNavigateSave();
  } else if (action === "quick_save") {
    try {
      await saveGame(0);
      toast.value?.show("快速保存成功", "success");
    } catch {
      toast.value?.show("快速保存失败", "error");
    }
  } else if (action === "quick_load") {
    try {
      await loadGame(0);
      toast.value?.show("快速读取成功", "success");
    } catch {
      toast.value?.show("没有快速存档", "info");
    }
  } else if (action === "navigate_settings") {
    navigateTo("settings");
  }
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
  await Promise.all([initScreens(), initTheme()]);
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
      @action="handleTitleAction"
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
        @quick-action="handleQuickAction"
      />
      <div v-else class="loading">加载中...</div>

      <InGameMenu
        v-if="showInGameMenu"
        @action="handleInGameMenuAction"
      />
    </div>

    <!-- Game Menu (Save/Load/Settings/History) -->
    <GameMenuFrame
      v-else-if="isInGameMenu()"
      :active-nav="activeGameMenuNav"
      @action="handleGameMenuAction"
    >
      <SaveLoadScreen
        v-if="currentScreen === 'save'"
        mode="save"
        :saves="saves"
        @save="onSave"
      />
      <SaveLoadScreen
        v-else-if="currentScreen === 'load'"
        mode="load"
        :saves="saves"
        @load="onLoad"
      />
      <SettingsScreen
        v-else-if="currentScreen === 'settings'"
      />
      <HistoryScreen
        v-else-if="currentScreen === 'history'"
      />
    </GameMenuFrame>

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
