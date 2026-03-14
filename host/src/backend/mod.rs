//! # 渲染后端模块
//!
//! 基于 winit + wgpu + egui 的渲染后端实现。
//!
//! ## 架构
//!
//! - `WgpuBackend`: 渲染后端门面，编排帧渲染流程
//! - `GpuContext`: GPU 设备、队列、表面管理
//! - `EguiIntegration`: egui 输入/输出/渲染桥接
//! - `SpriteRenderer`: 2D textured quad batch 渲染器
//! - `GpuTexture`: wgpu 纹理封装

pub mod dissolve_renderer;
pub mod gpu_texture;
pub mod math;
pub mod sprite_renderer;

pub use gpu_texture::GpuTexture;
pub use sprite_renderer::SpriteRenderer;

pub use crate::rendering_types::DrawCommand;

use crate::rendering_types::{Texture, TextureFactory};
use std::sync::Arc;
use tracing::{info, warn};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

// ── WgpuTextureFactory ───────────────────────────────────────────────────────

/// wgpu 纹理工厂（[`TextureFactory`] 的 wgpu 实现）
struct WgpuTextureFactory {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    sprite_renderer: Arc<SpriteRenderer>,
}

impl TextureFactory for WgpuTextureFactory {
    fn create_texture(
        &self,
        width: u32,
        height: u32,
        rgba_data: &[u8],
        label: Option<&str>,
    ) -> Arc<dyn Texture> {
        self.sprite_renderer.create_texture(
            &self.device,
            &self.queue,
            width,
            height,
            rgba_data,
            label,
        )
    }
}

// ── GpuContext ────────────────────────────────────────────────────────────────

/// GPU 设备、队列与渲染表面
pub struct GpuContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_cfg: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    fn new(window: &Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("[GpuContext] no suitable GPU adapter found");

        info!(
            adapter = %adapter.get_info().name,
            backend = ?adapter.get_info().backend,
            "GPU adapter selected"
        );

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("[GpuContext] GPU device creation failed");
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let mut surface_cfg = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("[GpuContext] surface format unsupported");
        surface_cfg.usage |= wgpu::TextureUsages::COPY_SRC;
        surface.configure(&device, &surface_cfg);

        Self {
            device,
            queue,
            surface,
            surface_cfg,
        }
    }

    /// 当前表面尺寸（物理像素）
    pub fn size(&self) -> (u32, u32) {
        (self.surface_cfg.width, self.surface_cfg.height)
    }

    /// 表面纹理格式
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_cfg.format
    }

    /// 处理窗口大小变更
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_cfg.width = new_size.width;
            self.surface_cfg.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_cfg);
        }
    }

    fn acquire_frame(&mut self) -> Option<(wgpu::SurfaceTexture, wgpu::TextureView)> {
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&Default::default());
                Some((frame, view))
            }
            Err(e) => {
                warn!("Surface texture acquisition failed: {e}, reconfiguring");
                self.surface.configure(&self.device, &self.surface_cfg);
                None
            }
        }
    }
}

// ── EguiIntegration ──────────────────────────────────────────────────────────

/// egui 集成层（输入桥接 + UI 渲染）
pub struct EguiIntegration {
    pub ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EguiIntegration {
    fn new(
        window: &Window,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        font_data: Option<Vec<u8>>,
    ) -> Self {
        let ctx = egui::Context::default();
        Self::configure_fonts(&ctx, font_data);

        let state =
            egui_winit::State::new(ctx.clone(), ctx.viewport_id(), window, None, None, None);
        let renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1, false);

        Self {
            ctx,
            state,
            renderer,
        }
    }

    fn configure_fonts(ctx: &egui::Context, font_data: Option<Vec<u8>>) {
        let Some(data) = font_data else {
            warn!("No CJK font data provided; Chinese text will render as tofu");
            return;
        };

        let mut fonts = egui::FontDefinitions::default();
        fonts
            .font_data
            .insert("cjk".to_owned(), egui::FontData::from_owned(data).into());
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "cjk".to_owned());
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "cjk".to_owned());
        ctx.set_fonts(fonts);
    }

    /// 将 winit 窗口事件传递给 egui，返回是否已消费
    pub fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    /// 处理 egui 平台输出（光标、IME 等）
    fn handle_platform_output(&mut self, window: &Window, output: egui::PlatformOutput) {
        self.state.handle_platform_output(window, output);
    }
}

// ── WgpuBackend ──────────────────────────────────────────────────────────────

/// wgpu + egui 渲染后端
///
/// 组合 [`GpuContext`] 和 [`EguiIntegration`]，编排帧渲染流程。
pub struct WgpuBackend {
    window: Arc<Window>,
    pub gpu: GpuContext,
    pub egui: EguiIntegration,

    sprite_renderer: Arc<SpriteRenderer>,
    dissolve_renderer: dissolve_renderer::DissolveRenderer,

    frame_start: std::time::Instant,
    frame_delta: f32,

    /// 视频帧纹理（cutscene 播放时使用）
    video_texture: Option<Arc<GpuTexture>>,
    video_texture_size: (u32, u32),

    /// 截图请求标志
    screenshot_requested: bool,
    /// 上一帧截图的 RGBA 像素数据
    last_screenshot: Option<Vec<u8>>,
    last_screenshot_size: (u32, u32),
}

impl WgpuBackend {
    /// 初始化渲染后端
    pub fn new(window: Arc<Window>, font_data: Option<Vec<u8>>) -> Self {
        let gpu = GpuContext::new(&window);

        let sprite_renderer = Arc::new(SpriteRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.surface_format(),
        ));
        let (w, h) = gpu.size();
        sprite_renderer.update_projection(&gpu.queue, w as f32, h as f32);

        let dissolve_renderer = dissolve_renderer::DissolveRenderer::new(
            &gpu.device,
            gpu.surface_format(),
            &sprite_renderer.texture_bind_group_layout,
        );

        let egui = EguiIntegration::new(&window, &gpu.device, gpu.surface_format(), font_data);

        info!(
            width = w,
            height = h,
            format = ?gpu.surface_format(),
            "WgpuBackend initialized"
        );

        Self {
            window,
            gpu,
            egui,
            sprite_renderer,
            dissolve_renderer,
            frame_start: std::time::Instant::now(),
            frame_delta: 0.0,
            video_texture: None,
            video_texture_size: (0, 0),
            screenshot_requested: false,
            last_screenshot: None,
            last_screenshot_size: (0, 0),
        }
    }

    // ── 访问器 ──────────────────────────────────────────────────────────────

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.gpu.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.gpu.queue
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.gpu.surface_format()
    }

    /// 当前窗口尺寸（物理像素）
    pub fn size(&self) -> (u32, u32) {
        self.gpu.size()
    }

    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    /// 请求在下一帧渲染结束后捕获截图
    pub fn request_screenshot(&mut self) {
        self.screenshot_requested = true;
    }

    /// 取走上一次捕获的截图（RGBA 像素 + 尺寸），仅可取一次
    pub fn take_screenshot(&mut self) -> Option<(Vec<u8>, u32, u32)> {
        let data = self.last_screenshot.take()?;
        let (w, h) = self.last_screenshot_size;
        Some((data, w, h))
    }

    /// 上一帧耗时（秒）
    pub fn frame_delta(&self) -> f32 {
        self.frame_delta
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui.ctx
    }

    pub fn sprite_renderer(&self) -> &SpriteRenderer {
        &self.sprite_renderer
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// 创建纹理上下文（注入到 ResourceManager 中）
    pub fn texture_context(&self) -> crate::rendering_types::TextureContext {
        let factory = Arc::new(WgpuTextureFactory {
            device: Arc::clone(&self.gpu.device),
            queue: Arc::clone(&self.gpu.queue),
            sprite_renderer: Arc::clone(&self.sprite_renderer),
        });
        crate::rendering_types::TextureContext::new(factory)
    }

    // ── 纹理工厂 ─────────────────────────────────────────────────────────────

    /// 从 RGBA 字节数据创建 GPU 纹理
    pub fn create_texture(
        &self,
        width: u32,
        height: u32,
        rgba_data: &[u8],
        label: Option<&str>,
    ) -> Arc<GpuTexture> {
        self.sprite_renderer.create_texture(
            &self.gpu.device,
            &self.gpu.queue,
            width,
            height,
            rgba_data,
            label,
        )
    }

    // ── 事件处理 ─────────────────────────────────────────────────────────────

    /// 将 winit 窗口事件传递给 egui，返回是否已消费
    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui.handle_window_event(&self.window, event)
    }

    /// 处理窗口大小变更
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.gpu.resize(new_size);
        if new_size.width > 0 && new_size.height > 0 {
            self.sprite_renderer.update_projection(
                &self.gpu.queue,
                new_size.width as f32,
                new_size.height as f32,
            );
        }
    }

    // ── 帧渲染 ──────────────────────────────────────────────────────────────

    /// 渲染一帧
    ///
    /// 渲染顺序：Clear -> sprite 绘制 -> dissolve 效果 -> egui UI 叠加
    pub fn render_frame(
        &mut self,
        build_ui: impl FnMut(&egui::Context),
        sprite_commands: &[DrawCommand],
    ) {
        let now = std::time::Instant::now();
        self.frame_delta = now.duration_since(self.frame_start).as_secs_f32();
        self.frame_start = now;

        let Some((frame, target_view)) = self.gpu.acquire_frame() else {
            return;
        };

        // egui: 采集输入 -> 构建 UI -> 输出
        let raw_input = self.egui.state.take_egui_input(&self.window);
        let full_output = self.egui.ctx.run(raw_input, build_ui);

        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [self.gpu.surface_cfg.width, self.gpu.surface_cfg.height],
            pixels_per_point: self.window.scale_factor() as f32,
        };
        let primitives = self
            .egui
            .ctx
            .tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.egui
                .renderer
                .update_texture(&self.gpu.device, &self.gpu.queue, *id, delta);
        }

        // GPU 命令编码
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        self.egui.renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
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
            let mut rpass = rpass.forget_lifetime();

            self.sprite_renderer
                .draw_sprites(&self.gpu.queue, &mut rpass, sprite_commands);

            let (sw, sh) = self.gpu.size();
            for cmd in sprite_commands {
                if let DrawCommand::Dissolve {
                    mask_texture,
                    progress,
                    ramp,
                    reversed,
                    overlay_color,
                    x,
                    y,
                    width,
                    height,
                } = cmd
                {
                    let gpu_mask = mask_texture
                        .as_any()
                        .downcast_ref::<GpuTexture>()
                        .expect("WgpuBackend requires GpuTexture for dissolve mask");
                    self.dissolve_renderer.draw(
                        &self.gpu.queue,
                        &mut rpass,
                        gpu_mask,
                        sw as f32,
                        sh as f32,
                        *progress,
                        *ramp,
                        *reversed,
                        *overlay_color,
                        *x,
                        *y,
                        *width,
                        *height,
                    );
                }
            }

            self.egui.renderer.render(&mut rpass, &primitives, &screen);
        }

        if self.screenshot_requested {
            self.screenshot_requested = false;
            let (w, h) = self.gpu.size();
            let bytes_per_pixel = 4u32;
            let unpadded_row = w * bytes_per_pixel;
            let padded_row = (unpadded_row + 255) & !255; // wgpu requires 256-byte alignment
            let buffer_size = (padded_row * h) as u64;

            let staging = self.gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("screenshot_staging"),
                size: buffer_size,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: &frame.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &staging,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_row),
                        rows_per_image: Some(h),
                    },
                },
                wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
            );

            self.gpu.queue.submit(std::iter::once(encoder.finish()));

            let slice = staging.slice(..);
            let (tx, rx) = std::sync::mpsc::channel();
            slice.map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
            self.gpu.device.poll(wgpu::Maintain::Wait);

            if rx.recv().is_ok_and(|r| r.is_ok()) {
                let mapped = slice.get_mapped_range();
                let mut pixels = Vec::with_capacity((w * h * bytes_per_pixel) as usize);
                for row in 0..h {
                    let start = (row * padded_row) as usize;
                    let end = start + unpadded_row as usize;
                    pixels.extend_from_slice(&mapped[start..end]);
                }
                drop(mapped);

                // Surface format may be BGRA; convert to RGBA
                if self.gpu.surface_format() == wgpu::TextureFormat::Bgra8UnormSrgb
                    || self.gpu.surface_format() == wgpu::TextureFormat::Bgra8Unorm
                {
                    for chunk in pixels.chunks_exact_mut(4) {
                        chunk.swap(0, 2); // B <-> R
                    }
                }

                self.last_screenshot = Some(pixels);
                self.last_screenshot_size = (w, h);
            }
        } else {
            self.gpu.queue.submit(std::iter::once(encoder.finish()));
        }
        frame.present();

        for id in &full_output.textures_delta.free {
            self.egui.renderer.free_texture(id);
        }
        self.egui
            .handle_platform_output(&self.window, full_output.platform_output);
    }

    /// 上传视频帧 RGBA 数据到 GPU 纹理。
    ///
    /// 首次调用或分辨率变化时创建新纹理，否则就地更新像素数据。
    pub fn upload_video_frame(&mut self, data: &[u8], width: u32, height: u32) {
        if self.video_texture_size != (width, height) || self.video_texture.is_none() {
            self.video_texture = Some(gpu_texture::create_gpu_texture(
                &self.gpu.device,
                &self.gpu.queue,
                &self.sprite_renderer.texture_bind_group_layout,
                &self.sprite_renderer.sampler,
                width,
                height,
                data,
                Some("video_frame"),
            ));
            self.video_texture_size = (width, height);
            return;
        }

        if let Some(tex) = &self.video_texture {
            self.gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * width),
                    rows_per_image: Some(height),
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    /// 生成视频全屏渲染的 DrawCommand（带信箱黑边）。
    ///
    /// 保持视频宽高比，在屏幕上居中渲染，剩余区域由 clear color（黑色）填充。
    pub fn video_draw_command(&self) -> Option<DrawCommand> {
        let tex = self.video_texture.as_ref()?;
        let (sw, sh) = self.gpu.size();
        let (sw, sh) = (sw as f32, sh as f32);
        let (vw, vh) = (tex.width(), tex.height());

        let scale = (sw / vw).min(sh / vh);
        let dst_w = vw * scale;
        let dst_h = vh * scale;
        let x = (sw - dst_w) / 2.0;
        let y = (sh - dst_h) / 2.0;

        Some(DrawCommand::Sprite {
            texture: tex.clone(),
            x,
            y,
            width: dst_w,
            height: dst_h,
            color: [1.0, 1.0, 1.0, 1.0],
        })
    }

    /// 释放视频纹理，播放结束后调用。
    pub fn clear_video_texture(&mut self) {
        self.video_texture = None;
        self.video_texture_size = (0, 0);
    }
}
