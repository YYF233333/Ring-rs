//! HostApp：winit ApplicationHandler 实现
//!
//! 管理窗口生命周期、事件分发和帧渲染循环。

use std::sync::Arc;

use host::app::{self, AppState};
use host::backend::WgpuBackend;
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

const SAVE_SLOTS_SHOWN: u32 = 20;

pub struct HostApp {
    backend: Option<WgpuBackend>,
    app_state: Option<AppState>,
    pub config: AppConfig,
    pub font_data: Option<Vec<u8>>,
    initialized: bool,
    settings_draft: Option<UserSettings>,
    save_load_tab: SaveLoadTab,
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
            ..
        } = self;
        let (Some(backend), Some(app_state)) = (backend.as_mut(), app_state.as_mut()) else {
            return;
        };

        if backend.handle_window_event(&event) {
            backend.request_redraw();
            return;
        }
        app_state.input_manager.process_event(&event);

        match event {
            WindowEvent::CloseRequested => el.exit(),
            WindowEvent::Resized(s) => {
                backend.resize(s);
                app_state
                    .core
                    .renderer
                    .set_screen_size(s.width as f32, s.height as f32);
            }
            WindowEvent::RedrawRequested => {
                if !*initialized {
                    *initialized = true;
                    app::load_resources(app_state);
                    let start_path = app_state.config.start_script_path.clone();
                    if app::load_script_from_logical_path(app_state, &start_path) {
                        info!(path = %start_path, "Start script loaded");
                        app::run_script_tick(app_state, None);
                    }
                }

                let dt = backend.frame_delta().min(0.1);
                app_state.input_manager.begin_frame(dt);
                app::ensure_render_resources(app_state);

                let (w, h) = backend.size();
                app_state.core.renderer.set_screen_size(w as f32, h as f32);
                let mode_before_update = app_state.ui.navigation.current();
                app::update(app_state, dt);

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

                let sprite_cmds = if current_mode.is_in_game() {
                    app::build_game_draw_commands(app_state)
                } else {
                    Vec::new()
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

                let mut ui_action = EguiAction::None;
                backend.render_frame(
                    |ctx| {
                        ui_action = match current_mode {
                            AppMode::Title => egui_screens::title::build_title_ui(ctx, app_state),
                            AppMode::InGame => {
                                egui_screens::ingame::build_ingame_ui(
                                    ctx,
                                    &app_state.core.render_state,
                                );
                                EguiAction::None
                            }
                            AppMode::InGameMenu => {
                                egui_screens::ingame_menu::build_ingame_menu_ui(ctx)
                            }
                            AppMode::Settings => {
                                egui_screens::settings::build_settings_ui(ctx, settings_draft)
                            }
                            AppMode::SaveLoad => egui_screens::save_load::build_save_load_ui(
                                ctx,
                                sl_tab,
                                &save_infos,
                                can_save,
                            ),
                            AppMode::History => {
                                egui_screens::history::build_history_ui(ctx, app_state)
                            }
                        };
                        egui_screens::toast::build_toast_overlay(ctx, &app_state.ui.toast_manager);
                    },
                    &sprite_cmds,
                );

                egui_actions::handle_egui_action(app_state, ui_action, save_load_tab, el);

                app_state.input_manager.end_frame();
                backend.request_redraw();
            }
            _ => {}
        }
    }
}
