import { readonly, ref } from "vue";
import type {
  AppConfig,
  HistoryEntry,
  PlaybackMode,
  RenderState,
  SaveInfo,
} from "../types/render-state";
import { useAssets } from "./useAssets";
import { callBackend } from "./useBackend";
import { createLogger } from "./useLogger";
import { captureScene } from "./useSceneCapture";

const log = createLogger("engine");

// ── 模块级单例状态 ──────────────────────────────────────────────────────────

const renderState = ref<RenderState | null>(null);
const isRunning = ref(false);
const playbackMode = ref<PlaybackMode>("Normal");
let animFrameId: number | null = null;
let lastTime = 0;
let tickCount = 0;

export function useEngine() {
  // ── 游戏循环 ────────────────────────────────────────────────────────────

  function gameLoop() {
    if (document.hidden) {
      animFrameId = requestAnimationFrame(gameLoop);
      return;
    }

    const now = performance.now();
    const dt = Math.min((now - lastTime) / 1000, 0.1);
    lastTime = now;

    callBackend<RenderState>("tick", { dt })
      .then((state) => {
        renderState.value = state;
        tickCount++;
        if (tickCount <= 5 || tickCount % 300 === 0) {
          log.debug(
            `tick #${tickCount}: bg=${state.current_background}, dialogue=${state.dialogue?.content?.slice(0, 30)}, transition=${!!state.scene_transition}, ui=${state.ui_visible}`,
          );
        }
      })
      .catch((err) => {
        if (tickCount <= 5) {
          log.error("tick error", err);
        }
      })
      .finally(() => {
        if (isRunning.value) {
          animFrameId = requestAnimationFrame(gameLoop);
        }
      });
  }

  // ── 生命周期 ────────────────────────────────────────────────────────────

  async function startGame(scriptPath: string) {
    log.info("startGame", scriptPath);
    const state = await callBackend<RenderState>("init_game", { scriptPath });
    log.debug("init_game returned", JSON.stringify(state).slice(0, 500));
    renderState.value = state;
    isRunning.value = true;
    lastTime = performance.now();
    gameLoop();
  }

  function stop() {
    isRunning.value = false;
    tickCount = 0;
    if (animFrameId !== null) cancelAnimationFrame(animFrameId);
  }

  // ── 交互 ────────────────────────────────────────────────────────────────

  async function handleClick() {
    if (!isRunning.value) return;
    const state = await callBackend<RenderState>("click");
    renderState.value = state;
  }

  async function handleChoose(index: number) {
    if (!isRunning.value) return;
    const state = await callBackend<RenderState>("choose", { index });
    renderState.value = state;
  }

  async function continueGame(): Promise<RenderState> {
    stop();
    const state = await callBackend<RenderState>("continue_game");
    renderState.value = state;
    isRunning.value = true;
    lastTime = performance.now();
    gameLoop();
    return state;
  }

  async function returnToTitle() {
    stop();
    await callBackend("return_to_title");
  }

  async function setPlaybackMode(mode: PlaybackMode) {
    playbackMode.value = mode;
    await callBackend("set_playback_mode", { mode: mode.toLowerCase() });
  }

  async function backspace() {
    if (!isRunning.value) return;
    try {
      const state = await callBackend<RenderState>("backspace");
      renderState.value = state;
    } catch {
      // no snapshot available
    }
  }

  async function frontendConnected() {
    await callBackend("frontend_connected").catch(() => {});
  }

  async function finishCutscene() {
    const state = await callBackend<RenderState>("finish_cutscene");
    renderState.value = state;
  }

  async function getHistory(): Promise<HistoryEntry[]> {
    return await callBackend<HistoryEntry[]>("get_history");
  }

  async function quitGame() {
    try {
      await callBackend("quit_game");
    } catch {
      window.close();
    }
  }

  // ── 存档 ────────────────────────────────────────────────────────────────

  async function saveGame(slot: number) {
    const { assetUrl } = useAssets();
    const rs = renderState.value;
    if (rs) {
      const thumbnail_base64 = await captureScene(rs, assetUrl);
      if (thumbnail_base64) {
        await callBackend("save_game_with_thumbnail", { slot, thumbnail_base64 });
        return;
      }
    }
    await callBackend("save_game", { slot });
  }

  async function loadGame(slot: number) {
    stop();
    const state = await callBackend<RenderState>("load_game", { slot });
    renderState.value = state;
    isRunning.value = true;
    lastTime = performance.now();
    gameLoop();
  }

  async function listSaves(): Promise<SaveInfo[]> {
    return await callBackend<SaveInfo[]>("list_saves");
  }

  async function deleteSave(slot: number) {
    await callBackend("delete_save", { slot });
  }

  async function getThumbnail(slot: number): Promise<string | null> {
    return await callBackend<string | null>("get_thumbnail", { slot });
  }

  async function getConfig(): Promise<AppConfig | null> {
    return await callBackend<AppConfig>("get_config");
  }

  return {
    renderState: readonly(renderState),
    isRunning: readonly(isRunning),
    playbackMode: readonly(playbackMode),
    startGame,
    handleClick,
    handleChoose,
    stop,
    saveGame,
    loadGame,
    continueGame,
    listSaves,
    deleteSave,
    getThumbnail,
    getConfig,
    returnToTitle,
    setPlaybackMode,
    backspace,
    frontendConnected,
    finishCutscene,
    getHistory,
    quitGame,
  };
}
