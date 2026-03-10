//! Title 页面 UI

use host::AppMode;
use host::app::AppState;

use crate::egui_actions::EguiAction;
use crate::egui_screens::helpers::{GOLD, dark_frame, menu_btn};

pub fn build_title_ui(ctx: &egui::Context, app_state: &AppState) -> EguiAction {
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
