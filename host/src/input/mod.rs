//! # Input 模块
//!
//! 输入处理系统，负责采集用户输入并转换为 RuntimeInput。
//!
//! ## 设计说明
//!
//! - `InputManager` 消费 winit 的 `WindowEvent` 维护内部键盘/鼠标状态
//! - 每帧调用 `begin_frame(dt)` 清理 per-frame 状态并推进内部时钟
//! - `update()` 根据当前 `WaitingReason` 将输入状态转换为 `RuntimeInput`
//! - 实现输入防抖和长按快进
//! - 支持选择分支的键盘导航和鼠标交互

use std::collections::HashSet;

use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

/// 输入防抖配置
const CLICK_DEBOUNCE_SECONDS: f64 = 0.15;

/// 长按快进 — 首次按下后等待时间，之后开始快进
const HOLD_INITIAL_DELAY: f64 = 0.3;
/// 长按快进重复间隔（秒），越小越快
const HOLD_REPEAT_INTERVAL: f64 = 0.05;

/// 输入管理器
///
/// 消费 winit `WindowEvent` 维护按键/鼠标状态，
/// 每帧通过 `update()` 将其转换为 `RuntimeInput`。
#[derive(Debug)]
pub struct InputManager {
    // ── per-frame state（begin_frame 清除）───────────────────────────
    just_pressed_keys: HashSet<KeyCode>,
    mouse_just_pressed: bool,

    // ── persistent state ─────────────────────────────────────────────
    pressed_keys: HashSet<KeyCode>,
    mouse_pressed: bool,
    mouse_position: (f32, f32),

    // ── 内部时钟 ────────────────────────────────────────────────────
    current_time: f64,

    // ── 游戏逻辑状态 ────────────────────────────────────────────────
    last_click_time: f64,
    /// 当前选择索引（用于选择分支）
    pub selected_index: usize,
    /// 鼠标悬停索引（用于选择分支）
    pub hovered_index: Option<usize>,
    choice_count: usize,
    pending_input: Option<RuntimeInput>,
    choice_rects: Vec<(f32, f32, f32, f32)>,
    hold_timer: f64,
    last_hold_trigger_time: f64,
}

impl InputManager {
    /// 创建新的输入管理器
    pub fn new() -> Self {
        Self {
            just_pressed_keys: HashSet::new(),
            mouse_just_pressed: false,
            pressed_keys: HashSet::new(),
            mouse_pressed: false,
            mouse_position: (0.0, 0.0),
            current_time: 0.0,
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

    // ── 事件接口 ─────────────────────────────────────────────────────

    /// 消费 winit WindowEvent 更新内部按键/鼠标状态
    pub fn process_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    match event.state {
                        ElementState::Pressed if !event.repeat => {
                            self.just_pressed_keys.insert(key);
                            self.pressed_keys.insert(key);
                        }
                        ElementState::Pressed => {
                            self.pressed_keys.insert(key);
                        }
                        ElementState::Released => {
                            self.pressed_keys.remove(&key);
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => {
                    self.mouse_just_pressed = true;
                    self.mouse_pressed = true;
                }
                ElementState::Released => {
                    self.mouse_pressed = false;
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = (position.x as f32, position.y as f32);
            }
            _ => {}
        }
    }

    /// 帧开始时调用：推进内部时钟（不清除 per-frame 状态）
    pub fn begin_frame(&mut self, dt: f32) {
        self.current_time += dt as f64;
    }

    /// 帧结束时调用：清除 per-frame 状态（just_pressed 等）
    ///
    /// 必须在游戏逻辑消费输入之后、下一帧事件到来之前调用。
    pub fn end_frame(&mut self) {
        self.just_pressed_keys.clear();
        self.mouse_just_pressed = false;
    }

    /// 获取当前鼠标位置
    pub fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    /// 获取当前鼠标是否按下
    pub fn is_mouse_pressed(&self) -> bool {
        self.mouse_pressed
    }

    /// 获取当前鼠标是否刚按下（本帧）
    pub fn is_mouse_just_pressed(&self) -> bool {
        self.mouse_just_pressed
    }

    /// 抑制本帧鼠标点击（当 egui 交互元素处于指针下方时调用）
    pub fn suppress_mouse_click(&mut self) {
        self.mouse_just_pressed = false;
    }

    // ── 选择分支 API ─────────────────────────────────────────────────

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

    // ── 主更新 ───────────────────────────────────────────────────────

    /// 根据当前的 `WaitingReason` 将输入状态转换为 RuntimeInput
    pub fn update(&mut self, waiting: &WaitingReason, dt: f32) -> Option<RuntimeInput> {
        if let Some(input) = self.pending_input.take() {
            return Some(input);
        }

        match waiting {
            WaitingReason::None => {
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                None
            }
            WaitingReason::WaitForClick => self.handle_click_input(dt),
            WaitingReason::WaitForChoice { choice_count } => {
                if self.choice_count != *choice_count {
                    self.reset_choice(*choice_count);
                }
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                self.handle_choice_input()
            }
            WaitingReason::WaitForTime(_) => {
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                self.handle_time_wait_input()
            }
            WaitingReason::WaitForSignal(_) => {
                self.hold_timer = 0.0;
                self.last_hold_trigger_time = 0.0;
                None
            }
        }
    }

    /// 检查是否刚刚发生点击（不消耗输入），用于 UI 反馈
    pub fn is_clicking(&self) -> bool {
        self.mouse_just_pressed
            || self.is_key_just_pressed(KeyCode::Space)
            || self.is_key_just_pressed(KeyCode::Enter)
    }

    /// 设置待处理的输入（用于外部系统注入，如信号）
    pub fn inject_input(&mut self, input: RuntimeInput) {
        self.pending_input = Some(input);
    }

    /// 获取当前选中的索引
    pub fn get_selected_index(&self) -> usize {
        self.selected_index
    }

    // ── 按键查询 ─────────────────────────────────────────────────────

    /// 检查指定按键是否在本帧刚被按下
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    /// 检查指定按键是否正在被按住
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    fn handle_click_input(&mut self, dt: f32) -> Option<RuntimeInput> {
        let current_time = self.current_time;
        let dt_f64 = dt as f64;

        let just_pressed =
            self.is_key_just_pressed(KeyCode::Space) || self.is_key_just_pressed(KeyCode::Enter);
        let mouse_clicked = self.mouse_just_pressed;
        let is_holding = self.is_key_down(KeyCode::Space) || self.is_key_down(KeyCode::Enter);

        if (just_pressed || mouse_clicked)
            && current_time - self.last_click_time >= CLICK_DEBOUNCE_SECONDS
        {
            self.last_click_time = current_time;
            self.hold_timer = 0.0;
            self.last_hold_trigger_time = 0.0;
            return Some(RuntimeInput::Click);
        }

        if is_holding {
            self.hold_timer += dt_f64;
            if self.hold_timer >= HOLD_INITIAL_DELAY
                && current_time - self.last_hold_trigger_time >= HOLD_REPEAT_INTERVAL
            {
                self.last_hold_trigger_time = current_time;
                self.last_click_time = current_time;
                return Some(RuntimeInput::Click);
            }
        } else {
            self.hold_timer = 0.0;
            self.last_hold_trigger_time = 0.0;
        }

        None
    }

    fn handle_time_wait_input(&mut self) -> Option<RuntimeInput> {
        let current_time = self.current_time;
        let clicked = self.mouse_just_pressed
            || self.is_key_just_pressed(KeyCode::Space)
            || self.is_key_just_pressed(KeyCode::Enter);

        if clicked && current_time - self.last_click_time >= CLICK_DEBOUNCE_SECONDS {
            self.last_click_time = current_time;
            return Some(RuntimeInput::Click);
        }

        None
    }

    fn handle_choice_input(&mut self) -> Option<RuntimeInput> {
        if self.choice_count == 0 {
            return None;
        }

        self.update_hover_state();

        // 键盘导航
        if self.is_key_just_pressed(KeyCode::ArrowUp) || self.is_key_just_pressed(KeyCode::KeyW) {
            self.selected_index = self.selected_index.saturating_sub(1);
            self.hovered_index = None;
        }
        if self.is_key_just_pressed(KeyCode::ArrowDown) || self.is_key_just_pressed(KeyCode::KeyS) {
            self.selected_index = (self.selected_index + 1).min(self.choice_count - 1);
            self.hovered_index = None;
        }

        // 键盘确认
        if self.is_key_just_pressed(KeyCode::Enter) || self.is_key_just_pressed(KeyCode::Space) {
            let current_time = self.current_time;
            if current_time - self.last_click_time >= CLICK_DEBOUNCE_SECONDS {
                self.last_click_time = current_time;
                return Some(RuntimeInput::ChoiceSelected {
                    index: self.selected_index,
                });
            }
        }

        // 鼠标点击选择
        if self.mouse_just_pressed
            && let Some(hover_idx) = self.hovered_index
        {
            let current_time = self.current_time;
            if current_time - self.last_click_time >= CLICK_DEBOUNCE_SECONDS {
                self.last_click_time = current_time;
                self.selected_index = hover_idx;
                return Some(RuntimeInput::ChoiceSelected { index: hover_idx });
            }
        }

        None
    }

    fn update_hover_state(&mut self) {
        let (mouse_x, mouse_y) = self.mouse_position;
        self.hovered_index = None;

        for (i, &(x, y, w, h)) in self.choice_rects.iter().enumerate() {
            if mouse_x >= x && mouse_x <= x + w && mouse_y >= y && mouse_y <= y + h {
                self.hovered_index = Some(i);
                break;
            }
        }
    }

    /// 根据鼠标位置更新选中的选项索引，返回是否有选项被悬停
    pub fn handle_choice_hover(&mut self, choice_rects: &[(f32, f32, f32, f32)]) -> bool {
        let (mouse_x, mouse_y) = self.mouse_position;

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
        assert!(manager.pending_input.is_some());

        let result = manager.update(&WaitingReason::WaitForClick, 0.016);
        assert_eq!(result, Some(RuntimeInput::Click));
        assert!(manager.pending_input.is_none());
    }
}
