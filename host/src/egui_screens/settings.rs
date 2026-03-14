//! Settings 页面 UI

use host::UserSettings;
use host::ui::asset_cache::UiAssetCache;
use host::ui::image_slider::{self, SliderTextures};
use host::ui::layout::{ScaleContext, UiLayoutConfig};

use crate::egui_actions::EguiAction;

/// 设置页面内容区（由 `game_menu_frame` 包裹）
pub fn build_settings_content(
    ui: &mut egui::Ui,
    draft: &mut Option<UserSettings>,
    layout: &UiLayoutConfig,
    assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
    let Some(ref mut d) = *draft else {
        return EguiAction::GoBack;
    };
    let mut action = EguiAction::None;

    let text_size = scale.uniform(layout.fonts.interface_text_size);
    let label_text_size = scale.uniform(layout.fonts.label_text_size);
    let pref_spacing = scale.y(layout.settings.pref_spacing);
    let interface_color = layout.colors.interface_text.to_egui();

    let label_w = scale.x(200.0);
    let slider_w = scale.x(400.0);
    let bar_h = scale.y(16.0);
    let thumb_sz = scale.uniform(22.0);

    let slider_tex = resolve_slider_textures(assets);

    setting_slider(
        ui,
        label_w,
        text_size,
        pref_spacing,
        slider_w,
        bar_h,
        thumb_sz,
        "文字速度",
        &mut d.text_speed,
        5.0..=100.0,
        " cps",
        1.0,
        interface_color,
        slider_tex.as_ref(),
    );
    setting_slider(
        ui,
        label_w,
        text_size,
        pref_spacing,
        slider_w,
        bar_h,
        thumb_sz,
        "自动延迟",
        &mut d.auto_delay,
        0.5..=5.0,
        " s",
        0.1,
        interface_color,
        slider_tex.as_ref(),
    );

    let mut bgm_pct = d.bgm_volume * 100.0;
    setting_slider(
        ui,
        label_w,
        text_size,
        pref_spacing,
        slider_w,
        bar_h,
        thumb_sz,
        "BGM 音量",
        &mut bgm_pct,
        0.0..=100.0,
        "%",
        1.0,
        interface_color,
        slider_tex.as_ref(),
    );
    d.bgm_volume = bgm_pct / 100.0;

    let mut sfx_pct = d.sfx_volume * 100.0;
    setting_slider(
        ui,
        label_w,
        text_size,
        pref_spacing,
        slider_w,
        bar_h,
        thumb_sz,
        "SFX 音量",
        &mut sfx_pct,
        0.0..=100.0,
        "%",
        1.0,
        interface_color,
        slider_tex.as_ref(),
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
            .button(egui::RichText::new("应用").size(label_text_size))
            .clicked()
        {
            action = EguiAction::ApplySettings(d.clone());
        }
    });

    action
}

fn resolve_slider_textures<'a>(assets: Option<&'a UiAssetCache>) -> Option<SliderTextures<'a>> {
    let assets = assets?;
    Some(SliderTextures {
        idle_bar: assets.get("slider_idle_bar")?,
        hover_bar: assets.get("slider_hover_bar")?,
        idle_thumb: assets.get("slider_idle_thumb")?,
        hover_thumb: assets.get("slider_hover_thumb")?,
    })
}

#[allow(clippy::too_many_arguments)]
fn setting_slider(
    ui: &mut egui::Ui,
    label_w: f32,
    text_size: f32,
    spacing: f32,
    slider_w: f32,
    bar_h: f32,
    thumb_sz: f32,
    label: &str,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    suffix: &str,
    step: f64,
    text_color: egui::Color32,
    textures: Option<&SliderTextures<'_>>,
) {
    ui.horizontal(|ui| {
        ui.allocate_ui(egui::vec2(label_w, 20.0), |ui| {
            ui.label(egui::RichText::new(label).size(text_size).color(text_color));
        });
        image_slider::image_slider(
            ui,
            value,
            range.clone(),
            textures,
            bar_h,
            thumb_sz,
            slider_w,
        );
        // Snap to step
        let inv = 1.0 / step as f32;
        *value = (*value * inv).round() / inv;
        // Value display
        let display = format!("{:.1}{}", *value, suffix);
        ui.label(
            egui::RichText::new(display)
                .size(text_size * 0.85)
                .color(text_color),
        );
    });
    ui.add_space(spacing);
}
