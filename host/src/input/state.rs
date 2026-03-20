//! 原始按键/鼠标状态、帧生命周期与点击防抖/长按计时。

use std::collections::HashSet;

use vn_runtime::input::RuntimeInput;
use winit::event::MouseScrollDelta;
use winit::keyboard::KeyCode;

use super::recording::{InputEvent, MouseButtonName};

/// 输入防抖配置（秒）
pub(crate) const CLICK_DEBOUNCE_SECONDS: f64 = 0.15;

/// 长按快进 — 首次按下后等待时间，之后开始快进
pub(crate) const HOLD_INITIAL_DELAY: f64 = 0.3;
/// 长按快进重复间隔（秒），越小越快
pub(crate) const HOLD_REPEAT_INTERVAL: f64 = 0.05;

/// 键盘/鼠标的原始状态、内部时钟与点击相关计时器。
#[derive(Debug)]
pub(crate) struct InputState {
    // ── per-frame state（end_frame 清除）────────────────────────────
    pub(crate) just_pressed_keys: HashSet<KeyCode>,
    pub(crate) mouse_just_pressed: bool,
    pub(crate) mouse_delta: (f32, f32),
    pub(crate) scroll_delta: (f32, f32),

    // ── persistent state ────────────────────────────────────────────
    pub(crate) pressed_keys: HashSet<KeyCode>,
    pub(crate) mouse_pressed: bool,
    pub(crate) mouse_position: (f32, f32),

    // ── 内部时钟 ──────────────────────────────────────────────────
    pub(crate) current_time: f64,
    pub(crate) elapsed_ms: u64,

    // ── 点击防抖 / 长按 ───────────────────────────────────────────
    pub(crate) last_click_time: f64,
    pub(crate) hold_timer: f64,
    pub(crate) last_hold_trigger_time: f64,
}

impl InputState {
    pub(crate) fn new() -> Self {
        Self {
            just_pressed_keys: HashSet::new(),
            mouse_just_pressed: false,
            mouse_delta: (0.0, 0.0),
            scroll_delta: (0.0, 0.0),
            pressed_keys: HashSet::new(),
            mouse_pressed: false,
            mouse_position: (0.0, 0.0),
            current_time: 0.0,
            elapsed_ms: 0,
            last_click_time: 0.0,
            hold_timer: 0.0,
            last_hold_trigger_time: 0.0,
        }
    }

    /// 帧开始时调用：推进内部时钟（不清除 per-frame 状态）
    pub(crate) fn begin_frame(&mut self, dt: f32) {
        self.current_time += dt as f64;
        self.elapsed_ms += (dt * 1000.0) as u64;
    }

    /// 帧结束时调用：清除 per-frame 状态（just_pressed、鼠标点击边沿、增量等）
    pub(crate) fn end_frame(&mut self) {
        self.just_pressed_keys.clear();
        self.mouse_just_pressed = false;
        self.mouse_delta = (0.0, 0.0);
        self.scroll_delta = (0.0, 0.0);
    }

    pub(crate) fn mouse_position(&self) -> (f32, f32) {
        self.mouse_position
    }

    pub(crate) fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }

    pub(crate) fn is_mouse_just_pressed(&self) -> bool {
        self.mouse_just_pressed
    }

    pub(crate) fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    pub(crate) fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    pub(crate) fn suppress_mouse_click(&mut self) {
        self.mouse_just_pressed = false;
    }

    /// 处理 winit 键盘 repeat 等 `convert_window_event` 未覆盖的路径。
    pub(crate) fn note_key_pressed_repeat(&mut self, key: KeyCode) {
        self.pressed_keys.insert(key);
    }

    pub(crate) fn accumulate_wheel(&mut self, delta: &MouseScrollDelta) {
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (*x * 20.0, *y * 20.0),
            MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
        };
        self.scroll_delta.0 += dx;
        self.scroll_delta.1 += dy;
    }

    /// 语义 `InputEvent`（录制/回放与 `process_input_event` 共用）
    pub(crate) fn apply_input_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::KeyPress { key } => {
                if let Some(code) = key.to_key_code() {
                    self.just_pressed_keys.insert(code);
                    self.pressed_keys.insert(code);
                }
            }
            InputEvent::KeyRelease { key } => {
                if let Some(code) = key.to_key_code() {
                    self.pressed_keys.remove(&code);
                }
            }
            InputEvent::MousePress {
                button: MouseButtonName::Left,
                x,
                y,
            } => {
                let old = self.mouse_position;
                self.mouse_just_pressed = true;
                self.mouse_pressed = true;
                self.mouse_position = (*x, *y);
                self.mouse_delta = (self.mouse_position.0 - old.0, self.mouse_position.1 - old.1);
            }
            InputEvent::MouseRelease {
                button: MouseButtonName::Left,
                ..
            } => {
                self.mouse_pressed = false;
            }
            InputEvent::MouseMove { x, y } => {
                let old = self.mouse_position;
                self.mouse_position = (*x, *y);
                self.mouse_delta = (self.mouse_position.0 - old.0, self.mouse_position.1 - old.1);
            }
            InputEvent::MouseWheel { delta_x, delta_y } => {
                self.scroll_delta.0 += *delta_x;
                self.scroll_delta.1 += *delta_y;
            }
            _ => {}
        }
    }

    pub(crate) fn reset_hold_timers(&mut self) {
        self.hold_timer = 0.0;
        self.last_hold_trigger_time = 0.0;
    }

    pub(crate) fn handle_click_input(&mut self, dt: f32) -> Option<RuntimeInput> {
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
            self.reset_hold_timers();
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
            self.reset_hold_timers();
        }

        None
    }

    pub(crate) fn handle_time_wait_input(&mut self) -> Option<RuntimeInput> {
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
}
