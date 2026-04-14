mod game_lifecycle;
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

/// 动画/过渡计时器（仅在 tick 和 interaction 中使用）
#[derive(Default)]
pub struct AnimationTimers {
    /// 背景过渡内部计时器
    pub bg_transition_elapsed: f32,
    /// 场景过渡内部计时器
    pub scene_transition_elapsed: f32,
    /// 活跃的 shake 动画状态
    pub active_shake: Option<ShakeAnimation>,
    /// 是否有活跃的场景效果（用于 signal 解析）
    pub scene_effect_active: bool,
}

/// Debug session 管理（完全隔离在 session.rs 中）
#[derive(Default)]
pub struct SessionAuthority {
    /// 当前持有 session authority 的客户端。
    pub client_owner: Option<SessionOwner>,
    /// client token 单调递增计数器。
    pub next_client_id: u64,
}

/// 应用状态内部结构（被 Mutex 保护）
pub struct AppStateInner {
    // ── 核心游戏状态 ──
    pub runtime: Option<vn_runtime::VNRuntime>,
    pub command_executor: CommandExecutor,
    pub render_state: RenderState,
    pub host_screen: HostScreen,
    pub waiting: WaitingFor,
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

    // ── 播放控制 ──
    /// 播放模式
    pub playback_mode: PlaybackMode,
    /// Auto 模式计时器
    pub auto_timer: f32,
    pub typewriter_timer: f32,
    /// 打字机基础速度（字符/秒）
    pub text_speed: f32,

    // ── 子结构 ──
    pub anim: AnimationTimers,
    pub session: SessionAuthority,
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
            script_finished: false,
            services: None,
            history: Vec::new(),
            user_settings: UserSettings::default(),
            persistent_store: PersistentStore::empty(),
            snapshot_stack: SnapshotStack::new(50),
            playback_mode: PlaybackMode::Normal,
            auto_timer: 0.0,
            typewriter_timer: 0.0,
            text_speed: 30.0,
            anim: AnimationTimers::default(),
            session: SessionAuthority::default(),
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
