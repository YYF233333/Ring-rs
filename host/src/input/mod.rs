//! 输入处理：`InputManager` 编排 `state::InputState`（设备状态/防抖/长按）与 [`ChoiceNavigator`]（选项导航），产出 `RuntimeInput`。

mod choice_navigator;
mod state;

pub mod recording;

pub use choice_navigator::ChoiceNavigator;

use vn_runtime::input::RuntimeInput;
use vn_runtime::state::WaitingReason;
use winit::event::{ElementState, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

use self::recording::{InputEvent, RecordingBuffer};
use self::state::InputState;

/// 输入管理器
///
/// 消费 winit `WindowEvent` 维护按键/鼠标状态，
/// 每帧通过 `update()` 将其转换为 `RuntimeInput`。
#[derive(Debug)]
pub struct InputManager {
    pub(crate) state: InputState,
    /// 选择分支导航子状态（`selected_index` / `hovered_index` 在此字段上）
    pub choice: ChoiceNavigator,
    pub(crate) pending_input: Option<RuntimeInput>,
    recording_buffer: Option<RecordingBuffer>,
}

#[allow(clippy::new_without_default)]
impl InputManager {
    /// 创建新的输入管理器
    pub fn new() -> Self {
        Self {
            state: InputState::new(),
            choice: ChoiceNavigator::new(),
            pending_input: None,
            recording_buffer: None,
        }
    }

    /// 启用后台录制缓冲区
    pub fn enable_recording(&mut self, size_mb: u32) {
        if size_mb > 0 {
            self.recording_buffer = Some(RecordingBuffer::new(size_mb));
        }
    }

    /// 消费 winit WindowEvent 更新内部按键/鼠标状态，同时写入录制缓冲区
    pub fn process_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::MouseWheel { delta, .. } = event {
            self.state.accumulate_wheel(delta);
            return;
        }

        if let Some(input_event) =
            recording::convert_window_event(event, self.state.mouse_position())
        {
            if let Some(ref mut buffer) = self.recording_buffer {
                buffer.push(self.state.elapsed_ms, input_event.clone());
            }
            self.state.apply_input_event(&input_event);
            return;
        }

        if let WindowEvent::KeyboardInput { event: key_ev, .. } = event
            && let PhysicalKey::Code(key) = key_ev.physical_key
            && key_ev.state == ElementState::Pressed
            && key_ev.repeat
        {
            self.state.note_key_pressed_repeat(key);
        }
    }

    /// 处理语义 InputEvent（录制/回放共用入口）
    pub fn process_input_event(&mut self, event: &InputEvent) {
        self.state.apply_input_event(event);
    }

    /// 注入回放事件（headless 用）
    pub fn inject_replay_events(&mut self, events: &[InputEvent]) {
        for event in events {
            self.process_input_event(event);
        }
    }

    /// 返回录制缓冲区快照（导出用）
    pub fn recording_snapshot(&self) -> Option<&std::collections::VecDeque<(u64, InputEvent)>> {
        self.recording_buffer.as_ref().map(|b| b.snapshot())
    }

    /// 帧开始时调用：推进内部时钟（不清除 per-frame 状态）
    pub fn begin_frame(&mut self, dt: f32) {
        self.state.begin_frame(dt);
    }

    /// 帧结束时调用：清除 per-frame 状态（just_pressed 等）
    ///
    /// 必须在游戏逻辑消费输入之后、下一帧事件到来之前调用。
    pub fn end_frame(&mut self) {
        self.state.end_frame();
    }

    /// 获取当前鼠标位置
    pub fn mouse_position(&self) -> (f32, f32) {
        self.state.mouse_position()
    }

    /// 本帧指针位移增量（像素），在 `end_frame` 时清零
    pub fn mouse_delta(&self) -> (f32, f32) {
        self.state.mouse_delta
    }

    /// 本帧滚轮累积增量，在 `end_frame` 时清零
    pub fn scroll_delta(&self) -> (f32, f32) {
        self.state.scroll_delta()
    }

    /// 获取当前鼠标是否按下
    pub fn is_mouse_pressed(&self) -> bool {
        self.state.mouse_pressed
    }

    /// 获取当前鼠标是否刚按下（本帧）
    pub fn is_mouse_just_pressed(&self) -> bool {
        self.state.is_mouse_just_pressed()
    }

    /// 抑制本帧鼠标点击（当 egui 交互元素处于指针下方时调用）
    pub fn suppress_mouse_click(&mut self) {
        self.state.suppress_mouse_click();
    }

    /// 重置选择状态
    pub fn reset_choice(&mut self, choice_count: usize) {
        self.choice.reset(choice_count);
    }

    /// 设置选择框矩形区域（每帧更新）
    pub fn set_choice_rects(&mut self, rects: Vec<(f32, f32, f32, f32)>) {
        self.choice.set_choice_rects(rects);
    }

    /// 根据当前的 `WaitingReason` 将输入状态转换为 RuntimeInput
    pub fn update(&mut self, waiting: &WaitingReason, dt: f32) -> Option<RuntimeInput> {
        if let Some(input) = self.pending_input.take() {
            return Some(input);
        }

        match waiting {
            WaitingReason::None => {
                self.state.reset_hold_timers();
                None
            }
            WaitingReason::WaitForClick => self.state.handle_click_input(dt),
            WaitingReason::WaitForChoice { choice_count } => {
                if self.choice.choice_count != *choice_count {
                    self.reset_choice(*choice_count);
                }
                self.state.reset_hold_timers();
                self.choice.handle_choice_input(&mut self.state)
            }
            WaitingReason::WaitForTime(_) => {
                self.state.reset_hold_timers();
                self.state.handle_time_wait_input()
            }
            WaitingReason::WaitForSignal(_) => {
                self.state.reset_hold_timers();
                None
            }
            WaitingReason::WaitForUIResult { .. } => {
                self.state.reset_hold_timers();
                None
            }
        }
    }

    /// 检查是否刚刚发生点击（不消耗输入），用于 UI 反馈
    pub fn is_clicking(&self) -> bool {
        self.state.is_mouse_just_pressed()
            || self.state.is_key_just_pressed(KeyCode::Space)
            || self.state.is_key_just_pressed(KeyCode::Enter)
    }

    /// 设置待处理的输入（用于外部系统注入，如信号）
    pub fn inject_input(&mut self, input: RuntimeInput) {
        self.pending_input = Some(input);
    }

    /// 获取当前选中的索引
    pub fn get_selected_index(&self) -> usize {
        self.choice.selected_index
    }

    /// 检查指定按键是否在本帧刚被按下
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.state.is_key_just_pressed(key)
    }

    /// 检查指定按键是否正在被按住
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.state.is_key_down(key)
    }

    /// 根据鼠标位置更新选中的选项索引，返回是否有选项被悬停
    pub fn handle_choice_hover(&mut self, choice_rects: &[(f32, f32, f32, f32)]) -> bool {
        self.choice
            .handle_choice_hover(self.state.mouse_position(), choice_rects)
    }
}

#[cfg(test)]
mod tests;
