//! Settings 页面 UI

use host::UserSettings;

use crate::egui_actions::EguiAction;
use crate::egui_screens::helpers::panel_frame;

pub fn build_settings_ui(ctx: &egui::Context, draft: &mut Option<UserSettings>) -> EguiAction {
    let Some(ref mut d) = *draft else {
        return EguiAction::GoBack;
    };
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(panel_frame())
        .show(ctx, |ui| {
            ui.heading(
                egui::RichText::new("Settings")
                    .size(28.0)
                    .color(egui::Color32::WHITE),
            );
            ui.add_space(24.0);

            let label_w = 140.0;

            setting_slider(
                ui,
                label_w,
                "Text Speed",
                &mut d.text_speed,
                5.0..=100.0,
                " cps",
                1.0,
            );
            setting_slider(
                ui,
                label_w,
                "Auto Delay",
                &mut d.auto_delay,
                0.5..=5.0,
                " s",
                0.1,
            );

            let mut bgm_pct = d.bgm_volume * 100.0;
            setting_slider(
                ui,
                label_w,
                "BGM Volume",
                &mut bgm_pct,
                0.0..=100.0,
                "%",
                1.0,
            );
            d.bgm_volume = bgm_pct / 100.0;

            let mut sfx_pct = d.sfx_volume * 100.0;
            setting_slider(
                ui,
                label_w,
                "SFX Volume",
                &mut sfx_pct,
                0.0..=100.0,
                "%",
                1.0,
            );
            d.sfx_volume = sfx_pct / 100.0;

            // Muted
            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
                    ui.label(
                        egui::RichText::new("Muted")
                            .size(16.0)
                            .color(egui::Color32::WHITE),
                    );
                });
                ui.checkbox(&mut d.muted, "");
            });
            ui.add_space(24.0);

            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("Apply & Back").size(16.0))
                    .clicked()
                {
                    action = EguiAction::ApplySettings(d.clone());
                }
                ui.add_space(16.0);
                if ui
                    .button(egui::RichText::new("Cancel").size(16.0))
                    .clicked()
                {
                    action = EguiAction::GoBack;
                }
            });
        });

    action
}

fn setting_slider(
    ui: &mut egui::Ui,
    label_w: f32,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    step: f64,
) {
    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
            ui.label(
                egui::RichText::new(label)
                    .size(16.0)
                    .color(egui::Color32::WHITE),
            );
        });
        ui.add(egui::Slider::new(value, range).suffix(suffix).step_by(step));
    });
    ui.add_space(12.0);
}
