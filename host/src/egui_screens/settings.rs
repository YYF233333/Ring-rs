//! Settings 页面 UI

use host::UserSettings;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};

use crate::egui_actions::EguiAction;

pub fn build_settings_ui(
    ctx: &egui::Context,
    draft: &mut Option<UserSettings>,
    layout: &UiLayoutConfig,
    _assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
    let Some(ref mut d) = *draft else {
        return EguiAction::GoBack;
    };
    let mut action = EguiAction::None;

    let text_size = scale.uniform(layout.fonts.interface_text_size);
    let label_text_size = scale.uniform(layout.fonts.label_text_size);
    let pref_spacing = scale.y(layout.settings.pref_spacing);

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(15, 15, 35, 240))
                .inner_margin(scale.uniform(40.0)),
        )
        .show(ctx, |ui| {
            let interface_color = layout.colors.interface_text.to_egui();

            ui.heading(
                egui::RichText::new("设置")
                    .size(scale.uniform(layout.fonts.title_text_size * 0.6))
                    .color(interface_color),
            );
            ui.add_space(scale.y(24.0));

            let label_w = scale.x(200.0);

            setting_slider(
                ui,
                label_w,
                text_size,
                pref_spacing,
                "文字速度",
                &mut d.text_speed,
                5.0..=100.0,
                " cps",
                1.0,
                interface_color,
            );
            setting_slider(
                ui,
                label_w,
                text_size,
                pref_spacing,
                "自动延迟",
                &mut d.auto_delay,
                0.5..=5.0,
                " s",
                0.1,
                interface_color,
            );

            let mut bgm_pct = d.bgm_volume * 100.0;
            setting_slider(
                ui,
                label_w,
                text_size,
                pref_spacing,
                "BGM 音量",
                &mut bgm_pct,
                0.0..=100.0,
                "%",
                1.0,
                interface_color,
            );
            d.bgm_volume = bgm_pct / 100.0;

            let mut sfx_pct = d.sfx_volume * 100.0;
            setting_slider(
                ui,
                label_w,
                text_size,
                pref_spacing,
                "SFX 音量",
                &mut sfx_pct,
                0.0..=100.0,
                "%",
                1.0,
                interface_color,
            );
            d.sfx_volume = sfx_pct / 100.0;

            ui.horizontal(|ui| {
                ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
                    ui.label(
                        egui::RichText::new("静音")
                            .size(text_size)
                            .color(interface_color),
                    );
                });
                ui.checkbox(&mut d.muted, "");
            });
            ui.add_space(scale.y(24.0));

            ui.horizontal(|ui| {
                if ui
                    .button(egui::RichText::new("应用并返回").size(label_text_size))
                    .clicked()
                {
                    action = EguiAction::ApplySettings(d.clone());
                }
                ui.add_space(scale.x(16.0));
                if ui
                    .button(egui::RichText::new("取消").size(label_text_size))
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
    text_size: f32,
    spacing: f32,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    step: f64,
    text_color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
            ui.label(egui::RichText::new(label).size(text_size).color(text_color));
        });
        ui.add(egui::Slider::new(value, range).suffix(suffix).step_by(step));
    });
    ui.add_space(spacing);
}
