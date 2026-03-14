//! Toast 覆盖层 UI

use host::ui::ToastManager;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};
use host::ui::nine_patch::{Borders, NinePatch};
use host::ui::toast::ToastType;

pub fn build_toast_overlay(
    ctx: &egui::Context,
    toast_manager: &ToastManager,
    layout: &UiLayoutConfig,
    assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) {
    let text_size = scale.uniform(layout.fonts.notify_text_size);
    let ypos = scale.y(layout.notify.ypos);
    let toast_h = text_size + scale.y(20.0);
    let spacing = scale.y(8.0);
    let margin_x = scale.x(16.0);
    let borders_raw = layout.notify.frame_borders;
    let borders = Borders::new(
        borders_raw[0],
        borders_raw[1],
        borders_raw[2],
        borders_raw[3],
    );

    for (i, toast) in toast_manager.toasts().iter().enumerate() {
        let alpha = ((1.0 - toast.fade_progress) * 230.0) as u8;
        let text_alpha = ((1.0 - toast.fade_progress) * 255.0) as u8;
        let y_offset = ypos + i as f32 * (toast_h + spacing);

        egui::Area::new(egui::Id::new("toast").with(i))
            .anchor(egui::Align2::RIGHT_TOP, [-margin_x, y_offset])
            .interactable(false)
            .order(egui::Order::Tooltip)
            .show(ctx, |ui| {
                let galley = ui.fonts(|f| {
                    f.layout_no_wrap(
                        toast.message.clone(),
                        egui::FontId::proportional(text_size),
                        egui::Color32::WHITE,
                    )
                });
                let text_w = galley.size().x;
                let frame_w = text_w + margin_x * 2.0;
                let frame_h = toast_h;

                let (rect, _) =
                    ui.allocate_exact_size(egui::vec2(frame_w, frame_h), egui::Sense::hover());

                let painter = ui.painter();

                let mut drew_ninepatch = false;
                if let Some(assets) = assets {
                    if let Some(tex) = assets.get("notify") {
                        let np = NinePatch::new(tex, borders);
                        let tint = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
                        np.paint(painter, rect, tint);
                        drew_ninepatch = true;
                    }
                }

                if !drew_ninepatch {
                    let bg = match toast.toast_type {
                        ToastType::Success => {
                            egui::Color32::from_rgba_unmultiplied(30, 80, 40, alpha)
                        }
                        ToastType::Error => {
                            egui::Color32::from_rgba_unmultiplied(100, 30, 30, alpha)
                        }
                        ToastType::Warning => {
                            egui::Color32::from_rgba_unmultiplied(100, 80, 20, alpha)
                        }
                        ToastType::Info => egui::Color32::from_rgba_unmultiplied(40, 40, 80, alpha),
                    };
                    painter.rect_filled(rect, 6.0, bg);
                }

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &toast.message,
                    egui::FontId::proportional(text_size),
                    egui::Color32::from_rgba_unmultiplied(255, 255, 255, text_alpha),
                );
            });
    }
}
