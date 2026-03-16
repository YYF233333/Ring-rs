use std::sync::Arc;
use tracing::{info, warn};
use winit::dpi::PhysicalSize;
use winit::window::Window;

/// GPU 设备、队列与渲染表面
pub struct GpuContext {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_cfg: wgpu::SurfaceConfiguration,
}

impl GpuContext {
    pub(super) fn new(window: &Arc<Window>) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let surface = instance
            .create_surface(window.clone())
            .expect("surface creation failed");

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

    pub(super) fn acquire_frame(&mut self) -> Option<(wgpu::SurfaceTexture, wgpu::TextureView)> {
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
