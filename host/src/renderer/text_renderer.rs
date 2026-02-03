//! # TextRenderer 模块
//!
//! 文本渲染器，负责对话框、角色名、章节标题等文本的渲染。

use macroquad::prelude::*;

/// 对话框配置
const DIALOGUE_BOX_MARGIN: f32 = 40.0;
const DIALOGUE_BOX_HEIGHT: f32 = 200.0;
const DIALOGUE_BOX_PADDING: f32 = 20.0;
const DIALOGUE_BOX_ALPHA: f32 = 0.85;

/// 文本配置
const SPEAKER_FONT_SIZE: f32 = 28.0;
const CONTENT_FONT_SIZE: f32 = 24.0;
const CHAPTER_FONT_SIZE: f32 = 48.0;

/// 颜色配置
const DIALOGUE_BOX_COLOR: Color = Color::new(0.1, 0.1, 0.15, DIALOGUE_BOX_ALPHA);
const SPEAKER_NAME_COLOR: Color = Color::new(0.95, 0.85, 0.6, 1.0); // 金黄色
const CONTENT_COLOR: Color = WHITE;

/// 辅助函数：为颜色应用 alpha 系数
#[inline]
fn color_with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, color.a * alpha)
}

/// 文本渲染器
#[derive(Debug)]
pub struct TextRenderer {
    /// 自定义字体（用于中文）
    font: Option<Font>,
    /// 是否已初始化
    initialized: bool,
    /// 是否使用自定义字体
    use_custom_font: bool,
}

impl TextRenderer {
    /// 创建新的文本渲染器
    pub fn new() -> Self {
        Self {
            font: None,
            initialized: false,
            use_custom_font: false,
        }
    }

    /// 加载字体
    pub async fn load_font(&mut self, path: &str) -> Result<(), String> {
        // 使用 macroquad 的异步加载方法
        match load_ttf_font(path).await {
            Ok(font) => {
                self.font = Some(font);
                self.initialized = true;
                self.use_custom_font = true;
                println!("✅ 成功加载字体: {}", path);
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ 加载字体失败: {} - {}", path, e);
                self.initialized = true;
                self.use_custom_font = false;
                Err(format!("加载字体失败: {}", e))
            }
        }
    }

    /// 检查是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// 检查是否使用自定义字体
    pub fn has_custom_font(&self) -> bool {
        self.use_custom_font && self.font.is_some()
    }

    /// 绘制 UI 文本（公开方法，用于调试信息、提示等）
    pub fn draw_ui_text(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        self.draw_text(text, x, y, font_size, color);
    }

    /// 渲染对话框
    pub fn render_dialogue_box(&self, speaker: Option<&str>, content: &str, visible_chars: usize) {
        self.render_dialogue_box_with_alpha(speaker, content, visible_chars, 1.0);
    }

    /// 渲染对话框（带全局透明度）
    ///
    /// 用于 changeScene 后 UI 淡入效果。
    pub fn render_dialogue_box_with_alpha(
        &self,
        speaker: Option<&str>,
        content: &str,
        visible_chars: usize,
        alpha: f32,
    ) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // 计算对话框位置和大小
        let box_x = DIALOGUE_BOX_MARGIN;
        let box_y = screen_h - DIALOGUE_BOX_HEIGHT - DIALOGUE_BOX_MARGIN;
        let box_w = screen_w - DIALOGUE_BOX_MARGIN * 2.0;
        let box_h = DIALOGUE_BOX_HEIGHT;

        // 边框颜色常量
        const BORDER_COLOR: Color = Color::new(0.5, 0.5, 0.6, 0.8);

        // 绘制对话框背景和边框
        draw_rectangle(
            box_x,
            box_y,
            box_w,
            box_h,
            color_with_alpha(DIALOGUE_BOX_COLOR, alpha),
        );
        draw_rectangle_lines(
            box_x,
            box_y,
            box_w,
            box_h,
            2.0,
            color_with_alpha(BORDER_COLOR, alpha),
        );

        // 绘制说话者名称
        let mut text_y = box_y + DIALOGUE_BOX_PADDING;
        if let Some(name) = speaker {
            self.draw_text(
                name,
                box_x + DIALOGUE_BOX_PADDING,
                text_y + SPEAKER_FONT_SIZE,
                SPEAKER_FONT_SIZE,
                color_with_alpha(SPEAKER_NAME_COLOR, alpha),
            );
            text_y += SPEAKER_FONT_SIZE + 10.0;
        }

        // 绘制对话内容（支持打字机效果）
        let visible_content: String = content.chars().take(visible_chars).collect();
        let content_x = box_x + DIALOGUE_BOX_PADDING;
        let content_y = text_y + CONTENT_FONT_SIZE + 5.0;
        let max_width = box_w - DIALOGUE_BOX_PADDING * 2.0;

        self.draw_text_wrapped(
            &visible_content,
            content_x,
            content_y,
            CONTENT_FONT_SIZE,
            color_with_alpha(CONTENT_COLOR, alpha),
            max_width,
        );

        // 绘制继续提示（如果文本已完全显示且 alpha > 0.5）
        if visible_chars >= content.chars().count() && alpha > 0.5 {
            self.draw_continue_indicator(box_x + box_w - 40.0, box_y + box_h - 30.0, alpha);
        }
    }

    /// 渲染章节标题（左上角显示，不遮挡内容）
    pub fn render_chapter_title(&self, title: &str, alpha: f32) {
        // 左上角位置
        let margin = 30.0;
        let x = margin;
        let y = margin + CHAPTER_FONT_SIZE;

        // 测量文本尺寸
        let text_size = self.measure_text(title, CHAPTER_FONT_SIZE);

        // 绘制半透明背景
        let bg_padding_x = 20.0;
        let bg_padding_y = 10.0;
        draw_rectangle(
            x - bg_padding_x,
            y - CHAPTER_FONT_SIZE - bg_padding_y,
            text_size.width + bg_padding_x * 2.0,
            CHAPTER_FONT_SIZE + bg_padding_y * 2.0,
            Color::new(0.0, 0.0, 0.0, 0.6 * alpha),
        );

        // 绘制装饰线（左侧竖线）
        draw_rectangle(
            x - bg_padding_x,
            y - CHAPTER_FONT_SIZE - bg_padding_y,
            4.0,
            CHAPTER_FONT_SIZE + bg_padding_y * 2.0,
            Color::new(0.95, 0.85, 0.6, alpha), // 金色
        );

        // 绘制标题文本
        let color = Color::new(1.0, 1.0, 1.0, alpha);
        self.draw_text(title, x, y, CHAPTER_FONT_SIZE, color);
    }

    /// 计算选择框的矩形区域
    ///
    /// 返回每个选项的 (x, y, width, height) 元组数组
    pub fn get_choice_rects(&self, choice_count: usize) -> Vec<(f32, f32, f32, f32)> {
        let screen_w = screen_width();
        let screen_h = screen_height();

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

    /// 渲染选择界面
    ///
    /// # 参数
    /// - `choices`: 选项列表
    /// - `selected_index`: 当前选中的索引
    /// - `hovered_index`: 鼠标悬停的索引（可选）
    pub fn render_choices(
        &self,
        choices: &[super::render_state::ChoiceItem],
        selected_index: usize,
        hovered_index: Option<usize>,
    ) {
        self.render_choices_with_alpha(choices, selected_index, hovered_index, 1.0);
    }

    /// 渲染选择界面（带全局透明度）
    ///
    /// 用于 changeScene 后 UI 淡入效果。
    pub fn render_choices_with_alpha(
        &self,
        choices: &[super::render_state::ChoiceItem],
        selected_index: usize,
        hovered_index: Option<usize>,
        alpha: f32,
    ) {
        let screen_w = screen_width();
        let screen_h = screen_height();

        // 选项布局常量
        const CHOICE_HEIGHT: f32 = 50.0;
        const CHOICE_SPACING: f32 = 10.0;

        let total_height = choices.len() as f32 * (CHOICE_HEIGHT + CHOICE_SPACING) - CHOICE_SPACING;
        let start_y = (screen_h - total_height) / 2.0;

        // 选项颜色定义
        const BG_SELECTED: Color = Color::new(0.3, 0.4, 0.6, 0.9);
        const BG_HOVERED: Color = Color::new(0.25, 0.35, 0.5, 0.85);
        const BG_NORMAL: Color = Color::new(0.2, 0.2, 0.3, 0.8);
        const BORDER_SELECTED: Color = Color::new(0.6, 0.7, 0.9, 1.0);
        const BORDER_HOVERED: Color = Color::new(0.5, 0.6, 0.8, 0.9);
        const BORDER_NORMAL: Color = Color::new(0.4, 0.4, 0.5, 0.8);
        const TEXT_ACTIVE: Color = WHITE;
        const TEXT_NORMAL: Color = Color::new(0.8, 0.8, 0.8, 1.0);

        // 绘制半透明背景
        draw_rectangle(
            0.0,
            start_y - 30.0,
            screen_w,
            total_height + 60.0,
            Color::new(0.0, 0.0, 0.0, 0.7 * alpha),
        );

        let box_w = screen_w * 0.6;
        let box_x = (screen_w - box_w) / 2.0;

        for (i, choice) in choices.iter().enumerate() {
            let y = start_y + i as f32 * (CHOICE_HEIGHT + CHOICE_SPACING);
            let is_selected = i == selected_index;
            let is_hovered = hovered_index == Some(i);

            // 选择合适的颜色
            let (bg_color, border_color, text_color) = if is_selected {
                (BG_SELECTED, BORDER_SELECTED, TEXT_ACTIVE)
            } else if is_hovered {
                (BG_HOVERED, BORDER_HOVERED, TEXT_ACTIVE)
            } else {
                (BG_NORMAL, BORDER_NORMAL, TEXT_NORMAL)
            };

            // 绘制选项背景和边框
            draw_rectangle(
                box_x,
                y,
                box_w,
                CHOICE_HEIGHT,
                color_with_alpha(bg_color, alpha),
            );
            draw_rectangle_lines(
                box_x,
                y,
                box_w,
                CHOICE_HEIGHT,
                if is_selected || is_hovered { 3.0 } else { 2.0 },
                color_with_alpha(border_color, alpha),
            );

            // 绘制选项文本
            let text_size = self.measure_text(&choice.text, CONTENT_FONT_SIZE);
            let text_x = box_x + (box_w - text_size.width) / 2.0;
            let text_y = y + (CHOICE_HEIGHT + CONTENT_FONT_SIZE) / 2.0 - 5.0;
            self.draw_text(
                &choice.text,
                text_x,
                text_y,
                CONTENT_FONT_SIZE,
                color_with_alpha(text_color, alpha),
            );

            // 绘制选中指示器
            if is_selected {
                let indicator_x = box_x - 30.0;
                let indicator_y = y + CHOICE_HEIGHT / 2.0;
                draw_triangle(
                    vec2(indicator_x, indicator_y - 10.0),
                    vec2(indicator_x, indicator_y + 10.0),
                    vec2(indicator_x + 15.0, indicator_y),
                    color_with_alpha(SPEAKER_NAME_COLOR, alpha),
                );
            }
        }
    }

    /// 绘制文本（使用自定义字体或默认字体）
    fn draw_text(&self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        if self.use_custom_font && self.font.is_some() {
            // 使用自定义字体
            let params = TextParams {
                font: self.font.as_ref(),
                font_size: font_size as u16,
                font_scale: 1.0,
                font_scale_aspect: 1.0,
                color,
                ..Default::default()
            };
            draw_text_ex(text, x, y, params);
        } else {
            // 使用默认字体（仅支持 ASCII）
            draw_text(text, x, y, font_size, color);
        }
    }

    /// 绘制自动换行文本
    fn draw_text_wrapped(
        &self,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        color: Color,
        max_width: f32,
    ) {
        let line_height = font_size * 1.4;
        let mut current_y = y;
        let mut current_line = String::new();
        let mut current_width = 0.0;

        for ch in text.chars() {
            // 处理换行符
            if ch == '\n' {
                self.draw_text(&current_line, x, current_y, font_size, color);
                current_y += line_height;
                current_line.clear();
                current_width = 0.0;
                continue;
            }

            // 计算字符宽度
            let char_str = ch.to_string();
            let char_width = self.measure_text(&char_str, font_size).width;

            // 检查是否需要换行
            if current_width + char_width > max_width && !current_line.is_empty() {
                self.draw_text(&current_line, x, current_y, font_size, color);
                current_y += line_height;
                current_line.clear();
                current_width = 0.0;
            }

            current_line.push(ch);
            current_width += char_width;
        }

        // 绘制最后一行
        if !current_line.is_empty() {
            self.draw_text(&current_line, x, current_y, font_size, color);
        }
    }

    /// 测量文本尺寸
    fn measure_text(&self, text: &str, font_size: f32) -> TextDimensions {
        if self.use_custom_font && self.font.is_some() {
            measure_text(text, self.font.as_ref(), font_size as u16, 1.0)
        } else {
            // 默认字体的简单估算
            let width = text.len() as f32 * font_size * 0.5;
            TextDimensions {
                width,
                height: font_size,
                offset_y: font_size * 0.8,
            }
        }
    }

    /// 绘制继续提示（闪烁的三角形）
    fn draw_continue_indicator(&self, x: f32, y: f32, alpha: f32) {
        // 使用时间产生闪烁效果
        let blink_alpha = ((get_time() * 3.0).sin() * 0.5 + 0.5) as f32;
        let color = Color::new(1.0, 1.0, 1.0, blink_alpha * alpha);

        draw_triangle(
            vec2(x, y),
            vec2(x + 15.0, y + 10.0),
            vec2(x, y + 20.0),
            color,
        );
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}
