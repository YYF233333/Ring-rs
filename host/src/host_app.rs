//! HostApp：winit ApplicationHandler 实现
//!
//! 管理窗口生命周期、事件分发和帧渲染循环。

use std::sync::Arc;

use host::app::{self, AppState};
use host::backend::WgpuBackend;
use host::build_ui::{UiFrameState, build_frame_ui};
use host::ui::{ConditionContext, UiAssetCache, UiRenderContext};
use host::{AppConfig, AppMode, LogicalPath};
use tracing::info;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

use host::egui_actions::{self, EguiAction};
use host::egui_screens;

use host::game_mode::{GameCompletion, GameMode};

/// WebView 小游戏运行时状态
struct MiniGameRuntime {
    game_mode: GameMode,
    webview: Option<wry::WebView>,
    result_rx: Option<std::sync::mpsc::Receiver<GameCompletion>>,
}

impl MiniGameRuntime {
    fn new() -> Self {
        Self {
            game_mode: GameMode::new(),
            webview: None,
            result_rx: None,
        }
    }
}

pub struct HostApp {
    backend: Option<WgpuBackend>,
    app_state: Option<AppState>,
    pub config: AppConfig,
    initialized: bool,
    ui_frame_state: UiFrameState,
    /// 待保存缩略图的存档槽位（截图已请求，等待下一帧捕获）
    pending_thumbnail_slot: Option<u32>,
    /// 事件流输出路径（CLI 传入）
    event_stream_path: Option<std::path::PathBuf>,
    mini_game: MiniGameRuntime,
}

impl HostApp {
    pub fn new(config: AppConfig, event_stream_path: Option<std::path::PathBuf>) -> Self {
        Self {
            backend: None,
            app_state: None,
            config,
            initialized: false,
            ui_frame_state: UiFrameState::default(),
            pending_thumbnail_slot: None,
            event_stream_path,
            mini_game: MiniGameRuntime::new(),
        }
    }
}

impl ApplicationHandler for HostApp {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.backend.is_some() {
            return;
        }

        let win = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title(&self.config.window.title)
                    .with_inner_size(LogicalSize::new(
                        self.config.window.width,
                        self.config.window.height,
                    ))
                    .with_resizable(false),
            )
            .expect("window creation failed"),
        );

        let mut app_state = AppState::new(
            self.config.clone(),
            app::AppInit {
                headless: false,
                event_stream_path: self.event_stream_path.clone(),
            },
        );
        let font_path = LogicalPath::new(&app_state.config.default_font);
        let font_data = match app_state.core.resource_manager.read_bytes(&font_path) {
            Ok(data) => {
                info!(font = %font_path, "CJK font loaded");
                Some(data)
            }
            Err(e) => {
                tracing::warn!(font = %font_path, error = %e, "Cannot load CJK font");
                None
            }
        };
        let backend = WgpuBackend::new(win, font_data);
        app_state
            .core
            .resource_manager
            .set_texture_context(backend.texture_context());

        let (w, h) = backend.size();
        app_state.core.renderer.set_screen_size(w as f32, h as f32);

        self.backend = Some(backend);
        self.app_state = Some(app_state);
        info!("Window created, AppState initialized");
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let HostApp {
            backend,
            app_state,
            initialized,
            ui_frame_state,
            pending_thumbnail_slot,
            mini_game,
            ..
        } = self;
        let (Some(backend), Some(app_state)) = (backend.as_mut(), app_state.as_mut()) else {
            return;
        };

        // 先记录原始输入，再交给 egui——保证 input_manager 总能看到事件
        app_state.input_manager.process_event(&event);
        backend.handle_window_event(&event);

        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => {
                backend.resize(s);
                app_state
                    .core
                    .renderer
                    .set_screen_size(s.width as f32, s.height as f32);
                // egui 使用逻辑像素坐标，ScaleContext 需要逻辑尺寸而非物理尺寸
                let sf = backend.scale_factor();
                app_state.host_state.scale_factor = sf as f64;
                app_state.ui.ui_context.set_screen_size(
                    s.width as f32 / sf,
                    s.height as f32 / sf,
                    &app_state.ui.layout,
                );
            }
            WindowEvent::RedrawRequested => {
                if !*initialized {
                    *initialized = true;
                    app::load_resources(app_state);

                    let egui_ctx = backend.egui_ctx();
                    let cache = UiAssetCache::load(
                        &app_state.ui.layout.assets,
                        app_state.core.resource_manager.source(),
                        egui_ctx,
                    );
                    app_state.ui.asset_cache = Some(cache);

                    let start_path = app_state.config.start_script_path.clone();
                    if app::load_script_from_logical_path(app_state, &start_path) {
                        info!(path = %start_path, "Start script loaded");
                        app::run_script_tick(app_state, None);
                    }
                }

                let dt = backend.frame_delta().min(0.1);
                app_state.input_manager.begin_frame(dt);

                if app_state.input_manager.is_key_just_pressed(KeyCode::F8) {
                    app::export_recording(app_state);
                }

                // 利用上一帧的 egui 布局：若指针在交互式控件上，抑制鼠标点击
                // 以避免游戏推进与 UI 按钮动作同帧冲突
                if backend.egui_ctx().wants_pointer_input() {
                    app_state.input_manager.suppress_mouse_click();
                }

                let (w, h) = backend.size();
                app_state.core.renderer.set_screen_size(w as f32, h as f32);
                let mode_before_update = app_state.ui.navigation.current();
                app::update(app_state, dt);

                // ── 小游戏 WebView 生命周期 ──
                // 1. 检查待启动的小游戏请求
                if let Some(launch) = app_state.pending_game_launch.take() {
                    let (w, h) = backend.size();
                    match mini_game.game_mode.start(
                        backend.window(),
                        (w, h),
                        &launch,
                        &app_state.config.assets_root,
                    ) {
                        Ok((webview, rx)) => {
                            mini_game.webview = Some(webview);
                            mini_game.result_rx = Some(rx);
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "WebView 创建失败，降级处理");
                            app_state.input_manager.inject_ui_result(
                                launch.request_key,
                                vn_runtime::state::VarValue::String(String::new()),
                            );
                        }
                    }
                }
                // 小游戏完成轮询在 about_to_wait 中执行，
                // 避免 WebView 遮挡父窗口导致 RedrawRequested 不触发。

                // 在 update 之后、渲染之前加载缺失纹理，
                // 确保 update 中新增的资源引用（如差分切换）在本帧即可渲染
                app::ensure_render_resources(app_state);

                // Esc 返回：仅当更新前已处于菜单/子界面时才触发，
                // 避免与 update_ingame 中 Esc 打开菜单在同一帧冲突。
                if matches!(
                    mode_before_update,
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

                // 视频播放时：上传帧数据到 GPU 纹理，生成全屏视频精灵
                let sprite_cmds = if app_state.core.video_player.is_playing() {
                    if let Some(frame) = app_state.core.video_player.current_frame() {
                        backend.upload_video_frame(&frame.data, frame.width, frame.height);
                    }
                    backend.video_draw_command().into_iter().collect::<Vec<_>>()
                } else {
                    backend.clear_video_texture();
                    if current_mode.is_in_game() {
                        app::build_game_draw_commands(app_state)
                    } else {
                        Vec::new()
                    }
                };

                let mut slot_thumbnails = std::collections::HashMap::new();
                if current_mode == AppMode::SaveLoad {
                    for slot in ui_frame_state.save_load_page.slot_range() {
                        if let Some(png_bytes) = app_state.save_manager.load_thumbnail_bytes(slot)
                            && let Ok(img) = image::load_from_memory(&png_bytes)
                        {
                            let rgba = img.to_rgba8();
                            let (w, h) = rgba.dimensions();
                            let tex = backend.egui_ctx().load_texture(
                                format!("thumb_{slot}"),
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [w as usize, h as usize],
                                    &rgba,
                                ),
                                egui::TextureOptions::LINEAR,
                            );
                            slot_thumbnails.insert(slot, tex);
                        }
                    }
                }

                let ui_ctx = UiRenderContext {
                    layout: &app_state.ui.layout,
                    assets: app_state.ui.asset_cache.as_ref(),
                    scale: &app_state.ui.ui_context.scale,
                    screen_defs: &app_state.ui.screen_defs,
                    conditions: ConditionContext {
                        has_continue: app_state.save_manager.has_continue(),
                        persistent: &app_state.persistent_store,
                    },
                };

                let mut ui_action = EguiAction::None;
                let mut confirm_resolved = false;
                backend.render_frame(
                    |ctx| {
                        let (action, resolved) = build_frame_ui(
                            ctx,
                            app_state,
                            &ui_ctx,
                            ui_frame_state,
                            &slot_thumbnails,
                        );
                        ui_action = action;
                        confirm_resolved = resolved;
                    },
                    &sprite_cmds,
                );

                if confirm_resolved {
                    ui_frame_state.pending_confirm = None;
                }

                // Process pending thumbnail save (from previous frame's screenshot request)
                if let Some(slot) = *pending_thumbnail_slot
                    && let Some((rgba, w, h)) = backend.take_screenshot()
                {
                    let thumb_w = 384u32;
                    let thumb_h = 216u32;
                    if let Err(e) = app_state
                        .save_manager
                        .save_thumbnail(slot, &rgba, w, h, thumb_w, thumb_h)
                    {
                        tracing::warn!(slot, error = %e, "缩略图保存失败");
                    }
                    *pending_thumbnail_slot = None;
                }

                // Track save slot for screenshot capture
                let save_slot = match &ui_action {
                    EguiAction::SaveToSlot(s) => Some(*s),
                    EguiAction::QuickSave => Some(app_state.current_save_slot),
                    _ => None,
                };

                match ui_action {
                    EguiAction::ShowConfirm {
                        message,
                        on_confirm,
                    } => {
                        if let EguiAction::SaveToSlot(s) = *on_confirm {
                            *pending_thumbnail_slot = Some(s);
                            backend.request_screenshot();
                        }
                        ui_frame_state.pending_confirm =
                            Some(egui_screens::confirm::ConfirmDialog {
                                message,
                                on_confirm: *on_confirm,
                                on_cancel: EguiAction::None,
                            });
                    }
                    _ => {
                        if let Some(slot) = save_slot {
                            *pending_thumbnail_slot = Some(slot);
                            backend.request_screenshot();
                        }
                        egui_actions::handle_egui_action(
                            app_state,
                            ui_action,
                            &mut ui_frame_state.save_load_tab,
                            Some(el),
                        );
                    }
                }

                app_state.input_manager.end_frame();
                backend.request_redraw();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        let completion = self
            .mini_game
            .result_rx
            .as_ref()
            .and_then(|rx| rx.try_recv().ok());

        if let Some(completion) = completion
            && let Some(request_key) = self.mini_game.game_mode.complete()
        {
            if let Some(ref webview) = self.mini_game.webview {
                let _ = webview.set_visible(false);
            }
            self.mini_game.webview = None;
            self.mini_game.result_rx = None;

            if let Some(app_state) = self.app_state.as_mut() {
                info!(key = %request_key, "小游戏完成，回传结果");
                app_state
                    .input_manager
                    .inject_ui_result(request_key, completion.result);
            }

            if let Some(backend) = self.backend.as_ref() {
                backend.request_redraw();
            }
        }
    }
}
