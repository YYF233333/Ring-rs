//! Title 页面 UI

use crate::ui::UiRenderContext;
use crate::ui::screen_defs::ConditionalAsset;

use crate::egui_actions::{self, EguiAction};

pub fn build_title_ui(ctx: &egui::Context, ui_ctx: &UiRenderContext<'_>) -> EguiAction {
    let title_def = &ui_ctx.screen_defs.title;
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();

            if let Some(assets) = ui_ctx.assets {
                if let Some(bg_key) =
                    ConditionalAsset::resolve(&title_def.background, &ui_ctx.conditions)
                    && let Some(tex) = assets.get(bg_key)
                {
                    ui.painter().image(
                        tex.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                if let Some(overlay_key) = &title_def.overlay
                    && let Some(overlay) = assets.get(overlay_key)
                {
                    ui.painter().image(
                        overlay.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }

            let layout = ui_ctx.layout;
            let scale = ui_ctx.scale;

            let nav_x = scale.x(layout.title.navigation_xpos);
            let nav_spacing = scale.y(layout.title.navigation_spacing);
            let text_size = scale.uniform(layout.fonts.interface_text_size);
            let btn_w = scale.x(240.0);
            let btn_h = text_size + 16.0;

            let visible_buttons: Vec<_> = title_def
                .buttons
                .iter()
                .filter(|btn| {
                    btn.visible
                        .as_ref()
                        .is_none_or(|cond| cond.evaluate(&ui_ctx.conditions))
                })
                .collect();

            let total_h = visible_buttons.len() as f32 * (btn_h + nav_spacing);
            let start_y = screen_rect.center().y - total_h / 2.0;

            let idle_color = layout.colors.idle.to_egui();
            let hover_color = layout.colors.hover.to_egui();

            let mut y = start_y;
            for btn_def in &visible_buttons {
                let btn_rect =
                    egui::Rect::from_min_size(egui::pos2(nav_x, y), egui::vec2(btn_w, btn_h));

                let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
                let is_hover = resp.hovered();

                let text_color = if is_hover { hover_color } else { idle_color };
                ui.painter().text(
                    egui::pos2(btn_rect.left() + 10.0, btn_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &btn_def.label,
                    egui::FontId::proportional(text_size),
                    text_color,
                );

                if resp.clicked() {
                    action = egui_actions::button_def_to_egui(btn_def);
                }

                y += btn_h + nav_spacing;
            }
        });

    action
}
