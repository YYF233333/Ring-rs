//! InGameMenu 页面 UI

use host::ui::UiRenderContext;

use crate::egui_actions::{self, EguiAction};

pub fn build_ingame_menu_ui(ctx: &egui::Context, ui_ctx: &UiRenderContext<'_>) -> EguiAction {
    let mut action = EguiAction::None;
    let buttons = &ui_ctx.screen_defs.ingame_menu.buttons;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();
            let layout = ui_ctx.layout;
            let scale = ui_ctx.scale;
            let text_size = scale.uniform(layout.fonts.interface_text_size);
            let btn_h = text_size + 16.0;
            let spacing = scale.y(10.0);

            let visible_buttons: Vec<_> = buttons
                .iter()
                .filter(|btn| {
                    btn.visible
                        .as_ref()
                        .is_none_or(|cond| cond.evaluate(&ui_ctx.conditions))
                })
                .collect();

            let total_h = visible_buttons.len() as f32 * (btn_h + spacing);
            let start_y = screen_rect.center().y - total_h / 2.0;
            let btn_w = scale.x(260.0);
            let center_x = screen_rect.center().x - btn_w / 2.0;

            let idle_color = layout.colors.idle.to_egui();
            let hover_color = layout.colors.hover.to_egui();

            let mut y = start_y;
            for btn_def in &visible_buttons {
                let btn_rect =
                    egui::Rect::from_min_size(egui::pos2(center_x, y), egui::vec2(btn_w, btn_h));
                let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
                let is_hover = resp.hovered();

                let bg = if is_hover {
                    egui::Color32::from_rgba_unmultiplied(60, 60, 100, 150)
                } else {
                    egui::Color32::from_rgba_unmultiplied(30, 30, 60, 100)
                };
                ui.painter().rect_filled(btn_rect, 4.0, bg);

                let text_color = if is_hover { hover_color } else { idle_color };
                ui.painter().text(
                    btn_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &btn_def.label,
                    egui::FontId::proportional(text_size),
                    text_color,
                );

                if resp.clicked() {
                    action = egui_actions::button_def_to_egui(btn_def);
                }
                y += btn_h + spacing;
            }
        });

    action
}
