//! # Input 模块
//!
//! 输入处理系统，负责采集用户输入并转换为 RuntimeInput。
//!
//! ## 设计说明
//!
//! - `InputManager` 采集 macroquad 的鼠标和键盘事件
//! - 根据当前 `WaitingReason` 决定如何处理输入
//! - 实现输入防抖，避免重复触发
//! - 支持选择分支的鼠标交互

use macroquad::prelude::*;
use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;

/// 输入防抖配置
const CLICK_DEBOUNCE_SECONDS: f64 = 0.15;

/// 长按快进配置
/// 第一次按下后等待的时间，之后才开始快进
const HOLD_INITIAL_DELAY: f64 = 0.3;
/// 长按快进时的重复间隔（秒），越小越快
const HOLD_REPEAT_INTERVAL: f64 = 0.05;

/// 输入管理器
///
/// 负责采集用户输入并转换为 RuntimeInput。
#[derive(Debug)]
pub struct InputManager {
    /// 上次点击时间（用于防抖）
    last_click_time: f64,
    /// 当前选择索引（用于选择分支）
    pub selected_index: usize,
    /// 鼠标悬停索引（用于选择分支）
    pub hovered_index: Option<usize>,
    /// 选项数量（用于边界检查）
    choice_count: usize,
    /// 是否有待处理的输入
    pending_input: Option<RuntimeInput>,
    /// 选择框矩形区域缓存
    choice_rects: Vec<(f32, f32, f32, f32)>,
    /// 长按计时器（用于快进）
    hold_timer: f64,
    /// 上次快进触发时间
    last_hold_trigger_time: f64,
}

impl InputManager {
    /// 创建新的输入管理器
    pub fn new() -> Self {
        Self {
            last_click_time: 0.0,
            selected_index: 0,
            hovered_index: None,
            choice_count: 0,
            pending_input: None,
            choice_rects: Vec::new(),
            hold_timer: 0.0,
            last_hold_trigger_time: 0.0,
        }
    }

    /// 重置选择状态
    pub fn reset_choice(&mut self, choice_count: usize) {
        self.selected_index = 0;
        self.hovered_index = None;
        self.choice_count = choice_count;
        self.choice_rects.clear();
    }

    /// 设置选择框矩形区域（每帧更新）
    pub fn set_choice_rects(&mut self, rects: Vec<(f32, f32, f32, f32)>) {
        self.choice_rects = rects;
    }

    /// 更新输入状态
    ///
    /// 根据当前的 `WaitingReason` 采集相应的输入。
    /// 返回可能产生的 `RuntimeInput`。
    ///
    /// # 参数
    /// - `waiting`: 当前的等待状态
    /// - `dt`: 帧时间（秒），用于长按快进计时
    pub fn update(&mut self, waiting: &WaitingReason, dt: f32) -> Option<RuntimeInput> {
        // 如果有待处理的输入，先返回它
        if let Some(input) = self.pending_input.take() {
            return Some(input);
        }

        match waiting {
            WaitingReason::None => {
                // 不等待时，不处理输入，重置长按计时器
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                None
            }
            WaitingReason::WaitForClick => self.handle_click_input(dt),
            WaitingReason::WaitForChoice { choice_count } => {
                // 如果选项数量变化，重置选择
                if self.choice_count != *choice_count {
                    self.reset_choice(*choice_count);
                }
                // 选择分支时不支持长按快进，重置计时器
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                self.handle_choice_input()
            }
            WaitingReason::WaitForTime(_) => {
                // 时间等待由 Host 处理，不需要用户输入
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                None
            }
            WaitingReason::WaitForSignal(_) => {
                // 信号等待由外部系统触发，暂不处理
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                None
            }
        }
    }

    /// 处理点击输入（支持长按快进）
    fn handle_click_input(&mut self, dt: f32) -> Option<RuntimeInput> {
        let current_time = get_time();
        let dt_f64 = dt as f64;

        // 检查是否刚刚按下（单次点击检测，优先处理）
        let just_pressed = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter);
        // 检查鼠标点击（不支持长按，只支持单次点击）
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
        // 检查是否按下空格或回车（长按检测）
        let is_holding = is_key_down(KeyCode::Space) || is_key_down(KeyCode::Enter);

        // 优先处理单次按下或鼠标点击（立即响应）
        if just_pressed || mouse_clicked {
            // 检查防抖
            if current_time - self.last_click_time >= CLICK_DEBOUNCE_SECONDS {
                self.last_click_time = current_time;
                self.hold_timer = 0.0; // 重置长按计时器
                self.last_hold_trigger_time = 0.0;
                return Some(RuntimeInput::Click);
            }
        }

        // 处理长按快进
        if is_holding {
            // 长按状态：更新计时器
            self.hold_timer += dt_f64;

            // 检查是否超过初始延迟
            if self.hold_timer >= HOLD_INITIAL_DELAY {
                // 开始快进：以固定频率触发输入
                if current_time - self.last_hold_trigger_time >= HOLD_REPEAT_INTERVAL {
                    self.last_hold_trigger_time = current_time;
                    self.last_click_time = current_time;
                    return Some(RuntimeInput::Click);
                }
            }
        } else {
            // 没有按下：重置长按计时器
            self.hold_timer = 0.0;
            self.last_hold_trigger_time = 0.0;
        }

        None
    }

    /// 处理选择输入
    fn handle_choice_input(&mut self) -> Option<RuntimeInput> {
        if self.choice_count == 0 {
            return None;
        }

        // 更新鼠标悬停状态
        self.update_hover_state();

        // 键盘导航
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.hovered_index = None; // 键盘操作时清除悬停状态
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.selected_index = (self.selected_index + 1).min(self.choice_count - 1);
            self.hovered_index = None; // 键盘操作时清除悬停状态
        }

        // 键盘确认选择
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            let current_time = get_time();
            if current_time - self.last_click_time >= CLICK_DEBOUNCE_SECONDS {
                self.last_click_time = current_time;
                return Some(RuntimeInput::ChoiceSelected {
                    index: self.selected_index,
                });
            }
        }

        // 鼠标点击选择（点击悬停的选项）
        if is_mouse_button_pressed(MouseButton::Left)
            && let Some(hover_idx) = self.hovered_index
        {
            let current_time = get_time();
            if current_time - self.last_click_time >= CLICK_DEBOUNCE_SECONDS {
                self.last_click_time = current_time;
                self.selected_index = hover_idx;
                return Some(RuntimeInput::ChoiceSelected { index: hover_idx });
            }
        }

        None
    }

    /// 更新鼠标悬停状态
    fn update_hover_state(&mut self) {
        let (mouse_x, mouse_y) = mouse_position();
        self.hovered_index = None;

        for (i, &(x, y, w, h)) in self.choice_rects.iter().enumerate() {
            if mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h {
                self.hovered_index = Some(i);
                break;
            }
        }
    }

    /// 处理选择项的鼠标悬停（外部调用版本）
    ///
    /// 根据鼠标位置更新选中的选项索引。
    /// 返回是否有选项被悬停。
    pub fn handle_choice_hover(&mut self, choice_rects: &[(f32, f32, f32, f32)]) -> bool {
        let (mouse_x, mouse_y) = mouse_position();

        for (i, &(x, y, w, h)) in choice_rects.iter().enumerate() {
            if mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h {
                self.hovered_index = Some(i);
                return true;
            }
        }

        self.hovered_index = None;
        false
    }

    /// 设置待处理的输入
    ///
    /// 用于外部系统注入输入（如信号）。
    pub fn inject_input(&mut self, input: RuntimeInput) {
        self.pending_input = Some(input);
    }

    /// 获取当前选中的索引
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    /// 检查是否刚刚发生点击（不消耗输入）
    ///
    /// 用于 UI 反馈，不会触发 RuntimeInput。
    pub fn is_clicking(&self) -> bool {
        is_mouse_button_pressed(MouseButton::Left)
            || is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::Enter)
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_manager_creation() {
        let manager = InputManager::new();
        assert_eq!(manager.selected_index, 0);
        assert_eq!(manager.choice_count, 0);
        assert!(manager.pending_input.is_none());
    }

    #[test]
    fn test_reset_choice() {
        let mut manager = InputManager::new();
        manager.selected_index = 5;
        manager.reset_choice(3);
        assert_eq!(manager.selected_index, 0);
        assert_eq!(manager.choice_count, 3);
    }

    #[test]
    fn test_inject_input() {
        let mut manager = InputManager::new();
        manager.inject_input(RuntimeInput::Click);

        // 模拟 update，应该返回注入的输入
        // 注意：这个测试需要 macroquad 环境，在单元测试中可能无法完全运行
        assert!(manager.pending_input.is_some());
    }
}
