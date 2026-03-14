//! InGameMenu 页面 UI

use host::AppMode;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};

use crate::egui_actions::EguiAction;

pub fn build_ingame_menu_ui(
    ctx: &egui::Context,
    layout: &UiLayoutConfig,
    _assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
    let mut action = EguiAction::None;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 180))
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();
            let text_size = scale.uniform(layout.fonts.interface_text_size);
            let btn_h = text_size + 16.0;
            let spacing = scale.y(10.0);

            let entries: &[(&str, EguiAction)] = &[
                ("继续", EguiAction::GoBack),
                ("保存", EguiAction::OpenSave),
                ("读取", EguiAction::OpenLoad),
                ("设置", EguiAction::NavigateTo(AppMode::Settings)),
                ("历史", EguiAction::NavigateTo(AppMode::History)),
                (
                    "返回标题",
                    EguiAction::ShowConfirm {
                        message: "确定返回标题画面？".into(),
                        on_confirm: Box::new(EguiAction::ReturnToTitle),
                    },
                ),
                (
                    "退出",
                    EguiAction::ShowConfirm {
                        message: "确定退出游戏？".into(),
                        on_confirm: Box::new(EguiAction::Exit),
                    },
                ),
            ];

            let total_h = entries.len() as f32 * (btn_h + spacing);
            let start_y = screen_rect.center().y - total_h / 2.0;
            let btn_w = scale.x(260.0);
            let center_x = screen_rect.center().x - btn_w / 2.0;

            let idle_color = layout.colors.idle.to_egui();
            let hover_color = layout.colors.hover.to_egui();

            let mut y = start_y;
            for (label, btn_action) in entries {
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
                    *label,
                    egui::FontId::proportional(text_size),
                    text_color,
                );

                if resp.clicked() {
                    action = btn_action.clone();
                }
                y += btn_h + spacing;
            }
        });

    action
}
