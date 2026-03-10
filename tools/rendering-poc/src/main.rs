/// PoC: winit + wgpu + egui rendering architecture
///
/// Validates:
///  1. winit window management  (full control of window & event loop)
///  2. wgpu texture rendering   (background image as full-screen GPU quad)
///  3. egui UI overlay          (dialogue box + control panel on top of wgpu)
///  4. dynamic texture updates  (simulates video-frame injection per frame)
///
/// Usage:  cargo run -p rendering-poc [path/to/background.webp]
use std::sync::Arc;
use std::time::Instant;

use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

// ── Vertex ──────────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &Self::ATTRS,
    };
}

#[rustfmt::skip]
const FULLSCREEN_QUAD: [Vertex; 6] = [
    Vertex { pos: [-1.0, -1.0], uv: [0.0, 1.0] },
    Vertex { pos: [ 1.0, -1.0], uv: [1.0, 1.0] },
    Vertex { pos: [ 1.0,  1.0], uv: [1.0, 0.0] },
    Vertex { pos: [-1.0, -1.0], uv: [0.0, 1.0] },
    Vertex { pos: [ 1.0,  1.0], uv: [1.0, 0.0] },
    Vertex { pos: [-1.0,  1.0], uv: [0.0, 0.0] },
];

// ── WGSL Shader ─────────────────────────────────────────────────────────────

const WGSL: &str = "
struct V { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };
@vertex fn vs(@location(0) p: vec2<f32>, @location(1) uv: vec2<f32>) -> V {
    return V(vec4(p, 0.0, 1.0), uv);
}
@group(0) @binding(0) var tex: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@fragment fn fs(v: V) -> @location(0) vec4<f32> {
    return textureSample(tex, samp, v.uv);
}
";

// ── Helpers ─────────────────────────────────────────────────────────────────

fn make_bind_group(
    dev: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    dev.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn upload_texture(
    dev: &wgpu::Device,
    queue: &wgpu::Queue,
    w: u32,
    h: u32,
    data: &[u8],
    extra_usage: wgpu::TextureUsages,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let tex = dev.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | extra_usage,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        data,
    );
    let view = tex.create_view(&Default::default());
    (tex, view)
}

// ── Gpu state ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
struct Gpu {
    window: Arc<Window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_cfg: wgpu::SurfaceConfiguration,

    quad_pipeline: wgpu::RenderPipeline,
    quad_vbuf: wgpu::Buffer,
    bind_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    bg_bind: wgpu::BindGroup,

    dyn_tex: wgpu::Texture,
    dyn_bind: wgpu::BindGroup,
    dyn_w: u32,
    dyn_h: u32,

    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    t0: Instant,
    show_dynamic: bool,
}

impl Gpu {
    fn init(window: Arc<Window>, bg_path: &str) -> Self {
        let size = window.inner_size();

        // ── wgpu bootstrap ──
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("no suitable GPU adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("device creation failed");

        let surface_cfg = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface format unsupported");
        surface.configure(&device, &surface_cfg);

        // ── shader + pipeline ──
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(WGSL.into()),
        });

        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
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
            label: None,
            bind_group_layouts: &[&bind_layout],
            push_constant_ranges: &[],
        });

        let quad_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipe_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[Vertex::LAYOUT],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_cfg.format,
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

        let quad_vbuf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&FULLSCREEN_QUAD),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // ── background texture ──
        let bg_img = image::open(bg_path).expect("cannot open background image");
        let bg_rgba = bg_img.to_rgba8();
        let (bw, bh) = (bg_rgba.width(), bg_rgba.height());
        let (_, bg_view) = upload_texture(
            &device,
            &queue,
            bw,
            bh,
            &bg_rgba,
            wgpu::TextureUsages::empty(),
            "bg",
        );
        let bg_bind = make_bind_group(&device, &bind_layout, &bg_view, &sampler);

        // ── dynamic texture (simulates video frames) ──
        let (dw, dh) = (640u32, 480);
        let zeros = vec![0u8; (dw * dh * 4) as usize];
        let (dyn_tex, dyn_view) = upload_texture(
            &device,
            &queue,
            dw,
            dh,
            &zeros,
            wgpu::TextureUsages::COPY_DST,
            "dynamic",
        );
        let dyn_bind = make_bind_group(&device, &bind_layout, &dyn_view, &sampler);

        // ── egui ──
        let egui_ctx = egui::Context::default();

        // Load CJK font — egui's built-in font has no Chinese glyphs
        let font_path = "assets/fonts/simhei.ttf";
        if let Ok(font_data) = std::fs::read(font_path) {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert(
                "simhei".to_owned(),
                egui::FontData::from_owned(font_data).into(),
            );
            fonts
                .families
                .get_mut(&egui::FontFamily::Proportional)
                .unwrap()
                .insert(0, "simhei".to_owned());
            fonts
                .families
                .get_mut(&egui::FontFamily::Monospace)
                .unwrap()
                .insert(0, "simhei".to_owned());
            egui_ctx.set_fonts(fonts);
        } else {
            eprintln!("warning: cannot load {font_path}, Chinese text will be broken");
        }

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui_ctx.viewport_id(),
            &window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_cfg.format, None, 1, false);

        Self {
            window,
            device,
            queue,
            surface,
            surface_cfg,
            quad_pipeline,
            quad_vbuf,
            bind_layout,
            sampler,
            bg_bind,
            dyn_tex,
            dyn_bind,
            dyn_w: dw,
            dyn_h: dh,
            egui_ctx,
            egui_state,
            egui_renderer,
            t0: Instant::now(),
            show_dynamic: false,
        }
    }

    fn on_resize(&mut self, s: winit::dpi::PhysicalSize<u32>) {
        if s.width > 0 && s.height > 0 {
            self.surface_cfg.width = s.width;
            self.surface_cfg.height = s.height;
            self.surface.configure(&self.device, &self.surface_cfg);
        }
    }

    /// Writes procedurally-generated pixel data into the dynamic texture,
    /// simulating what a video decoder would do each frame.
    fn write_video_frame(&self) {
        let (w, h) = (self.dyn_w, self.dyn_h);
        let t = self.t0.elapsed().as_secs_f32();
        let mut px = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);
                px[i] = ((fx * 255.0 + t * 60.0) % 256.0) as u8;
                px[i + 1] = ((fy * 255.0 + t * 40.0) % 256.0) as u8;
                px[i + 2] = (((fx + fy) * 128.0 + t * 80.0) % 256.0) as u8;
                px[i + 3] = 255;
            }
        }
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.dyn_tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &px,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(w * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    fn draw_frame(&mut self) {
        if self.show_dynamic {
            self.write_video_frame();
        }

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => {
                self.surface.configure(&self.device, &self.surface_cfg);
                return;
            }
        };
        let target_view = frame.texture.create_view(&Default::default());

        // ── egui UI ──
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let mut show = self.show_dynamic;
        let elapsed = self.t0.elapsed().as_secs_f32();

        let full_output = self.egui_ctx.run(raw_input, |ctx| {
            // Dialogue box (bottom panel)
            egui::TopBottomPanel::bottom("dialogue")
                .min_height(100.0)
                .frame(
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_premultiplied(15, 15, 35, 220))
                        .inner_margin(16.0),
                )
                .show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 200, 255),
                        egui::RichText::new("[子文]").size(20.0).strong(),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        egui::RichText::new(concat!(
                            "winit 管理窗口, wgpu 渲染背景纹理, egui 绘制对话框叠加层。\n",
                            "视频帧可作为 wgpu 纹理动态上传, 与渲染管线无缝集成。"
                        ))
                        .size(16.0)
                        .color(egui::Color32::WHITE),
                    );
                });

            // Control panel
            egui::Window::new("PoC 控制面板")
                .default_pos([16.0, 16.0])
                .show(ctx, |ui| {
                    ui.heading("方案 A: winit + wgpu + egui");
                    ui.separator();
                    ui.checkbox(&mut show, "启用动态纹理 (模拟视频帧)");
                    ui.label(format!("已运行 {elapsed:.1}s"));
                });
        });
        self.show_dynamic = show;

        // ── egui tessellation ──
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.surface_cfg.width, self.surface_cfg.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        let primitives = self
            .egui_ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_renderer
                .update_texture(&self.device, &self.queue, *id, delta);
        }

        // ── GPU commands ──
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.egui_renderer.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &primitives,
            &screen,
        );

        {
            let rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            // Release encoder borrow so we can call encoder.finish() later
            let mut rpass = rpass.forget_lifetime();

            // 1) Background / dynamic texture as full-screen quad
            let bind = if self.show_dynamic {
                &self.dyn_bind
            } else {
                &self.bg_bind
            };
            rpass.set_pipeline(&self.quad_pipeline);
            rpass.set_bind_group(0, bind, &[]);
            rpass.set_vertex_buffer(0, self.quad_vbuf.slice(..));
            rpass.draw(0..6, 0..1);

            // 2) egui overlay
            self.egui_renderer.render(&mut rpass, &primitives, &screen);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);
    }
}

// ── Application ─────────────────────────────────────────────────────────────

struct App {
    gpu: Option<Gpu>,
    bg_path: String,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.gpu.is_some() {
            return;
        }
        let win = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("Ring-rs PoC: winit + wgpu + egui")
                    .with_inner_size(LogicalSize::new(1280u32, 960)),
            )
            .unwrap(),
        );
        self.gpu = Some(Gpu::init(win, &self.bg_path));
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(g) = &mut self.gpu else { return };

        if g.egui_state.on_window_event(&g.window, &event).consumed {
            g.window.request_redraw();
            return;
        }

        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => g.on_resize(s),
            WindowEvent::RedrawRequested => {
                g.draw_frame();
                g.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();

    let bg = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "assets/backgrounds/高中教室/bg001_sashool_h_19201440.webp".into());
    println!("Ring-rs rendering PoC  |  bg = {bg}");

    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut App {
        gpu: None,
        bg_path: bg,
    })
    .unwrap();
}
