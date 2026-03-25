use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use vn_runtime::state::{VarValue, WaitingReason};
use vn_runtime::{Parser, RuntimeInput, Script, ScriptNode, VNRuntime};

use crate::audio::AudioManager;
use crate::command_executor::{
    AudioCommand, BatchOutput, CommandExecutor, ExecuteResult, SceneEffectKind,
    SceneEffectRequest,
};
use crate::config::AppConfig;
use crate::render_state::{CutsceneState, PlaybackMode, RenderState, SceneTransitionPhaseState};
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

/// Tauri 托管的全局应用状态
pub struct AppState {
    pub inner: std::sync::Arc<std::sync::Mutex<AppStateInner>>,
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
    pub fn save(&self) -> Result<(), String> {
        if !self.saves_dir.exists() {
            fs::create_dir_all(&self.saves_dir).map_err(|e| format!("无法创建存档目录: {e}"))?;
        }
        let path = self.saves_dir.join(PERSISTENT_FILE);
        let content = serde_json::to_string_pretty(&self.variables)
            .map_err(|e| format!("持久化变量序列化失败: {e}"))?;
        fs::write(&path, content).map_err(|e| format!("持久化变量写入失败: {e}"))?;
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
    pub history_len: usize,
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
}

/// 应用状态内部结构（被 Mutex 保护）
pub struct AppStateInner {
    pub runtime: Option<VNRuntime>,
    pub command_executor: CommandExecutor,
    pub render_state: RenderState,
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
}

/// Shake 动画的运行时状态
struct ShakeAnimation {
    amplitude_x: f32,
    amplitude_y: f32,
    duration: f32,
    elapsed: f32,
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
    /// 等待 Host 侧事件完成（场景过渡、标题卡片等），对应 Runtime 的 WaitForSignal
    Signal(String),
}

impl AppStateInner {
    pub fn new() -> Self {
        Self {
            runtime: None,
            command_executor: CommandExecutor::new(),
            render_state: RenderState::new(),
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
    }

    /// 解析脚本并初始化运行时
    pub fn init_game(&mut self, script_content: &str) -> Result<(), String> {
        self.reset_session();

        let mut parser = Parser::new();
        let script = parser
            .parse("main", script_content)
            .map_err(|e| format!("脚本解析失败: {e}"))?;
        self.runtime = Some(VNRuntime::new(script));

        self.inject_persistent_vars();
        self.run_script_tick();
        Ok(())
    }

    /// 通过 ResourceManager 读取脚本并初始化运行时
    ///
    /// 除入口脚本外，递归预加载所有 `callScript` 引用的子脚本，
    /// 保证运行时执行到跨文件调用时不会因脚本未注册而失败。
    pub fn init_game_from_resource(&mut self, script_path: &str) -> Result<(), String> {
        let rm = &self.services().resources;

        let logical = LogicalPath::new(script_path);
        let content = rm
            .read_text(&logical)
            .map_err(|e| format!("读取脚本失败: {e}"))?;

        let script_id = logical.file_stem().to_string();
        let base_dir = logical.parent_dir().to_string();

        let mut parser = Parser::new();
        let script = parser
            .parse_with_base_path(&script_id, &content, &base_dir)
            .map_err(|e| format!("脚本解析失败: {e}"))?;

        for w in parser.warnings() {
            warn!(warning = %w, "脚本解析警告");
        }

        let mut runtime = VNRuntime::new(script.clone());
        let normalized = logical.as_str().to_string();
        runtime.register_script(&normalized, script.clone());
        runtime.state_mut().position.set_path(&normalized);

        let mut visited = HashSet::new();
        visited.insert(normalized);
        Self::preload_called_scripts(&mut runtime, rm, &script, &mut visited);

        self.reset_session();
        self.runtime = Some(runtime);

        self.inject_persistent_vars();
        self.run_script_tick();
        Ok(())
    }

    /// 递归预加载 `callScript` 引用的所有子脚本
    fn preload_called_scripts(
        runtime: &mut VNRuntime,
        rm: &ResourceManager,
        script: &Script,
        visited: &mut HashSet<String>,
    ) {
        for node in Self::collect_call_nodes(&script.nodes) {
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
            Self::preload_called_scripts(runtime, rm, &child_script, visited);
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
                        result.extend(Self::collect_call_nodes(&branch.body));
                    }
                }
                _ => {}
            }
        }
        result
    }

    /// 每帧调用，推进打字机和计时器
    pub fn process_tick(&mut self, dt: f32) {
        self.advance_playback_mode(dt);
        self.update_animations(dt);
        self.resolve_waits(dt);

        if self.waiting == WaitingFor::Nothing && !self.script_finished {
            self.run_script_tick();
        }

        self.advance_typewriter(dt);
        self.sync_audio(dt);
        self.render_state.playback_mode = self.playback_mode.clone();
    }

    /// Skip 模式立即推进 + Auto 模式计时推进
    fn advance_playback_mode(&mut self, dt: f32) {
        if self.playback_mode == PlaybackMode::Skip && self.waiting == WaitingFor::Click {
            self.render_state.complete_typewriter();
            self.clear_click_wait();
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
        if let WaitingFor::Signal(ref signal_id) = self.waiting {
            let resolved = match signal_id.as_str() {
                "scene_transition" => self
                    .render_state
                    .scene_transition
                    .as_ref()
                    .is_none_or(|st| st.phase == SceneTransitionPhaseState::Completed),
                "title_card" => self.render_state.title_card.is_none(),
                "scene_effect" => !self.scene_effect_active,
                "cutscene" => self.render_state.cutscene.is_none(),
                _ => false,
            };
            if resolved {
                if let Some(rt) = self.runtime.as_mut() {
                    rt.state_mut().clear_wait();
                }
                self.waiting = WaitingFor::Nothing;
            }
        }

        if let WaitingFor::Time { remaining_ms } = &self.waiting {
            let elapsed_ms = (dt * 1000.0) as u64;
            if elapsed_ms >= *remaining_ms {
                if let Some(rt) = self.runtime.as_mut() {
                    rt.state_mut().clear_wait();
                }
                self.waiting = WaitingFor::Nothing;
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
        self.auto_timer = 0.0;

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

    /// 清除 Click 等待——同时清除 host 侧和 runtime 侧的等待状态
    fn clear_click_wait(&mut self) {
        if let Some(rt) = self.runtime.as_mut() {
            rt.state_mut().clear_wait();
        }
        self.waiting = WaitingFor::Nothing;
    }

    /// 捕获当前状态快照（用于 Backspace 回退）
    fn capture_snapshot(&mut self) {
        let Some(rt) = self.runtime.as_ref() else {
            return;
        };
        let snapshot = Snapshot {
            render_state: self.render_state.clone(),
            runtime_state: rt.state().clone(),
            history_len: self.history.len(),
        };
        self.snapshot_stack.push(snapshot);
    }

    /// 恢复到最近的快照
    pub fn restore_snapshot(&mut self) -> bool {
        let Some(snapshot) = self.snapshot_stack.pop() else {
            return false;
        };
        if let Some(rt) = self.runtime.as_mut() {
            rt.restore_state(snapshot.runtime_state);
        }
        self.render_state = snapshot.render_state;
        self.history.truncate(snapshot.history_len);
        self.waiting = WaitingFor::Click;
        self.typewriter_timer = 0.0;
        self.auto_timer = 0.0;
        true
    }

    /// 结束视频过场
    pub fn finish_cutscene(&mut self) {
        self.render_state.cutscene = None;
        if self.waiting == WaitingFor::Cutscene {
            self.waiting = WaitingFor::Nothing;
        }
    }

    /// 处理用户选择
    pub fn process_choose(&mut self, index: usize) {
        if self.waiting != WaitingFor::Choice {
            return;
        }
        if let Some(rt) = self.runtime.as_mut() {
            let input = RuntimeInput::choice(index);
            let _ = rt.tick(Some(input));
        }
        self.render_state.clear_choices();
        self.waiting = WaitingFor::Nothing;
    }

    /// 调用 runtime.tick() 并执行产出的 commands
    pub(crate) fn run_script_tick(&mut self) {
        let Some(rt) = self.runtime.as_mut() else {
            return;
        };

        match rt.tick(None) {
            Ok((commands, waiting_reason)) => {
                let BatchOutput {
                    result,
                    audio_commands,
                    scene_effect_request,
                } = self
                    .command_executor
                    .execute_batch(&commands, &mut self.render_state);

                if let Some(ref d) = self.render_state.dialogue
                    && (d.visible_chars == 0 || !d.content.is_empty()) {
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
                    self.return_to_title();
                    return;
                }

                if let ExecuteResult::RequestUI { key, mode } = &result {
                    warn!(
                        key = %key, mode = %mode,
                        "RequestUI 暂不支持，回传空字符串降级"
                    );
                    if let Some(rt) = self.runtime.as_mut() {
                        let input = RuntimeInput::UIResult {
                            key: key.clone(),
                            value: VarValue::String(String::new()),
                        };
                        let _ = rt.tick(Some(input));
                    }
                    return;
                }

                if let ExecuteResult::WaitForCutscene { video_path } = &result {
                    self.render_state.cutscene = Some(CutsceneState {
                        video_path: video_path.clone(),
                        is_playing: true,
                    });
                }

                // 用 Runtime 的 waiting_reason（权威来源）映射 Host 等待状态
                match &waiting_reason {
                    WaitingReason::None => {}
                    WaitingReason::WaitForClick => {
                        self.waiting = WaitingFor::Click;
                    }
                    WaitingReason::WaitForChoice { .. } => {
                        self.waiting = WaitingFor::Choice;
                    }
                    WaitingReason::WaitForTime(duration) => {
                        self.waiting = WaitingFor::Time {
                            remaining_ms: duration.as_millis() as u64,
                        };
                    }
                    WaitingReason::WaitForSignal(signal_id) => {
                        self.waiting = WaitingFor::Signal(signal_id.as_str().to_string());
                    }
                    WaitingReason::WaitForUIResult { key, .. } => {
                        warn!(
                            key = %key,
                            "WaitForUIResult 暂不支持，回传空字符串降级"
                        );
                        if let Some(rt) = self.runtime.as_mut() {
                            let input = RuntimeInput::UIResult {
                                key: key.clone(),
                                value: VarValue::String(String::new()),
                            };
                            let _ = rt.tick(Some(input));
                        }
                    }
                }

                // 同步 runtime persistent 变量到 PersistentStore
                if let Some(rt) = self.runtime.as_ref() {
                    let pv = &rt.state().persistent_variables;
                    if !pv.is_empty() {
                        self.persistent_store.merge_from(pv);
                    }
                }

                if waiting_reason == WaitingReason::None && commands.is_empty() {
                    self.script_finished = true;
                }
            }
            Err(_) => {
                self.script_finished = true;
            }
        }
    }

    /// 追加对话历史
    pub fn push_history(&mut self, speaker: Option<String>, text: String) {
        self.history.insert(0, HistoryEntry { speaker, text });
    }

    /// 从存档恢复游戏状态
    ///
    /// 统一处理 `load_game` 和 `continue_game` 的恢复逻辑：
    /// 若 runtime 尚未初始化，根据存档中的 `script_path` 加载入口脚本并预加载子脚本。
    pub fn restore_from_save(&mut self, save_data: vn_runtime::SaveData) -> Result<(), String> {
        if self.runtime.is_none() {
            let path = if !save_data.runtime_state.position.script_path.is_empty() {
                &save_data.runtime_state.position.script_path
            } else {
                &save_data.runtime_state.position.script_id
            };
            self.init_game_from_resource(path)?;
        }

        let rt = self.runtime.as_mut().ok_or("游戏未启动")?;
        let svc = self
            .services
            .as_ref()
            .expect("invariant: services initialized in setup()");

        // 恢复 call_stack 中引用的脚本到 registry
        for frame in &save_data.runtime_state.call_stack {
            let frame_path = if !frame.script_path.is_empty() {
                &frame.script_path
            } else {
                &frame.script_id
            };
            let logical = LogicalPath::new(frame_path);
            if let Ok(content) = svc.resources.read_text(&logical) {
                let fid = logical.file_stem().to_string();
                let fbase = logical.parent_dir().to_string();
                let mut parser = Parser::new();
                if let Ok(s) = parser.parse_with_base_path(&fid, &content, &fbase) {
                    rt.register_script(frame_path, s);
                }
            }
        }

        rt.restore_state(save_data.runtime_state);
        rt.restore_history(save_data.history);

        self.render_state = RenderState::new();
        if let Some(bg) = save_data.render.background {
            self.render_state.set_background(bg);
        }
        self.waiting = WaitingFor::Nothing;
        self.typewriter_timer = 0.0;
        self.script_finished = false;
        self.run_script_tick();
        Ok(())
    }

    /// 重置到标题画面状态
    pub fn return_to_title(&mut self) {
        if let Err(e) = self.persistent_store.save() {
            warn!("返回标题时持久化变量保存失败: {e}");
        }
        self.reset_session();
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
            self.render_state.scene_effect.shake_offset_x =
                shake.amplitude_x * decay * phase.sin();
            self.render_state.scene_effect.shake_offset_y =
                shake.amplitude_y * decay * phase.cos();
        }
    }
}
