use dioxus::prelude::*;

use crate::render_state::{RenderState, SceneTransitionKind, SceneTransitionPhaseState};

/// 资源 URL 构建（Windows wry 格式）
fn asset_url(path: &str) -> String {
    format!("http://ring-asset.localhost/{path}")
}

/// Rule 遮罩过渡组件：使用 WebGL shader 实现遮罩过渡。
///
/// 架构：
/// - Rust 侧计算 progress 并写入 `window.__ruleProgress`
/// - JS 侧 `requestAnimationFrame` 自主渲染循环读取 progress 绘制
///
/// 这个模式在 PoC 阶段已验证可靠。
#[component]
pub fn RuleTransitionCanvas(render_state: Signal<RenderState>) -> Element {
    let rs = render_state.read();

    let transition = match &rs.scene_transition {
        Some(t) => t,
        None => return rsx! {},
    };

    let (mask_path, _reversed, _ramp) = match &transition.transition_type {
        SceneTransitionKind::Rule {
            mask_path,
            reversed,
            ramp,
        } => (mask_path.clone(), *reversed, *ramp),
        _ => return rsx! {},
    };

    if transition.phase == SceneTransitionPhaseState::Completed {
        return rsx! {};
    }

    let mask_url = asset_url(&mask_path);

    // 初始化 WebGL（仅首次 mount）
    use_effect({
        let mask_url = mask_url.clone();
        move || {
            document::eval(&format!(
                r#"
                (function() {{
                    const canvas = document.getElementById("vn-rule-canvas");
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
                            fragColor = vec4(0.0, 0.0, 0.0, edge);
                        }}
                    `);
                    gl.compileShader(fs);

                    const prog = gl.createProgram();
                    gl.attachShader(prog, vs);
                    gl.attachShader(prog, fs);
                    gl.linkProgram(prog);
                    gl.useProgram(prog);

                    const buf = gl.createBuffer();
                    gl.bindBuffer(gl.ARRAY_BUFFER, buf);
                    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1,-1, 1,-1, -1,1, 1,1]), gl.STATIC_DRAW);
                    const posLoc = gl.getAttribLocation(prog, "a_pos");
                    gl.enableVertexAttribArray(posLoc);
                    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

                    gl.enable(gl.BLEND);
                    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

                    const progressLoc = gl.getUniformLocation(prog, "u_progress");
                    window.__ruleProgress = 0.0;
                    window.__ruleReady = false;

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
                    }};

                    function renderLoop() {{
                        if (window.__ruleReady) {{
                            gl.uniform1f(progressLoc, window.__ruleProgress);
                            gl.clear(gl.COLOR_BUFFER_BIT);
                            gl.drawArrays(gl.TRIANGLE_STRIP, 0, 4);
                        }}
                        requestAnimationFrame(renderLoop);
                    }}
                    renderLoop();
                }})();
            "#
            ));
        }
    });

    rsx! {
        canvas {
            id: "vn-rule-canvas",
            class: "vn-rule-canvas",
            width: "960",
            height: "540",
        }
    }
}
