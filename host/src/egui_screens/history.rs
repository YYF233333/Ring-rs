//! History 页面 UI

use host::app::AppState;
use vn_runtime::HistoryEvent;

use crate::egui_actions::EguiAction;
use crate::egui_screens::helpers::{GOLD, panel_frame};

pub fn build_history_ui(ctx: &egui::Context, app_state: &AppState) -> EguiAction {
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
