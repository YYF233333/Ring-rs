//! 音频命令处理
//!
//! 阶段 27：函数签名从 `&mut AppState` 改为 `(&mut CoreSystems, &AppConfig)`，
//! 不再依赖完整的应用状态。

use crate::command_executor::AudioCommand;
use crate::config::AppConfig;
use crate::resources::LogicalPath;

use super::super::CoreSystems;
use tracing::error;

/// 处理音频命令
///
/// 始终通过 ResourceManager 读取音频字节并缓存到 AudioManager，
/// 不再区分 FS/ZIP 模式。
pub fn handle_audio_command(core: &mut CoreSystems, _config: &AppConfig) {
    let audio_cmd = core.command_executor.last_output.audio_command.clone();

    if let Some(cmd) = audio_cmd {
        let path_to_cache = match &cmd {
            AudioCommand::PlayBgm { path, .. } => Some(path.clone()),
            AudioCommand::PlaySfx { path } => Some(path.clone()),
            AudioCommand::StopBgm { .. } | AudioCommand::BgmDuck | AudioCommand::BgmUnduck => None,
        };

        if let Some(path) = path_to_cache {
            let logical_path = LogicalPath::new(&path);
            match core.resource_manager.read_bytes(&logical_path) {
                Ok(bytes) => {
                    if let Some(ref mut audio) = core.audio_manager {
                        audio.cache_audio_bytes(logical_path.as_str(), bytes);
                    }
                }
                Err(e) => {
                    error!(path = %logical_path, error = %e, "Cannot read audio file");
                    return;
                }
            }
        }

        if let Some(ref mut audio_manager) = core.audio_manager {
            match cmd {
                AudioCommand::PlayBgm {
                    path,
                    looping,
                    fade_in: _,
                } => {
                    const CROSSFADE_DURATION: f32 = 1.0;
                    if audio_manager.is_bgm_playing() {
                        audio_manager.crossfade_bgm(&path, looping, CROSSFADE_DURATION);
                    } else {
                        audio_manager.play_bgm(&path, looping, Some(CROSSFADE_DURATION));
                    }
                }
                AudioCommand::StopBgm { fade_out } => {
                    audio_manager.stop_bgm(fade_out);
                }
                AudioCommand::BgmDuck => {
                    audio_manager.duck();
                }
                AudioCommand::BgmUnduck => {
                    audio_manager.unduck();
                }
                AudioCommand::PlaySfx { path } => {
                    audio_manager.play_sfx(&path);
                }
            }
        }
    }
}
