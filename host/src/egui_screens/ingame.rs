//! InGame 页面 UI（对话框 + 选项）

use host::RenderState;

pub fn build_ingame_ui(ctx: &egui::Context, render_state: &RenderState) {
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
