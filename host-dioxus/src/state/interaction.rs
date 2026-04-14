use tracing::warn;
use vn_runtime::RuntimeInput;
use vn_runtime::command::Command;
use vn_runtime::state::WaitingReason;

use crate::command_executor::{
    AudioCommand, BatchOutput, ExecuteResult, SceneEffectKind, SceneEffectRequest,
};
use crate::error::{HostError, HostResult};
use crate::render_state::{CutsceneState, HostScreen, PlaybackMode};

use super::*;

impl AppStateInner {
    /// 处理用户点击
    pub fn process_click(&mut self) {
        if !self.host_screen.allows_progression() {
            return;
        }
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

    /// 同时清除 host 侧和 runtime 侧的等待状态
    pub(super) fn clear_wait(&mut self) {
        if let Some(rt) = self.runtime.as_mut() {
            rt.state_mut().clear_wait();
        }
        self.waiting = WaitingFor::Nothing;
    }

    pub(super) fn clear_click_wait(&mut self) {
        self.clear_wait();
    }

    /// 恢复操作（restore_from_save / restore_snapshot）的公共收尾逻辑
    pub(super) fn finish_restore(&mut self) {
        self.script_finished = false;
        self.typewriter_timer = 0.0;
        self.auto_timer = 0.0;
        self.playback_mode = PlaybackMode::Normal;
        self.set_host_screen(HostScreen::InGame);
        self.project_render_state();
    }

    /// 捕获当前状态快照（用于 Backspace 回退）
    pub(super) fn capture_snapshot(&mut self) {
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
        match tick_result {
            Ok((commands, waiting_reason)) => {
                self.apply_runtime_tick_output(commands, waiting_reason)
            }
            Err(error) => {
                warn!(%error, "choice selection tick failed");
                self.script_finished = true;
            }
        }
    }

    pub(super) fn apply_runtime_tick_output(
        &mut self,
        commands: Vec<Command>,
        waiting_reason: WaitingReason,
    ) {
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
            self.return_to_title(false);
            return;
        }

        self.project_render_state();
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
            }
        }
    }

    /// 追加对话历史
    pub fn push_history(&mut self, speaker: Option<String>, text: String) {
        self.history.insert(0, HistoryEntry { speaker, text });
    }

    pub fn set_host_screen(&mut self, screen: HostScreen) {
        if self.host_screen == screen {
            self.project_render_state();
            return;
        }

        self.host_screen = screen;

        if !self.host_screen.allows_progression() && self.playback_mode != PlaybackMode::Normal {
            self.playback_mode = PlaybackMode::Normal;
            self.auto_timer = 0.0;
        }

        self.project_render_state();
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

        self.playback_mode = next_mode;
        self.auto_timer = 0.0;
        self.project_render_state();
    }

    /// 分派音频命令到 AudioManager
    pub(super) fn dispatch_audio_command(&mut self, cmd: AudioCommand) {
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
    pub(super) fn apply_scene_effect(&mut self, req: SceneEffectRequest) {
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
}
