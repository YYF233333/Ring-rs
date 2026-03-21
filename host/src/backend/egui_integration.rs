use tracing::warn;
use winit::event::WindowEvent;
use winit::window::Window;

/// 为 egui Context 配置 CJK 字体（windowed / headless 共用）
pub fn configure_egui_fonts(ctx: &egui::Context, font_data: Option<Vec<u8>>) {
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
        .expect("egui builtin font family must exist")
        .insert(0, "cjk".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .expect("egui builtin font family must exist")
        .insert(0, "cjk".to_owned());
    ctx.set_fonts(fonts);
}

/// egui 集成层（输入桥接 + UI 渲染）
pub struct EguiIntegration {
    pub ctx: egui::Context,
    pub(super) state: egui_winit::State,
    pub(super) renderer: egui_wgpu::Renderer,
}

impl EguiIntegration {
    pub(super) fn new(
        window: &Window,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        font_data: Option<Vec<u8>>,
    ) -> Self {
        let ctx = egui::Context::default();
        configure_egui_fonts(&ctx, font_data);

        let state =
            egui_winit::State::new(ctx.clone(), ctx.viewport_id(), window, None, None, None);
        let renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1, false);

        Self {
            ctx,
            state,
            renderer,
        }
    }

    /// 将 winit 窗口事件传递给 egui，返回是否已消费
    pub fn handle_window_event(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    /// 处理 egui 平台输出（光标、IME 等）
    pub(super) fn handle_platform_output(&mut self, window: &Window, output: egui::PlatformOutput) {
        self.state.handle_platform_output(window, output);
    }
}
