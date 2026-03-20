//! 选择分支的键盘导航与鼠标悬停/点击命中。

use vn_runtime::input::RuntimeInput;
use winit::keyboard::KeyCode;

use super::state::{CLICK_DEBOUNCE_SECONDS, InputState};

/// 选择分支的键盘/鼠标导航状态与矩形命中。
#[derive(Debug)]
pub struct ChoiceNavigator {
    /// 当前选择索引（用于选择分支）
    pub selected_index: usize,
    /// 鼠标悬停索引（用于选择分支）
    pub hovered_index: Option<usize>,
    pub(crate) choice_count: usize,
    pub(crate) choice_rects: Vec<(f32, f32, f32, f32)>,
}

impl ChoiceNavigator {
    pub(crate) fn new() -> Self {
        Self {
            selected_index: 0,
            hovered_index: None,
            choice_count: 0,
            choice_rects: Vec::new(),
        }
    }

    pub(crate) fn reset(&mut self, choice_count: usize) {
        self.selected_index = 0;
        self.hovered_index = None;
        self.choice_count = choice_count;
        self.choice_rects.clear();
    }

    /// 设置选择框矩形区域（每帧更新）
    pub fn set_choice_rects(&mut self, rects: Vec<(f32, f32, f32, f32)>) {
        self.choice_rects = rects;
    }

    pub(crate) fn update_hover_state(&mut self, mouse_position: (f32, f32)) {
        let (mouse_x, mouse_y) = mouse_position;
        self.hovered_index = None;

        for (i, &(x, y, w, h)) in self.choice_rects.iter().enumerate() {
            if mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h {
                self.hovered_index = Some(i);
                break;
            }
        }
    }

    pub(crate) fn handle_choice_input(&mut self, state: &mut InputState) -> Option<RuntimeInput> {
        if self.choice_count == 0 {
            return None;
        }

        self.update_hover_state(state.mouse_position());

        if state.is_key_just_pressed(KeyCode::ArrowUp) || state.is_key_just_pressed(KeyCode::KeyW) {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.hovered_index = None;
        }
        if state.is_key_just_pressed(KeyCode::ArrowDown) || state.is_key_just_pressed(KeyCode::KeyS)
        {
            self.selected_index = (self.selected_index + 1).min(self.choice_count - 1);
            self.hovered_index = None;
        }

        if state.is_key_just_pressed(KeyCode::Enter) || state.is_key_just_pressed(KeyCode::Space) {
            let current_time = state.current_time;
            if current_time - state.last_click_time >= CLICK_DEBOUNCE_SECONDS {
                state.last_click_time = current_time;
                return Some(RuntimeInput::ChoiceSelected {
                    index: self.selected_index,
                });
            }
        }

        if state.is_mouse_just_pressed()
            && let Some(hover_idx) = self.hovered_index
        {
            let current_time = state.current_time;
            if current_time - state.last_click_time >= CLICK_DEBOUNCE_SECONDS {
                state.last_click_time = current_time;
                self.selected_index = hover_idx;
                return Some(RuntimeInput::ChoiceSelected { index: hover_idx });
            }
        }

        None
    }

    /// 使用当前鼠标坐标与给定矩形列表更新 [`Self::hovered_index`]，返回是否命中某一选项。
    pub fn handle_choice_hover(
        &mut self,
        mouse_position: (f32, f32),
        choice_rects: &[(f32, f32, f32, f32)],
    ) -> bool {
        let (mouse_x, mouse_y) = mouse_position;

        for (i, &(x, y, w, h)) in choice_rects.iter().enumerate() {
            if mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h {
                self.hovered_index = Some(i);
                return true;
            }
        }

        self.hovered_index = None;
        false
    }
}
