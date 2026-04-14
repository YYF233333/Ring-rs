use std::collections::HashSet;

use tracing::{error, warn};
use vn_runtime::history::HistoryEvent;
use vn_runtime::state::{RuntimeState, WaitingReason};
use vn_runtime::{Parser, Script, ScriptNode, VNRuntime};

use crate::error::HostResult;
use crate::render_state::{HostScreen, PlaybackMode};
use crate::resources::{LogicalPath, ResourceManager};

use super::*;

impl AppStateInner {
    /// 将 PersistentStore 中的变量注入到当前 runtime。
    pub(super) fn inject_persistent_vars(&mut self) {
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
    pub(super) fn reset_session(&mut self) {
        if let Some(svc) = self.services.as_mut() {
            svc.audio.stop_bgm(None);
        }
        self.runtime = None;
        self.render_state = crate::render_state::RenderState::new();
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
        self.project_render_state();
    }

    /// 解析脚本并初始化运行时
    pub fn init_game(&mut self, script_content: &str) -> HostResult<()> {
        let mut parser = Parser::new();
        let script = parser.parse("main", script_content)?;
        self.reset_session();
        self.runtime = Some(VNRuntime::new(script));
        self.inject_persistent_vars();
        self.set_host_screen(HostScreen::InGame);
        self.run_script_tick();
        Ok(())
    }

    /// 通过 ResourceManager 构建运行时，但不执行首帧。
    pub(super) fn build_runtime_from_resource(&self, script_path: &str) -> HostResult<VNRuntime> {
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

    pub(super) fn start_runtime(
        &mut self,
        mut runtime: VNRuntime,
        _script_path: &str,
        start_label: Option<&str>,
    ) -> HostResult<()> {
        use crate::error::HostError;
        if let Some(label) = start_label {
            let target = runtime
                .find_label(label)
                .ok_or_else(|| HostError::InvalidInput(format!("标签未找到: {label}")))?;
            runtime.state_mut().position.jump_to(target);
        }

        self.reset_session();
        self.runtime = Some(runtime);
        self.inject_persistent_vars();
        self.set_host_screen(HostScreen::InGame);
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

    /// 重置到标题画面状态
    pub fn return_to_title(&mut self, save_continue: bool) {
        if save_continue {
            if let Err(error) = self.save_continue() {
                warn!(%error, "返回标题时保存 continue 失败");
            }
        } else if let Err(error) = self.delete_continue() {
            warn!(%error, "返回标题时删除 continue 失败");
        }
        if let Err(e) = self.persistent_store.save() {
            warn!("返回标题时持久化变量保存失败: {e}");
        }
        self.reset_session();
        self.set_host_screen(HostScreen::Title);
    }

    /// 执行数据驱动的 ActionDef（从 screens.json 按钮定义映射到应用操作）
    pub fn execute_action(&mut self, action: &crate::screen_defs::ActionDef) {
        use crate::screen_defs::ActionDef;
        match action {
            ActionDef::StartGame => {
                let path = self.services().config.start_script_path.clone();
                if let Err(e) = self.delete_continue() {
                    warn!("删除 continue 失败: {e}");
                }
                if let Err(e) = self.init_game_from_resource(&path) {
                    error!("开始游戏失败: {e}");
                }
            }
            ActionDef::StartAtLabel(label) => {
                let path = self.services().config.start_script_path.clone();
                let label = label.clone();
                if let Err(e) = self.delete_continue() {
                    warn!("删除 continue 失败: {e}");
                }
                if let Err(e) = self.init_game_from_resource_at_label(&path, &label) {
                    error!("从标签 {label} 开始游戏失败: {e}");
                }
            }
            ActionDef::ContinueGame => match self.services().saves.load_continue() {
                Ok(save_data) => {
                    if let Err(e) = self.restore_from_save(save_data) {
                        error!("继续游戏失败: {e}");
                    }
                }
                Err(e) => warn!("加载 continue 存档失败: {e}"),
            },
            ActionDef::OpenLoad => self.set_host_screen(HostScreen::Load),
            ActionDef::OpenSave => self.set_host_screen(HostScreen::Save),
            ActionDef::NavigateSettings => self.set_host_screen(HostScreen::Settings),
            ActionDef::NavigateHistory => self.set_host_screen(HostScreen::History),
            ActionDef::ReplaceSettings => self.set_host_screen(HostScreen::Settings),
            ActionDef::ReplaceHistory => self.set_host_screen(HostScreen::History),
            ActionDef::QuickSave => {
                if let Err(e) = self.save_to_slot(55) {
                    error!("快存失败: {e}");
                }
            }
            ActionDef::QuickLoad => match self.services().saves.load(55) {
                Ok(save_data) => {
                    if let Err(e) = self.restore_from_save(save_data) {
                        error!("快读失败: {e}");
                    }
                }
                Err(e) => warn!("快读加载失败: {e}"),
            },
            ActionDef::ToggleSkip => {
                let mode = if self.playback_mode == PlaybackMode::Skip {
                    PlaybackMode::Normal
                } else {
                    PlaybackMode::Skip
                };
                self.set_playback_mode(mode);
            }
            ActionDef::ToggleAuto => {
                let mode = if self.playback_mode == PlaybackMode::Auto {
                    PlaybackMode::Normal
                } else {
                    PlaybackMode::Auto
                };
                self.set_playback_mode(mode);
            }
            ActionDef::GoBack => self.set_host_screen(HostScreen::InGame),
            ActionDef::ReturnToTitle => self.return_to_title(true),
            ActionDef::ReturnToGame => self.set_host_screen(HostScreen::InGame),
            ActionDef::Exit => {
                // 由前端层处理窗口关闭
                tracing::info!("Exit action received — 由前端处理窗口关闭");
            }
        }
    }

    /// 构建条件求值上下文
    pub fn condition_context(&self) -> crate::screen_defs::ConditionContext<'_> {
        let has_continue = self
            .services
            .as_ref()
            .is_some_and(|svc| svc.saves.has_continue());
        crate::screen_defs::ConditionContext {
            has_continue,
            persistent: &self.persistent_store,
        }
    }
}

pub(crate) fn preload_called_scripts(
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

pub(crate) fn load_call_stack_scripts(
    runtime: &mut VNRuntime,
    resources: &ResourceManager,
    runtime_state: &RuntimeState,
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

pub(crate) fn host_history_from_runtime(
    history: &vn_runtime::history::History,
) -> Vec<HistoryEntry> {
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

pub(crate) fn map_runtime_waiting(waiting_reason: &WaitingReason) -> WaitingFor {
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
