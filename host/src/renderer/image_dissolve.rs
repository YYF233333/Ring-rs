//! # ImageDissolve 模块
//!
//! 实现类似 Ren'Py 的 ImageDissolve 效果。
//! 使用灰度遮罩图，根据像素亮度控制溶解顺序。

use macroquad::prelude::*;

/// ImageDissolve 顶点 shader
const VERTEX_SHADER: &str = r#"
#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;

/// ImageDissolve 片段 shader
/// 
/// 根据灰度遮罩图的像素亮度控制溶解顺序：
/// - progress: 过渡进度 (0.0 - 1.0)
/// - ramp: 渐变带宽，控制边缘柔和度
/// - reversed: 是否反转遮罩
const FRAGMENT_SHADER: &str = r#"
#version 100
precision mediump float;

varying vec2 uv;
varying vec4 color;

uniform sampler2D Texture;        // 新背景纹理
uniform sampler2D _mask_texture;  // 灰度遮罩纹理
uniform sampler2D _old_texture;   // 旧背景纹理
uniform float _progress;          // 过渡进度 (0.0 - 1.0)
uniform float _ramp;              // 渐变带宽
uniform float _reversed;          // 是否反转 (0.0 = 正常, 1.0 = 反转)

void main() {
    // 采样新背景
    vec4 newColor = texture2D(Texture, uv);
    
    // 采样旧背景
    vec4 oldColor = texture2D(_old_texture, uv);
    
    // 采样遮罩图（灰度）
    vec4 maskColor = texture2D(_mask_texture, uv);
    float maskValue = maskColor.r; // 使用红色通道作为亮度值
    
    // 如果反转，翻转遮罩值
    if (_reversed > 0.5) {
        maskValue = 1.0 - maskValue;
    }
    
    // 计算混合因子
    // 原理：progress 从 0 到 1 变化
    // 当 maskValue <= progress 时，显示新内容（factor = 1）
    float factor;
    if (_ramp > 0.001) {
        // 带渐变的溶解 - 使用 smoothstep 产生平滑过渡边缘
        float lower = _progress - _ramp * 0.5;
        float upper = _progress + _ramp * 0.5;
        factor = 1.0 - smoothstep(lower, upper, maskValue);
    } else {
        // 硬边溶解：maskValue <= progress 显示新内容
        factor = step(maskValue, _progress);
    }
    
    // 直接在 shader 中混合新旧背景（不依赖 alpha blending）
    gl_FragColor = mix(oldColor, newColor, factor);
}
"#;

/// ImageDissolve 效果管理器
pub struct ImageDissolve {
    /// shader 材质
    material: Option<Material>,
    /// 渐变带宽（默认 0.1）
    ramp: f32,
}

impl ImageDissolve {
    /// 创建新的 ImageDissolve 效果管理器
    pub fn new() -> Self {
        Self {
            material: None,
            ramp: 0.0, // 默认硬边（更符合 rule 灰度图的预期）
        }
    }

    /// 初始化 shader
    pub fn init(&mut self) -> Result<(), String> {
        match load_material(
            ShaderSource::Glsl {
                vertex: VERTEX_SHADER,
                fragment: FRAGMENT_SHADER,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("_progress", UniformType::Float1),
                    UniformDesc::new("_ramp", UniformType::Float1),
                    UniformDesc::new("_reversed", UniformType::Float1),
                ],
                textures: vec![
                    "_mask_texture".to_string(),
                    "_old_texture".to_string(),
                ],
                ..Default::default()
            },
        ) {
            Ok(material) => {
                self.material = Some(material);
                println!("✅ ImageDissolve shader 初始化成功");
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ ImageDissolve shader 初始化失败: {}", e);
                Err(format!("Shader 初始化失败: {}", e))
            }
        }
    }

    /// 设置渐变带宽
    pub fn set_ramp(&mut self, ramp: f32) {
        self.ramp = ramp.clamp(0.0, 1.0);
    }

    /// 绘制带 ImageDissolve 效果的纹理
    ///
    /// # 参数
    /// - `new_texture`: 新背景纹理
    /// - `old_texture`: 旧背景纹理
    /// - `mask_texture`: 灰度遮罩纹理
    /// - `progress`: 过渡进度 (0.0 - 1.0)
    /// - `reversed`: 是否反转遮罩
    /// - `dest_rect`: 目标矩形 (x, y, width, height)
    pub fn draw(
        &mut self,
        new_texture: &Texture2D,
        old_texture: &Texture2D,
        mask_texture: &Texture2D,
        progress: f32,
        reversed: bool,
        dest_rect: (f32, f32, f32, f32),
    ) {
        let (x, y, w, h) = dest_rect;

        if let Some(material) = &self.material {
            // 设置 shader uniforms
            material.set_uniform("_progress", progress);
            material.set_uniform("_ramp", self.ramp);
            material.set_uniform("_reversed", if reversed { 1.0f32 } else { 0.0f32 });
            // 设置纹理
            material.set_texture("_mask_texture", mask_texture.clone());
            material.set_texture("_old_texture", old_texture.clone());

            // 使用自定义材质绘制
            gl_use_material(material);
            draw_texture_ex(
                new_texture,
                x,
                y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(w, h)),
                    ..Default::default()
                },
            );
            gl_use_default_material();
        } else {
            panic!("ImageDissolve shader 未初始化");
        }
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.material.is_some()
    }
}
impl Default for ImageDissolve {
    fn default() -> Self {
        Self::new()
    }
}
