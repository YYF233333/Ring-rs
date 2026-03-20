//! 杂项指令解析：audio、sceneEffect、titleCard、cutscene

use crate::command::TransitionArg;
use crate::error::ParseError;
use crate::script::ast::ScriptNode;

use super::super::helpers::{extract_audio_src, parse_transition, parse_transition_args};
use super::Phase2Parser;

impl Phase2Parser {
    /// 解析 audio 标签
    ///
    /// 语法:
    /// - `<audio src="path/to/audio.mp3"></audio>` - SFX（播放一次）
    /// - `<audio src="path/to/audio.mp3"></audio> loop` - BGM（循环播放）
    pub(super) fn parse_audio(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let path = extract_audio_src(line).ok_or_else(|| ParseError::MissingParameter {
            line: line_number,
            command: "audio".to_string(),
            param: "音频路径 (<audio src=\"...\">)".to_string(),
        })?;

        let is_bgm = if let Some(close_tag_pos) = line.to_lowercase().find("</audio>") {
            let after_tag = &line[close_tag_pos + 8..];
            after_tag.to_lowercase().contains("loop") || after_tag.contains('\u{267E}')
        } else {
            false
        };

        Ok(Some(ScriptNode::PlayAudio {
            path: path.to_string(),
            is_bgm,
        }))
    }

    /// 解析场景效果命令
    ///
    /// 语法: `sceneEffect name` 或 `sceneEffect name(args...)`
    pub(super) fn parse_scene_effect(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("sceneEffect".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "sceneEffect".to_string(),
                param: "effect name".to_string(),
            })?;

        let effect = parse_transition(content).ok_or_else(|| ParseError::InvalidParameter {
            line: line_number,
            param: "effect".to_string(),
            message: format!("unable to parse scene effect: '{}'", content),
        })?;

        Ok(Some(ScriptNode::SceneEffect { effect }))
    }

    /// 解析章节字卡命令
    ///
    /// 语法: `titleCard "text" (duration: N)`
    pub(super) fn parse_title_card(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("titleCard".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "titleCard".to_string(),
                param: "text and duration".to_string(),
            })?;

        let quote_start = content
            .find('"')
            .ok_or_else(|| ParseError::InvalidParameter {
                line: line_number,
                param: "text".to_string(),
                message: "titleCard requires quoted text".to_string(),
            })?;
        let quote_end =
            content[quote_start + 1..]
                .find('"')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "text".to_string(),
                    message: "missing closing quote".to_string(),
                })?
                + quote_start
                + 1;
        let text = content[quote_start + 1..quote_end].to_string();

        let rest = content[quote_end + 1..].trim();
        let duration = if let Some(paren_start) = rest.find('(') {
            let paren_end = rest
                .rfind(')')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "duration".to_string(),
                    message: "missing closing parenthesis".to_string(),
                })?;
            let args_str = &rest[paren_start + 1..paren_end];
            let args =
                parse_transition_args(args_str).map_err(|e| ParseError::InvalidParameter {
                    line: line_number,
                    param: "duration".to_string(),
                    message: e,
                })?;

            args.iter()
                .find(|(k, _)| k.as_deref() == Some("duration"))
                .and_then(|(_, v)| match v {
                    TransitionArg::Number(n) => Some(*n),
                    _ => None,
                })
                .or_else(|| {
                    args.first().and_then(|(_, v)| match v {
                        TransitionArg::Number(n) => Some(*n),
                        _ => None,
                    })
                })
                .unwrap_or(1.0)
        } else {
            1.0
        };

        if duration <= 0.0 {
            return Err(ParseError::InvalidParameter {
                line: line_number,
                param: "duration".to_string(),
                message: format!("duration must be positive, got: {}", duration),
            });
        }

        Ok(Some(ScriptNode::TitleCard { text, duration }))
    }

    /// 解析视频过场命令
    ///
    /// 语法: `cutscene "path"`
    pub(super) fn parse_cutscene(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("cutscene".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "cutscene".to_string(),
                param: "path".to_string(),
            })?;

        let quote_start = content
            .find('"')
            .ok_or_else(|| ParseError::InvalidParameter {
                line: line_number,
                param: "path".to_string(),
                message: "cutscene requires a quoted path".to_string(),
            })?;
        let quote_end =
            content[quote_start + 1..]
                .find('"')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "path".to_string(),
                    message: "missing closing quote".to_string(),
                })?
                + quote_start
                + 1;
        let path = content[quote_start + 1..quote_end].to_string();

        if path.is_empty() {
            return Err(ParseError::InvalidParameter {
                line: line_number,
                param: "path".to_string(),
                message: "cutscene path cannot be empty".to_string(),
            });
        }

        Ok(Some(ScriptNode::Cutscene { path }))
    }
}
