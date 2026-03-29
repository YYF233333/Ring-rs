<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
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
import type { HistoryEntry, HostScreen, SaveInfo } from "./types/render-state";
import type { ButtonDef } from "./types/screens";
import VNScene from "./vn/VNScene.vue";

type ToastHandle = {
  show: (message: string, type?: "success" | "error" | "info") => void;
};

const {
  renderState,
  playbackMode,
  startGame,
  startGameAtLabel,
  handleClick,
  handleChoose,
  saveGame,
  loadGame,
  continueGame,
  deleteSave,
  listSaves,
  getHistory,
  getConfig,
  returnToTitle,
  setPlaybackMode,
  setHostScreen,
  backspace,
  frontendConnected,
  finishCutscene,
  submitUiResult,
  quitGame,
} = useEngine();

const audioState = computed(() => renderState.value?.audio);
const { dispose: disposeAudio } = useAudio(audioState);

const { currentScreen, navigateTo, replaceGameMenuPage, goBack, syncFromHostScreen, isInGameMenu } =
  useNavigation();

const { init: initAssets } = useAssets();
const { screens, init: initScreens, refreshConditions, actionId } = useScreens();
const { init: initTheme } = useTheme();

const showInGameMenu = ref(false);
const toast = ref<ToastHandle | null>(null);
const saves = ref<SaveInfo[]>([]);
const historyEntries = ref<HistoryEntry[]>([]);

function applyBackendHostScreen(hostScreen: HostScreen) {
  showInGameMenu.value = hostScreen === "InGameMenu";
  syncFromHostScreen(hostScreen);
  if (hostScreen === "Save" || hostScreen === "Load") {
    void refreshSaves();
  }
}

const desiredHostScreen = computed<HostScreen>(() => {
  if (showInGameMenu.value) return "InGameMenu";
  switch (currentScreen.value) {
    case "ingame":
      return "InGame";
    case "save":
      return "Save";
    case "load":
      return "Load";
    case "settings":
      return "Settings";
    case "history":
      return "History";
    case "title":
    default:
      return "Title";
  }
});

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

async function refreshHistory() {
  historyEntries.value = await getHistory();
}

function confirmMessageForAction(action: string): string | undefined {
  const defs = screens.value;
  if (!defs) return undefined;

  const candidates: ButtonDef[] = [
    ...defs.title.buttons,
    ...defs.ingame_menu.buttons,
    ...defs.game_menu.nav_buttons,
    defs.game_menu.return_button,
  ];

  return candidates.find((btn) => actionId(btn.action) === action)?.confirm;
}

async function confirmActionIfNeeded(action: string): Promise<boolean> {
  const message = confirmMessageForAction(action);
  if (!message) return true;
  return askConfirm("确认", message);
}

async function onNewGame() {
  try {
    const config = await getConfig();
    if (!config) throw new Error("config missing");
    const scriptPath = config.start_script_path;
    await startGame(scriptPath);
  } catch {
    toast.value?.show("启动失败", "error");
  }
}

async function onContinue() {
  try {
    await continueGame();
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
    showInGameMenu.value = false;
    toast.value?.show("读取成功", "success");
  } catch {
    toast.value?.show("读取失败", "error");
  }
}

async function onReturnToTitle() {
  showInGameMenu.value = false;
  try {
    await returnToTitle();
  } catch {
    /* best-effort */
  }
  await refreshConditions();
}

async function handleTitleAction(action: string) {
  if (!(await confirmActionIfNeeded(action))) return;

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
      if (!config) throw new Error("config missing");
      const scriptPath = config.start_script_path;
      const label = action.slice("start_at_label:".length);
      await startGameAtLabel(scriptPath, label);
    } catch {
      toast.value?.show("启动失败", "error");
    }
  }
}

async function handleInGameMenuAction(action: string) {
  if (action === "go_back") {
    showInGameMenu.value = false;
    return;
  }
  if (!(await confirmActionIfNeeded(action))) return;

  showInGameMenu.value = false;
  if (action === "open_save") {
    await onNavigateSave();
  } else if (action === "open_load") {
    await onNavigateLoad();
  } else if (action === "navigate_history") {
    await refreshHistory();
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
    await refreshHistory();
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
    if (!(await confirmActionIfNeeded(action))) return;
    await onReturnToTitle();
  } else if (action === "return_to_game") {
    goBack();
  } else if (action === "exit") {
    if (!(await confirmActionIfNeeded(action))) return;
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
    await refreshHistory();
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

async function onDeleteSave(slot: number) {
  try {
    await deleteSave(slot);
    await refreshSaves();
    toast.value?.show("删除成功", "success");
  } catch {
    toast.value?.show("删除失败", "error");
  }
}

function onSceneClick() {
  if (showInGameMenu.value) return;
  if (renderState.value?.active_ui_mode) return;
  handleClick();
}

function onRightClick() {
  if (currentScreen.value === "ingame" && !renderState.value?.active_ui_mode) {
    showInGameMenu.value = !showInGameMenu.value;
  }
}

function onKeyDown(e: KeyboardEvent) {
  if (currentScreen.value !== "ingame") return;

  if (renderState.value?.active_ui_mode) {
    return;
  }

  if (e.key === "Escape" && currentScreen.value === "ingame") {
    showInGameMenu.value = !showInGameMenu.value;
    return;
  }
  if (showInGameMenu.value) return;

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

watch(
  () => renderState.value?.host_screen,
  (hostScreen) => {
    if (!hostScreen) return;
    applyBackendHostScreen(hostScreen);
  },
  { immediate: true },
);

watch(desiredHostScreen, (hostScreen) => {
  if (!renderState.value) return;
  if (renderState.value.host_screen === hostScreen) return;
  void setHostScreen(hostScreen).catch(() => {
    /* owner may have switched during teardown */
  });
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
        @ui-result="(key: string, value: unknown) => submitUiResult(key, value)"
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
        @delete="onDeleteSave"
      />
      <SaveLoadScreen
        v-else-if="currentScreen === 'load'"
        mode="load"
        :saves="saves"
        @load="onLoad"
        @delete="onDeleteSave"
      />
      <SettingsScreen
        v-else-if="currentScreen === 'settings'"
      />
      <HistoryScreen
        v-else-if="currentScreen === 'history'"
        :entries="historyEntries"
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
  --vn-ease-scene: cubic-bezier(0.455, 0.03, 0.515, 0.955);
  --vn-ease-character: cubic-bezier(0.645, 0.045, 0.355, 1);
  --vn-ease-ui: ease;
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
