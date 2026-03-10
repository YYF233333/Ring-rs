//! Toast 覆盖层 UI

use host::ui::ToastManager;
use host::ui::toast::ToastType;

pub fn build_toast_overlay(ctx: &egui::Context, toast_manager: &ToastManager) {
    for (i, toast) in toast_manager.toasts().iter().enumerate() {
        let alpha = ((1.0 - toast.fade_progress) * 230.0) as u8;
        let bg = match toast.toast_type {
            ToastType::Success => egui::Color32::from_rgba_unmultiplied(30, 80, 40, alpha),
            ToastType::Error => egui::Color32::from_rgba_unmultiplied(100, 30, 30, alpha),
            ToastType::Warning => egui::Color32::from_rgba_unmultiplied(100, 80, 20, alpha),
            ToastType::Info => egui::Color32::from_rgba_unmultiplied(40, 40, 80, alpha),
        };
        let text_alpha = ((1.0 - toast.fade_progress) * 255.0) as u8;

        egui::Area::new(egui::Id::new("toast").with(i))
            .anchor(egui::Align2::RIGHT_TOP, [-16.0, 16.0 + i as f32 * 56.0])
            .interactable(false)
            .show(ctx, |ui| {
                egui::Frame::new()
                    .fill(bg)
                    .corner_radius(6.0)
                    .inner_margin(egui::Margin::symmetric(16, 10))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new(&toast.message).size(14.0).color(
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, text_alpha),
                        ));
                    });
            });
    }
}
