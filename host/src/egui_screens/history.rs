//! History 页面 UI

use host::app::AppState;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};
use vn_runtime::HistoryEvent;

use crate::egui_actions::EguiAction;

pub fn build_history_ui(
    ctx: &egui::Context,
    app_state: &AppState,
    layout: &UiLayoutConfig,
    _assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
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

    let text_size = scale.uniform(layout.fonts.text_size);
    let name_width = scale.x(layout.history.name_width);
    let text_width = scale.x(layout.history.text_width);
    let _entry_height = scale.y(layout.history.entry_height);
    let interface_color = layout.colors.interface_text.to_egui();
    let accent_color = layout.colors.accent.to_egui();
    let idle_color = layout.colors.idle.to_egui();

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(15, 15, 35, 240))
                .inner_margin(scale.uniform(40.0)),
        )
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("历史")
                        .size(scale.uniform(layout.fonts.title_text_size * 0.6))
                        .color(interface_color),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(
                            egui::RichText::new("返回")
                                .size(scale.uniform(layout.fonts.interface_text_size)),
                        )
                        .clicked()
                    {
                        action = EguiAction::GoBack;
                    }
                });
            });
            ui.add_space(scale.y(12.0));
            ui.separator();
            ui.add_space(scale.y(8.0));

            if events.is_empty() {
                ui.label(
                    egui::RichText::new("暂无历史记录。")
                        .size(text_size)
                        .color(idle_color),
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
                                    let row_min_h = text_size + scale.y(8.0);
                                    ui.horizontal(|ui| {
                                        ui.allocate_ui(egui::vec2(name_width, row_min_h), |ui| {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::TOP),
                                                |ui| {
                                                    if let Some(name) = speaker {
                                                        ui.label(
                                                            egui::RichText::new(name)
                                                                .size(text_size)
                                                                .strong()
                                                                .color(accent_color),
                                                        );
                                                    }
                                                },
                                            );
                                        });
                                        ui.add_space(scale.x(22.0));
                                        ui.allocate_ui(egui::vec2(text_width, row_min_h), |ui| {
                                            ui.label(
                                                egui::RichText::new(content)
                                                    .size(text_size)
                                                    .color(interface_color),
                                            );
                                        });
                                    });
                                    ui.add_space(scale.y(6.0));
                                }
                                HistoryEvent::ChapterMark { title, .. } => {
                                    ui.add_space(scale.y(8.0));
                                    ui.separator();
                                    ui.label(
                                        egui::RichText::new(title)
                                            .size(scale.uniform(layout.fonts.label_text_size))
                                            .strong()
                                            .color(accent_color),
                                    );
                                    ui.separator();
                                    ui.add_space(scale.y(8.0));
                                }
                                _ => {}
                            }
                        }
                    });
            }
        });

    action
}
