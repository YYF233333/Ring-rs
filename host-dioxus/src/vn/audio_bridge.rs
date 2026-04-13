use dioxus::prelude::*;

use crate::render_state::RenderState;

/// 资源 URL 构建
fn asset_url(path: &str) -> String {
    format!("http://ring-asset.localhost/{path}")
}

/// 音频桥接组件：监听 `AudioRenderState` 变化，通过 JS Web Audio API 播放。
///
/// 不渲染任何 DOM。在 mount 时注入 JS AudioManager，
/// 每帧 diff BGM/SFX 状态并调用 JS 函数。
#[component]
pub fn AudioBridge(render_state: Signal<RenderState>) -> Element {
    // 注入 JS 音频管理器（仅首次 mount）
    use_effect(|| {
        document::eval(
            r#"
            window.__ringAudio = (function() {
                let bgmAudio = null;
                let bgmPath = null;
                let bgmFadeInterval = null;

                function stopBgmFade() {
                    if (bgmFadeInterval) {
                        clearInterval(bgmFadeInterval);
                        bgmFadeInterval = null;
                    }
                }

                function fadeBgmTo(targetVol, duration, onDone) {
                    stopBgmFade();
                    if (!bgmAudio || duration <= 0) {
                        if (bgmAudio) bgmAudio.volume = Math.max(0, Math.min(1, targetVol));
                        if (onDone) onDone();
                        return;
                    }
                    const startVol = bgmAudio.volume;
                    const steps = Math.max(1, Math.round(duration * 30));
                    const delta = (targetVol - startVol) / steps;
                    let step = 0;
                    bgmFadeInterval = setInterval(() => {
                        step++;
                        if (step >= steps) {
                            bgmAudio.volume = Math.max(0, Math.min(1, targetVol));
                            stopBgmFade();
                            if (onDone) onDone();
                        } else {
                            bgmAudio.volume = Math.max(0, Math.min(1, startVol + delta * step));
                        }
                    }, (duration * 1000) / steps);
                }

                return {
                    playBgm(url, loop_, volume, fadeDuration) {
                        if (bgmPath === url) {
                            // same track, just update volume
                            if (bgmAudio) {
                                const v = Math.max(0, Math.min(1, volume));
                                if (fadeDuration > 0) {
                                    fadeBgmTo(v, fadeDuration);
                                } else {
                                    stopBgmFade();
                                    bgmAudio.volume = v;
                                }
                            }
                            return;
                        }
                        // different track: crossfade
                        const oldAudio = bgmAudio;
                        const newAudio = new Audio(url);
                        newAudio.loop = loop_;
                        newAudio.volume = 0;
                        bgmAudio = newAudio;
                        bgmPath = url;

                        newAudio.play().catch(e => console.warn("[audio] BGM play failed:", e));

                        const targetVol = Math.max(0, Math.min(1, volume));
                        const fd = fadeDuration > 0 ? fadeDuration : 0.5;

                        // Fade in new
                        fadeBgmTo(targetVol, fd);

                        // Fade out old
                        if (oldAudio) {
                            const oldSteps = Math.max(1, Math.round(fd * 30));
                            const oldStart = oldAudio.volume;
                            const oldDelta = oldStart / oldSteps;
                            let oldStep = 0;
                            const oldFade = setInterval(() => {
                                oldStep++;
                                if (oldStep >= oldSteps) {
                                    oldAudio.pause();
                                    oldAudio.src = "";
                                    clearInterval(oldFade);
                                } else {
                                    oldAudio.volume = Math.max(0, oldStart - oldDelta * oldStep);
                                }
                            }, (fd * 1000) / oldSteps);
                        }
                    },

                    stopBgm(fadeDuration) {
                        if (!bgmAudio) return;
                        const fd = fadeDuration > 0 ? fadeDuration : 0;
                        if (fd > 0) {
                            fadeBgmTo(0, fd, () => {
                                if (bgmAudio) { bgmAudio.pause(); bgmAudio.src = ""; }
                                bgmAudio = null;
                                bgmPath = null;
                            });
                        } else {
                            stopBgmFade();
                            bgmAudio.pause();
                            bgmAudio.src = "";
                            bgmAudio = null;
                            bgmPath = null;
                        }
                    },

                    setBgmVolume(volume) {
                        if (bgmAudio) {
                            bgmAudio.volume = Math.max(0, Math.min(1, volume));
                        }
                    },

                    playSfx(url, volume) {
                        const audio = new Audio(url);
                        audio.volume = Math.max(0, Math.min(1, volume));
                        audio.play().catch(e => console.warn("[audio] SFX play failed:", e));
                    }
                };
            })();
            console.log("[audio] JS AudioManager initialized");
        "#,
        );
    });

    // 跟踪上一帧的 BGM 状态，用于 diff
    let mut prev_bgm_path = use_signal(|| Option::<String>::None);
    let mut prev_bgm_volume = use_signal(|| 0.0f32);

    // 每帧检查音频状态变化
    let rs = render_state.read();
    let audio = &rs.audio;

    // 处理 BGM 变化
    let current_bgm = &audio.bgm;
    let transition = &audio.bgm_transition;
    let fade_duration = transition.as_ref().map(|t| t.duration).unwrap_or(0.0);

    match current_bgm {
        Some(bgm) => {
            let url = asset_url(&bgm.path);
            let prev_path = prev_bgm_path.read().clone();
            let prev_vol = *prev_bgm_volume.read();

            if prev_path.as_deref() != Some(&url) {
                // BGM changed
                let looping = bgm.looping;
                let volume = bgm.volume;
                document::eval(&format!(
                    r#"if(window.__ringAudio) window.__ringAudio.playBgm("{url}", {looping}, {volume}, {fade_duration});"#
                ));
                prev_bgm_path.set(Some(url));
                prev_bgm_volume.set(volume);
            } else if (bgm.volume - prev_vol).abs() > 0.5 {
                // Volume changed significantly
                let volume = bgm.volume;
                document::eval(&format!(
                    r#"if(window.__ringAudio) window.__ringAudio.setBgmVolume({volume});"#
                ));
                prev_bgm_volume.set(volume);
            }
        }
        None => {
            if prev_bgm_path.read().is_some() {
                // BGM stopped
                document::eval(&format!(
                    r#"if(window.__ringAudio) window.__ringAudio.stopBgm({fade_duration});"#
                ));
                prev_bgm_path.set(None);
                prev_bgm_volume.set(0.0);
            }
        }
    }

    // 处理 SFX 队列（drain 语义——每帧只出现一次）
    for sfx in &audio.sfx_queue {
        let url = asset_url(&sfx.path);
        let volume = sfx.volume;
        document::eval(&format!(
            r#"if(window.__ringAudio) window.__ringAudio.playSfx("{url}", {volume});"#
        ));
    }

    // 不渲染任何 DOM
    rsx! {}
}
