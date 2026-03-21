//! Headless 测试模式
//!
//! 无窗口、无 GPU 运行环境，用于 AI 自动调试管线的快进回放。
//! 通过 `--headless --replay-input=<path>` 启动。
//!
//! 包含 CPU-only egui 集成：运行完整 UI 逻辑（布局、命中测试、交互）
//! 但不执行 GPU 渲染。这使得地图选择、选项分支等 egui 驱动的交互
//! 可以在 headless 模式下正确重现。

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use tracing::info;
use winit::keyboard::KeyCode;

use crate::AppConfig;
use crate::AppMode;
use crate::LogicalPath;
use crate::app::{self, AppInit, AppState};
use crate::backend::configure_egui_fonts;
use crate::build_ui::{UiFrameState, build_frame_ui};
use crate::egui_actions::{self, EguiAction};
use crate::egui_screens;
use crate::input::recording::{InputEvent, InputReplayer, MouseButtonName};
use crate::rendering_types::{NullTextureFactory, TextureContext};
use crate::ui::{ConditionContext, UiRenderContext};

/// Headless 模式错误
#[derive(Debug)]
pub enum HeadlessError {
    ReplayLoad(crate::input::recording::ReplayLoadError),
    EventStream(std::io::Error),
    Validation(String),
}

impl std::fmt::Display for HeadlessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReplayLoad(e) => write!(f, "Replay load error: {e}"),
            Self::EventStream(e) => write!(f, "Event stream error: {e}"),
            Self::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl std::error::Error for HeadlessError {}

impl From<crate::input::recording::ReplayLoadError> for HeadlessError {
    fn from(e: crate::input::recording::ReplayLoadError) -> Self {
        Self::ReplayLoad(e)
    }
}

/// Headless CLI 参数（从 main.rs 的 Cli 传入的子集）
pub struct HeadlessCli {
    pub replay_input: PathBuf,
    pub event_stream: Option<PathBuf>,
    pub exit_on: String,
    pub max_frames: Option<u64>,
    pub timeout_sec: Option<u64>,
}

// ─── HeadlessEgui ───────────────────────────────────────────────────

/// CPU-only egui 集成（无 GPU 渲染，仅运行 UI 逻辑）
struct HeadlessEgui {
    ctx: egui::Context,
    pointer_pos: egui::Pos2,
    scale_factor: f32,
    screen_rect: egui::Rect,
}

impl HeadlessEgui {
    fn new(
        font_data: Option<Vec<u8>>,
        logical_width: f32,
        logical_height: f32,
        scale_factor: f64,
    ) -> Self {
        let ctx = egui::Context::default();
        configure_egui_fonts(&ctx, font_data);

        let screen_rect =
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(logical_width, logical_height));

        Self {
            ctx,
            pointer_pos: egui::Pos2::ZERO,
            scale_factor: scale_factor as f32,
            screen_rect,
        }
    }

    /// 将录制的 InputEvent 转换为 egui::RawInput
    fn build_raw_input(&mut self, events: &[InputEvent]) -> egui::RawInput {
        let mut egui_events = Vec::new();

        for event in events {
            match event {
                InputEvent::MouseMove { x, y } => {
                    self.pointer_pos = egui::pos2(x / self.scale_factor, y / self.scale_factor);
                    egui_events.push(egui::Event::PointerMoved(self.pointer_pos));
                }
                InputEvent::MousePress { button, x, y } => {
                    self.pointer_pos = egui::pos2(x / self.scale_factor, y / self.scale_factor);
                    if let Some(egui_button) = to_egui_button(button) {
                        egui_events.push(egui::Event::PointerButton {
                            pos: self.pointer_pos,
                            button: egui_button,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
                InputEvent::MouseRelease { button, x, y } => {
                    self.pointer_pos = egui::pos2(x / self.scale_factor, y / self.scale_factor);
                    if let Some(egui_button) = to_egui_button(button) {
                        egui_events.push(egui::Event::PointerButton {
                            pos: self.pointer_pos,
                            button: egui_button,
                            pressed: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
                InputEvent::KeyPress { key } => {
                    if let Some(egui_key) = to_egui_key(&key.0) {
                        egui_events.push(egui::Event::Key {
                            key: egui_key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
                InputEvent::KeyRelease { key } => {
                    if let Some(egui_key) = to_egui_key(&key.0) {
                        egui_events.push(egui::Event::Key {
                            key: egui_key,
                            physical_key: None,
                            pressed: false,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
                InputEvent::MouseWheel { delta_x, delta_y } => {
                    egui_events.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(*delta_x, *delta_y),
                        modifiers: egui::Modifiers::NONE,
                    });
                }
                InputEvent::UIResult { .. } => {}
            }
        }

        egui::RawInput {
            screen_rect: Some(self.screen_rect),
            // pixels_per_point = 1.0：坐标已转换为逻辑像素
            ..Default::default()
        }
        .tap(|input| input.events = egui_events)
    }
}

/// 辅助 trait：在 RawInput 上设置 events（避免 struct update 与 Vec 冲突）
trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&mut Self)) -> Self;
}

impl Tap for egui::RawInput {
    fn tap(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}

fn to_egui_button(button: &MouseButtonName) -> Option<egui::PointerButton> {
    match button {
        MouseButtonName::Left => Some(egui::PointerButton::Primary),
        MouseButtonName::Right => Some(egui::PointerButton::Secondary),
        MouseButtonName::Middle => Some(egui::PointerButton::Middle),
    }
}

fn to_egui_key(name: &str) -> Option<egui::Key> {
    match name {
        "Space" => Some(egui::Key::Space),
        "Enter" => Some(egui::Key::Enter),
        "Escape" => Some(egui::Key::Escape),
        "ArrowUp" => Some(egui::Key::ArrowUp),
        "ArrowDown" => Some(egui::Key::ArrowDown),
        "ArrowLeft" => Some(egui::Key::ArrowLeft),
        "ArrowRight" => Some(egui::Key::ArrowRight),
        _ => None,
    }
}

// ─── Headless 入口 ──────────────────────────────────────────────────

/// 运行 headless 模式
pub fn run(config: AppConfig, cli: &crate::headless::HeadlessCli) -> Result<(), HeadlessError> {
    let es_path = cli
        .event_stream
        .clone()
        .unwrap_or_else(|| PathBuf::from("events.jsonl"));

    let mut app_state = AppState::new(
        config.clone(),
        AppInit {
            headless: true,
            event_stream_path: Some(es_path),
        },
    );

    app_state
        .core
        .resource_manager
        .set_texture_context(TextureContext::new(std::sync::Arc::new(NullTextureFactory)));

    let logical_w = config.window.width as f32;
    let logical_h = config.window.height as f32;
    app_state
        .core
        .renderer
        .set_screen_size(logical_w, logical_h);

    let mut replayer = InputReplayer::load(&cli.replay_input)?;

    if replayer.meta().logical_width != config.window.width
        || replayer.meta().logical_height != config.window.height
    {
        tracing::warn!(
            recording_size = format!(
                "{}x{}",
                replayer.meta().logical_width,
                replayer.meta().logical_height
            ),
            config_size = format!("{}x{}", config.window.width, config.window.height),
            "录制分辨率与配置不一致"
        );
    }

    let scale_factor = replayer.meta().scale_factor;

    // 加载 CJK 字体供 CPU egui 使用（精确还原布局）
    let font_path = LogicalPath::new(&config.default_font);
    let font_data = match app_state.core.resource_manager.read_bytes(&font_path) {
        Ok(data) => {
            info!(font = %font_path, "CJK font loaded (headless)");
            Some(data)
        }
        Err(e) => {
            tracing::warn!(font = %font_path, error = %e, "Cannot load CJK font (headless)");
            None
        }
    };

    let mut headless_egui = HeadlessEgui::new(font_data, logical_w, logical_h, scale_factor);

    // 设置 UI 缩放上下文（使用逻辑尺寸）
    app_state
        .ui
        .ui_context
        .set_screen_size(logical_w, logical_h, &app_state.ui.layout);

    app::load_resources(&mut app_state);
    let start_path = app_state.config.start_script_path.clone();
    if app::load_script_from_logical_path(&mut app_state, &start_path) {
        info!(path = %start_path, "Start script loaded (headless)");
        app::run_script_tick(&mut app_state, None);
    }

    app_state.ui.navigation.switch_to(AppMode::InGame);

    headless_loop(&mut app_state, &mut replayer, cli, &mut headless_egui)?;

    app_state.event_stream.flush();
    info!("Headless 运行完成");
    Ok(())
}

fn headless_loop(
    app_state: &mut AppState,
    replayer: &mut InputReplayer,
    cli: &HeadlessCli,
    egui: &mut HeadlessEgui,
) -> Result<(), HeadlessError> {
    let fixed_dt: f32 = 1.0 / 60.0;
    let dt_ms: u64 = 16;
    let mut elapsed_ms: u64 = 0;
    let mut frame_count: u64 = 0;
    let wall_start = Instant::now();
    let mut ui_frame_state = UiFrameState::default();

    loop {
        app_state.event_stream.set_logical_time_ms(elapsed_ms);
        let events = replayer.drain_until(elapsed_ms);
        app_state.input_manager.inject_replay_events(&events);

        app_state.input_manager.begin_frame(fixed_dt);

        // 利用上一帧 egui 布局：若指针在交互控件上，抑制游戏层鼠标点击
        if egui.ctx.wants_pointer_input() {
            app_state.input_manager.suppress_mouse_click();
        }

        app::update(app_state, fixed_dt);

        // ── Esc 导航（与 host_app.rs 相同逻辑）──
        let mode_before = app_state.ui.navigation.current();
        if matches!(
            mode_before,
            AppMode::InGameMenu | AppMode::SaveLoad | AppMode::Settings | AppMode::History
        ) && app_state.input_manager.is_key_just_pressed(KeyCode::Escape)
        {
            app_state.ui.navigation.go_back();
            ui_frame_state.settings_draft = None;
        }

        let current_mode = app_state.ui.navigation.current();
        if current_mode == AppMode::Settings && ui_frame_state.settings_draft.is_none() {
            ui_frame_state.settings_draft = Some(app_state.user_settings.clone());
        } else if current_mode != AppMode::Settings {
            ui_frame_state.settings_draft = None;
        }

        // ── CPU egui 帧 ──
        let raw_input = egui.build_raw_input(&events);
        let ui_ctx = UiRenderContext {
            layout: &app_state.ui.layout,
            assets: None,
            scale: &app_state.ui.ui_context.scale,
            screen_defs: &app_state.ui.screen_defs,
            conditions: ConditionContext {
                has_continue: app_state.save_manager.has_continue(),
                persistent: &app_state.persistent_store,
            },
        };
        let slot_thumbnails: HashMap<u32, egui::TextureHandle> = HashMap::new();

        let mut ui_action = EguiAction::None;
        let mut confirm_resolved = false;
        let _full_output = egui.ctx.run(raw_input, |ctx| {
            let (action, resolved) = build_frame_ui(
                ctx,
                app_state,
                &ui_ctx,
                &mut ui_frame_state,
                &slot_thumbnails,
            );
            ui_action = action;
            confirm_resolved = resolved;
        });

        if confirm_resolved {
            ui_frame_state.pending_confirm = None;
        }

        match ui_action {
            EguiAction::ShowConfirm {
                message,
                on_confirm,
            } => {
                ui_frame_state.pending_confirm = Some(egui_screens::confirm::ConfirmDialog {
                    message,
                    on_confirm: *on_confirm,
                    on_cancel: EguiAction::None,
                });
            }
            _ => {
                egui_actions::handle_egui_action(
                    app_state,
                    ui_action,
                    &mut ui_frame_state.save_load_tab,
                    None,
                );
            }
        }

        app_state.input_manager.end_frame();

        elapsed_ms += dt_ms;
        frame_count += 1;

        if app_state.host_state.exit_requested
            || should_exit(app_state, replayer, cli, frame_count, &wall_start)
        {
            break;
        }
    }
    Ok(())
}

fn should_exit(
    app_state: &AppState,
    replayer: &InputReplayer,
    cli: &HeadlessCli,
    frame_count: u64,
    wall_start: &Instant,
) -> bool {
    if let Some(max) = cli.max_frames
        && frame_count >= max
    {
        info!(frames = frame_count, "已达最大帧数限制");
        return true;
    }

    if let Some(timeout) = cli.timeout_sec
        && wall_start.elapsed().as_secs() >= timeout
    {
        info!(timeout_sec = timeout, "已达超时限制");
        return true;
    }

    match cli.exit_on.as_str() {
        "replay-end" => {
            if replayer.is_exhausted() {
                info!("回放数据已耗尽");
                return true;
            }
        }
        "script-finished" => {
            if app_state.session.script_finished {
                info!("脚本执行完毕");
                return true;
            }
        }
        _ => {}
    }

    false
}
