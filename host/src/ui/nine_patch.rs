//! # NinePatch 九宫格渲染
//!
//! 将一张图片按边框值切为 9 块，渲染到任意大小的矩形区域。
//! 四角保持原始尺寸，边条拉伸填充，中心区域拉伸铺满。

/// 九宫格边框值（像素）
#[derive(Debug, Clone, Copy)]
pub struct Borders {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Borders {
    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn from_array(arr: [f32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

/// 九宫格渲染器
pub struct NinePatch<'a> {
    texture: &'a egui::TextureHandle,
    borders: Borders,
}

impl<'a> NinePatch<'a> {
    pub fn new(texture: &'a egui::TextureHandle, borders: Borders) -> Self {
        Self { texture, borders }
    }

    /// 在指定矩形区域内绘制九宫格图片
    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect, tint: egui::Color32) {
        let [tex_w, tex_h] = self.texture.size();
        let tw = tex_w as f32;
        let th = tex_h as f32;

        let b = &self.borders;
        let tex_id = self.texture.id();

        // UV 边界
        let u_left = b.left / tw;
        let u_right = 1.0 - b.right / tw;
        let v_top = b.top / th;
        let v_bottom = 1.0 - b.bottom / th;

        // 像素边界
        let px_left = rect.left() + b.left;
        let px_right = rect.right() - b.right;
        let px_top = rect.top() + b.top;
        let px_bottom = rect.bottom() - b.bottom;

        // 如果目标矩形小于边框之和，做 clamp 避免翻转
        let px_left = px_left.min(px_right);
        let px_top = px_top.min(px_bottom);

        let patches: [(egui::Rect, egui::Rect); 9] = [
            // top-left
            (
                egui::Rect::from_min_max(rect.left_top(), egui::pos2(px_left, px_top)),
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(u_left, v_top)),
            ),
            // top-center
            (
                egui::Rect::from_min_max(
                    egui::pos2(px_left, rect.top()),
                    egui::pos2(px_right, px_top),
                ),
                egui::Rect::from_min_max(egui::pos2(u_left, 0.0), egui::pos2(u_right, v_top)),
            ),
            // top-right
            (
                egui::Rect::from_min_max(
                    egui::pos2(px_right, rect.top()),
                    egui::pos2(rect.right(), px_top),
                ),
                egui::Rect::from_min_max(egui::pos2(u_right, 0.0), egui::pos2(1.0, v_top)),
            ),
            // middle-left
            (
                egui::Rect::from_min_max(
                    egui::pos2(rect.left(), px_top),
                    egui::pos2(px_left, px_bottom),
                ),
                egui::Rect::from_min_max(egui::pos2(0.0, v_top), egui::pos2(u_left, v_bottom)),
            ),
            // center
            (
                egui::Rect::from_min_max(
                    egui::pos2(px_left, px_top),
                    egui::pos2(px_right, px_bottom),
                ),
                egui::Rect::from_min_max(egui::pos2(u_left, v_top), egui::pos2(u_right, v_bottom)),
            ),
            // middle-right
            (
                egui::Rect::from_min_max(
                    egui::pos2(px_right, px_top),
                    egui::pos2(rect.right(), px_bottom),
                ),
                egui::Rect::from_min_max(egui::pos2(u_right, v_top), egui::pos2(1.0, v_bottom)),
            ),
            // bottom-left
            (
                egui::Rect::from_min_max(
                    egui::pos2(rect.left(), px_bottom),
                    egui::pos2(px_left, rect.bottom()),
                ),
                egui::Rect::from_min_max(egui::pos2(0.0, v_bottom), egui::pos2(u_left, 1.0)),
            ),
            // bottom-center
            (
                egui::Rect::from_min_max(
                    egui::pos2(px_left, px_bottom),
                    egui::pos2(px_right, rect.bottom()),
                ),
                egui::Rect::from_min_max(egui::pos2(u_left, v_bottom), egui::pos2(u_right, 1.0)),
            ),
            // bottom-right
            (
                egui::Rect::from_min_max(egui::pos2(px_right, px_bottom), rect.right_bottom()),
                egui::Rect::from_min_max(egui::pos2(u_right, v_bottom), egui::pos2(1.0, 1.0)),
            ),
        ];

        for (pixel_rect, uv_rect) in &patches {
            if pixel_rect.width() > 0.0 && pixel_rect.height() > 0.0 {
                painter.image(tex_id, *pixel_rect, *uv_rect, tint);
            }
        }
    }
}
