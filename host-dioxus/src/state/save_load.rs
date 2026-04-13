use tracing::warn;
use vn_runtime::command::Position;
use vn_runtime::state::WaitingReason;

use crate::error::{HostError, HostResult};
use crate::render_state::RenderState;

use super::*;

pub fn waiting_requires_snapshot_fallback(waiting: &WaitingFor) -> bool {
    matches!(
        waiting,
        WaitingFor::Choice
            | WaitingFor::Cutscene
            | WaitingFor::Signal(_)
            | WaitingFor::UIResult { .. }
    )
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

impl AppStateInner {
    pub(super) fn build_save_data(&self, slot: u32) -> HostResult<vn_runtime::SaveData> {
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

    pub(super) fn apply_render_snapshot(&mut self, render: &vn_runtime::RenderSnapshot) {
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

    pub(super) fn apply_audio_state(&mut self, audio: &vn_runtime::AudioState) {
        let manager = &mut self.services_mut().audio;
        match &audio.current_bgm {
            Some(path) => manager.play_bgm(path, audio.bgm_looping, None),
            None => manager.stop_bgm(None),
        }
        self.sync_audio(0.0);
    }

    pub(super) fn normalize_restored_waiting(&mut self) {
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
}
