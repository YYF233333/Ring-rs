//! InGameMenu 页面 UI

use host::AppMode;

use crate::egui_actions::EguiAction;

pub fn build_ingame_menu_ui(ctx: &egui::Context) -> EguiAction {
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
