//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。
//!
//! 使用 winit 事件循环 + wgpu 渲染 + egui UI。

use std::sync::Arc;

use host::app::{self, AppState};
use host::backend::WgpuBackend;
use host::renderer::RenderState;
use host::save_manager::SaveInfo;
use host::ui::toast::ToastType;
use host::{AppConfig, AppMode, SaveLoadTab, UserSettings};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use vn_runtime::HistoryEvent;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

const CONFIG_PATH: &str = "config.json";
const USER_SETTINGS_PATH: &str = "user_settings.json";
const SAVE_SLOTS_SHOWN: u32 = 20;

struct HostApp {
    backend: Option<WgpuBackend>,
    app_state: Option<AppState>,
    config: AppConfig,
    font_data: Option<Vec<u8>>,
    initialized: bool,
    settings_draft: Option<UserSettings>,
    save_load_tab: SaveLoadTab,
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
            .set_gpu_context(backend.gpu_resource_context());

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
                app::update(app_state, dt);

                let current_mode = app_state.ui.navigation.current();

                // ESC to go back from overlay screens
                if matches!(
                    current_mode,
                    AppMode::InGameMenu | AppMode::SaveLoad | AppMode::Settings | AppMode::History
                ) && app_state
                    .input_manager
                    .is_key_just_pressed_pub(KeyCode::Escape)
                {
                    app_state.ui.navigation.go_back();
                    *settings_draft = None;
                }

                let current_mode = app_state.ui.navigation.current();

                // Initialize settings draft when entering Settings
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

                // Pre-load save infos for SaveLoad screen
                let save_infos: Vec<Option<SaveInfo>> = if current_mode == AppMode::SaveLoad {
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
                            AppMode::Title => build_title_ui(ctx, app_state),
                            AppMode::InGame => {
                                build_ingame_ui(ctx, &app_state.core.render_state);
                                EguiAction::None
                            }
                            AppMode::InGameMenu => build_ingame_menu_ui(ctx),
                            AppMode::Settings => build_settings_ui(ctx, settings_draft),
                            AppMode::SaveLoad => {
                                build_save_load_ui(ctx, sl_tab, &save_infos, can_save)
                            }
                            AppMode::History => build_history_ui(ctx, app_state),
                        };
                        build_toast_overlay(ctx, &app_state.ui.toast_manager);
                    },
                    &sprite_cmds,
                );

                handle_egui_action(app_state, ui_action, save_load_tab, el);

                app_state.input_manager.end_frame();
                backend.request_redraw();
            }
            _ => {}
        }
    }
}

// -- EguiAction -----------------------------------------------------------------

#[derive(Debug, Clone)]
enum EguiAction {
    None,
    StartGame,
    ContinueGame,
    NavigateTo(AppMode),
    GoBack,
    ReturnToTitle,
    Exit,
    ApplySettings(UserSettings),
    OpenSave,
    OpenLoad,
    SaveToSlot(u32),
    LoadFromSlot(u32),
    DeleteSlot(u32),
}

fn handle_egui_action(
    app_state: &mut AppState,
    action: EguiAction,
    save_load_tab: &mut SaveLoadTab,
    el: &ActiveEventLoop,
) {
    match action {
        EguiAction::None => {}
        EguiAction::StartGame => {
            let _ = app_state.save_manager.delete_continue();
            app::start_new_game(app_state);
        }
        EguiAction::ContinueGame => {
            app::load_continue(app_state);
        }
        EguiAction::NavigateTo(mode) => {
            app_state.ui.navigation.navigate_to(mode);
        }
        EguiAction::GoBack => {
            app_state.ui.navigation.go_back();
        }
        EguiAction::ReturnToTitle => {
            app::return_to_title_from_game(app_state, true);
        }
        EguiAction::Exit => {
            el.exit();
        }
        EguiAction::ApplySettings(new_settings) => {
            app_state.user_settings = new_settings;
            if let Some(ref mut audio) = app_state.core.audio_manager {
                audio.set_bgm_volume(app_state.user_settings.bgm_volume);
                audio.set_sfx_volume(app_state.user_settings.sfx_volume);
                audio.set_muted(app_state.user_settings.muted);
            }
            if let Err(e) = app_state.user_settings.save(USER_SETTINGS_PATH) {
                tracing::warn!(error = %e, "保存用户设置失败");
                app_state.ui.toast_manager.error("Settings save failed");
            } else {
                app_state.ui.toast_manager.success("Settings saved");
            }
            app_state.ui.navigation.go_back();
        }
        EguiAction::OpenSave => {
            *save_load_tab = SaveLoadTab::Save;
            app_state.ui.navigation.navigate_to(AppMode::SaveLoad);
        }
        EguiAction::OpenLoad => {
            *save_load_tab = SaveLoadTab::Load;
            app_state.ui.navigation.navigate_to(AppMode::SaveLoad);
        }
        EguiAction::SaveToSlot(slot) => {
            app_state.current_save_slot = slot;
            app::quick_save(app_state);
            app_state
                .ui
                .toast_manager
                .success(format!("Saved to slot {slot}"));
        }
        EguiAction::LoadFromSlot(slot) => {
            app::load_game(app_state, slot);
            app_state
                .ui
                .toast_manager
                .success(format!("Loaded slot {slot}"));
        }
        EguiAction::DeleteSlot(slot) => {
            if app_state.save_manager.delete(slot).is_ok() {
                app_state
                    .ui
                    .toast_manager
                    .info(format!("Deleted slot {slot}"));
            } else {
                app_state.ui.toast_manager.error("Delete failed");
            }
        }
    }
}

// -- Screen builders ------------------------------------------------------------

const DARK_BG: egui::Color32 = egui::Color32::from_rgb(20, 20, 40);
const PANEL_BG: egui::Color32 = egui::Color32::from_rgb(25, 25, 50);
const GOLD: egui::Color32 = egui::Color32::from_rgb(220, 200, 160);

fn dark_frame() -> egui::Frame {
    egui::Frame::new().fill(DARK_BG).inner_margin(0.0)
}

fn panel_frame() -> egui::Frame {
    egui::Frame::new().fill(PANEL_BG).inner_margin(40.0)
}

// -- Title ----------------------------------------------------------------------

fn build_title_ui(ctx: &egui::Context, app_state: &AppState) -> EguiAction {
    let has_continue = app_state.save_manager.has_continue();
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(dark_frame())
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.2);
                ui.heading(
                    egui::RichText::new("Visual Novel Engine")
                        .size(36.0)
                        .color(GOLD),
                );
                ui.add_space(40.0);

                let btn = egui::vec2(240.0, 44.0);

                if menu_btn(ui, btn, "New Game") {
                    action = EguiAction::StartGame;
                }
                if has_continue && menu_btn(ui, btn, "Continue") {
                    action = EguiAction::ContinueGame;
                }
                if menu_btn(ui, btn, "Load") {
                    action = EguiAction::OpenLoad;
                }
                if menu_btn(ui, btn, "Settings") {
                    action = EguiAction::NavigateTo(AppMode::Settings);
                }
                if menu_btn(ui, btn, "Exit") {
                    action = EguiAction::Exit;
                }
            });
        });

    action
}

// -- InGame ---------------------------------------------------------------------

fn build_ingame_ui(ctx: &egui::Context, render_state: &RenderState) {
    if let Some(ref dialogue) = render_state.dialogue {
        egui::TopBottomPanel::bottom("dialogue_panel")
            .min_height(120.0)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(15, 15, 35, 220))
                    .inner_margin(16.0),
            )
            .show(ctx, |ui| {
                if let Some(ref speaker) = dialogue.speaker {
                    ui.colored_label(
                        egui::Color32::from_rgb(240, 210, 140),
                        egui::RichText::new(speaker).size(22.0).strong(),
                    );
                    ui.add_space(4.0);
                }
                let visible_text: String = dialogue
                    .content
                    .chars()
                    .take(dialogue.visible_chars)
                    .collect();
                ui.label(
                    egui::RichText::new(&visible_text)
                        .size(18.0)
                        .color(egui::Color32::WHITE),
                );
            });
    }

    if let Some(ref choices) = render_state.choices {
        egui::Area::new(egui::Id::new("choices_area"))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
                    .inner_margin(24.0)
                    .corner_radius(8.0)
                    .show(ui, |ui| {
                        for (i, choice) in choices.choices.iter().enumerate() {
                            let selected = i == choices.selected_index;
                            let color = if selected {
                                egui::Color32::from_rgb(255, 220, 100)
                            } else {
                                egui::Color32::WHITE
                            };
                            ui.add_space(4.0);
                            ui.label(egui::RichText::new(&choice.text).size(18.0).color(color));
                        }
                    });
            });
    }
}

// -- InGameMenu -----------------------------------------------------------------

fn build_ingame_menu_ui(ctx: &egui::Context) -> EguiAction {
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(ui.available_height() * 0.15);
                ui.heading(
                    egui::RichText::new("Menu")
                        .size(28.0)
                        .color(egui::Color32::WHITE),
                );
                ui.add_space(30.0);

                let btn = egui::vec2(220.0, 40.0);
                let entries: &[(&str, EguiAction)] = &[
                    ("Resume", EguiAction::GoBack),
                    ("Save", EguiAction::OpenSave),
                    ("Load", EguiAction::OpenLoad),
                    ("Settings", EguiAction::NavigateTo(AppMode::Settings)),
                    ("History", EguiAction::NavigateTo(AppMode::History)),
                    ("Return to Title", EguiAction::ReturnToTitle),
                    ("Exit", EguiAction::Exit),
                ];

                for (label, btn_action) in entries {
                    if ui
                        .add_sized(
                            btn,
                            egui::Button::new(egui::RichText::new(*label).size(16.0)),
                        )
                        .clicked()
                    {
                        action = btn_action.clone();
                    }
                    ui.add_space(6.0);
                }
            });
        });

    action
}

// -- Settings -------------------------------------------------------------------

fn build_settings_ui(ctx: &egui::Context, draft: &mut Option<UserSettings>) -> EguiAction {
    let Some(ref mut d) = *draft else {
        return EguiAction::GoBack;
    };
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(panel_frame())
        .show(ctx, |ui| {
            ui.heading(
                egui::RichText::new("Settings")
                    .size(28.0)
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(24.0);

            let label_w = 140.0;

            // Text Speed
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
                    ui.label(
                        egui::RichText::new("Text Speed")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.add(
                    egui::Slider::new(&mut d.text_speed, 5.0..=100.0)
                        .suffix(" cps")
                        .step_by(1.0),
                );
            });
            ui.add_space(12.0);

            // Auto Delay
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
                    ui.label(
                        egui::RichText::new("Auto Delay")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.add(
                    egui::Slider::new(&mut d.auto_delay, 0.5..=5.0)
                        .suffix(" s")
                        .step_by(0.1),
                );
            });
            ui.add_space(12.0);

            // BGM Volume
            let mut bgm_pct = d.bgm_volume * 100.0;
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
                    ui.label(
                        egui::RichText::new("BGM Volume")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.add(
                    egui::Slider::new(&mut bgm_pct, 0.0..=100.0)
                        .suffix("%")
                        .step_by(1.0),
                );
            });
            d.bgm_volume = bgm_pct / 100.0;
            ui.add_space(12.0);

            // SFX Volume
            let mut sfx_pct = d.sfx_volume * 100.0;
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
                    ui.label(
                        egui::RichText::new("SFX Volume")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.add(
                    egui::Slider::new(&mut sfx_pct, 0.0..=100.0)
                        .suffix("%")
                        .step_by(1.0),
                );
            });
            d.sfx_volume = sfx_pct / 100.0;
            ui.add_space(12.0);

            // Muted
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
                    ui.label(
                        egui::RichText::new("Muted")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.checkbox(&mut d.muted, "");
            });
            ui.add_space(24.0);

            // Buttons
            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("Apply & Back").size(16.0))
                    .clicked()
                {
                    action = EguiAction::ApplySettings(d.clone());
                }
                ui.add_space(16.0);
                if ui
                    .button(egui::RichText::new("Cancel").size(16.0))
                    .clicked()
                {
                    action = EguiAction::GoBack;
                }
            });
        });

    action
}

// -- SaveLoad -------------------------------------------------------------------

fn build_save_load_ui(
    ctx: &egui::Context,
    tab: SaveLoadTab,
    save_infos: &[Option<SaveInfo>],
    can_save: bool,
) -> EguiAction {
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(panel_frame())
        .show(ctx, |ui| {
            // Header with tabs
            ui.horizontal(|ui| {
                let save_label = if tab == SaveLoadTab::Save {
                    egui::RichText::new("[ Save ]")
                        .size(22.0)
                        .strong()
                        .color(GOLD)
                } else {
                    egui::RichText::new("  Save  ")
                        .size(22.0)
                        .color(egui::Color32::GRAY)
                };
                let load_label = if tab == SaveLoadTab::Load {
                    egui::RichText::new("[ Load ]")
                        .size(22.0)
                        .strong()
                        .color(GOLD)
                } else {
                    egui::RichText::new("  Load  ")
                        .size(22.0)
                        .color(egui::Color32::GRAY)
                };

                if ui.selectable_label(false, save_label).clicked() && can_save {
                    action = EguiAction::OpenSave;
                }
                if ui.selectable_label(false, load_label).clicked() {
                    action = EguiAction::OpenLoad;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("Back").size(16.0)).clicked() {
                        action = EguiAction::GoBack;
                    }
                });
            });
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            // Slot list
            egui::ScrollArea::vertical()
                .auto_shrink(false)
                .show(ui, |ui| {
                    for (i, info) in save_infos.iter().enumerate() {
                        let slot = (i as u32) + 1;
                        ui.push_id(slot, |ui| {
                            let frame = egui::Frame::new()
                                .fill(egui::Color32::from_rgb(30, 30, 55))
                                .inner_margin(12.0)
                                .corner_radius(4.0);

                            frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Slot number
                                    ui.label(
                                        egui::RichText::new(format!("Slot {slot:02}"))
                                            .size(16.0)
                                            .strong()
                                            .color(GOLD),
                                    );
                                    ui.add_space(12.0);

                                    if let Some(si) = info {
                                        // Save info
                                        let chapter = si.chapter_title.as_deref().unwrap_or("---");
                                        ui.label(
                                            egui::RichText::new(chapter)
                                                .size(14.0)
                                                .color(egui::Color32::WHITE),
                                        );
                                        ui.add_space(8.0);
                                        ui.label(
                                            egui::RichText::new(si.formatted_timestamp())
                                                .size(13.0)
                                                .color(egui::Color32::LIGHT_GRAY),
                                        );
                                        ui.add_space(8.0);
                                        ui.label(
                                            egui::RichText::new(si.formatted_play_time())
                                                .size(13.0)
                                                .color(egui::Color32::LIGHT_GRAY),
                                        );
                                    } else {
                                        ui.label(
                                            egui::RichText::new("-- Empty --")
                                                .size(14.0)
                                                .color(egui::Color32::DARK_GRAY),
                                        );
                                    }

                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            // Delete button (only if save exists)
                                            if info.is_some()
                                                && ui
                                                    .small_button(egui::RichText::new("Del").color(
                                                        egui::Color32::from_rgb(200, 80, 80),
                                                    ))
                                                    .clicked()
                                            {
                                                action = EguiAction::DeleteSlot(slot);
                                            }

                                            // Save/Load button
                                            match tab {
                                                SaveLoadTab::Save if can_save => {
                                                    if ui.small_button("Save").clicked() {
                                                        action = EguiAction::SaveToSlot(slot);
                                                    }
                                                }
                                                SaveLoadTab::Load if info.is_some() => {
                                                    if ui.small_button("Load").clicked() {
                                                        action = EguiAction::LoadFromSlot(slot);
                                                    }
                                                }
                                                _ => {}
                                            }
                                        },
                                    );
                                });
                            });
                            ui.add_space(4.0);
                        });
                    }
                });
        });

    action
}

// -- History --------------------------------------------------------------------

fn build_history_ui(ctx: &egui::Context, app_state: &AppState) -> EguiAction {
    let mut action = EguiAction::None;

    let events: Vec<&HistoryEvent> = app_state
        .session
        .vn_runtime
        .as_ref()
        .map(|rt| {
            rt.history()
                .events()
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        HistoryEvent::Dialogue { .. } | HistoryEvent::ChapterMark { .. }
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    egui::CentralPanel::default()
        .frame(panel_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("History")
                        .size(28.0)
                        .color(egui::Color32::WHITE),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("Back").size(16.0)).clicked() {
                        action = EguiAction::GoBack;
                    }
                });
            });
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            if events.is_empty() {
                ui.label(
                    egui::RichText::new("No history yet.")
                        .size(16.0)
                        .color(egui::Color32::GRAY),
                );
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for event in &events {
                            match event {
                                HistoryEvent::Dialogue {
                                    speaker, content, ..
                                } => {
                                    ui.horizontal_wrapped(|ui| {
                                        if let Some(name) = speaker {
                                            ui.label(
                                                egui::RichText::new(format!("{name}:"))
                                                    .size(15.0)
                                                    .strong()
                                                    .color(egui::Color32::from_rgb(240, 210, 140)),
                                            );
                                        }
                                        ui.label(
                                            egui::RichText::new(content)
                                                .size(15.0)
                                                .color(egui::Color32::WHITE),
                                        );
                                    });
                                    ui.add_space(6.0);
                                }
                                HistoryEvent::ChapterMark { title, .. } => {
                                    ui.add_space(8.0);
                                    ui.separator();
                                    ui.label(
                                        egui::RichText::new(title).size(18.0).strong().color(GOLD),
                                    );
                                    ui.separator();
                                    ui.add_space(8.0);
                                }
                                _ => {}
                            }
                        }
                    });
            }
        });

    action
}

// -- Toast overlay --------------------------------------------------------------

fn build_toast_overlay(ctx: &egui::Context, toast_manager: &host::ui::ToastManager) {
    for (i, toast) in toast_manager.toasts().iter().enumerate() {
        let alpha = ((1.0 - toast.fade_progress) * 230.0) as u8;
        let bg = match toast.toast_type {
            ToastType::Success => egui::Color32::from_rgba_unmultiplied(30, 80, 40, alpha),
            ToastType::Error => egui::Color32::from_rgba_unmultiplied(100, 30, 30, alpha),
            ToastType::Warning => egui::Color32::from_rgba_unmultiplied(100, 80, 20, alpha),
            ToastType::Info => egui::Color32::from_rgba_unmultiplied(40, 40, 80, alpha),
        };
        let text_alpha = ((1.0 - toast.fade_progress) * 255.0) as u8;

        egui::Area::new(egui::Id::new("toast").with(i))
            .anchor(egui::Align2::RIGHT_TOP, [-16.0, 16.0 + i as f32 * 56.0])
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(16, 10))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&toast.message).size(14.0).color(
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, text_alpha),
                        ));
                    });
            });
    }
}

// -- Helpers --------------------------------------------------------------------

fn menu_btn(ui: &mut egui::Ui, size: egui::Vec2, label: &str) -> bool {
    let clicked = ui
        .add_sized(
            size,
            egui::Button::new(egui::RichText::new(label).size(18.0)),
        )
        .clicked();
    ui.add_space(8.0);
    clicked
}

// -- main -----------------------------------------------------------------------

fn main() {
    let config = AppConfig::load(CONFIG_PATH);

    let configured = config
        .debug
        .log_level
        .as_deref()
        .unwrap_or("info")
        .trim()
        .to_ascii_lowercase();

    let level = match configured.as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" | "warning" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        "off" => LevelFilter::OFF,
        other => {
            eprintln!("Invalid log_level: '{other}', fallback to info.");
            LevelFilter::INFO
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .without_time()
        .compact()
        .with_target(false)
        .init();

    info!(path = ?CONFIG_PATH, "Config loaded");

    if let Err(e) = config.validate() {
        panic!("Config validation failed: {}", e);
    }

    let font_path = config.assets_root.join(&config.default_font);
    let font_data = match std::fs::read(&font_path) {
        Ok(data) => {
            info!(path = ?font_path, "CJK font loaded");
            Some(data)
        }
        Err(e) => {
            tracing::warn!(path = ?font_path, error = %e, "Cannot load CJK font");
            None
        }
    };

    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut HostApp {
        backend: None,
        app_state: None,
        config,
        font_data,
        initialized: false,
        settings_draft: None,
        save_load_tab: SaveLoadTab::Load,
    })
    .unwrap();
}
