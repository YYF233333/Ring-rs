//! Title 页面 UI

use crate::egui_actions::EguiAction;
use host::AppMode;
use host::app::AppState;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};

pub fn build_title_ui(
    ctx: &egui::Context,
    app_state: &AppState,
    layout: &UiLayoutConfig,
    assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
    let has_continue = app_state.save_manager.has_continue();
    let is_winter = app_state.persistent_store.is_season_complete("summer");
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();

            if let Some(assets) = assets {
                let bg_key = if is_winter {
                    "main_winter"
                } else {
                    "main_summer"
                };
                if let Some(tex) = assets.get(bg_key) {
                    ui.painter().image(
                        tex.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                if let Some(overlay) = assets.get("main_menu_overlay") {
                    ui.painter().image(
                        overlay.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }

            let nav_x = scale.x(layout.title.navigation_xpos);
            let nav_spacing = scale.y(layout.title.navigation_spacing);
            let text_size = scale.uniform(layout.fonts.interface_text_size);
            let btn_w = scale.x(240.0);
            let btn_h = text_size + 16.0;

            let entries: Vec<(&str, EguiAction, bool)> = vec![
                ("开始游戏", EguiAction::StartGame, true),
                ("冬篇", EguiAction::StartWinter, is_winter),
                ("继续游戏", EguiAction::ContinueGame, has_continue),
                ("读取游戏", EguiAction::OpenLoad, true),
                ("设置", EguiAction::NavigateTo(AppMode::Settings), true),
                (
                    "退出",
                    EguiAction::ShowConfirm {
                        message: "确定退出游戏？".into(),
                        on_confirm: Box::new(EguiAction::Exit),
                    },
                    true,
                ),
            ];

            let total_h =
                entries.iter().filter(|(_, _, show)| *show).count() as f32 * (btn_h + nav_spacing);
            let start_y = screen_rect.center().y - total_h / 2.0;

            let idle_color = layout.colors.idle.to_egui();
            let hover_color = layout.colors.hover.to_egui();

            let mut y = start_y;
            for (label, btn_action, show) in &entries {
                if !show {
                    continue;
                }

                let btn_rect =
                    egui::Rect::from_min_size(egui::pos2(nav_x, y), egui::vec2(btn_w, btn_h));

                let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
                let is_hover = resp.hovered();

                let text_color = if is_hover { hover_color } else { idle_color };
                ui.painter().text(
                    egui::pos2(btn_rect.left() + 10.0, btn_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    *label,
                    egui::FontId::proportional(text_size),
                    text_color,
                );

                if resp.clicked() {
                    action = btn_action.clone();
                }

                y += btn_h + nav_spacing;
            }
        });

    action
}
