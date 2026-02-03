//! # 音频相关命令执行
//!
//! 处理 PlayBgm、StopBgm、PlaySfx 命令。

use super::CommandExecutor;
use super::types::{AudioCommand, ExecuteResult};
use tracing::debug;

impl CommandExecutor {
    /// 执行 PlayBgm
    pub(super) fn execute_play_bgm(&mut self, path: &str, looping: bool) -> ExecuteResult {
        // 记录音频命令，由 main.rs 处理实际播放
        self.last_output.audio_command = Some(AudioCommand::PlayBgm {
            path: path.to_string(),
            looping,
            fade_in: Some(0.5), // 默认 0.5 秒淡入
        });
        debug!(path = %path, looping = looping, "播放 BGM");
        ExecuteResult::Ok
    }

    /// 执行 StopBgm
    pub(super) fn execute_stop_bgm(&mut self, fade_out: Option<f64>) -> ExecuteResult {
        // 记录音频命令
        self.last_output.audio_command = Some(AudioCommand::StopBgm {
            fade_out: fade_out.map(|d| d as f32),
        });
        debug!(fade_out = ?fade_out, "停止 BGM");
        ExecuteResult::Ok
    }

    /// 执行 PlaySfx
    pub(super) fn execute_play_sfx(&mut self, path: &str) -> ExecuteResult {
        // 记录音频命令
        self.last_output.audio_command = Some(AudioCommand::PlaySfx {
            path: path.to_string(),
        });
        debug!(path = %path, "播放音效");
        ExecuteResult::Ok
    }
}
