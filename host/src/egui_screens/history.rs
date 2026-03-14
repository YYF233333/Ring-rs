//! History 页面 UI

use host::app::AppState;
use host::ui::layout::{ScaleContext, UiLayoutConfig};
use vn_runtime::HistoryEvent;

use crate::egui_actions::EguiAction;

/// 历史页面内容区（由 `game_menu_frame` 包裹）
pub fn build_history_content(
    ui: &mut egui::Ui,
    app_state: &AppState,
    layout: &UiLayoutConfig,
    scale: &ScaleContext,
) -> EguiAction {
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
    let interface_color = layout.colors.interface_text.to_egui();
    let accent_color = layout.colors.accent.to_egui();
    let idle_color = layout.colors.idle.to_egui();

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

    EguiAction::None
}
