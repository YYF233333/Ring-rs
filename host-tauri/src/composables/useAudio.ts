import { type DeepReadonly, type Ref, ref, watch } from "vue";
import type { AudioRenderState } from "../types/render-state";
import { useAssets } from "./useAssets";
import { createLogger } from "./useLogger";

const log = createLogger("audio");

/**
 * 响应式音频管理 composable
 *
 * 监听 RenderState.audio 变化，通过 Web Audio API / HTMLAudioElement 实现播放。
 * - BGM: path 变化 + bgm_transition → crossfade；volume 变化 → 平滑调整；null → fade out
 * - SFX: sfx_queue 有条目 → 播放一次性音效
 */
export function useAudio(audioState: Ref<DeepReadonly<AudioRenderState> | undefined>) {
  const { assetUrl } = useAssets();

  let audioCtx: AudioContext | null = null;
  let bgmElement: HTMLAudioElement | null = null;
  let bgmGainNode: GainNode | null = null;
  let bgmSourceNode: MediaElementAudioSourceNode | null = null;
  let currentBgmPath: string | null = null;
  let fadeOutTimer: ReturnType<typeof setTimeout> | null = null;

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

  // ── BGM 生命周期 ────────────────────────────────────────────────────────

  function cleanupAudioNodes(
    element: HTMLAudioElement,
    source: MediaElementAudioSourceNode,
    gain: GainNode,
  ) {
    element.pause();
    element.src = "";
    element.load();
    source.disconnect();
    gain.disconnect();
  }

  /** 立即停止当前 BGM（硬切，无淡出） */
  function stopBgmImmediate() {
    if (fadeOutTimer) {
      clearTimeout(fadeOutTimer);
      fadeOutTimer = null;
    }
    if (bgmElement && bgmSourceNode && bgmGainNode) {
      cleanupAudioNodes(bgmElement, bgmSourceNode, bgmGainNode);
    }
    bgmElement = null;
    bgmSourceNode = null;
    bgmGainNode = null;
    currentBgmPath = null;
  }

  /**
   * 将当前 BGM 节点移交给淡出流程，并清空 current 引用。
   * 淡出结束后自动 disconnect。
   */
  function detachAndFadeOut(duration: number) {
    const el = bgmElement;
    const src = bgmSourceNode;
    const gain = bgmGainNode;
    if (!el || !src || !gain || !audioCtx) return;

    gain.gain.cancelScheduledValues(audioCtx.currentTime);
    gain.gain.setValueAtTime(gain.gain.value, audioCtx.currentTime);
    gain.gain.linearRampToValueAtTime(0, audioCtx.currentTime + duration);

    const timer = setTimeout(() => cleanupAudioNodes(el, src, gain), duration * 1000 + 100);

    if (fadeOutTimer) clearTimeout(fadeOutTimer);
    fadeOutTimer = timer;

    bgmElement = null;
    bgmSourceNode = null;
    bgmGainNode = null;
    currentBgmPath = null;
  }

  /** 创建新的 BGM 播放节点，返回 null 表示 URL 解析失败 */
  function createBgmNodes(
    path: string,
    looping: boolean,
    initialGain: number,
  ): {
    audio: HTMLAudioElement;
    source: MediaElementAudioSourceNode;
    gain: GainNode;
  } | null {
    const ctx = ensureAudioContext();
    const url = assetUrl(path);
    if (!url) {
      log.warn("Cannot resolve asset URL for BGM:", path);
      return null;
    }

    const audio = new Audio();
    audio.crossOrigin = "anonymous";
    audio.loop = looping;
    audio.preload = "auto";
    audio.src = url;

    const source = ctx.createMediaElementSource(audio);
    const gain = ctx.createGain();
    gain.gain.value = initialGain;
    source.connect(gain);
    gain.connect(ctx.destination);

    return { audio, source, gain };
  }

  /** 安装新 BGM 并开始播放（直接设定目标音量，无淡入） */
  function playBgmImmediate(path: string, looping: boolean, volume: number) {
    stopBgmImmediate();
    const nodes = createBgmNodes(path, looping, volume);
    if (!nodes) return;

    nodes.audio.play().catch((err) => {
      log.warn("BGM play failed (autoplay policy?):", err);
    });

    bgmElement = nodes.audio;
    bgmSourceNode = nodes.source;
    bgmGainNode = nodes.gain;
    currentBgmPath = path;
    log.debug(`BGM playing: ${path}, volume=${volume}, loop=${looping}`);
  }

  /** 从静音淡入新 BGM */
  function playBgmWithFadeIn(
    path: string,
    looping: boolean,
    targetVolume: number,
    duration: number,
  ) {
    stopBgmImmediate();
    const ctx = ensureAudioContext();
    const nodes = createBgmNodes(path, looping, 0);
    if (!nodes) return;

    nodes.gain.gain.setValueAtTime(0, ctx.currentTime);
    nodes.gain.gain.linearRampToValueAtTime(targetVolume, ctx.currentTime + duration);

    nodes.audio.play().catch((err) => {
      log.warn("BGM play failed (autoplay policy?):", err);
    });

    bgmElement = nodes.audio;
    bgmSourceNode = nodes.source;
    bgmGainNode = nodes.gain;
    currentBgmPath = path;
    log.debug(`BGM fade in: ${path}, duration=${duration}s, target=${targetVolume}`);
  }

  /** 交叉淡入淡出：旧 BGM 淡出，新 BGM 同时淡入 */
  function crossfadeBgm(path: string, looping: boolean, targetVolume: number, duration: number) {
    const ctx = ensureAudioContext();

    detachAndFadeOut(duration);

    const nodes = createBgmNodes(path, looping, 0);
    if (!nodes) return;

    nodes.gain.gain.setValueAtTime(0, ctx.currentTime);
    nodes.gain.gain.linearRampToValueAtTime(targetVolume, ctx.currentTime + duration);

    nodes.audio.play().catch((err) => {
      log.warn("BGM play failed (autoplay policy?):", err);
    });

    bgmElement = nodes.audio;
    bgmSourceNode = nodes.source;
    bgmGainNode = nodes.gain;
    currentBgmPath = path;
    log.debug(`BGM crossfade → ${path}, duration=${duration}s`);
  }

  function setBgmVolume(volume: number) {
    if (bgmGainNode && audioCtx) {
      bgmGainNode.gain.linearRampToValueAtTime(volume, audioCtx.currentTime + 0.1);
    }
  }

  // ── SFX ─────────────────────────────────────────────────────────────────

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
      const transition = audioState.value?.bgm_transition ?? null;

      if (!bgm) {
        if (currentBgmPath) {
          if (transition) {
            detachAndFadeOut(transition.duration);
            log.debug(`BGM fade out, duration=${transition.duration}s`);
          } else {
            stopBgmImmediate();
            log.debug("BGM stopped (hard)");
          }
        }
        return;
      }

      const pathChanged = bgm.path !== currentBgmPath;
      if (pathChanged) {
        if (transition) {
          if (currentBgmPath) {
            crossfadeBgm(bgm.path, bgm.looping, bgm.volume, transition.duration);
          } else {
            playBgmWithFadeIn(bgm.path, bgm.looping, bgm.volume, transition.duration);
          }
        } else {
          playBgmImmediate(bgm.path, bgm.looping, bgm.volume);
        }
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
    stopBgmImmediate();
    if (audioCtx) {
      audioCtx.close();
      audioCtx = null;
    }
    document.removeEventListener("click", resumeHandler);
    document.removeEventListener("keydown", resumeHandler);
  }

  return { dispose, tryResume };
}
