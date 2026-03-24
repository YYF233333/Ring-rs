//! 状态快照与回退
//!
//! 在每个停止点（WaitForClick / WaitForChoice）自动保存引擎完整状态，
//! 支持 Backspace 单步回退和历史记录跳转。
//!
//! 快照不持久化到存档，加载存档或新开游戏后快照栈为空。

use tracing::debug;
use vn_runtime::history::History;
use vn_runtime::state::{RuntimeState, WaitingReason};

use crate::PlaybackMode;
use crate::renderer::render_state::RenderState;

use super::AppState;

/// 单个停止点的完整状态快照
pub struct StateSnapshot {
    pub runtime_state: RuntimeState,
    pub history: History,
    pub render_state: RenderState,
    pub waiting_reason: WaitingReason,
    pub current_bgm: Option<String>,
}

/// 快照栈（不设上限、不持久化）
pub struct SnapshotStack {
    snapshots: Vec<StateSnapshot>,
}

impl SnapshotStack {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn push(&mut self, snapshot: StateSnapshot) {
        self.snapshots.push(snapshot);
    }

    pub fn pop(&mut self) -> Option<StateSnapshot> {
        self.snapshots.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }
}

/// 在停止点捕获当前状态快照
///
/// 仅在 WaitForClick / WaitForChoice 时调用，在 `run_script_tick` 推进前保存当前状态。
pub fn capture_snapshot(app_state: &mut AppState) {
    let runtime = match app_state.session.vn_runtime.as_ref() {
        Some(r) => r,
        None => return,
    };

    let snapshot = StateSnapshot {
        runtime_state: runtime.state().clone(),
        history: runtime.history().clone(),
        render_state: app_state.core.render_state.clone(),
        waiting_reason: app_state.session.waiting_reason.clone(),
        current_bgm: app_state
            .core
            .audio_manager
            .as_ref()
            .and_then(|a| a.current_bgm_path().map(|s| s.to_string())),
    };

    app_state.snapshot_stack.push(snapshot);
    debug!(count = app_state.snapshot_stack.len(), "快照已保存");
}

/// 回退到上一个快照
///
/// 恢复 Runtime、RenderState、音频三层状态。返回 `true` 表示回退成功。
pub fn rollback(app_state: &mut AppState) -> bool {
    let snapshot = match app_state.snapshot_stack.pop() {
        Some(s) => s,
        None => {
            debug!("快照栈为空，无法回退");
            return false;
        }
    };

    let target_bgm = snapshot.current_bgm;

    // 1. 恢复 Runtime 状态与历史记录
    if let Some(ref mut runtime) = app_state.session.vn_runtime {
        runtime.restore_state(snapshot.runtime_state);
        runtime.restore_history(snapshot.history);
    }

    // 2. 清理活跃的动画和过渡效果
    if app_state.core.animation_system.has_active_animations() {
        app_state.core.animation_system.skip_all();
        let _ = app_state.core.animation_system.update(0.0);
    }
    if app_state.core.renderer.transition.is_active() {
        app_state.core.renderer.transition.skip();
    }
    if app_state.core.renderer.is_scene_transition_active() {
        let _ = app_state.core.renderer.skip_scene_transition_to_end();
    }

    // 3. 恢复 RenderState（整体替换）
    app_state.core.render_state = snapshot.render_state;
    app_state.core.character_object_ids.clear();

    // 4. 恢复等待状态
    if let WaitingReason::WaitForChoice { choice_count } = &snapshot.waiting_reason {
        app_state.input_manager.reset_choice(*choice_count);
    }
    app_state.session.waiting_reason = snapshot.waiting_reason;

    // 5. 重置推进相关计时器
    app_state.session.typewriter_timer = 0.0;
    app_state.session.auto_timer = 0.0;
    app_state.session.playback_mode = PlaybackMode::Normal;
    app_state.session.script_finished = false;

    // 6. 恢复 BGM（仅在变化时切换）
    let current_bgm = app_state
        .core
        .audio_manager
        .as_ref()
        .and_then(|a| a.current_bgm_path().map(|s| s.to_string()));
    if current_bgm != target_bgm
        && let Some(ref mut audio) = app_state.core.audio_manager
    {
        match &target_bgm {
            Some(path) => audio.play_bgm(path, true, Some(0.3)),
            None => audio.stop_bgm(Some(0.3)),
        }
    }

    debug!(
        remaining = app_state.snapshot_stack.len(),
        "已回退到上一个快照"
    );
    true
}
