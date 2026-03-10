//! 2D Sprite 渲染器
//!
//! 基于 wgpu 的 textured quad 渲染器，用于绘制背景、角色立绘和矩形遮罩。
//!
//! ## 渲染模型
//!
//! - 使用正交投影（像素坐标，左上角原点）
//! - 每个 sprite 生成 6 个顶点（两个三角形）
//! - 每个纹理一次 draw call
//! - 支持 alpha 混合和颜色调制

use std::sync::Arc;
use wgpu::util::DeviceExt;

use super::gpu_texture::{GpuTexture, create_gpu_texture};
use super::math::{QuadVertex, orthographic_projection, quad_vertices};
use crate::rendering_types::DrawCommand;

const MAX_SPRITES: usize = 128;
const VERTS_PER_SPRITE: usize = 6;
const MAX_VERTS: usize = MAX_SPRITES * VERTS_PER_SPRITE;

// ── WGSL Shader ─────────────────────────────────────────────────────────────

const SPRITE_WGSL: &str = r"
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

struct Projection {
    matrix: mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> proj: Projection;

@vertex fn vs(
    @location(0) pos: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = proj.matrix * vec4(pos, 0.0, 1.0);
    out.uv = uv;
    out.color = color;
    return out;
}

@group(1) @binding(0) var tex: texture_2d<f32>;
@group(1) @binding(1) var samp: sampler;

@fragment fn fs_textured(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(tex, samp, in.uv);
    return tex_color * in.color;
}

@fragment fn fs_solid(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
";

// ── 1x1 白色纹理 ──────────────────────────────────────────────────────────

const WHITE_PIXEL: [u8; 4] = [255, 255, 255, 255];

// ── SpriteRenderer ──────────────────────────────────────────────────────────

/// 2D Sprite 渲染器
pub struct SpriteRenderer {
    textured_pipeline: wgpu::RenderPipeline,
    solid_pipeline: wgpu::RenderPipeline,

    proj_buffer: wgpu::Buffer,
    proj_bind_group: wgpu::BindGroup,

    pub(crate) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) sampler: wgpu::Sampler,

    vertex_buffer: wgpu::Buffer,

    /// 1x1 白色纹理（用于纯色矩形绘制）
    white_texture: Arc<GpuTexture>,
}

impl SpriteRenderer {
    /// 创建新的 Sprite 渲染器
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite_shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_WGSL.into()),
        });

        let proj_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite_proj_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sprite_tex_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let pipe_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite_pipe_layout"),
            bind_group_layouts: &[&proj_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let make_pipeline = |entry_point: &str, label: &str| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipe_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    buffers: &[QuadVertex::LAYOUT],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry_point),
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
            })
        };

        let textured_pipeline = make_pipeline("fs_textured", "sprite_textured_pipeline");
        let solid_pipeline = make_pipeline("fs_solid", "sprite_solid_pipeline");

        let proj_data = orthographic_projection(1.0, 1.0);
        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite_proj_buf"),
            contents: bytemuck::cast_slice(&proj_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let proj_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sprite_proj_bg"),
            layout: &proj_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: proj_buffer.as_entire_binding(),
            }],
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sprite_vbuf"),
            size: (MAX_VERTS * size_of::<QuadVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let white_texture = create_gpu_texture(
            device,
            queue,
            &texture_bind_group_layout,
            &sampler,
            1,
            1,
            &WHITE_PIXEL,
            Some("white_1x1"),
        );

        Self {
            textured_pipeline,
            solid_pipeline,
            proj_buffer,
            proj_bind_group,
            texture_bind_group_layout,
            sampler,
            vertex_buffer,
            white_texture,
        }
    }

    /// 创建 GPU 纹理（从 RGBA 字节数据）
    pub fn create_texture(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        rgba_data: &[u8],
        label: Option<&str>,
    ) -> Arc<GpuTexture> {
        create_gpu_texture(
            device,
            queue,
            &self.texture_bind_group_layout,
            &self.sampler,
            width,
            height,
            rgba_data,
            label,
        )
    }

    /// 更新正交投影矩阵（窗口大小变化时调用）
    pub fn update_projection(&self, queue: &wgpu::Queue, width: f32, height: f32) {
        let proj = orthographic_projection(width, height);
        queue.write_buffer(&self.proj_buffer, 0, bytemuck::cast_slice(&proj));
    }

    /// 在 render pass 中绘制 sprites
    pub fn draw_sprites(
        &self,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'_>,
        sprites: &[DrawCommand],
    ) {
        if sprites.is_empty() {
            return;
        }

        let mut vertices: Vec<QuadVertex> = Vec::with_capacity(sprites.len() * VERTS_PER_SPRITE);
        for cmd in sprites {
            if let Some((x, y, w, h, color, uv)) = cmd.sprite_params() {
                vertices.extend_from_slice(&quad_vertices(x, y, w, h, uv, color));
            }
        }

        let byte_data = bytemuck::cast_slice(&vertices);
        queue.write_buffer(&self.vertex_buffer, 0, byte_data);

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_bind_group(0, &self.proj_bind_group, &[]);

        let mut vertex_offset: u32 = 0;
        for cmd in sprites {
            match cmd {
                DrawCommand::Sprite { texture, .. } => {
                    let gpu_tex = texture
                        .as_any()
                        .downcast_ref::<GpuTexture>()
                        .expect("WgpuBackend requires GpuTexture");
                    pass.set_pipeline(&self.textured_pipeline);
                    pass.set_bind_group(1, &gpu_tex.bind_group, &[]);
                }
                DrawCommand::Rect { .. } => {
                    pass.set_pipeline(&self.solid_pipeline);
                    pass.set_bind_group(1, &self.white_texture.bind_group, &[]);
                }
                DrawCommand::Dissolve { .. } => continue,
            }
            pass.draw(vertex_offset..vertex_offset + VERTS_PER_SPRITE as u32, 0..1);
            vertex_offset += VERTS_PER_SPRITE as u32;
        }
    }
}
