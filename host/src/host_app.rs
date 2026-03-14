//! HostApp：winit ApplicationHandler 实现
//!
//! 管理窗口生命周期、事件分发和帧渲染循环。

use std::sync::Arc;

use host::app::{self, AppState};
use host::backend::WgpuBackend;
use host::ui::UiAssetCache;
use host::{AppConfig, AppMode, SaveLoadPage, SaveLoadTab, UserSettings};
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

pub struct HostApp {
    backend: Option<WgpuBackend>,
    app_state: Option<AppState>,
    pub config: AppConfig,
    pub font_data: Option<Vec<u8>>,
    initialized: bool,
    settings_draft: Option<UserSettings>,
    save_load_tab: SaveLoadTab,
    save_load_page: SaveLoadPage,
    pending_confirm: Option<ConfirmDialog>,
    /// 待保存缩略图的存档槽位（截图已请求，等待下一帧捕获）
    pending_thumbnail_slot: Option<u32>,
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
            save_load_page: SaveLoadPage::default(),
            pending_confirm: None,
            pending_thumbnail_slot: None,
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
            save_load_page,
            pending_confirm,
            pending_thumbnail_slot,
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
                        save_load_page
                            .slot_range()
                            .map(|s| app_state.save_manager.get_save_info(s))
                            .collect()
                    } else {
                        Vec::new()
                    };

                let mut slot_thumbnails = std::collections::HashMap::new();
                if current_mode == AppMode::SaveLoad {
                    for slot in save_load_page.slot_range() {
                        if let Some(png_bytes) = app_state.save_manager.load_thumbnail_bytes(slot) {
                            if let Ok(img) = image::load_from_memory(&png_bytes) {
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
                }

                let can_save = app_state.session.vn_runtime.is_some();
                let sl_tab = *save_load_tab;

                let layout = &app_state.ui.layout;
                let asset_cache = app_state.ui.asset_cache.as_ref();
                let scale = &app_state.ui.ui_context.scale;
                let is_winter = app_state
                    .persistent_store
                    .variables
                    .get("complete_summer")
                    .is_some_and(|v| matches!(v, vn_runtime::state::VarValue::Bool(true)));

                let mut ui_action = EguiAction::None;
                let mut confirm_resolved = false;
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
                                AppMode::Settings => {
                                    egui_screens::game_menu::build_game_menu_frame(
                                        ctx,
                                        "设置",
                                        is_winter,
                                        layout,
                                        asset_cache,
                                        scale,
                                        |ui| {
                                            egui_screens::settings::build_settings_content(
                                                ui,
                                                settings_draft,
                                                layout,
                                                asset_cache,
                                                scale,
                                            )
                                        },
                                    )
                                }
                                AppMode::SaveLoad => {
                                    egui_screens::game_menu::build_game_menu_frame(
                                        ctx,
                                        if sl_tab == SaveLoadTab::Save {
                                            "保存"
                                        } else {
                                            "读取"
                                        },
                                        is_winter,
                                        layout,
                                        asset_cache,
                                        scale,
                                        |ui| {
                                            egui_screens::save_load::build_save_load_content(
                                                ui,
                                                sl_tab,
                                                save_load_page,
                                                &save_infos,
                                                can_save,
                                                layout,
                                                asset_cache,
                                                scale,
                                                &slot_thumbnails,
                                            )
                                        },
                                    )
                                }
                                AppMode::History => egui_screens::game_menu::build_game_menu_frame(
                                    ctx,
                                    "历史",
                                    is_winter,
                                    layout,
                                    asset_cache,
                                    scale,
                                    |ui| {
                                        egui_screens::history::build_history_content(
                                            ui, app_state, layout, scale,
                                        )
                                    },
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
                                confirm_resolved = true;
                            }
                        }

                        egui_screens::toast::build_toast_overlay(
                            ctx,
                            &app_state.ui.toast_manager,
                            layout,
                            asset_cache,
                            scale,
                        );
                    },
                    &sprite_cmds,
                );

                if confirm_resolved {
                    *pending_confirm = None;
                }

                // Process pending thumbnail save (from previous frame's screenshot request)
                if let Some(slot) = *pending_thumbnail_slot {
                    if let Some((rgba, w, h)) = backend.take_screenshot() {
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
                        // If the confirmed action is SaveToSlot, track it for screenshot
                        if let EguiAction::SaveToSlot(s) = *on_confirm {
                            *pending_thumbnail_slot = Some(s);
                            backend.request_screenshot();
                        }
                        *pending_confirm = Some(ConfirmDialog {
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
