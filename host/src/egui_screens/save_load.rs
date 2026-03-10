//! SaveLoad 页面 UI

use host::SaveLoadTab;
use host::save_manager::SaveInfo;

use crate::egui_actions::EguiAction;
use crate::egui_screens::helpers::{GOLD, panel_frame};

pub fn build_save_load_ui(
    ctx: &egui::Context,
    tab: SaveLoadTab,
    save_infos: &[Option<SaveInfo>],
    can_save: bool,
) -> EguiAction {
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(panel_frame())
        .show(ctx, |ui| {
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
                                    ui.label(
                                        egui::RichText::new(format!("Slot {slot:02}"))
                                            .size(16.0)
                                            .strong()
                                            .color(GOLD),
                                    );
                                    ui.add_space(12.0);

                                    if let Some(si) = info {
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
                                            if info.is_some()
                                                && ui
                                                    .small_button(egui::RichText::new("Del").color(
                                                        egui::Color32::from_rgb(200, 80, 80),
                                                    ))
                                                    .clicked()
                                            {
                                                action = EguiAction::DeleteSlot(slot);
                                            }

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
