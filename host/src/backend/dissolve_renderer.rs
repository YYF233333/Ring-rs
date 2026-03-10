//! ImageDissolve 渲染器
//!
//! 基于 wgpu 的遮罩溶解效果，用于 Rule 场景过渡。
//!
//! ## 原理
//!
//! 通过灰度遮罩图控制每个像素的溶解顺序：
//! - `progress` 从 0→1，mask 值 ≤ progress 的像素显示 overlay 色
//! - `ramp > 0` 时 smoothstep 产生柔和过渡边缘
//! - `reversed` 反转遮罩方向

use std::mem::size_of;

use wgpu::util::DeviceExt;

use super::gpu_texture::GpuTexture;

// ── Uniform 数据 ─────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DissolveUniforms {
    projection: [f32; 16],
    progress: f32,
    ramp: f32,
    reversed: f32,
    _pad: f32,
    overlay_color: [f32; 4],
}

// ── WGSL Shader ──────────────────────────────────────────────────────────────

const DISSOLVE_WGSL: &str = r"
struct Uniforms {
    projection: mat4x4<f32>,
    progress: f32,
    ramp: f32,
    reversed: f32,
    _pad: f32,
    overlay_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

@vertex fn vs(
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) _color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = u.projection * vec4(pos, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@group(1) @binding(0) var mask_tex: texture_2d<f32>;
@group(1) @binding(1) var mask_samp: sampler;

@fragment fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    let mask_raw = textureSample(mask_tex, mask_samp, in.uv).r;

    var mask_value = mask_raw;
    if u.reversed > 0.5 {
        mask_value = 1.0 - mask_value;
    }

    var factor: f32;
    if u.ramp > 0.001 {
        let lower = u.progress - u.ramp * 0.5;
        let upper = u.progress + u.ramp * 0.5;
        factor = 1.0 - smoothstep(lower, upper, mask_value);
    } else {
        factor = step(mask_value, u.progress);
    }

    return vec4(u.overlay_color.rgb, u.overlay_color.a * factor);
}
";

// ── 顶点定义（与 SpriteVertex 布局一致） ────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DissolveVertex {
    pos: [f32; 2],
    uv: [f32; 2],
    _color: [f32; 4],
}

impl DissolveVertex {
    const ATTRS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &Self::ATTRS,
    };
}

// ── DissolveRenderer ─────────────────────────────────────────────────────────

/// 遮罩溶解渲染器
///
/// 绘制一个全屏 quad，通过灰度遮罩图控制 overlay 颜色的逐像素 alpha。
/// 用于 Rule 场景过渡的 FadeIn/FadeOut 阶段。
pub struct DissolveRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    vertex_buffer: wgpu::Buffer,
}

impl DissolveRenderer {
    /// 创建 DissolveRenderer
    ///
    /// `texture_bind_group_layout` 必须与 `SpriteRenderer` 共享，
    /// 以保证 `GpuTexture::bind_group` 在两个管线间通用。
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("dissolve_shader"),
            source: wgpu::ShaderSource::Wgsl(DISSOLVE_WGSL.into()),
        });

        // Group 0: dissolve uniforms (projection + params)
        let uniform_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("dissolve_uniform_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("dissolve_pipe_layout"),
            bind_group_layouts: &[&uniform_layout, texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("dissolve_pipeline"),
            layout: Some(&pipe_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[DissolveVertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });

        // Uniform buffer
        let initial = DissolveUniforms {
            projection: [0.0; 16],
            progress: 0.0,
            ramp: 0.0,
            reversed: 0.0,
            _pad: 0.0,
            overlay_color: [0.0, 0.0, 0.0, 1.0],
        };
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("dissolve_uniform_buf"),
            contents: bytemuck::bytes_of(&initial),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("dissolve_uniform_bg"),
            layout: &uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Vertex buffer for a single quad (6 vertices)
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("dissolve_vbuf"),
            size: (6 * size_of::<DissolveVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
        }
    }

    /// 绘制一次溶解叠加
    ///
    /// 在当前 render pass 中绘制一个带遮罩 alpha 的全屏 quad。
    /// `mask` 的 `bind_group` 必须与 `texture_bind_group_layout` 兼容。
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        mask: &GpuTexture,
        screen_width: f32,
        screen_height: f32,
        progress: f32,
        ramp: f32,
        reversed: bool,
        overlay_color: [f32; 4],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let projection = orthographic_projection(screen_width, screen_height);
        let uniforms = DissolveUniforms {
            projection,
            progress,
            ramp,
            reversed: if reversed { 1.0 } else { 0.0 },
            _pad: 0.0,
            overlay_color,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let vertices = quad_vertices(x, y, width, height);
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        pass.set_bind_group(1, &mask.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..6, 0..1);
    }
}

// ── 辅助函数 ────────────────────────────────────────────────────────────────

fn orthographic_projection(width: f32, height: f32) -> [f32; 16] {
    #[rustfmt::skip]
    let m = [
        2.0 / width,   0.0,            0.0,  0.0,
        0.0,          -2.0 / height,    0.0,  0.0,
        0.0,           0.0,            1.0,  0.0,
       -1.0,           1.0,            0.0,  1.0,
    ];
    m
}

fn quad_vertices(x: f32, y: f32, w: f32, h: f32) -> [DissolveVertex; 6] {
    let c = [0.0f32; 4];
    let tl = DissolveVertex {
        pos: [x, y],
        uv: [0.0, 0.0],
        _color: c,
    };
    let tr = DissolveVertex {
        pos: [x + w, y],
        uv: [1.0, 0.0],
        _color: c,
    };
    let bl = DissolveVertex {
        pos: [x, y + h],
        uv: [0.0, 1.0],
        _color: c,
    };
    let br = DissolveVertex {
        pos: [x + w, y + h],
        uv: [1.0, 1.0],
        _color: c,
    };
    [tl, tr, br, tl, br, bl]
}
