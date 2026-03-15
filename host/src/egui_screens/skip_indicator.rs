//! 快进指示器 UI
//!
//! 在游戏画面左上角显示"正在快进"提示，仅在 Skip 模式下可见。

use host::ui::UiRenderContext;
use host::ui::nine_patch::{Borders, NinePatch};

pub fn build_skip_indicator(ctx: &egui::Context, ui_ctx: &UiRenderContext<'_>) {
    let layout = ui_ctx.layout;
    let scale = ui_ctx.scale;
    let y = scale.y(layout.skip_indicator.ypos);
    let text_size = scale.uniform(layout.fonts.notify_text_size);
    let borders = layout.skip_indicator.frame_borders;

    // Animate arrow cycling

    // Animate arrow cycling
    let time = ctx.input(|i| i.time);
    let arrow_count = ((time * 3.0) as usize % 4).max(1);
    let arrows: String = "›".repeat(arrow_count);
    let full_label = format!("正在快进 {arrows}");

    let frame_w = scale.x(borders[0] + borders[2]) + text_size * full_label.len() as f32 * 0.55;
    let frame_h = scale.y(borders[1] + borders[3]) + text_size + 8.0;

    egui::Area::new(egui::Id::new("skip_indicator"))
        .anchor(egui::Align2::LEFT_TOP, [scale.x(10.0), y])
        .interactable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(frame_w, frame_h), egui::Sense::hover());

            let painter = ui.painter();

            if let Some(assets) = ui_ctx.assets {
                if let Some(tex) = assets.get("skip") {
                    let np = NinePatch::new(tex, Borders::from_array(borders));
                    np.paint(painter, rect, egui::Color32::WHITE);
                }
            } else {
                painter.rect_filled(
                    rect,
                    4.0,
                    egui::Color32::from_rgba_unmultiplied(20, 60, 40, 200),
                );
            }

            let interface_color = layout.colors.interface_text.to_egui();
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &full_label,
                egui::FontId::proportional(text_size),
                interface_color,
            );
        });

    ctx.request_repaint();
}
