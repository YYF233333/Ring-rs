import { ref, readonly } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { RenderState, SaveInfo, AppConfig } from "../types/render-state";

export function useEngine() {
  const renderState = ref<RenderState | null>(null);
  const isRunning = ref(false);
  let animFrameId: number | null = null;
  let lastTime = 0;

  async function startGame(scriptPath: string) {
    console.debug("[engine] startGame:", scriptPath);
    const state = await invoke<RenderState>("init_game", { scriptPath });
    console.debug("[engine] init_game returned:", JSON.stringify(state).slice(0, 500));
    renderState.value = state;
    isRunning.value = true;
    lastTime = performance.now();
    gameLoop();
  }

  let tickCount = 0;
  function gameLoop() {
    const now = performance.now();
    const dt = (now - lastTime) / 1000;
    lastTime = now;

    invoke<RenderState>("tick", { dt })
      .then((state) => {
        renderState.value = state;
        tickCount++;
        if (tickCount <= 5 || tickCount % 300 === 0) {
          console.debug(`[engine] tick #${tickCount}: bg=${state.current_background}, dialogue=${state.dialogue?.content?.slice(0, 30)}, transition=${!!state.scene_transition}, ui=${state.ui_visible}`);
        }
      })
      .catch((err) => {
        if (tickCount <= 5) {
          console.error("[engine] tick error:", err);
        }
      })
      .finally(() => {
        if (isRunning.value) {
          animFrameId = requestAnimationFrame(gameLoop);
        }
      });
  }

  async function handleClick() {
    if (!isRunning.value) return;
    const state = await invoke<RenderState>("click");
    renderState.value = state;
  }

  async function handleChoose(index: number) {
    if (!isRunning.value) return;
    const state = await invoke<RenderState>("choose", { index });
    renderState.value = state;
  }

  function stop() {
    isRunning.value = false;
    if (animFrameId !== null) cancelAnimationFrame(animFrameId);
  }

  // ── 存档 ───────────────────────────────────────────────────────────────

  async function saveGame(slot: number) {
    await invoke("save_game", { slot });
  }

  async function loadGame(slot: number) {
    const state = await invoke<RenderState>("load_game", { slot });
    renderState.value = state;
    isRunning.value = true;
    lastTime = performance.now();
    gameLoop();
  }

  async function listSaves(): Promise<SaveInfo[]> {
    return await invoke<SaveInfo[]>("list_saves");
  }

  async function deleteSave(slot: number) {
    await invoke("delete_save", { slot });
  }

  async function getConfig(): Promise<AppConfig | null> {
    return await invoke<AppConfig>("get_config");
  }

  return {
    renderState: readonly(renderState),
    isRunning: readonly(isRunning),
    startGame,
    handleClick,
    handleChoose,
    stop,
    saveGame,
    loadGame,
    listSaves,
    deleteSave,
    getConfig,
  };
}
