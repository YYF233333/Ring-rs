//! 确认弹窗 UI
//!
//! 模态覆盖层，在需要用户确认的操作前显示（退出/覆盖存档/返回标题等）。

use crate::ui::UiRenderContext;
use crate::ui::nine_patch::{Borders, NinePatch};

use crate::egui_actions::EguiAction;

/// 确认弹窗数据
#[derive(Debug, Clone)]
pub struct ConfirmDialog {
    pub message: String,
    pub on_confirm: EguiAction,
    pub on_cancel: EguiAction,
}

/// 绘制确认弹窗（如果有 pending confirm）。
///
/// 返回用户选择的 action（确认/取消），或 `None` 表示无操作。
pub fn build_confirm_overlay(
    ctx: &egui::Context,
    dialog: &ConfirmDialog,
    ui_ctx: &UiRenderContext<'_>,
) -> Option<EguiAction> {
    let mut result = None;
    let layout = ui_ctx.layout;
    let scale = ui_ctx.scale;

    // Semi-transparent overlay
    egui::Area::new(egui::Id::new("confirm_overlay"))
        .order(egui::Order::Foreground)
        .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            let (_, resp) = ui.allocate_exact_size(screen_rect.size(), egui::Sense::click());

            if let Some(assets) = ui_ctx.assets {
                if let Some(tex) = assets.get("confirm_overlay") {
                    ui.painter().image(
                        tex.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            } else {
                ui.painter().rect_filled(
                    screen_rect,
                    0.0,
                    egui::Color32::from_rgba_unmultiplied(0, 0, 0, 180),
                );
            }

            // Center dialog frame
            let borders = layout.confirm.frame_borders;
            let frame_w = scale.x(600.0);
            let frame_h = scale.y(300.0);
            let frame_rect =
                egui::Rect::from_center_size(screen_rect.center(), egui::vec2(frame_w, frame_h));

            if let Some(assets) = ui_ctx.assets {
                if let Some(tex) = assets.get("frame") {
                    let np = NinePatch::new(tex, Borders::from_array(borders));
                    np.paint(ui.painter(), frame_rect, egui::Color32::WHITE);
                }
            } else {
                ui.painter()
                    .rect_filled(frame_rect, 8.0, egui::Color32::from_rgb(25, 25, 50));
            }

            // Message text
            let text_size = scale.uniform(layout.fonts.interface_text_size);
            let interface_color = layout.colors.interface_text.to_egui();
            ui.painter().text(
                egui::pos2(frame_rect.center().x, frame_rect.center().y - scale.y(30.0)),
                egui::Align2::CENTER_CENTER,
                &dialog.message,
                egui::FontId::proportional(text_size),
                interface_color,
            );

            // Buttons
            let btn_w = scale.x(140.0);
            let btn_h = text_size + 16.0;
            let btn_spacing = scale.x(40.0);
            let btn_y = frame_rect.center().y + scale.y(40.0);

            let confirm_rect = egui::Rect::from_min_size(
                egui::pos2(frame_rect.center().x - btn_w - btn_spacing / 2.0, btn_y),
                egui::vec2(btn_w, btn_h),
            );
            let cancel_rect = egui::Rect::from_min_size(
                egui::pos2(frame_rect.center().x + btn_spacing / 2.0, btn_y),
                egui::vec2(btn_w, btn_h),
            );

            let idle_color = layout.colors.idle.to_egui();
            let hover_color = layout.colors.hover.to_egui();

            // Confirm button
            let confirm_resp = ui.allocate_rect(confirm_rect, egui::Sense::click());
            let conf_hover = confirm_resp.hovered();
            ui.painter().rect_filled(
                confirm_rect,
                4.0,
                if conf_hover {
                    egui::Color32::from_rgb(60, 60, 100)
                } else {
                    egui::Color32::from_rgb(40, 40, 70)
                },
            );
            ui.painter().text(
                confirm_rect.center(),
                egui::Align2::CENTER_CENTER,
                "确定",
                egui::FontId::proportional(text_size),
                if conf_hover { hover_color } else { idle_color },
            );

            // Cancel button
            let cancel_resp = ui.allocate_rect(cancel_rect, egui::Sense::click());
            let canc_hover = cancel_resp.hovered();
            ui.painter().rect_filled(
                cancel_rect,
                4.0,
                if canc_hover {
                    egui::Color32::from_rgb(60, 60, 100)
                } else {
                    egui::Color32::from_rgb(40, 40, 70)
                },
            );
            ui.painter().text(
                cancel_rect.center(),
                egui::Align2::CENTER_CENTER,
                "取消",
                egui::FontId::proportional(text_size),
                if canc_hover { hover_color } else { idle_color },
            );

            if confirm_resp.clicked() {
                result = Some(dialog.on_confirm.clone());
            } else if cancel_resp.clicked() || resp.clicked() {
                result = Some(dialog.on_cancel.clone());
            }
        });

    result
}
