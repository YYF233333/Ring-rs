//! # TextRenderer 模块
//!
//! 保留选项布局计算；实际文本渲染已迁移到 egui。

/// 文本渲染器（保留用于选项区域计算）
#[derive(Debug)]
pub struct TextRenderer;

impl TextRenderer {
    pub fn new() -> Self {
        Self
    }

    /// 计算选项矩形区域（用于点击检测）
    ///
    /// 返回每个选项的 (x, y, width, height) 元组数组
    pub fn get_choice_rects(
        &self,
        choice_count: usize,
        screen_w: f32,
        screen_h: f32,
    ) -> Vec<(f32, f32, f32, f32)> {
        let choice_height = 50.0;
        let choice_spacing = 10.0;
        let total_height = choice_count as f32 * (choice_height + choice_spacing) - choice_spacing;
        let start_y = (screen_h - total_height) / 2.0;

        let box_w = screen_w * 0.6;
        let box_x = (screen_w - box_w) / 2.0;

        (0..choice_count)
            .map(|i| {
                let y = start_y + i as f32 * (choice_height + choice_spacing);
                (box_x, y, box_w, choice_height)
            })
            .collect()
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}
