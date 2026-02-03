//! 音频命令处理

use crate::AssetSourceType;
use crate::command_executor::AudioCommand;
use crate::resources::normalize_logical_path;

use super::super::AppState;
use tracing::error;

/// 处理音频命令
pub fn handle_audio_command(app_state: &mut AppState) {
    let audio_cmd = app_state.command_executor.last_output.audio_command.clone();

    if let Some(cmd) = audio_cmd {
        // ZIP 模式下需要先缓存音频字节
        if let AssetSourceType::Zip = app_state.config.asset_source {
            let path_to_cache = match &cmd {
                AudioCommand::PlayBgm { path, .. } => Some(path.clone()),
                AudioCommand::PlaySfx { path } => Some(path.clone()),
                AudioCommand::StopBgm { .. } => None,
            };

            if let Some(path) = path_to_cache {
                let logical_path = normalize_logical_path(&path);
                // 读取音频字节并缓存
                match app_state.resource_manager.read_bytes(&logical_path) {
                    Ok(bytes) => {
                        if let Some(ref mut audio) = app_state.audio_manager {
                            audio.cache_audio_bytes(&logical_path, bytes);
                        }
                    }
                    Err(e) => {
                        error!(path = %logical_path, error = %e, "无法读取音频文件");
                        return;
                    }
                }
            }
        }

        if let Some(ref mut audio_manager) = app_state.audio_manager {
            match cmd {
                AudioCommand::PlayBgm {
                    path,
                    looping,
                    fade_in: _,
                } => {
                    // BGM 切换自带交叉淡化效果（规范要求）
                    // 如果当前有 BGM 在播放，使用交叉淡化；否则直接播放（带淡入）
                    const CROSSFADE_DURATION: f32 = 1.0; // 交叉淡化时长
                    if audio_manager.is_bgm_playing() {
                        audio_manager.crossfade_bgm(&path, looping, CROSSFADE_DURATION);
                    } else {
                        audio_manager.play_bgm(&path, looping, Some(CROSSFADE_DURATION));
                    }
                }
                AudioCommand::StopBgm { fade_out } => {
                    audio_manager.stop_bgm(fade_out);
                }
                AudioCommand::PlaySfx { path } => {
                    audio_manager.play_sfx(&path);
                }
            }
        }
    }
}
