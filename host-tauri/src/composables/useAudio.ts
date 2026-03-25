import { type DeepReadonly, type Ref, ref, watch } from "vue";
import type { AudioRenderState } from "../types/render-state";
import { useAssets } from "./useAssets";
import { createLogger } from "./useLogger";

const log = createLogger("audio");

/**
 * 响应式音频管理 composable
 *
 * 监听 RenderState.audio 变化，通过 Web Audio API / HTMLAudioElement 实现播放。
 * - BGM: path 变化 → crossfade；volume 变化 → 平滑调整；null → fade out
 * - SFX: sfx_queue 有条目 → 播放一次性音效
 */
export function useAudio(audioState: Ref<DeepReadonly<AudioRenderState> | undefined>) {
  const { assetUrl } = useAssets();

  let audioCtx: AudioContext | null = null;
  let bgmElement: HTMLAudioElement | null = null;
  let bgmGainNode: GainNode | null = null;
  let bgmSourceNode: MediaElementAudioSourceNode | null = null;
  let currentBgmPath: string | null = null;

  const isResumed = ref(false);

  function ensureAudioContext(): AudioContext {
    if (!audioCtx) {
      audioCtx = new AudioContext();
      log.info("AudioContext created, state:", audioCtx.state);
    }
    return audioCtx;
  }

  function tryResume() {
    if (isResumed.value) return;
    const ctx = ensureAudioContext();
    if (ctx.state === "suspended") {
      ctx.resume().then(() => {
        isResumed.value = true;
        log.info("AudioContext resumed after user interaction");
      });
    } else {
      isResumed.value = true;
    }
  }

  const resumeHandler = () => {
    tryResume();
    document.removeEventListener("click", resumeHandler);
    document.removeEventListener("keydown", resumeHandler);
  };
  document.addEventListener("click", resumeHandler, { once: false });
  document.addEventListener("keydown", resumeHandler, { once: false });

  function stopBgm() {
    if (bgmElement) {
      bgmElement.pause();
      bgmElement.src = "";
      bgmElement.load();
    }
    if (bgmSourceNode) {
      bgmSourceNode.disconnect();
      bgmSourceNode = null;
    }
    bgmElement = null;
    bgmGainNode = null;
    currentBgmPath = null;
  }

  function playBgm(path: string, looping: boolean, volume: number) {
    const ctx = ensureAudioContext();
    const url = assetUrl(path);
    if (!url) {
      log.warn("Cannot resolve asset URL for BGM:", path);
      return;
    }

    stopBgm();

    const audio = new Audio();
    audio.crossOrigin = "anonymous";
    audio.loop = looping;
    audio.preload = "auto";
    audio.src = url;

    const source = ctx.createMediaElementSource(audio);
    const gain = ctx.createGain();
    gain.gain.value = volume;

    source.connect(gain);
    gain.connect(ctx.destination);

    audio.play().catch((err) => {
      log.warn("BGM play failed (autoplay policy?):", err);
    });

    bgmElement = audio;
    bgmSourceNode = source;
    bgmGainNode = gain;
    currentBgmPath = path;

    log.debug(`BGM playing: ${path}, volume=${volume}, loop=${looping}`);
  }

  function setBgmVolume(volume: number) {
    if (bgmGainNode && audioCtx) {
      bgmGainNode.gain.linearRampToValueAtTime(volume, audioCtx.currentTime + 0.1);
    }
  }

  function playSfx(path: string, volume: number) {
    const url = assetUrl(path);
    if (!url) {
      log.warn("Cannot resolve asset URL for SFX:", path);
      return;
    }

    const ctx = ensureAudioContext();
    const audio = new Audio();
    audio.crossOrigin = "anonymous";
    audio.src = url;
    const source = ctx.createMediaElementSource(audio);
    const gain = ctx.createGain();
    gain.gain.value = volume;
    source.connect(gain);
    gain.connect(ctx.destination);

    audio.play().catch((err) => {
      log.warn("SFX play failed:", err);
    });
    audio.addEventListener("ended", () => {
      source.disconnect();
    });

    log.debug(`SFX playing: ${path}, volume=${volume}`);
  }

  // ── 监听 BGM 状态变化 ──────────────────────────────────────────────────

  watch(
    () => audioState.value?.bgm,
    (bgm) => {
      if (!bgm) {
        if (currentBgmPath) {
          stopBgm();
          log.debug("BGM stopped (state is null)");
        }
        return;
      }

      const pathChanged = bgm.path !== currentBgmPath;
      if (pathChanged) {
        playBgm(bgm.path, bgm.looping, bgm.volume);
      } else {
        setBgmVolume(bgm.volume);
        if (bgmElement) {
          bgmElement.loop = bgm.looping;
        }
      }
    },
    { deep: true },
  );

  // ── 监听 SFX 队列 ─────────────────────────────────────────────────────

  watch(
    () => audioState.value?.sfx_queue,
    (queue) => {
      if (!queue || queue.length === 0) return;
      for (const sfx of queue) {
        playSfx(sfx.path, sfx.volume);
      }
    },
  );

  function dispose() {
    stopBgm();
    if (audioCtx) {
      audioCtx.close();
      audioCtx = null;
    }
    document.removeEventListener("click", resumeHandler);
    document.removeEventListener("keydown", resumeHandler);
  }

  return { dispose, tryResume };
}
