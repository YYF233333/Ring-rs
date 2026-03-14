//! HostApp：winit ApplicationHandler 实现
//!
//! 管理窗口生命周期、事件分发和帧渲染循环。

use std::sync::Arc;

use host::app::{self, AppState};
use host::backend::WgpuBackend;
use host::ui::UiAssetCache;
use host::{AppConfig, AppMode, SaveLoadTab, UserSettings};
use tracing::info;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

use crate::egui_actions::{self, EguiAction};
use crate::egui_screens;
use crate::egui_screens::confirm::ConfirmDialog;

const SAVE_SLOTS_SHOWN: u32 = 20;

pub struct HostApp {
    backend: Option<WgpuBackend>,
    app_state: Option<AppState>,
    pub config: AppConfig,
    pub font_data: Option<Vec<u8>>,
    initialized: bool,
    settings_draft: Option<UserSettings>,
    save_load_tab: SaveLoadTab,
    pending_confirm: Option<ConfirmDialog>,
}

impl HostApp {
    pub fn new(config: AppConfig, font_data: Option<Vec<u8>>) -> Self {
        Self {
            backend: None,
            app_state: None,
            config,
            font_data,
            initialized: false,
            settings_draft: None,
            save_load_tab: SaveLoadTab::Load,
            pending_confirm: None,
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
            .unwrap(),
        );

        let backend = WgpuBackend::new(win, self.font_data.take());
        let mut app_state = AppState::new(self.config.clone());
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
            settings_draft,
            save_load_tab,
            pending_confirm,
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

                // 利用上一帧的 egui 布局：若指针在交互式控件上，抑制鼠标点击
                // 以避免游戏推进与 UI 按钮动作同帧冲突
                if backend.egui_ctx().wants_pointer_input() {
                    app_state.input_manager.suppress_mouse_click();
                }

                let (w, h) = backend.size();
                app_state.core.renderer.set_screen_size(w as f32, h as f32);
                let mode_before_update = app_state.ui.navigation.current();
                app::update(app_state, dt);

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
                    *settings_draft = None;
                }

                let current_mode = app_state.ui.navigation.current();

                if current_mode == AppMode::Settings && settings_draft.is_none() {
                    *settings_draft = Some(app_state.user_settings.clone());
                } else if current_mode != AppMode::Settings {
                    *settings_draft = None;
                }

                // 视频播放时：上传帧数据到 GPU 纹理，生成全屏视频精灵
                let sprite_cmds = if app_state.video_player.is_playing() {
                    if let Some(frame) = app_state.video_player.current_frame() {
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

                let save_infos: Vec<Option<host::save_manager::SaveInfo>> =
                    if current_mode == AppMode::SaveLoad {
                        (1..=SAVE_SLOTS_SHOWN)
                            .map(|s| app_state.save_manager.get_save_info(s))
                            .collect()
                    } else {
                        Vec::new()
                    };
                let can_save = app_state.session.vn_runtime.is_some();
                let sl_tab = *save_load_tab;

                let layout = &app_state.ui.layout;
                let asset_cache = app_state.ui.asset_cache.as_ref();
                let scale = &app_state.ui.ui_context.scale;

                let mut ui_action = EguiAction::None;
                backend.render_frame(
                    |ctx| {
                        ui_action = if app_state.video_player.is_playing() {
                            EguiAction::None
                        } else {
                            match current_mode {
                                AppMode::Title => egui_screens::title::build_title_ui(
                                    ctx,
                                    app_state,
                                    layout,
                                    asset_cache,
                                    scale,
                                ),
                                AppMode::InGame => egui_screens::ingame::build_ingame_ui(
                                    ctx,
                                    &app_state.core.render_state,
                                    layout,
                                    asset_cache,
                                    scale,
                                ),
                                AppMode::InGameMenu => {
                                    egui_screens::ingame_menu::build_ingame_menu_ui(
                                        ctx,
                                        layout,
                                        asset_cache,
                                        scale,
                                    )
                                }
                                AppMode::Settings => egui_screens::settings::build_settings_ui(
                                    ctx,
                                    settings_draft,
                                    layout,
                                    asset_cache,
                                    scale,
                                ),
                                AppMode::SaveLoad => egui_screens::save_load::build_save_load_ui(
                                    ctx,
                                    sl_tab,
                                    &save_infos,
                                    can_save,
                                    layout,
                                    asset_cache,
                                    scale,
                                ),
                                AppMode::History => egui_screens::history::build_history_ui(
                                    ctx,
                                    app_state,
                                    layout,
                                    asset_cache,
                                    scale,
                                ),
                            }
                        };
                        // Skip indicator (InGame + Skip mode)
                        if current_mode == AppMode::InGame
                            && app_state.session.playback_mode == host::PlaybackMode::Skip
                        {
                            egui_screens::skip_indicator::build_skip_indicator(
                                ctx,
                                layout,
                                asset_cache,
                                scale,
                            );
                        }

                        // Confirm dialog overlay (drawn on top of everything)
                        if let Some(dialog) = pending_confirm.as_ref() {
                            if let Some(confirm_action) =
                                egui_screens::confirm::build_confirm_overlay(
                                    ctx,
                                    dialog,
                                    layout,
                                    asset_cache,
                                    scale,
                                )
                            {
                                ui_action = confirm_action;
                            }
                        }

                        egui_screens::toast::build_toast_overlay(ctx, &app_state.ui.toast_manager);
                    },
                    &sprite_cmds,
                );

                // If confirm dialog was resolved, clear it
                if pending_confirm.is_some() {
                    if !matches!(ui_action, EguiAction::None) {
                        let resolved_action = ui_action.clone();
                        *pending_confirm = None;
                        ui_action = resolved_action;
                    }
                }

                match ui_action {
                    EguiAction::ShowConfirm {
                        message,
                        on_confirm,
                    } => {
                        *pending_confirm = Some(ConfirmDialog {
                            message,
                            on_confirm: *on_confirm,
                            on_cancel: EguiAction::None,
                        });
                    }
                    _ => {
                        egui_actions::handle_egui_action(app_state, ui_action, save_load_tab, el);
                    }
                }

                app_state.input_manager.end_frame();
                backend.request_redraw();
            }
            _ => {}
        }
    }
}
