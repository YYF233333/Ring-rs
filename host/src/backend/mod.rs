//! # 渲染后端模块
//!
//! 基于 winit + wgpu + egui 的渲染后端实现。
//!
//! ## 架构
//!
//! - [`WgpuBackend`]: GPU 初始化、窗口管理、egui 集成、帧渲染循环
//! - [`SpriteRenderer`]: 2D textured quad batch 渲染器（背景/角色/遮罩）
//! - [`GpuTexture`]: wgpu 纹理封装（替代 macroquad Texture2D）

pub mod dissolve_renderer;
pub mod gpu_texture;
pub mod sprite_renderer;

pub use gpu_texture::GpuTexture;
pub use sprite_renderer::{DrawCommand, SpriteRenderer};

use std::sync::Arc;
use tracing::{info, warn};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::window::Window;

/// wgpu + egui 渲染后端
///
/// 封装了 GPU 设备、表面、渲染管线和 egui 集成。
/// 提供帧渲染 API，支持自定义 wgpu 渲染与 egui UI 叠加。
pub struct WgpuBackend {
    window: Arc<Window>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_cfg: wgpu::SurfaceConfiguration,

    sprite_renderer: Arc<SpriteRenderer>,
    dissolve_renderer: dissolve_renderer::DissolveRenderer,

    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    frame_start: std::time::Instant,
    frame_delta: f32,
}

impl WgpuBackend {
    /// 初始化渲染后端
    ///
    /// # 参数
    /// - `window`: 已创建的 winit 窗口
    /// - `font_data`: CJK 字体数据（如 simhei.ttf），用于 egui 中文渲染
    pub fn new(window: Arc<Window>, font_data: Option<Vec<u8>>) -> Self {
        let size = window.inner_size();

        // wgpu 初始化
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(&surface),
            ..Default::default()
        }))
        .expect("no suitable GPU adapter found");

        info!(
            adapter = %adapter.get_info().name,
            backend = ?adapter.get_info().backend,
            "GPU adapter selected"
        );

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default(), None))
            .expect("GPU device creation failed");
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let surface_cfg = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("surface format unsupported");
        surface.configure(&device, &surface_cfg);

        // Sprite 渲染器
        let sprite_renderer = Arc::new(SpriteRenderer::new(&device, &queue, surface_cfg.format));
        sprite_renderer.update_projection(&queue, size.width as f32, size.height as f32);

        let dissolve_renderer = dissolve_renderer::DissolveRenderer::new(
            &device,
            surface_cfg.format,
            &sprite_renderer.texture_bind_group_layout,
        );

        // egui 初始化
        let egui_ctx = egui::Context::default();
        Self::configure_egui_fonts(&egui_ctx, font_data);

        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui_ctx.viewport_id(),
            &window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, surface_cfg.format, None, 1, false);

        info!(
            width = size.width,
            height = size.height,
            format = ?surface_cfg.format,
            "WgpuBackend initialized"
        );

        Self {
            window,
            device,
            queue,
            surface,
            surface_cfg,
            sprite_renderer,
            dissolve_renderer,
            egui_ctx,
            egui_state,
            egui_renderer,
            frame_start: std::time::Instant::now(),
            frame_delta: 0.0,
        }
    }

    /// 配置 egui 字体（加入 CJK 字体支持）
    fn configure_egui_fonts(ctx: &egui::Context, font_data: Option<Vec<u8>>) {
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

    // ── 访问器 ──────────────────────────────────────────────────────────────

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_cfg.format
    }

    /// 当前窗口尺寸（物理像素）
    pub fn size(&self) -> (u32, u32) {
        (self.surface_cfg.width, self.surface_cfg.height)
    }

    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    /// 上一帧耗时（秒）
    pub fn frame_delta(&self) -> f32 {
        self.frame_delta
    }

    pub fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }

    pub fn sprite_renderer(&self) -> &SpriteRenderer {
        &self.sprite_renderer
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// 创建 GPU 资源上下文（注入到 ResourceManager 中）
    pub fn gpu_resource_context(&self) -> crate::resources::GpuResourceContext {
        crate::resources::GpuResourceContext {
            device: Arc::clone(&self.device),
            queue: Arc::clone(&self.queue),
            sprite_renderer: Arc::clone(&self.sprite_renderer),
        }
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
            &self.device,
            &self.queue,
            width,
            height,
            rgba_data,
            label,
        )
    }

    // ── 事件处理 ─────────────────────────────────────────────────────────────

    /// 将 winit 窗口事件传递给 egui。
    ///
    /// 返回 `true` 表示 egui 消费了该事件。
    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        self.egui_state
            .on_window_event(&self.window, event)
            .consumed
    }

    /// 处理窗口大小变更
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_cfg.width = new_size.width;
            self.surface_cfg.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_cfg);
            self.sprite_renderer.update_projection(
                &self.queue,
                new_size.width as f32,
                new_size.height as f32,
            );
        }
    }

    // ── 帧渲染 ──────────────────────────────────────────────────────────────

    /// 渲染一帧
    ///
    /// # 参数
    /// - `build_ui`: 构建 egui UI 的闭包
    /// - `sprite_commands`: 自定义 sprite 绘制命令（背景/角色/遮罩）
    ///
    /// 渲染顺序：
    /// 1. Clear → sprite 绘制（背景/角色/遮罩）
    /// 2. egui UI 叠加层
    pub fn render_frame(
        &mut self,
        build_ui: impl FnMut(&egui::Context),
        sprite_commands: &[DrawCommand],
    ) {
        // 更新帧时间
        let now = std::time::Instant::now();
        self.frame_delta = now.duration_since(self.frame_start).as_secs_f32();
        self.frame_start = now;

        // 获取 surface 纹理
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(_) => {
                self.surface.configure(&self.device, &self.surface_cfg);
                return;
            }
        };
        let target_view = frame.texture.create_view(&Default::default());

        // egui: 采集输入 → 构建 UI → 输出
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_ctx.run(raw_input, build_ui);

        // egui: 纹理更新 + tessellation
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

        // GPU 命令编码
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
            let mut rpass = rpass.forget_lifetime();

            // 1) Sprite 渲染（背景/角色/遮罩等，跳过 Dissolve 命令）
            self.sprite_renderer
                .draw_sprites(&self.queue, &mut rpass, sprite_commands);

            // 2) Dissolve 叠加（遮罩溶解效果）
            let sw = self.surface_cfg.width as f32;
            let sh = self.surface_cfg.height as f32;
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
                    self.dissolve_renderer.draw(
                        &self.queue,
                        &mut rpass,
                        mask_texture,
                        sw,
                        sh,
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

            // 3) egui UI 叠加
            self.egui_renderer.render(&mut rpass, &primitives, &screen);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        // egui: 清理
        for id in &full_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }
        self.egui_state
            .handle_platform_output(&self.window, full_output.platform_output);
    }
}
