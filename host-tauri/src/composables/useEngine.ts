import { readonly, ref } from "vue";
import type {
  AppConfig,
  FrontendSession,
  HarnessTraceBundle,
  HistoryEntry,
  HostScreen,
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
const clientToken = ref<string | null>(null);
let animFrameId: number | null = null;
let lastTime = 0;
let tickCount = 0;

export function useEngine() {
  function applyRenderState(state: RenderState) {
    renderState.value = state;
    playbackMode.value = state.playback_mode;
  }

  function requireClientToken(): string {
    if (!clientToken.value) {
      throw new Error("frontend client token missing; call frontendConnected first");
    }
    return clientToken.value;
  }

  function sessionArgs(args?: Record<string, unknown>) {
    return {
      clientToken: requireClientToken(),
      ...(args ?? {}),
    };
  }

  function hostScreenArg(screen: HostScreen): string {
    switch (screen) {
      case "InGame":
        return "ingame";
      case "InGameMenu":
        return "ingame_menu";
      case "Save":
        return "save";
      case "Load":
        return "load";
      case "Settings":
        return "settings";
      case "History":
        return "history";
      case "Title":
      default:
        return "title";
    }
  }

  // ── 游戏循环 ────────────────────────────────────────────────────────────

  function gameLoop() {
    if (document.hidden) {
      animFrameId = requestAnimationFrame(gameLoop);
      return;
    }

    const now = performance.now();
    const dt = Math.min((now - lastTime) / 1000, 0.1);
    lastTime = now;

    callBackend<RenderState>("tick", sessionArgs({ dt }))
      .then((state) => {
        applyRenderState(state);
        tickCount++;
        if (tickCount <= 5 || tickCount % 300 === 0) {
          log.debug(
            `tick #${tickCount}: bg=${state.current_background}, dialogue=${state.dialogue?.content?.slice(0, 30)}, transition=${!!state.scene_transition}, ui=${state.ui_visible}`,
          );
        }
      })
      .catch((err) => {
        log.error("tick error", err);
        stop();
      })
      .finally(() => {
        if (isRunning.value) {
          animFrameId = requestAnimationFrame(gameLoop);
        }
      });
  }

  // ── 生命周期 ────────────────────────────────────────────────────────────

  async function startGame(scriptPath: string) {
    stop();
    log.info("startGame", scriptPath);
    const state = await callBackend<RenderState>("init_game", sessionArgs({ scriptPath }));
    log.debug("init_game returned", JSON.stringify(state).slice(0, 500));
    applyRenderState(state);
    isRunning.value = true;
    lastTime = performance.now();
    gameLoop();
  }

  async function startGameAtLabel(scriptPath: string, label: string) {
    stop();
    log.info("startGameAtLabel", `${scriptPath}#${label}`);
    const state = await callBackend<RenderState>(
      "init_game_at_label",
      sessionArgs({ scriptPath, label }),
    );
    applyRenderState(state);
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
    const state = await callBackend<RenderState>("click", sessionArgs());
    applyRenderState(state);
  }

  async function handleChoose(index: number) {
    if (!isRunning.value) return;
    const state = await callBackend<RenderState>("choose", sessionArgs({ index }));
    applyRenderState(state);
  }

  async function continueGame(): Promise<RenderState> {
    stop();
    const state = await callBackend<RenderState>("continue_game", sessionArgs());
    applyRenderState(state);
    isRunning.value = true;
    lastTime = performance.now();
    gameLoop();
    return state;
  }

  async function returnToTitle() {
    stop();
    const state = await callBackend<RenderState>(
      "return_to_title",
      sessionArgs({ saveContinue: true }),
    );
    applyRenderState(state);
  }

  async function setPlaybackMode(mode: PlaybackMode) {
    const state = await callBackend<RenderState>(
      "set_playback_mode",
      sessionArgs({ mode: mode.toLowerCase() }),
    );
    applyRenderState(state);
  }

  async function setHostScreen(screen: HostScreen) {
    const state = await callBackend<RenderState>(
      "set_host_screen",
      sessionArgs({ screen: hostScreenArg(screen) }),
    );
    applyRenderState(state);
  }

  async function backspace() {
    if (!isRunning.value) return;
    try {
      const state = await callBackend<RenderState>("backspace", sessionArgs());
      applyRenderState(state);
    } catch {
      // no snapshot available
    }
  }

  async function frontendConnected() {
    const session = await callBackend<FrontendSession>("frontend_connected", {
      clientLabel: "ui",
    });
    clientToken.value = session.client_token;
    applyRenderState(session.render_state);
    return session.render_state;
  }

  async function finishCutscene() {
    const state = await callBackend<RenderState>("finish_cutscene", sessionArgs());
    applyRenderState(state);
  }

  async function submitUiResult(key: string, value: unknown) {
    try {
      log.info(`submitUiResult: key=${key}, value=${JSON.stringify(value)}`);
      const state = await callBackend<RenderState>("submit_ui_result", {
        ...sessionArgs(),
        key,
        value: value ?? "",
      });
      applyRenderState(state);
    } catch (err) {
      log.error("submitUiResult failed", err);
    }
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
        await callBackend("save_game_with_thumbnail", sessionArgs({ slot, thumbnail_base64 }));
        return;
      }
    }
    await callBackend("save_game", sessionArgs({ slot }));
  }

  async function loadGame(slot: number) {
    stop();
    const state = await callBackend<RenderState>("load_game", sessionArgs({ slot }));
    applyRenderState(state);
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

  async function debugRunUntil(
    dt: number,
    maxSteps: number,
    stopOnWait = true,
    stopOnScriptFinished = true,
  ): Promise<HarnessTraceBundle> {
    return await callBackend<HarnessTraceBundle>(
      "debug_run_until",
      sessionArgs({ dt, maxSteps, stopOnWait, stopOnScriptFinished }),
    );
  }

  return {
    renderState: readonly(renderState),
    isRunning: readonly(isRunning),
    playbackMode: readonly(playbackMode),
    startGame,
    startGameAtLabel,
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
    setHostScreen,
    backspace,
    frontendConnected,
    finishCutscene,
    submitUiResult,
    getHistory,
    quitGame,
    debugRunUntil,
  };
}
