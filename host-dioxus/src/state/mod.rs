mod game_lifecycle;
mod harness;
mod interaction;
mod save_load;
mod session;
#[cfg(test)]
mod tests;
mod tick;
pub mod types;

pub use types::*;

// Re-export internal free functions so sub-modules using `use super::*` can call them.
pub(crate) use game_lifecycle::{
    host_history_from_runtime, load_call_stack_scripts, map_runtime_waiting,
};
pub use save_load::waiting_requires_snapshot_fallback;

use crate::command_executor::CommandExecutor;
use crate::render_state::{HostScreen, PlaybackMode, RenderState};

/// 应用状态内部结构（被 Mutex 保护）
pub struct AppStateInner {
    pub runtime: Option<vn_runtime::VNRuntime>,
    pub command_executor: CommandExecutor,
    pub render_state: RenderState,
    pub host_screen: HostScreen,
    pub waiting: WaitingFor,
    pub typewriter_timer: f32,
    /// 打字机基础速度（字符/秒）
    pub text_speed: f32,
    pub script_finished: bool,
    /// setup() 初始化的子系统集合
    pub services: Option<Services>,
    /// 对话历史（最新在前）
    pub history: Vec<HistoryEntry>,
    /// 用户设置
    pub user_settings: UserSettings,
    /// 持久化变量存储
    pub persistent_store: PersistentStore,
    /// 快照栈（Backspace 回退用）
    pub snapshot_stack: SnapshotStack,
    /// 播放模式
    pub playback_mode: PlaybackMode,
    /// Auto 模式计时器
    pub auto_timer: f32,
    /// 背景过渡内部计时器
    pub(self) bg_transition_elapsed: f32,
    /// 场景过渡内部计时器
    pub(self) scene_transition_elapsed: f32,
    /// 活跃的 shake 动画状态
    pub(self) active_shake: Option<ShakeAnimation>,
    /// 是否有活跃的场景效果（用于 signal 解析）
    pub(self) scene_effect_active: bool,
    /// 当前持有 session authority 的客户端。
    pub(self) client_owner: Option<SessionOwner>,
    /// client token 单调递增计数器。
    pub(self) next_client_id: u64,
    /// deterministic harness 的逻辑时间。
    pub(self) logical_time_ms: u64,
    /// 机读 trace 事件缓冲区。
    pub(self) trace_events: Vec<HarnessTraceEvent>,
    /// trace 事件序号。
    pub(self) trace_seq: u64,
}

impl Default for AppStateInner {
    fn default() -> Self {
        Self::new()
    }
}

impl AppStateInner {
    pub fn new() -> Self {
        Self {
            runtime: None,
            command_executor: CommandExecutor::new(),
            render_state: RenderState::new(),
            host_screen: HostScreen::Title,
            waiting: WaitingFor::Nothing,
            typewriter_timer: 0.0,
            text_speed: 30.0,
            script_finished: false,
            services: None,
            history: Vec::new(),
            user_settings: UserSettings::default(),
            persistent_store: PersistentStore::empty(),
            snapshot_stack: SnapshotStack::new(50),
            playback_mode: PlaybackMode::Normal,
            auto_timer: 0.0,
            bg_transition_elapsed: 0.0,
            scene_transition_elapsed: 0.0,
            active_shake: None,
            scene_effect_active: false,
            client_owner: None,
            next_client_id: 0,
            logical_time_ms: 0,
            trace_events: Vec::new(),
            trace_seq: 0,
        }
    }

    /// 获取已初始化的子系统引用。
    /// setup() 完成后此断言不会失败。
    pub fn services(&self) -> &Services {
        self.services
            .as_ref()
            .expect("invariant: services initialized in setup()")
    }

    /// 获取已初始化的子系统可变引用。
    pub fn services_mut(&mut self) -> &mut Services {
        self.services
            .as_mut()
            .expect("invariant: services initialized in setup()")
    }

    pub(self) fn project_render_state(&mut self) {
        self.render_state.playback_mode = self.playback_mode.clone();
        self.render_state.host_screen = self.host_screen.clone();
    }
}
