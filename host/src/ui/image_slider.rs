//! 基于图片素材的自定义水平滑块 widget
//!
//! 使用 NinePatch 渲染轨道，图片渲染拇指。
//! Fallback：素材不可用时使用 egui 原生绘制。

use super::nine_patch::{Borders, NinePatch};

/// 图片滑块所需的纹理引用
pub struct SliderTextures<'a> {
    pub idle_bar: &'a egui::TextureHandle,
    pub hover_bar: &'a egui::TextureHandle,
    pub idle_thumb: &'a egui::TextureHandle,
    pub hover_thumb: &'a egui::TextureHandle,
}

/// 绘制一个基于图片素材的水平滑块，返回值是否发生变化
pub fn image_slider(
    ui: &mut egui::Ui,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    textures: Option<&SliderTextures<'_>>,
    bar_height: f32,
    thumb_size: f32,
    width: f32,
) -> bool {
    let total_h = thumb_size.max(bar_height);
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(width, total_h), egui::Sense::click_and_drag());

    let range_min = *range.start();
    let range_max = *range.end();
    let range_span = range_max - range_min;
    if range_span <= 0.0 {
        return false;
    }

    let usable_left = rect.left() + thumb_size / 2.0;
    let usable_right = rect.right() - thumb_size / 2.0;
    let usable_width = usable_right - usable_left;

    let mut changed = false;
    if (response.dragged() || response.clicked())
        && let Some(pos) = response.interact_pointer_pos()
    {
        let t = ((pos.x - usable_left) / usable_width).clamp(0.0, 1.0);
        let new_val = range_min + t * range_span;
        if (new_val - *value).abs() > f32::EPSILON {
            *value = new_val;
            changed = true;
        }
    }

    let t = ((*value - range_min) / range_span).clamp(0.0, 1.0);
    let thumb_center_x = usable_left + t * usable_width;

    let painter = ui.painter();
    let is_hover = response.hovered() || response.dragged();

    if let Some(tex) = textures {
        let bar_tex = if is_hover {
            tex.hover_bar
        } else {
            tex.idle_bar
        };
        let bar_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(width, bar_height));
        let borders = Borders::new(6.0, 6.0, 6.0, 6.0);
        let np = NinePatch::new(bar_tex, borders);
        np.paint(painter, bar_rect, egui::Color32::WHITE);

        // Filled portion
        let filled_bar = egui::Rect::from_min_max(
            bar_rect.left_top(),
            egui::pos2(thumb_center_x, bar_rect.bottom()),
        );
        let fill_borders = Borders::new(6.0, 6.0, 0.0, 6.0);
        let fill_np = NinePatch::new(
            if is_hover {
                tex.hover_bar
            } else {
                tex.idle_bar
            },
            fill_borders,
        );
        fill_np.paint(painter, filled_bar, egui::Color32::WHITE);

        let thumb_tex = if is_hover {
            tex.hover_thumb
        } else {
            tex.idle_thumb
        };
        let thumb_rect = egui::Rect::from_center_size(
            egui::pos2(thumb_center_x, rect.center().y),
            egui::vec2(thumb_size, thumb_size),
        );
        painter.image(
            thumb_tex.id(),
            thumb_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );
    } else {
        let track_rect =
            egui::Rect::from_center_size(rect.center(), egui::vec2(width, bar_height * 0.4));
        painter.rect_filled(track_rect, 3.0, egui::Color32::from_rgb(60, 60, 80));

        let filled = egui::Rect::from_min_max(
            track_rect.left_top(),
            egui::pos2(thumb_center_x, track_rect.bottom()),
        );
        painter.rect_filled(filled, 3.0, egui::Color32::from_rgb(100, 130, 200));

        let thumb_color = if is_hover {
            egui::Color32::from_rgb(220, 220, 255)
        } else {
            egui::Color32::from_rgb(180, 180, 220)
        };
        painter.circle_filled(
            egui::pos2(thumb_center_x, rect.center().y),
            thumb_size / 2.0,
            thumb_color,
        );
    }

    changed
}
