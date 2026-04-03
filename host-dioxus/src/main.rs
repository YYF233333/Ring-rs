#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::time::Duration;

use dioxus::desktop::Config;
use dioxus::desktop::tao::dpi::LogicalSize;
use dioxus::desktop::tao::window::WindowBuilder;
use dioxus::desktop::wry::http;
use dioxus::prelude::*;

// ---------------------------------------------------------------------------
// CSS (inline to avoid external file loading issues)
// ---------------------------------------------------------------------------

const INLINE_CSS: &str = r#"
* { margin: 0; padding: 0; box-sizing: border-box; }
body { background: #1a1a2e; color: #eee; font-family: sans-serif; overflow-y: auto; }
.poc-container { padding: 20px; max-width: 1200px; margin: 0 auto; }
.demo-section {
    margin: 20px 0; padding: 16px;
    border: 1px solid #333; border-radius: 8px;
    background: rgba(255,255,255,0.03);
}
h1 { margin-bottom: 16px; }
h2 { margin-bottom: 12px; font-size: 1.2em; color: #aaa; }
button {
    padding: 8px 16px; margin: 8px 4px;
    cursor: pointer; border: 1px solid #555;
    background: #2a2a4e; color: #eee; border-radius: 4px;
}
button:hover { background: #3a3a6e; }

.fade-box {
    width: 200px; height: 100px; background: #e94560;
    display: flex; align-items: center; justify-content: center;
    transition: opacity 0.5s ease, transform 0.5s ease;
    margin: 12px 0; border-radius: 4px;
}
.fade-box.visible { opacity: 1; transform: scale(1); }
.fade-box.hidden { opacity: 0; transform: scale(0.8); }

.pulse-box {
    width: 200px; height: 100px; background: #0f3460;
    display: flex; align-items: center; justify-content: center;
    margin: 12px 0; border-radius: 4px;
    animation: pulse 2s ease-in-out infinite;
}
@keyframes pulse {
    0%, 100% { transform: scale(1); opacity: 1; }
    50% { transform: scale(1.05); opacity: 0.7; }
}

.scene-container {
    position: relative; width: 640px; height: 480px;
    overflow: hidden; border-radius: 8px; margin: 12px 0;
    background: #111;
}
.scene-bg { width: 100%; height: 100%; object-fit: cover; }
.dialogue-box {
    position: absolute; bottom: 0; left: 0; right: 0;
    background: rgba(0, 0, 0, 0.75); padding: 16px;
    cursor: pointer; min-height: 80px;
    font-size: 18px; line-height: 1.6;
}
.typing-indicator { animation: blink 0.8s infinite; color: #888; }
@keyframes blink { 0%, 100% { opacity: 1; } 50% { opacity: 0; } }
.tick-counter { position: absolute; top: 8px; right: 8px; font-size: 12px; color: #888; }

canvas { border: 1px solid #333; display: block; margin: 8px 0; }
video { border-radius: 4px; margin: 8px 0; }
.debug-log { font-size: 11px; color: #666; font-family: monospace; margin: 4px 0; word-break: break-all; }
"#;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let css_head = format!("<style>{INLINE_CSS}</style>");

    dioxus::LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title("Ring Engine - Dioxus PoC")
                        .with_inner_size(LogicalSize::new(1280, 900)),
                )
                .with_custom_head(css_head)
                .with_custom_protocol("ring-asset", ring_asset_handler),
        )
        .launch(App);
}

// ---------------------------------------------------------------------------
// Checklist #3: ring-asset custom protocol
// ---------------------------------------------------------------------------

fn ring_asset_handler(
    _id: dioxus::desktop::wry::WebViewId,
    request: http::Request<Vec<u8>>,
) -> http::Response<Cow<'static, [u8]>> {
    let uri = request.uri().to_string();
    let raw_path = request.uri().path();
    let path_clean = percent_decode(raw_path.trim_start_matches('/'));
    let assets_root = find_assets_root();
    let full_path = assets_root.join(&path_clean);

    eprintln!("[ring-asset] URI={uri}  path={raw_path}  resolved={}", full_path.display());

    let mime = guess_mime(&path_clean);

    match std::fs::read(&full_path) {
        Ok(bytes) => {
            eprintln!("[ring-asset] OK: {} ({} bytes, {mime})", path_clean, bytes.len());
            http::Response::builder()
                .status(200)
                .header("Content-Type", mime)
                .header("Access-Control-Allow-Origin", "*")
                .body(Cow::from(bytes))
                .unwrap()
        }
        Err(e) => {
            eprintln!("[ring-asset] FAIL: {path_clean} -> {e}");
            http::Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(Cow::from(format!("Not Found: {path_clean}").into_bytes()))
                .unwrap()
        }
    }
}

fn percent_decode(input: &str) -> String {
    let mut out = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(&input[i + 1..i + 3], 16) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| input.to_string())
}

fn find_assets_root() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_default();
    let mut dir: &Path = &cwd;
    loop {
        let candidate = dir.join("assets");
        if candidate.is_dir() {
            return candidate;
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }
    cwd.join("assets")
}

fn guess_mime(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webm" => "video/webm",
        "mp4" => "video/mp4",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "css" => "text/css",
        "js" => "application/javascript",
        "json" => "application/json",
        "html" => "text/html",
        _ => "application/octet-stream",
    }
}

// ---------------------------------------------------------------------------
// Asset URL helper — try both formats for Windows compatibility
// ---------------------------------------------------------------------------

/// On Windows, wry custom protocols use `http://{name}.localhost/` format.
fn asset_url(path: &str) -> String {
    format!("http://ring-asset.localhost/{path}")
}

// ---------------------------------------------------------------------------
// Checklist #8: Signal-driven state (simulated AppStateInner)
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct PocState {
    tick_count: u32,
    background: &'static str,
    dialogue_text: &'static str,
    visible_chars: usize,
    is_typing: bool,
}

impl PocState {
    fn new() -> Self {
        Self {
            tick_count: 0,
            background: "backgrounds/BG12_pl_n_19201440.jpg",
            dialogue_text: "Signal 驱动的打字机效果。点击对话框可以完成/重启打字。每 tick 推进一个字符，计数器在右上角。",
            visible_chars: 0,
            is_typing: true,
        }
    }

    fn process_tick(&mut self) {
        self.tick_count += 1;
        if self.is_typing {
            let total = self.dialogue_text.chars().count();
            if self.visible_chars < total {
                self.visible_chars += 1;
            } else {
                self.is_typing = false;
            }
        }
    }

    fn process_click(&mut self) {
        if self.is_typing {
            self.visible_chars = self.dialogue_text.chars().count();
            self.is_typing = false;
        } else {
            self.visible_chars = 0;
            self.is_typing = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Root component
// ---------------------------------------------------------------------------

fn App() -> Element {
    let mut state = use_signal(|| PocState::new());

    // Tick loop ~30fps
    use_hook(|| {
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(33)).await;
                state.write().process_tick();
            }
        });
    });

    let s = state.read();
    let visible_text: String = s.dialogue_text.chars().take(s.visible_chars).collect();
    let bg_url = asset_url(s.background);

    rsx! {
        div { class: "poc-container",
            h1 { "Ring Engine - Dioxus PoC" }

            // --- Checklist #3 + #8 ---
            div { class: "demo-section",
                h2 { "#3 Custom Protocol + #8 Signal State" }
                p { class: "debug-log", "img src: {bg_url}" }
                div { class: "scene-container",
                    img {
                        src: "{bg_url}",
                        class: "scene-bg",
                        onerror: |_| { eprintln!("[PoC] img onerror fired — protocol URL may be wrong"); },
                    }
                    div { class: "tick-counter", "tick: {s.tick_count}" }
                    div {
                        class: "dialogue-box",
                        onclick: move |_| state.write().process_click(),
                        "{visible_text}"
                        if s.is_typing {
                            span { class: "typing-indicator", " ..." }
                        }
                    }
                }
            }

            CssTransitionDemo {}
            WebGlDemo {}
            RuleTransitionDemo {}
            VideoDemo {}
        }
    }
}

// ---------------------------------------------------------------------------
// Checklist #4: CSS Transition / Animation
// ---------------------------------------------------------------------------

#[component]
fn CssTransitionDemo() -> Element {
    let mut visible = use_signal(|| true);
    let class_name = if visible() { "fade-box visible" } else { "fade-box hidden" };

    rsx! {
        div { class: "demo-section",
            h2 { "#4 CSS Transition & Animation" }
            button {
                onclick: move |_| {
                    let v = visible();
                    visible.set(!v);
                },
                "Toggle Fade (visible={visible})"
            }
            div {
                class: "{class_name}",
                "CSS Transition"
            }
            div { class: "pulse-box", "CSS @keyframes" }
        }
    }
}

// ---------------------------------------------------------------------------
// Checklist #5: WebGL 2.0 Shader
// ---------------------------------------------------------------------------

#[component]
fn WebGlDemo() -> Element {
    use_effect(|| {
        document::eval(r#"
            (function() {
                const canvas = document.getElementById("webgl-demo");
                if (!canvas) { console.error("canvas not found"); return; }
                const gl = canvas.getContext("webgl2");
                if (!gl) { console.error("WebGL2 not supported"); return; }

                const vs = gl.createShader(gl.VERTEX_SHADER);
                gl.shaderSource(vs, `#version 300 es
                    in vec2 a_pos;
                    out vec2 v_uv;
                    void main() {
                        gl_Position = vec4(a_pos, 0.0, 1.0);
                        v_uv = a_pos * 0.5 + 0.5;
                    }
                `);
                gl.compileShader(vs);

                const fs = gl.createShader(gl.FRAGMENT_SHADER);
                gl.shaderSource(fs, `#version 300 es
                    precision mediump float;
                    in vec2 v_uv;
                    out vec4 fragColor;
                    uniform float u_time;
                    void main() {
                        fragColor = vec4(
                            v_uv.x,
                            v_uv.y,
                            0.5 + 0.5 * sin(u_time),
                            1.0
                        );
                    }
                `);
                gl.compileShader(fs);

                const prog = gl.createProgram();
                gl.attachShader(prog, vs);
                gl.attachShader(prog, fs);
                gl.linkProgram(prog);
                gl.useProgram(prog);

                const buf = gl.createBuffer();
                gl.bindBuffer(gl.ARRAY_BUFFER, buf);
                gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
                    -1,-1, 1,-1, -1,1, 1,1
                ]), gl.STATIC_DRAW);
                const loc = gl.getAttribLocation(prog, "a_pos");
                gl.enableVertexAttribArray(loc);
                gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);

                const timeLoc = gl.getUniformLocation(prog, "u_time");
                const start = performance.now();
                function frame() {
                    const t = (performance.now() - start) / 1000.0;
                    gl.uniform1f(timeLoc, t);
                    gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
                    requestAnimationFrame(frame);
                }
                frame();
                console.log("[PoC] WebGL 2.0 shader OK");
            })();
        "#);
    });

    rsx! {
        div { class: "demo-section",
            h2 { "#5 WebGL 2.0 Shader" }
            p { "Color-shifting gradient via GLSL:" }
            canvas { id: "webgl-demo", width: "320", height: "240" }
        }
    }
}

// ---------------------------------------------------------------------------
// Checklist #6: RuleTransition (WebGL mask via eval bridge)
// ---------------------------------------------------------------------------

#[component]
fn RuleTransitionDemo() -> Element {
    let mut progress = use_signal(|| 0.0f32);
    let mut animating = use_signal(|| false);

    let mask_url = asset_url("backgrounds/rule_10.png");

    // Initialize WebGL + mask texture + JS-side render loop on mount
    use_effect({
        let mask_url = mask_url.clone();
        move || {
            document::eval(&format!(r#"
                (function() {{
                    const canvas = document.getElementById("rule-canvas");
                    if (!canvas) return;
                    const gl = canvas.getContext("webgl2", {{ premultipliedAlpha: false }});
                    if (!gl) {{ console.error("WebGL2 not supported for rule"); return; }}

                    const vs = gl.createShader(gl.VERTEX_SHADER);
                    gl.shaderSource(vs, `#version 300 es
                        in vec2 a_pos;
                        out vec2 v_uv;
                        void main() {{
                            gl_Position = vec4(a_pos, 0.0, 1.0);
                            v_uv = a_pos * 0.5 + 0.5;
                            v_uv.y = 1.0 - v_uv.y;
                        }}
                    `);
                    gl.compileShader(vs);
                    if (!gl.getShaderParameter(vs, gl.COMPILE_STATUS))
                        console.error("VS:", gl.getShaderInfoLog(vs));

                    const fs = gl.createShader(gl.FRAGMENT_SHADER);
                    gl.shaderSource(fs, `#version 300 es
                        precision mediump float;
                        in vec2 v_uv;
                        out vec4 fragColor;
                        uniform sampler2D u_mask;
                        uniform float u_progress;
                        void main() {{
                            float m = texture(u_mask, v_uv).r;
                            float edge = smoothstep(u_progress - 0.08, u_progress + 0.08, m);
                            // Black overlay: edge=1 means opaque black, edge=0 means transparent
                            fragColor = vec4(0.0, 0.0, 0.0, edge);
                        }}
                    `);
                    gl.compileShader(fs);
                    if (!gl.getShaderParameter(fs, gl.COMPILE_STATUS))
                        console.error("FS:", gl.getShaderInfoLog(fs));

                    const prog = gl.createProgram();
                    gl.attachShader(prog, vs);
                    gl.attachShader(prog, fs);
                    gl.linkProgram(prog);
                    if (!gl.getProgramParameter(prog, gl.LINK_STATUS))
                        console.error("Link:", gl.getProgramInfoLog(prog));
                    gl.useProgram(prog);

                    const buf = gl.createBuffer();
                    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
                    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
                        -1,-1, 1,-1, -1,1, 1,1
                    ]), gl.STATIC_DRAW);
                    const posLoc = gl.getAttribLocation(prog, "a_pos");
                    gl.enableVertexAttribArray(posLoc);
                    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

                    const progressLoc = gl.getUniformLocation(prog, "u_progress");
                    gl.enable(gl.BLEND);
                    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

                    // Global progress variable set by Rust via eval
                    window.__ruleProgress = 0.0;
                    window.__ruleReady = false;

                    // Load mask texture
                    const img = new Image();
                    img.crossOrigin = "anonymous";
                    img.src = "{mask_url}";
                    img.onload = () => {{
                        gl.activeTexture(gl.TEXTURE0);
                        const tex = gl.createTexture();
                        gl.bindTexture(gl.TEXTURE_2D, tex);
                        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, img);
                        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
                        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
                        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
                        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
                        gl.uniform1i(gl.getUniformLocation(prog, "u_mask"), 0);
                        window.__ruleReady = true;
                        console.log("[PoC] Rule mask loaded OK");
                    }};
                    img.onerror = (e) => {{
                        console.error("[PoC] Mask load failed:", "{mask_url}", e);
                    }};

                    // JS-side render loop: reads window.__ruleProgress and draws
                    function renderLoop() {{
                        if (window.__ruleReady) {{
                            gl.uniform1f(progressLoc, window.__ruleProgress);
                            gl.clear(gl.COLOR_BUFFER_BIT);
                            gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
                        }}
                        requestAnimationFrame(renderLoop);
                    }}
                    renderLoop();
                    console.log("[PoC] RuleTransition WebGL init OK");
                }})();
            "#));
        }
    });

    // Rust-side animation: updates window.__ruleProgress via eval
    use_hook(|| {
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(16)).await;
                if !animating() {
                    continue;
                }
                let p = progress() + 1.0 / 60.0;
                if p >= 1.0 {
                    progress.set(1.0);
                    animating.set(false);
                } else {
                    progress.set(p);
                }
                let p_val = progress();
                document::eval(&format!("window.__ruleProgress={p_val}"));
            }
        });
    });

    rsx! {
        div { class: "demo-section",
            h2 { "#6 RuleTransition (WebGL Mask via Eval Bridge)" }
            p { "Mask: rule_10.png | Progress: {progress:.2}" }
            p { class: "debug-log", "mask src: {mask_url}" }
            button {
                onclick: move |_| {
                    progress.set(0.0);
                    animating.set(true);
                },
                "Start Rule Transition"
            }
            canvas { id: "rule-canvas", width: "480", height: "360" }
        }
    }
}

// ---------------------------------------------------------------------------
// Checklist #7: HTML5 Video
// ---------------------------------------------------------------------------

#[component]
fn VideoDemo() -> Element {
    let mut playing = use_signal(|| false);
    let video_url = asset_url("audio/ending_HVC_bgm.webm");

    rsx! {
        div { class: "demo-section",
            h2 { "#7 HTML5 Video Playback" }
            p { class: "debug-log", "video src: {video_url}" }
            button {
                onclick: move |_| playing.set(!playing()),
                if playing() { "Hide Video" } else { "Show Video" }
            }
            if playing() {
                video {
                    src: "{video_url}",
                    autoplay: true,
                    controls: true,
                    width: "480",
                    height: "270",
                    onended: move |_| playing.set(false),
                }
            }
        }
    }
}
