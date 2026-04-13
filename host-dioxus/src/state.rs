use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use vn_runtime::command::{Command, Position};
use vn_runtime::history::HistoryEvent;
use vn_runtime::state::{VarValue, WaitingReason};
use vn_runtime::{Parser, RuntimeInput, Script, ScriptNode, VNRuntime};

use crate::audio::AudioManager;
use crate::command_executor::{
    AudioCommand, BatchOutput, CommandExecutor, ExecuteResult, SceneEffectKind, SceneEffectRequest,
};
use crate::config::AppConfig;
use crate::error::{HostError, HostResult};
use crate::render_state::{
    CutsceneState, HostScreen, PlaybackMode, RenderState, SceneTransitionPhaseState,
};
use crate::resources::{LogicalPath, ResourceManager};
use crate::save_manager::SaveManager;

/// 用户可调设置（前端 ↔ 后端同步）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub bgm_volume: f32,
    pub sfx_volume: f32,
    pub text_speed: f32,
    pub auto_delay: f32,
    pub fullscreen: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            bgm_volume: 80.0,
            sfx_volume: 100.0,
            text_speed: 40.0,
            auto_delay: 2.0,
            fullscreen: false,
        }
    }
}

/// 对话历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub speaker: Option<String>,
    pub text: String,
}

/// Dioxus Desktop 托管的全局应用状态
#[derive(Clone)]
pub struct AppState {
    pub inner: std::sync::Arc<std::sync::Mutex<AppStateInner>>,
}

/// 当前会话的前端 owner。
#[derive(Debug, Clone)]
struct SessionOwner {
    token: String,
    label: String,
}

/// 前端连接后返回的会话信息。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendSession {
    pub client_token: String,
    pub render_state: RenderState,
}

/// 机读 harness 结束态快照。
#[derive(Debug, Clone, Serialize)]
pub struct HarnessSnapshot {
    pub render_state: RenderState,
    pub waiting: WaitingFor,
    pub script_finished: bool,
    pub playback_mode: PlaybackMode,
    pub host_screen: HostScreen,
    pub history_count: usize,
}

/// trace 中的单条事件。
#[derive(Debug, Clone, Serialize)]
pub struct HarnessTraceEvent {
    pub seq: u64,
    pub logical_time_ms: u64,
    pub kind: String,
    pub data: serde_json::Value,
}

/// trace bundle 元信息。
#[derive(Debug, Clone, Serialize)]
pub struct HarnessTraceMetadata {
    pub dt_seconds: f32,
    pub steps_run: usize,
    pub stop_reason: String,
    pub owner_label: Option<String>,
}

/// deterministic harness 的机读产物。
#[derive(Debug, Clone, Serialize)]
pub struct HarnessTraceBundle {
    pub metadata: HarnessTraceMetadata,
    pub events: Vec<HarnessTraceEvent>,
    pub final_snapshot: HarnessSnapshot,
}

// ── 持久化存储 ──────────────────────────────────────────────────────────────

const PERSISTENT_FILE: &str = "persistent.json";

/// 持久化变量存储（跨会话保留的 `$persistent.key` 变量）
pub struct PersistentStore {
    saves_dir: PathBuf,
    pub variables: HashMap<String, VarValue>,
}

impl PersistentStore {
    /// 创建空 store
    pub fn empty() -> Self {
        Self {
            saves_dir: PathBuf::new(),
            variables: HashMap::new(),
        }
    }

    /// 从存档目录加载；文件不存在或解析失败时返回空 store
    pub fn load(saves_dir: impl AsRef<Path>) -> Self {
        let saves_dir = saves_dir.as_ref().to_path_buf();
        let path = saves_dir.join(PERSISTENT_FILE);

        let variables = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_else(|| {
                    warn!(path = %path.display(), "持久化变量加载失败，使用空 store");
                    HashMap::new()
                })
        } else {
            HashMap::new()
        };

        Self {
            saves_dir,
            variables,
        }
    }

    /// 写入磁盘
    pub fn save(&self) -> HostResult<()> {
        if !self.saves_dir.exists() {
            fs::create_dir_all(&self.saves_dir)?;
        }
        let path = self.saves_dir.join(PERSISTENT_FILE);
        let content = serde_json::to_string_pretty(&self.variables)
            .map_err(|e| HostError::Internal(format!("持久化变量序列化失败: {e}")))?;
        fs::write(&path, content)?;
        info!(path = %path.display(), count = self.variables.len(), "持久化变量保存成功");
        Ok(())
    }

    /// 将 runtime persistent_variables 合并入 store（runtime 值覆盖）
    pub fn merge_from(&mut self, vars: &HashMap<String, VarValue>) {
        for (k, v) in vars {
            self.variables.insert(k.clone(), v.clone());
        }
    }
}

// ── 快照栈 ──────────────────────────────────────────────────────────────────

/// 状态快照（用于 Backspace 回退）
pub struct Snapshot {
    pub render_state: RenderState,
    pub runtime_state: vn_runtime::state::RuntimeState,
    pub runtime_history: vn_runtime::History,
    pub current_bgm: Option<String>,
}

/// 快照栈
pub struct SnapshotStack {
    snapshots: Vec<Snapshot>,
    max_size: usize,
}

impl SnapshotStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, snapshot: Snapshot) {
        if self.snapshots.len() >= self.max_size {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot);
    }

    pub fn pop(&mut self) -> Option<Snapshot> {
        self.snapshots.pop()
    }

    pub fn last(&self) -> Option<&Snapshot> {
        self.snapshots.last()
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
}

// ── 应用状态 ────────────────────────────────────────────────────────────────

/// setup() 中一次性初始化的子系统集合。
/// 初始化后不可能为 None——通过 `services()` 访问器断言此不变量。
pub struct Services {
    pub audio: AudioManager,
    pub resources: ResourceManager,
    pub saves: SaveManager,
    pub config: AppConfig,
    pub manifest: crate::manifest::Manifest,
}

/// 应用状态内部结构（被 Mutex 保护）
pub struct AppStateInner {
    pub runtime: Option<VNRuntime>,
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
    bg_transition_elapsed: f32,
    /// 场景过渡内部计时器
    scene_transition_elapsed: f32,
    /// 活跃的 shake 动画状态
    active_shake: Option<ShakeAnimation>,
    /// 是否有活跃的场景效果（用于 signal 解析）
    scene_effect_active: bool,
    /// 当前持有 session authority 的客户端。
    client_owner: Option<SessionOwner>,
    /// client token 单调递增计数器。
    next_client_id: u64,
    /// deterministic harness 的逻辑时间。
    logical_time_ms: u64,
    /// 机读 trace 事件缓冲区。
    trace_events: Vec<HarnessTraceEvent>,
    /// trace 事件序号。
    trace_seq: u64,
}

/// Shake 动画的运行时状态
struct ShakeAnimation {
    amplitude_x: f32,
    amplitude_y: f32,
    duration: f32,
    elapsed: f32,
}

/// Host 侧 Signal 等待的具体种类。
///
/// 与 `vn_runtime::command::SIGNAL_*` 常量一一对应，
/// 编译期保证穷举，消除字符串拼写错误风险。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    SceneTransition,
    TitleCard,
    SceneEffect,
    Cutscene,
}

/// Host 侧的等待状态
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WaitingFor {
    Nothing,
    Click,
    Choice,
    Time {
        remaining_ms: u64,
    },
    Cutscene,
    Signal(SignalKind),
    /// 等待 UI 交互结果（`requestUI` / `callGame` / `showMap`），对应 Runtime 的 WaitForUIResult
    UIResult {
        key: String,
    },
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

    fn project_render_state(&mut self) {
        self.render_state.playback_mode = self.playback_mode.clone();
        self.render_state.host_screen = self.host_screen.clone();
    }

    fn snapshot_for_trace(&self) -> HarnessSnapshot {
        HarnessSnapshot {
            render_state: self.render_state.clone(),
            waiting: self.waiting.clone(),
            script_finished: self.script_finished,
            playback_mode: self.playback_mode.clone(),
            host_screen: self.host_screen.clone(),
            history_count: self.history.len(),
        }
    }

    fn clear_trace(&mut self) {
        self.trace_events.clear();
        self.trace_seq = 0;
        self.logical_time_ms = 0;
    }

    fn record_trace(&mut self, kind: &str, data: serde_json::Value) {
        self.trace_events.push(HarnessTraceEvent {
            seq: self.trace_seq,
            logical_time_ms: self.logical_time_ms,
            kind: kind.to_string(),
            data,
        });
        self.trace_seq += 1;
    }

    fn build_trace_bundle(
        &self,
        dt_seconds: f32,
        steps_run: usize,
        stop_reason: impl Into<String>,
    ) -> HarnessTraceBundle {
        HarnessTraceBundle {
            metadata: HarnessTraceMetadata {
                dt_seconds,
                steps_run,
                stop_reason: stop_reason.into(),
                owner_label: self.client_owner.as_ref().map(|owner| owner.label.clone()),
            },
            events: self.trace_events.clone(),
            final_snapshot: self.snapshot_for_trace(),
        }
    }

    pub fn frontend_connected(&mut self, label: Option<String>) -> FrontendSession {
        self.next_client_id += 1;
        let token = format!("client-{}", self.next_client_id);
        let label = label.unwrap_or_else(|| "frontend".to_string());
        self.client_owner = Some(SessionOwner {
            token: token.clone(),
            label: label.clone(),
        });

        if self.runtime.is_none() {
            self.host_screen = HostScreen::Title;
        }
        self.project_render_state();
        self.record_trace(
            "client_claimed",
            serde_json::json!({
                "label": label,
                "token": token,
                "host_screen": format!("{:?}", self.host_screen),
            }),
        );

        FrontendSession {
            client_token: token,
            render_state: self.render_state.clone(),
        }
    }

    pub fn assert_owner(&self, client_token: &str) -> HostResult<()> {
        let owner = self.client_owner.as_ref().ok_or_else(|| {
            HostError::Session(
                "当前没有已连接的前端 owner，请先调用 frontend_connected".to_string(),
            )
        })?;
        if owner.token == client_token {
            Ok(())
        } else {
            Err(HostError::Session(format!(
                "当前会话已被其他客户端接管（owner={}），拒绝推进请求",
                owner.label
            )))
        }
    }

    pub fn set_host_screen(&mut self, screen: HostScreen) {
        if self.host_screen == screen {
            self.project_render_state();
            return;
        }

        let previous = format!("{:?}", self.host_screen);
        self.host_screen = screen;

        if !self.host_screen.allows_progression() && self.playback_mode != PlaybackMode::Normal {
            self.playback_mode = PlaybackMode::Normal;
            self.auto_timer = 0.0;
        }

        self.project_render_state();
        self.record_trace(
            "host_screen_changed",
            serde_json::json!({
                "from": previous,
                "to": format!("{:?}", self.host_screen),
            }),
        );
    }

    pub fn set_playback_mode(&mut self, mode: PlaybackMode) {
        let next_mode = if self.host_screen.allows_progression() {
            mode
        } else {
            PlaybackMode::Normal
        };
        if self.playback_mode == next_mode {
            self.project_render_state();
            return;
        }

        let previous = format!("{:?}", self.playback_mode);
        self.playback_mode = next_mode;
        self.auto_timer = 0.0;
        self.project_render_state();
        self.record_trace(
            "playback_mode_changed",
            serde_json::json!({
                "from": previous,
                "to": format!("{:?}", self.playback_mode),
            }),
        );
    }

    fn build_save_data(&self, slot: u32) -> HostResult<vn_runtime::SaveData> {
        let (runtime_state, runtime_history, render_state, current_bgm) =
            if waiting_requires_snapshot_fallback(&self.waiting) {
                let snapshot = self.snapshot_stack.last().ok_or_else(|| {
                    HostError::Internal(format!(
                        "当前处于 {:?} 中间态，且没有可回退快照，无法安全保存",
                        self.waiting
                    ))
                })?;
                warn!(
                    waiting = ?self.waiting,
                    "保存时使用最近快照作为稳定边界，避免写入宿主无法直接恢复的中间态"
                );
                (
                    snapshot.runtime_state.clone(),
                    snapshot.runtime_history.clone(),
                    &snapshot.render_state,
                    snapshot.current_bgm.clone(),
                )
            } else {
                let runtime = self
                    .runtime
                    .as_ref()
                    .ok_or_else(|| HostError::Internal("游戏未启动".to_string()))?;
                (
                    runtime.state().clone(),
                    runtime.history().clone(),
                    &self.render_state,
                    self.services()
                        .audio
                        .current_bgm_path()
                        .map(|s| s.to_string()),
                )
            };

        let mut save_data = vn_runtime::SaveData::new(slot, runtime_state)
            .with_history(runtime_history)
            .with_render(vn_runtime::RenderSnapshot {
                background: render_state.current_background.clone(),
                characters: render_state
                    .visible_characters
                    .iter()
                    .map(|(alias, sprite)| vn_runtime::CharacterSnapshot {
                        alias: alias.clone(),
                        texture_path: sprite.texture_path.clone(),
                        position: format!("{:?}", sprite.position),
                    })
                    .collect(),
            })
            .with_audio(vn_runtime::AudioState {
                current_bgm,
                bgm_looping: true,
            });

        if let Some(ref chapter) = render_state.chapter_mark {
            save_data = save_data.with_chapter(&chapter.title);
        }

        Ok(save_data)
    }

    pub fn save_to_slot(&mut self, slot: u32) -> HostResult<()> {
        let save_data = self.build_save_data(slot)?;
        self.services().saves.save(&save_data)?;
        self.record_trace("save_slot_written", serde_json::json!({ "slot": slot }));
        Ok(())
    }

    pub fn save_to_slot_with_thumbnail(
        &mut self,
        slot: u32,
        thumbnail_png: &[u8],
    ) -> HostResult<()> {
        let save_data = self.build_save_data(slot)?;
        self.services()
            .saves
            .save_thumbnail_png(slot, thumbnail_png)?;
        self.services().saves.save(&save_data)?;
        self.record_trace(
            "save_slot_written",
            serde_json::json!({ "slot": slot, "thumbnail": true }),
        );
        Ok(())
    }

    pub fn save_continue(&mut self) -> HostResult<()> {
        let save_data = self.build_save_data(0)?;
        self.services().saves.save_continue(&save_data)?;
        self.record_trace("continue_written", serde_json::json!({}));
        Ok(())
    }

    pub fn delete_continue(&mut self) -> HostResult<()> {
        self.services().saves.delete_continue()?;
        self.record_trace("continue_deleted", serde_json::json!({}));
        Ok(())
    }

    /// 将 PersistentStore 中的变量注入到当前 runtime。
    fn inject_persistent_vars(&mut self) {
        if self.persistent_store.variables.is_empty() {
            return;
        }
        if let Some(rt) = self.runtime.as_mut() {
            for (k, v) in &self.persistent_store.variables {
                rt.state_mut().set_persistent_var(k.clone(), v.clone());
            }
        }
    }

    /// 重置游戏会话状态——停止音频、清空 runtime 与渲染状态。
    ///
    /// `return_to_title`、`init_game` 等需要"干净开始"的方法共用此逻辑，
    /// 确保不会遗漏子系统清理。
    fn reset_session(&mut self) {
        if let Some(svc) = self.services.as_mut() {
            svc.audio.stop_bgm(None);
        }
        self.runtime = None;
        self.render_state = RenderState::new();
        self.host_screen = HostScreen::Title;
        self.waiting = WaitingFor::Nothing;
        self.typewriter_timer = 0.0;
        self.script_finished = false;
        self.history.clear();
        self.snapshot_stack.clear();
        self.playback_mode = PlaybackMode::Normal;
        self.auto_timer = 0.0;
        self.bg_transition_elapsed = 0.0;
        self.scene_transition_elapsed = 0.0;
        self.active_shake = None;
        self.scene_effect_active = false;
        self.logical_time_ms = 0;
        self.project_render_state();
    }

    /// 解析脚本并初始化运行时
    pub fn init_game(&mut self, script_content: &str) -> HostResult<()> {
        let mut parser = Parser::new();
        let script = parser.parse("main", script_content)?;
        self.reset_session();
        self.clear_trace();
        self.runtime = Some(VNRuntime::new(script));
        self.inject_persistent_vars();
        self.set_host_screen(HostScreen::InGame);
        self.record_trace(
            "game_started",
            serde_json::json!({ "script_path": "inline" }),
        );
        self.run_script_tick();
        Ok(())
    }

    /// 通过 ResourceManager 构建运行时，但不执行首帧。
    fn build_runtime_from_resource(&self, script_path: &str) -> HostResult<VNRuntime> {
        let rm = &self.services().resources;

        let logical = LogicalPath::new(script_path);
        let content = rm.read_text(&logical)?;

        let script_id = logical.file_stem().to_string();
        let base_dir = logical.parent_dir().to_string();

        let mut parser = Parser::new();
        let script = parser.parse_with_base_path(&script_id, &content, &base_dir)?;

        for w in parser.warnings() {
            warn!(warning = %w, "脚本解析警告");
        }

        let mut runtime = VNRuntime::new(script.clone());
        let normalized = logical.as_str().to_string();
        runtime.register_script(&normalized, script.clone());
        runtime.state_mut().position.set_path(&normalized);

        let mut visited = HashSet::new();
        visited.insert(normalized);
        preload_called_scripts(&mut runtime, rm, &script, &mut visited);

        Ok(runtime)
    }

    fn start_runtime(
        &mut self,
        mut runtime: VNRuntime,
        script_path: &str,
        start_label: Option<&str>,
    ) -> HostResult<()> {
        if let Some(label) = start_label {
            let target = runtime
                .find_label(label)
                .ok_or_else(|| HostError::InvalidInput(format!("标签未找到: {label}")))?;
            runtime.state_mut().position.jump_to(target);
        }

        self.reset_session();
        self.clear_trace();
        self.runtime = Some(runtime);
        self.inject_persistent_vars();
        self.set_host_screen(HostScreen::InGame);
        self.record_trace(
            "game_started",
            serde_json::json!({
                "script_path": script_path,
                "start_label": start_label,
            }),
        );
        self.run_script_tick();
        Ok(())
    }

    /// 通过 ResourceManager 读取脚本并初始化运行时
    ///
    /// 除入口脚本外，递归预加载所有 `callScript` 引用的子脚本，
    /// 保证运行时执行到跨文件调用时不会因脚本未注册而失败。
    pub fn init_game_from_resource(&mut self, script_path: &str) -> HostResult<()> {
        let runtime = self.build_runtime_from_resource(script_path)?;
        self.start_runtime(runtime, script_path, None)
    }

    pub fn init_game_from_resource_at_label(
        &mut self,
        script_path: &str,
        label: &str,
    ) -> HostResult<()> {
        let runtime = self.build_runtime_from_resource(script_path)?;
        self.start_runtime(runtime, script_path, Some(label))
    }

    /// 递归预加载 `callScript` 引用的所有子脚本
    /// 每帧调用，推进打字机和计时器
    pub fn process_tick(&mut self, dt: f32) {
        if !self.host_screen.allows_progression() {
            self.project_render_state();
            return;
        }

        self.logical_time_ms += (dt * 1000.0).round() as u64;
        let waiting_before = format!("{:?}", self.waiting);
        self.advance_playback_mode(dt);
        self.update_animations(dt);
        self.resolve_waits(dt);

        if self.waiting == WaitingFor::Nothing && !self.script_finished {
            self.run_script_tick();
        }

        self.advance_typewriter(dt);
        self.sync_audio(dt);
        self.project_render_state();
        self.record_trace(
            "tick",
            serde_json::json!({
                "dt_seconds": dt,
                "waiting_before": waiting_before,
                "waiting_after": format!("{:?}", self.waiting),
                "script_finished": self.script_finished,
            }),
        );
    }

    /// Skip 模式立即推进 + Auto 模式计时推进
    fn advance_playback_mode(&mut self, dt: f32) {
        if self.playback_mode == PlaybackMode::Skip {
            if !self.render_state.is_dialogue_complete() {
                self.render_state.complete_typewriter();
            }

            match self.waiting.clone() {
                WaitingFor::Click => {
                    self.clear_click_wait();
                }
                WaitingFor::Time { .. } => {
                    self.clear_wait();
                }
                WaitingFor::Signal(signal_kind) => {
                    self.complete_signal_wait(signal_kind);
                }
                WaitingFor::Cutscene => {
                    self.finish_cutscene();
                }
                _ => {}
            }
        }

        if self.playback_mode == PlaybackMode::Auto
            && self.waiting == WaitingFor::Click
            && self.render_state.is_dialogue_complete()
        {
            self.auto_timer += dt;
            if self.auto_timer >= self.user_settings.auto_delay {
                self.auto_timer = 0.0;
                self.clear_click_wait();
            }
        }
    }

    fn complete_signal_wait(&mut self, signal_kind: SignalKind) {
        match signal_kind {
            SignalKind::SceneTransition => {
                if let Some(st) = self.render_state.scene_transition.as_mut() {
                    if let Some(bg) = st.pending_background.take() {
                        self.render_state.current_background = Some(bg);
                    }
                    st.phase = SceneTransitionPhaseState::Completed;
                }
            }
            SignalKind::TitleCard => {
                self.render_state.title_card = None;
            }
            SignalKind::SceneEffect => {
                self.scene_effect_active = false;
            }
            SignalKind::Cutscene => {
                self.finish_cutscene();
            }
        }

        self.clear_wait();
    }

    /// 推进 chapter_mark / title_card / background_transition / scene_transition / 角色 alpha
    fn update_animations(&mut self, dt: f32) {
        self.render_state.update_chapter_mark(dt);

        if let Some(tc) = self.render_state.title_card.as_mut() {
            tc.elapsed += dt;
            if tc.elapsed >= tc.duration {
                self.render_state.title_card = None;
            }
        }

        self.update_background_transition(dt);
        self.update_scene_transition(dt);
        self.update_character_alpha(dt);
        self.update_shake(dt);
    }

    /// 推进角色 alpha 过渡，淡出完成后移除
    fn update_character_alpha(&mut self, dt: f32) {
        for c in self.render_state.visible_characters.values_mut() {
            let duration = c.transition_duration.unwrap_or(0.0);
            if duration > 0.0 && (c.alpha - c.target_alpha).abs() > f32::EPSILON {
                let speed = dt / duration;
                if c.alpha < c.target_alpha {
                    c.alpha = (c.alpha + speed).min(c.target_alpha);
                } else {
                    c.alpha = (c.alpha - speed).max(c.target_alpha);
                }
                if (c.alpha - c.target_alpha).abs() <= f32::EPSILON {
                    c.alpha = c.target_alpha;
                    c.transition_duration = None;
                }
            } else if duration <= 0.0 {
                c.alpha = c.target_alpha;
            }
        }
        self.render_state
            .visible_characters
            .retain(|_, c| !(c.fading_out && c.alpha <= f32::EPSILON));
    }

    /// 解析 Signal 等待 + Time 等待
    fn resolve_waits(&mut self, dt: f32) {
        if let WaitingFor::Signal(signal_kind) = self.waiting {
            let resolved = match signal_kind {
                SignalKind::SceneTransition => self
                    .render_state
                    .scene_transition
                    .as_ref()
                    .is_none_or(|st| st.phase == SceneTransitionPhaseState::Completed),
                SignalKind::TitleCard => self.render_state.title_card.is_none(),
                SignalKind::SceneEffect => !self.scene_effect_active,
                SignalKind::Cutscene => self.render_state.cutscene.is_none(),
            };
            if resolved {
                self.clear_wait();
            }
        }

        if let WaitingFor::Time { remaining_ms } = &self.waiting {
            let elapsed_ms = (dt * 1000.0) as u64;
            if elapsed_ms >= *remaining_ms {
                self.clear_wait();
            } else {
                let decrement = elapsed_ms;
                if let WaitingFor::Time { remaining_ms } = &mut self.waiting {
                    *remaining_ms -= decrement;
                }
            }
        }
    }

    /// 推进打字机 + inline wait
    fn advance_typewriter(&mut self, dt: f32) {
        if !self.render_state.is_dialogue_complete() && !self.render_state.has_inline_wait() {
            let speed = self.render_state.effective_text_speed(self.text_speed);
            self.typewriter_timer += dt * speed;
            while self.typewriter_timer >= 1.0 {
                self.typewriter_timer -= 1.0;
                let done = self.render_state.advance_typewriter();
                if done {
                    self.typewriter_timer = 0.0;
                    if self
                        .render_state
                        .dialogue
                        .as_ref()
                        .is_some_and(|d| d.no_wait)
                        && self.waiting == WaitingFor::Click
                    {
                        self.clear_click_wait();
                    }
                    break;
                }
                if self.render_state.has_inline_wait() {
                    break;
                }
            }
        }

        if self.render_state.has_inline_wait() && !self.render_state.is_inline_click_wait() {
            let finished = self.render_state.update_inline_wait(dt as f64);
            if finished {
                // 定时等待结束，继续打字
            }
        }
    }

    /// 同步音频状态到 render_state
    fn sync_audio(&mut self, dt: f32) {
        if let Some(svc) = self.services.as_mut() {
            svc.audio.update(dt);
            self.render_state.audio = svc.audio.drain_audio_state();
        }
    }

    /// 推进背景 dissolve 过渡（内部计时器，不推到 RenderState）
    fn update_background_transition(&mut self, dt: f32) {
        if let Some(bt) = self.render_state.background_transition.as_mut() {
            self.bg_transition_elapsed += dt;
            if self.bg_transition_elapsed >= bt.duration {
                self.render_state.background_transition = None;
                self.bg_transition_elapsed = 0.0;
            }
        }
    }

    /// 推进场景遮罩过渡（内部计时器推进 phase，不计算渐变值）
    fn update_scene_transition(&mut self, dt: f32) {
        const HOLD_DURATION: f32 = 0.2;

        let Some(st) = self.render_state.scene_transition.as_mut() else {
            return;
        };

        self.scene_transition_elapsed += dt;

        match st.phase {
            SceneTransitionPhaseState::FadeIn => {
                if self.scene_transition_elapsed >= st.duration {
                    if let Some(bg) = st.pending_background.take() {
                        self.render_state.current_background = Some(bg);
                    }
                    st.phase = SceneTransitionPhaseState::Hold;
                    self.scene_transition_elapsed = 0.0;
                }
            }
            SceneTransitionPhaseState::Hold => {
                if self.scene_transition_elapsed >= HOLD_DURATION {
                    st.phase = SceneTransitionPhaseState::FadeOut;
                    self.scene_transition_elapsed = 0.0;
                }
            }
            SceneTransitionPhaseState::FadeOut => {
                if self.scene_transition_elapsed >= st.duration {
                    st.phase = SceneTransitionPhaseState::Completed;
                    self.scene_transition_elapsed = 0.0;
                }
            }
            SceneTransitionPhaseState::Completed => {
                self.render_state.scene_transition = None;
                self.scene_transition_elapsed = 0.0;
            }
        }
    }

    /// 处理用户点击
    pub fn process_click(&mut self) {
        if !self.host_screen.allows_progression() {
            return;
        }
        self.auto_timer = 0.0;
        self.record_trace(
            "click",
            serde_json::json!({
                "waiting": format!("{:?}", self.waiting),
                "dialogue_complete": self.render_state.is_dialogue_complete(),
            }),
        );

        if !self.render_state.is_dialogue_complete() {
            self.render_state.complete_typewriter();
            if self
                .render_state
                .dialogue
                .as_ref()
                .is_some_and(|d| d.no_wait)
                && self.waiting == WaitingFor::Click
            {
                self.clear_click_wait();
            }
            return;
        }

        if self.render_state.is_inline_click_wait() {
            self.render_state.clear_inline_wait();
            return;
        }

        if self.waiting == WaitingFor::Click {
            self.capture_snapshot();
            self.clear_click_wait();
        }
    }

    /// 同时清除 host 侧和 runtime 侧的等待状态
    fn clear_wait(&mut self) {
        if let Some(rt) = self.runtime.as_mut() {
            rt.state_mut().clear_wait();
        }
        self.waiting = WaitingFor::Nothing;
    }

    fn clear_click_wait(&mut self) {
        self.clear_wait();
    }

    /// 恢复操作（restore_from_save / restore_snapshot）的公共收尾逻辑
    fn finish_restore(&mut self) {
        self.script_finished = false;
        self.typewriter_timer = 0.0;
        self.auto_timer = 0.0;
        self.playback_mode = PlaybackMode::Normal;
        self.set_host_screen(HostScreen::InGame);
        self.project_render_state();
    }

    /// 捕获当前状态快照（用于 Backspace 回退）
    fn capture_snapshot(&mut self) {
        let Some(rt) = self.runtime.as_ref() else {
            return;
        };
        let snapshot = Snapshot {
            render_state: self.render_state.clone(),
            runtime_state: rt.state().clone(),
            runtime_history: rt.history().clone(),
            current_bgm: self
                .services
                .as_ref()
                .and_then(|svc| svc.audio.current_bgm_path().map(|s| s.to_string())),
        };
        self.snapshot_stack.push(snapshot);
    }

    /// 恢复到最近的快照
    pub fn restore_snapshot(&mut self) -> bool {
        let Some(snapshot) = self.snapshot_stack.pop() else {
            return false;
        };
        let target_bgm = snapshot.current_bgm.clone();
        if let Some(rt) = self.runtime.as_mut() {
            rt.restore_state(snapshot.runtime_state.clone());
            rt.restore_history(snapshot.runtime_history.clone());
        }
        self.render_state = snapshot.render_state;
        self.history = host_history_from_runtime(&snapshot.runtime_history);
        self.waiting = map_runtime_waiting(&snapshot.runtime_state.waiting);
        {
            let audio = &mut self.services_mut().audio;
            match target_bgm {
                Some(path) => audio.play_bgm(&path, true, None),
                None => audio.stop_bgm(None),
            }
        }
        self.sync_audio(0.0);
        self.finish_restore();
        self.record_trace("snapshot_restored", serde_json::json!({}));
        true
    }

    /// 结束视频过场
    pub fn finish_cutscene(&mut self) {
        self.render_state.cutscene = None;
        if self.waiting == WaitingFor::Cutscene {
            self.waiting = WaitingFor::Nothing;
        }
        if let Some(svc) = self.services.as_mut() {
            svc.audio.unduck();
        }
        self.sync_audio(0.0);
        self.record_trace("cutscene_finished", serde_json::json!({}));
    }

    /// 处理前端回传的 UI 交互结果
    pub fn handle_ui_result(&mut self, key: String, value: serde_json::Value) -> HostResult<()> {
        let expected_key = match &self.waiting {
            WaitingFor::UIResult { key } => key.clone(),
            _ => return Err(HostError::InvalidInput("当前未在等待 UI 结果".to_string())),
        };
        if key != expected_key {
            return Err(HostError::InvalidInput(format!(
                "UIResult key 不匹配：期望 '{expected_key}'，收到 '{key}'"
            )));
        }

        let var_value = crate::render_state::json_to_var_value(&value);

        let input = RuntimeInput::UIResult {
            key,
            value: var_value,
        };
        let tick_result = self
            .runtime
            .as_mut()
            .expect("invariant: UIResult requires loaded runtime")
            .tick(Some(input))?;
        self.render_state.active_ui_mode = None;
        self.waiting = WaitingFor::Nothing;
        self.apply_runtime_tick_output(tick_result.0, tick_result.1);

        self.record_trace("ui_result_submitted", serde_json::json!({}));

        Ok(())
    }

    /// 处理用户选择
    pub fn process_choose(&mut self, index: usize) {
        if !self.host_screen.allows_progression() {
            return;
        }
        if self.waiting != WaitingFor::Choice {
            return;
        }
        let tick_result = self
            .runtime
            .as_mut()
            .expect("invariant: choice selection requires loaded runtime")
            .tick(Some(RuntimeInput::choice(index)));
        self.render_state.clear_choices();
        self.waiting = WaitingFor::Nothing;
        self.record_trace("choice_selected", serde_json::json!({ "index": index }));
        match tick_result {
            Ok((commands, waiting_reason)) => {
                self.apply_runtime_tick_output(commands, waiting_reason)
            }
            Err(error) => {
                warn!(%error, "choice selection tick failed");
                self.script_finished = true;
                self.record_trace("script_tick_error", serde_json::json!({}));
            }
        }
    }

    fn apply_runtime_tick_output(&mut self, commands: Vec<Command>, waiting_reason: WaitingReason) {
        let manifest = &self
            .services
            .as_ref()
            .expect("invariant: services initialized in setup()")
            .manifest;
        let BatchOutput {
            result,
            audio_commands,
            scene_effect_request,
        } = self
            .command_executor
            .execute_batch(&commands, &mut self.render_state, manifest);

        if let Some(ref d) = self.render_state.dialogue
            && (d.visible_chars == 0 || !d.content.is_empty())
        {
            let last_text = self.history.first().map(|h| h.text.as_str());
            if last_text != Some(&d.content) {
                self.push_history(d.speaker.clone(), d.content.clone());
            }
        }

        for cmd in audio_commands {
            self.dispatch_audio_command(cmd);
        }

        if let Some(req) = scene_effect_request {
            self.apply_scene_effect(req);
        }

        if result == ExecuteResult::FullRestart {
            self.return_to_title(false);
            return;
        }

        if let ExecuteResult::RequestUI {
            key, mode, params, ..
        } = &result
        {
            let json_params = params
                .iter()
                .map(|(k, v)| (k.clone(), crate::render_state::var_value_to_json(v)))
                .collect();
            self.render_state.active_ui_mode = Some(crate::render_state::UiModeRequest {
                mode: mode.clone(),
                key: key.clone(),
                params: json_params,
            });
        }

        if let ExecuteResult::WaitForCutscene { video_path } = &result {
            self.render_state.cutscene = Some(CutsceneState {
                video_path: video_path.clone(),
                is_playing: true,
            });
            self.services_mut().audio.duck();
        }

        // 用 Runtime 的 waiting_reason（权威来源）映射 Host 等待状态
        self.waiting = map_runtime_waiting(&waiting_reason);

        // 同步 runtime persistent 变量到 PersistentStore
        if let Some(rt) = self.runtime.as_ref() {
            let pv = &rt.state().persistent_variables;
            if !pv.is_empty() {
                self.persistent_store.merge_from(pv);
            }
        }

        if waiting_reason == WaitingReason::None && commands.is_empty() {
            self.script_finished = true;
            self.record_trace("script_finished", serde_json::json!({}));
            self.return_to_title(false);
            return;
        }

        self.project_render_state();
        self.record_trace(
            "script_tick",
            serde_json::json!({
                "commands_count": commands.len(),
                "waiting": format!("{:?}", self.waiting),
                "execute_result": format!("{:?}", result),
                "script_finished": self.script_finished,
            }),
        );
    }

    /// 调用 runtime.tick() 并执行产出的 commands
    pub(crate) fn run_script_tick(&mut self) {
        let Some(rt) = self.runtime.as_mut() else {
            return;
        };

        match rt.tick(None) {
            Ok((commands, waiting_reason)) => {
                self.apply_runtime_tick_output(commands, waiting_reason)
            }
            Err(error) => {
                warn!(%error, "脚本 tick 执行失败");
                self.script_finished = true;
                self.record_trace(
                    "script_tick_error",
                    serde_json::json!({ "error": error.to_string() }),
                );
            }
        }
    }

    /// 追加对话历史
    pub fn push_history(&mut self, speaker: Option<String>, text: String) {
        self.history.insert(0, HistoryEntry { speaker, text });
    }

    fn apply_render_snapshot(&mut self, render: &vn_runtime::RenderSnapshot) {
        self.render_state = RenderState::new();
        if let Some(background) = &render.background {
            self.render_state.set_background(background.clone());
        }

        let manifest = self.services().manifest.clone();
        for character in &render.characters {
            self.render_state.show_character(
                character.alias.clone(),
                character.texture_path.clone(),
                parse_saved_position(&character.position),
                &manifest,
            );
            if let Some(sprite) = self
                .render_state
                .visible_characters
                .get_mut(&character.alias)
            {
                sprite.alpha = 1.0;
                sprite.target_alpha = 1.0;
                sprite.transition_duration = None;
            }
        }
    }

    fn apply_audio_state(&mut self, audio: &vn_runtime::AudioState) {
        let manager = &mut self.services_mut().audio;
        match &audio.current_bgm {
            Some(path) => manager.play_bgm(path, audio.bgm_looping, None),
            None => manager.stop_bgm(None),
        }
        self.sync_audio(0.0);
    }

    fn normalize_restored_waiting(&mut self) {
        if !waiting_requires_snapshot_fallback(&self.waiting) {
            return;
        }

        warn!(
            waiting = ?self.waiting,
            "读档命中了宿主无法直接重建的等待态，回退到 WaitForClick 稳定点"
        );
        if let Some(rt) = self.runtime.as_mut() {
            rt.state_mut().wait(WaitingReason::click());
        }
        self.render_state.clear_choices();
        self.render_state.active_ui_mode = None;
        self.render_state.cutscene = None;
        self.render_state.scene_transition = None;
        self.render_state.title_card = None;
        self.waiting = WaitingFor::Click;
    }

    /// 从存档恢复游戏状态
    ///
    /// 统一处理 `load_game` 和 `continue_game` 的恢复逻辑：
    /// 若 runtime 尚未初始化，根据存档中的 `script_path` 加载入口脚本并预加载子脚本。
    pub fn restore_from_save(&mut self, save_data: vn_runtime::SaveData) -> HostResult<()> {
        let runtime_state = save_data.runtime_state.clone();
        let history = save_data.history.clone();
        let render = save_data.render.clone();
        let audio = save_data.audio.clone();

        let path = if !runtime_state.position.script_path.is_empty() {
            runtime_state.position.script_path.clone()
        } else {
            runtime_state.position.script_id.clone()
        };

        let mut runtime = self.build_runtime_from_resource(&path)?;
        load_call_stack_scripts(&mut runtime, &self.services().resources, &runtime_state);
        runtime.restore_state(runtime_state.clone());
        runtime.restore_history(history.clone());
        runtime.state_mut().persistent_variables = self.persistent_store.variables.clone();

        self.reset_session();
        self.clear_trace();
        self.runtime = Some(runtime);
        self.apply_render_snapshot(&render);
        self.apply_audio_state(&audio);
        self.history = host_history_from_runtime(&history);
        self.waiting = map_runtime_waiting(&runtime_state.waiting);
        self.normalize_restored_waiting();
        self.snapshot_stack.clear();
        self.finish_restore();
        self.record_trace(
            "save_restored",
            serde_json::json!({
                "script_path": path,
                "waiting": format!("{:?}", self.waiting),
                "history_count": self.history.len(),
            }),
        );
        Ok(())
    }

    /// 重置到标题画面状态
    pub fn return_to_title(&mut self, save_continue: bool) {
        if save_continue && let Err(error) = self.save_continue() {
            warn!(%error, "返回标题时保存 continue 失败");
        }
        if !save_continue && let Err(error) = self.delete_continue() {
            warn!(%error, "返回标题时删除 continue 失败");
        }
        if let Err(e) = self.persistent_store.save() {
            warn!("返回标题时持久化变量保存失败: {e}");
        }
        self.reset_session();
        self.set_host_screen(HostScreen::Title);
        self.record_trace(
            "returned_to_title",
            serde_json::json!({ "save_continue": save_continue }),
        );
    }

    pub fn debug_run_until(
        &mut self,
        dt: f32,
        max_steps: usize,
        stop_on_wait: bool,
        stop_on_script_finished: bool,
    ) -> HarnessTraceBundle {
        self.clear_trace();
        let mut steps = 0usize;

        let stop_reason = loop {
            if steps >= max_steps {
                break "max_steps";
            }
            if !self.host_screen.allows_progression() {
                break "host_screen_blocked";
            }

            self.process_tick(dt);
            steps += 1;

            if stop_on_script_finished && self.script_finished {
                break "script_finished";
            }
            if stop_on_wait && self.waiting != WaitingFor::Nothing {
                break "waiting";
            }
        };

        self.build_trace_bundle(dt, steps, stop_reason)
    }

    /// 分派音频命令到 AudioManager（headless 状态追踪）
    fn dispatch_audio_command(&mut self, cmd: AudioCommand) {
        let audio = &mut self.services_mut().audio;
        match cmd {
            AudioCommand::PlayBgm {
                path,
                looping,
                fade_in,
            } => {
                audio.play_bgm(&path, looping, fade_in);
            }
            AudioCommand::StopBgm { fade_out } => {
                audio.stop_bgm(fade_out);
            }
            AudioCommand::BgmDuck => {
                audio.duck();
            }
            AudioCommand::BgmUnduck => {
                audio.unduck();
            }
            AudioCommand::PlaySfx { path } => {
                audio.play_sfx(&path);
            }
        }
    }

    /// 应用场景效果请求
    fn apply_scene_effect(&mut self, req: SceneEffectRequest) {
        match req.kind {
            SceneEffectKind::Shake {
                amplitude_x,
                amplitude_y,
            } => {
                self.active_shake = Some(ShakeAnimation {
                    amplitude_x,
                    amplitude_y,
                    duration: req.duration,
                    elapsed: 0.0,
                });
                self.scene_effect_active = true;
            }
            SceneEffectKind::Blur => {
                self.render_state.scene_effect.blur_amount = 1.0;
                self.scene_effect_active = false;
            }
            SceneEffectKind::BlurOut => {
                self.render_state.scene_effect.blur_amount = 0.0;
                self.scene_effect_active = false;
            }
            SceneEffectKind::Dim { level } => {
                self.render_state.scene_effect.dim_level = level;
                self.scene_effect_active = false;
            }
            SceneEffectKind::DimReset => {
                self.render_state.scene_effect.dim_level = 0.0;
                self.scene_effect_active = false;
            }
        }
    }

    /// 推进 shake 动画
    fn update_shake(&mut self, dt: f32) {
        let Some(shake) = self.active_shake.as_mut() else {
            return;
        };
        shake.elapsed += dt;
        if shake.elapsed >= shake.duration {
            self.render_state.scene_effect.shake_offset_x = 0.0;
            self.render_state.scene_effect.shake_offset_y = 0.0;
            self.active_shake = None;
            self.scene_effect_active = false;
        } else {
            let progress = shake.elapsed / shake.duration;
            let decay = 1.0 - progress;
            let freq = 30.0;
            let phase = shake.elapsed * freq;
            self.render_state.scene_effect.shake_offset_x = shake.amplitude_x * decay * phase.sin();
            self.render_state.scene_effect.shake_offset_y = shake.amplitude_y * decay * phase.cos();
        }
    }
}

fn preload_called_scripts(
    runtime: &mut VNRuntime,
    rm: &ResourceManager,
    script: &Script,
    visited: &mut HashSet<String>,
) {
    for node in collect_call_nodes(&script.nodes) {
        let ScriptNode::CallScript { path, .. } = node else {
            continue;
        };

        let resolved = script.resolve_path(path);
        if !visited.insert(resolved.clone()) {
            continue;
        }

        let logical = LogicalPath::new(&resolved);
        let content = match rm.read_text(&logical) {
            Ok(text) => text,
            Err(e) => {
                warn!(path = %resolved, error = %e, "callScript 目标脚本预加载失败");
                continue;
            }
        };

        let child_id = logical.file_stem().to_string();
        let child_base = logical.parent_dir().to_string();
        let mut parser = Parser::new();
        let child_script = match parser.parse_with_base_path(&child_id, &content, &child_base) {
            Ok(s) => s,
            Err(e) => {
                warn!(path = %resolved, error = %e, "callScript 目标脚本解析失败");
                continue;
            }
        };

        runtime.register_script(&resolved, child_script.clone());
        preload_called_scripts(runtime, rm, &child_script, visited);
    }
}

/// 从 AST 中收集所有 CallScript 节点（包括条件分支内部的）
fn collect_call_nodes(nodes: &[ScriptNode]) -> Vec<&ScriptNode> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            ScriptNode::CallScript { .. } => result.push(node),
            ScriptNode::Conditional { branches } => {
                for branch in branches {
                    result.extend(collect_call_nodes(&branch.body));
                }
            }
            _ => {}
        }
    }
    result
}

fn load_call_stack_scripts(
    runtime: &mut VNRuntime,
    resources: &ResourceManager,
    runtime_state: &vn_runtime::state::RuntimeState,
) {
    for frame in &runtime_state.call_stack {
        let frame_path = if !frame.script_path.is_empty() {
            &frame.script_path
        } else {
            &frame.script_id
        };
        let logical = LogicalPath::new(frame_path);
        if let Ok(content) = resources.read_text(&logical) {
            let fid = logical.file_stem().to_string();
            let fbase = logical.parent_dir().to_string();
            let mut parser = Parser::new();
            if let Ok(script) = parser.parse_with_base_path(&fid, &content, &fbase) {
                runtime.register_script(frame_path, script);
            }
        }
    }
}

fn host_history_from_runtime(history: &vn_runtime::history::History) -> Vec<HistoryEntry> {
    history
        .events()
        .iter()
        .filter_map(|event| match event {
            HistoryEvent::Dialogue {
                speaker, content, ..
            } => Some(HistoryEntry {
                speaker: speaker.clone(),
                text: content.clone(),
            }),
            _ => None,
        })
        .rev()
        .collect()
}

fn waiting_requires_snapshot_fallback(waiting: &WaitingFor) -> bool {
    matches!(
        waiting,
        WaitingFor::Choice
            | WaitingFor::Cutscene
            | WaitingFor::Signal(_)
            | WaitingFor::UIResult { .. }
    )
}

fn map_runtime_waiting(waiting_reason: &WaitingReason) -> WaitingFor {
    match waiting_reason {
        WaitingReason::None => WaitingFor::Nothing,
        WaitingReason::WaitForClick => WaitingFor::Click,
        WaitingReason::WaitForChoice { .. } => WaitingFor::Choice,
        WaitingReason::WaitForTime(duration) => WaitingFor::Time {
            remaining_ms: duration.as_millis() as u64,
        },
        WaitingReason::WaitForSignal(signal_id) => {
            let kind = match signal_id.as_str() {
                vn_runtime::command::SIGNAL_SCENE_TRANSITION => SignalKind::SceneTransition,
                vn_runtime::command::SIGNAL_TITLE_CARD => SignalKind::TitleCard,
                vn_runtime::command::SIGNAL_SCENE_EFFECT => SignalKind::SceneEffect,
                vn_runtime::command::SIGNAL_CUTSCENE => SignalKind::Cutscene,
                other => {
                    warn!(signal = other, "未知 signal ID，回退为 SceneEffect");
                    SignalKind::SceneEffect
                }
            };
            WaitingFor::Signal(kind)
        }
        WaitingReason::WaitForUIResult { key, .. } => WaitingFor::UIResult { key: key.clone() },
    }
}

fn parse_saved_position(name: &str) -> Position {
    match name {
        "Left" => Position::Left,
        "Right" => Position::Right,
        "Center" => Position::Center,
        "NearLeft" => Position::NearLeft,
        "NearRight" => Position::NearRight,
        "NearMiddle" => Position::NearMiddle,
        "FarLeft" => Position::FarLeft,
        "FarRight" => Position::FarRight,
        "FarMiddle" => Position::FarMiddle,
        _ => Position::Center,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("ring_host_dioxus_{name}_{suffix}"))
    }

    fn make_state_with_services(
        script_path: &str,
        script_content: &str,
    ) -> (AppStateInner, PathBuf) {
        let root = unique_temp_dir("state");
        let assets_dir = root.join("assets");
        let saves_dir = root.join("saves");
        std::fs::create_dir_all(assets_dir.join("scripts")).unwrap();
        std::fs::create_dir_all(&saves_dir).unwrap();
        std::fs::write(assets_dir.join(script_path), script_content).unwrap();

        let mut config = AppConfig::default();
        config.assets_root = assets_dir.clone();
        config.saves_dir = saves_dir.clone();
        config.start_script_path = script_path.to_string();

        let mut inner = AppStateInner::new();
        inner.persistent_store = PersistentStore::load(&saves_dir);
        inner.services = Some(Services {
            audio: AudioManager::new(),
            resources: ResourceManager::new(&assets_dir),
            saves: SaveManager::new(&saves_dir),
            config,
            manifest: crate::manifest::Manifest::with_defaults(),
        });
        (inner, root)
    }

    #[test]
    fn frontend_owner_reclaim_invalidates_stale_token() {
        let mut inner = AppStateInner::new();
        let first = inner.frontend_connected(Some("first".to_string()));
        assert!(inner.assert_owner(&first.client_token).is_ok());

        let second = inner.frontend_connected(Some("second".to_string()));
        assert!(inner.assert_owner(&first.client_token).is_err());
        assert!(inner.assert_owner(&second.client_token).is_ok());
    }

    #[test]
    fn overlay_screen_blocks_progression_and_resets_playback() {
        let mut inner = AppStateInner::new();
        inner.set_host_screen(HostScreen::InGame);
        inner.set_playback_mode(PlaybackMode::Auto);
        inner.waiting = WaitingFor::Time {
            remaining_ms: 1_000,
        };

        inner.set_host_screen(HostScreen::Save);
        inner.process_tick(1.0);

        assert_eq!(inner.playback_mode, PlaybackMode::Normal);
        assert_eq!(
            inner.waiting,
            WaitingFor::Time {
                remaining_ms: 1_000
            }
        );
    }

    #[test]
    fn return_to_title_with_save_continue_writes_continue_file() {
        let script = "changeBG <img src=\"../backgrounds/entry.png\" />\n";
        let (mut inner, root) = make_state_with_services("scripts/scene.md", script);

        inner.init_game_from_resource("scripts/scene.md").unwrap();
        inner.return_to_title(true);

        assert!(inner.services().saves.has_continue());

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn restore_from_save_keeps_saved_render_snapshot_without_entry_tick() {
        let script = "changeBG <img src=\"../backgrounds/entry.png\" />\n";
        let (mut inner, root) = make_state_with_services("scripts/scene.md", script);

        let mut runtime_state = vn_runtime::state::RuntimeState::new("scene");
        runtime_state.position.set_path("scripts/scene.md");

        let save_data = vn_runtime::SaveData::new(1, runtime_state)
            .with_render(vn_runtime::RenderSnapshot {
                background: Some("backgrounds/saved.png".to_string()),
                characters: Vec::new(),
            })
            .with_history(vn_runtime::History::new());

        inner.restore_from_save(save_data).unwrap();

        assert_eq!(
            inner.render_state.current_background.as_deref(),
            Some("backgrounds/saved.png")
        );
        assert_eq!(inner.host_screen, HostScreen::InGame);

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn build_save_data_uses_snapshot_boundary_while_waiting_for_choice() {
        let script = r#"
："选择前。"
| 选择 |        |
| ---- | ------ |
| 选项A | label_a |
| 选项B | label_b |
**label_a**
："选了A。"
**label_b**
："选了B。"
"#;
        let (mut inner, root) = make_state_with_services("scripts/choice.md", script);

        inner.init_game_from_resource("scripts/choice.md").unwrap();
        inner.render_state.complete_typewriter();
        inner.process_click();
        inner.process_tick(0.0);

        assert_eq!(inner.waiting, WaitingFor::Choice);
        assert!(inner.render_state.choices.is_some());
        assert!(inner.snapshot_stack.last().is_some());

        let save_data = inner.build_save_data(1).unwrap();
        assert!(matches!(
            save_data.runtime_state.waiting,
            WaitingReason::WaitForClick
        ));

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn build_save_data_uses_snapshot_boundary_while_waiting_for_ui_result() {
        let script = r#"
："第二部分：测试地图语法糖 showMap。"
showMap "demo_world" as $destination

if $destination == "town"
  ："你通过 showMap 选择了小镇。"
else
  ："你通过 showMap 选择了其他地点。"
endif
"#;
        let (mut inner, root) = make_state_with_services("scripts/ui_save.md", script);

        inner.init_game_from_resource("scripts/ui_save.md").unwrap();
        inner.render_state.complete_typewriter();
        inner.process_click();
        inner.run_script_tick();

        assert_eq!(
            inner.waiting,
            WaitingFor::UIResult {
                key: "show_map".to_string()
            }
        );
        assert!(inner.render_state.active_ui_mode.is_some());
        assert!(inner.snapshot_stack.last().is_some());

        let save_data = inner.build_save_data(1).unwrap();
        assert!(matches!(
            save_data.runtime_state.waiting,
            WaitingReason::WaitForClick
        ));

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn restore_from_save_normalizes_legacy_ui_wait_to_click() {
        let script = "：\"恢复等待态。\"\n";
        let (mut inner, root) = make_state_with_services("scripts/restore.md", script);

        let mut runtime_state = vn_runtime::state::RuntimeState::new("restore");
        runtime_state.position.set_path("scripts/restore.md");
        runtime_state.wait(WaitingReason::ui_result("show_map", "destination"));

        let save_data = vn_runtime::SaveData::new(1, runtime_state)
            .with_render(vn_runtime::RenderSnapshot {
                background: Some("backgrounds/saved.png".to_string()),
                characters: Vec::new(),
            })
            .with_history(vn_runtime::History::new());

        inner.restore_from_save(save_data).unwrap();

        assert_eq!(inner.waiting, WaitingFor::Click);
        assert!(matches!(
            inner
                .runtime
                .as_ref()
                .expect("runtime should be restored")
                .waiting(),
            WaitingReason::WaitForClick
        ));
        assert!(inner.render_state.active_ui_mode.is_none());

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn cutscene_ducks_and_finish_restores_bgm_volume() {
        let (mut inner, root) = make_state_with_services("scripts/cutscene.md", "");

        inner
            .services_mut()
            .audio
            .play_bgm("audio/theme.ogg", true, None);
        inner.sync_audio(0.0);
        let base_volume = inner
            .render_state
            .audio
            .bgm
            .as_ref()
            .expect("bgm should exist before cutscene")
            .volume;

        inner.apply_runtime_tick_output(
            vec![Command::Cutscene {
                path: "video/opening.webm".to_string(),
            }],
            WaitingReason::signal("cutscene"),
        );
        inner.sync_audio(0.5);

        let ducked_volume = inner
            .render_state
            .audio
            .bgm
            .as_ref()
            .expect("bgm should still exist during cutscene")
            .volume;
        assert!(inner.render_state.cutscene.is_some());
        assert!(ducked_volume < base_volume);

        inner.finish_cutscene();
        inner.sync_audio(0.5);

        let restored_volume = inner
            .render_state
            .audio
            .bgm
            .as_ref()
            .expect("bgm should remain after cutscene")
            .volume;
        assert!(inner.render_state.cutscene.is_none());
        assert!((restored_volume - base_volume).abs() < f32::EPSILON);

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn handle_ui_result_applies_follow_up_dialogue_without_skipping() {
        let script = r#"
："第二部分：测试地图语法糖 showMap。"
showMap "demo_world" as $destination

if $destination == "town"
  ："你通过 showMap 选择了小镇。"
else
  ："你通过 showMap 选择了其他地点。"
endif

："第三部分：直接测试底层 requestUI。"
"#;
        let (mut inner, root) = make_state_with_services("scripts/ui.md", script);

        inner.init_game_from_resource("scripts/ui.md").unwrap();
        inner.render_state.complete_typewriter();
        inner.process_click();
        inner.run_script_tick();

        let active_mode = inner
            .render_state
            .active_ui_mode
            .as_ref()
            .expect("showMap should activate a UI mode");
        assert_eq!(active_mode.mode, "show_map");
        assert_eq!(
            inner.waiting,
            WaitingFor::UIResult {
                key: "show_map".to_string()
            }
        );

        inner
            .handle_ui_result(
                "show_map".to_string(),
                serde_json::Value::String("town".to_string()),
            )
            .unwrap();

        assert!(inner.render_state.active_ui_mode.is_none());
        assert_eq!(inner.waiting, WaitingFor::Click);
        assert_eq!(
            inner
                .render_state
                .dialogue
                .as_ref()
                .expect("follow-up dialogue should be rendered")
                .content,
            "你通过 showMap 选择了小镇。"
        );

        std::fs::remove_dir_all(root).ok();
    }

    #[test]
    fn script_end_returns_to_title_screen() {
        let script = "：\"脚本结束测试。\"\n";
        let (mut inner, root) = make_state_with_services("scripts/end.md", script);

        inner.init_game_from_resource("scripts/end.md").unwrap();
        inner.render_state.complete_typewriter();
        inner.process_click();
        inner.process_tick(0.0);

        assert_eq!(inner.host_screen, HostScreen::Title);
        assert_eq!(inner.render_state.host_screen, HostScreen::Title);
        assert!(inner.runtime.is_none());
        assert!(inner.render_state.dialogue.is_none());

        std::fs::remove_dir_all(root).ok();
    }
}
