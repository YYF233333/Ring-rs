//! 杂项指令解析：audio、sceneEffect、titleCard、cutscene、requestUI、textMode

use crate::command::{TextMode, TransitionArg};
use crate::error::ParseError;
use crate::script::Expr;
use crate::script::ast::ScriptNode;

use super::super::expr_parser::parse_expression;
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

    /// 解析 callGame 命令（语法糖 -> RequestUI）
    ///
    /// 语法: `callGame "game_id" as $var` 或 `callGame "game_id" as $var (params)`
    /// 转译为: `RequestUI { mode: "call_game", result_var, params: [(game_id, ...), ...] }`
    pub(super) fn parse_call_game(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("callGame".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "callGame".to_string(),
                param: "game_id".to_string(),
            })?;

        let quote_start = content
            .find('"')
            .ok_or_else(|| ParseError::InvalidParameter {
                line: line_number,
                param: "game_id".to_string(),
                message: "callGame requires a quoted game ID".to_string(),
            })?;
        let quote_end =
            content[quote_start + 1..]
                .find('"')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "game_id".to_string(),
                    message: "missing closing quote for game ID".to_string(),
                })?
                + quote_start
                + 1;
        let game_id = content[quote_start + 1..quote_end].to_string();

        if game_id.is_empty() {
            return Err(ParseError::InvalidParameter {
                line: line_number,
                param: "game_id".to_string(),
                message: "game ID cannot be empty".to_string(),
            });
        }

        let rest = content[quote_end + 1..].trim();
        let as_keyword = "as ";
        let rest_lower = rest.to_lowercase();
        if !rest_lower.starts_with(as_keyword) {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "callGame".to_string(),
                param: "'as $var' clause".to_string(),
            });
        }
        let after_as = rest[as_keyword.len()..].trim();

        let var_end = after_as
            .find(|c: char| c == '(' || c.is_whitespace())
            .unwrap_or(after_as.len());
        let var_part = &after_as[..var_end];
        let result_var = var_part
            .strip_prefix('$')
            .ok_or_else(|| ParseError::InvalidLine {
                line: line_number,
                message: format!("callGame 变量名必须以 '$' 开头，实际: '{}'", var_part),
            })?;

        if result_var.is_empty() || !result_var.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidLine {
                line: line_number,
                message: format!(
                    "callGame 变量名格式无效，只能含字母、数字和下划线，实际: '{}'",
                    result_var
                ),
            });
        }

        let params_rest = after_as[var_end..].trim();
        let mut params = vec![(
            "game_id".to_string(),
            Expr::Literal(crate::state::VarValue::String(game_id)),
        )];

        if let Some(paren_start) = params_rest.find('(') {
            let paren_end = params_rest
                .rfind(')')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "params".to_string(),
                    message: "missing closing parenthesis".to_string(),
                })?;
            let args_str = &params_rest[paren_start + 1..paren_end];
            let extra_params = parse_request_ui_params(args_str, line_number)?;
            params.extend(extra_params);
        }

        Ok(Some(ScriptNode::RequestUI {
            mode: "call_game".to_string(),
            result_var: result_var.to_string(),
            params,
        }))
    }

    /// 解析 showMap 命令（语法糖 -> RequestUI）
    ///
    /// 语法: `showMap "map_id" as $var`
    /// 转译为: `RequestUI { mode: "show_map", result_var, params: [(map_id, "map_id")] }`
    pub(super) fn parse_show_map(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("showMap".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "showMap".to_string(),
                param: "map_id".to_string(),
            })?;

        let quote_start = content
            .find('"')
            .ok_or_else(|| ParseError::InvalidParameter {
                line: line_number,
                param: "map_id".to_string(),
                message: "showMap requires a quoted map ID".to_string(),
            })?;
        let quote_end =
            content[quote_start + 1..]
                .find('"')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "map_id".to_string(),
                    message: "missing closing quote for map ID".to_string(),
                })?
                + quote_start
                + 1;
        let map_id = content[quote_start + 1..quote_end].to_string();

        if map_id.is_empty() {
            return Err(ParseError::InvalidParameter {
                line: line_number,
                param: "map_id".to_string(),
                message: "map ID cannot be empty".to_string(),
            });
        }

        let rest = content[quote_end + 1..].trim();
        let as_keyword = "as ";
        let rest_lower = rest.to_lowercase();
        if !rest_lower.starts_with(as_keyword) {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "showMap".to_string(),
                param: "'as $var' clause".to_string(),
            });
        }
        let after_as = rest[as_keyword.len()..].trim();
        let result_var = after_as
            .strip_prefix('$')
            .ok_or_else(|| ParseError::InvalidLine {
                line: line_number,
                message: format!("showMap 变量名必须以 '$' 开头，实际: '{}'", after_as),
            })?;

        if result_var.is_empty() || !result_var.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidLine {
                line: line_number,
                message: format!(
                    "showMap 变量名格式无效，只能含字母、数字和下划线，实际: '{}'",
                    result_var
                ),
            });
        }

        Ok(Some(ScriptNode::RequestUI {
            mode: "show_map".to_string(),
            result_var: result_var.to_string(),
            params: vec![(
                "map_id".to_string(),
                Expr::Literal(crate::state::VarValue::String(map_id)),
            )],
        }))
    }

    /// 解析 textMode 命令
    ///
    /// 语法: `textMode nvl` / `textMode adv`
    pub(super) fn parse_text_mode(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("textMode".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "textMode".to_string(),
                param: "mode (nvl or adv)".to_string(),
            })?;

        let mode = match content.to_lowercase().as_str() {
            "nvl" => TextMode::NVL,
            "adv" => TextMode::ADV,
            _ => {
                return Err(ParseError::InvalidParameter {
                    line: line_number,
                    param: "mode".to_string(),
                    message: format!("textMode must be 'nvl' or 'adv', got: '{}'", content),
                });
            }
        };

        Ok(Some(ScriptNode::SetTextMode(mode)))
    }

    /// 解析 requestUI 命令
    ///
    /// 语法:
    /// - `requestUI "mode" as $var`
    /// - `requestUI "mode" as $var (param1: value1, param2: "str")`
    pub(super) fn parse_request_ui(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("requestUI".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "requestUI".to_string(),
                param: "mode".to_string(),
            })?;

        let quote_start = content
            .find('"')
            .ok_or_else(|| ParseError::InvalidParameter {
                line: line_number,
                param: "mode".to_string(),
                message: "requestUI requires a quoted mode name".to_string(),
            })?;
        let quote_end =
            content[quote_start + 1..]
                .find('"')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "mode".to_string(),
                    message: "missing closing quote for mode name".to_string(),
                })?
                + quote_start
                + 1;
        let mode = content[quote_start + 1..quote_end].to_string();

        if mode.is_empty() {
            return Err(ParseError::InvalidParameter {
                line: line_number,
                param: "mode".to_string(),
                message: "mode name cannot be empty".to_string(),
            });
        }

        let rest = content[quote_end + 1..].trim();

        let as_keyword = "as ";
        let rest_lower = rest.to_lowercase();
        if !rest_lower.starts_with(as_keyword) {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "requestUI".to_string(),
                param: "'as $var' clause".to_string(),
            });
        }
        let after_as = rest[as_keyword.len()..].trim();

        let var_end = after_as
            .find(|c: char| c == '(' || c.is_whitespace())
            .unwrap_or(after_as.len());
        let var_part = &after_as[..var_end];
        let result_var = var_part
            .strip_prefix('$')
            .ok_or_else(|| ParseError::InvalidLine {
                line: line_number,
                message: format!("requestUI 变量名必须以 '$' 开头，实际: '{}'", var_part),
            })?;

        if result_var.is_empty() || !result_var.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidLine {
                line: line_number,
                message: format!(
                    "requestUI 变量名格式无效，只能含字母、数字和下划线，实际: '{}'",
                    result_var
                ),
            });
        }

        let params_rest = after_as[var_end..].trim();
        let params = if let Some(paren_start) = params_rest.find('(') {
            let paren_end = params_rest
                .rfind(')')
                .ok_or_else(|| ParseError::InvalidParameter {
                    line: line_number,
                    param: "params".to_string(),
                    message: "missing closing parenthesis".to_string(),
                })?;
            let args_str = &params_rest[paren_start + 1..paren_end];
            parse_request_ui_params(args_str, line_number)?
        } else {
            Vec::new()
        };

        Ok(Some(ScriptNode::RequestUI {
            mode,
            result_var: result_var.to_string(),
            params,
        }))
    }
}

/// 解析 requestUI 参数列表：`key1: value1, key2: "str"`
fn parse_request_ui_params(
    args_str: &str,
    line_number: usize,
) -> Result<Vec<(String, Expr)>, ParseError> {
    let mut params = Vec::new();
    if args_str.trim().is_empty() {
        return Ok(params);
    }

    for part in split_params(args_str) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let colon_pos = part.find(':').ok_or_else(|| ParseError::InvalidParameter {
            line: line_number,
            param: "params".to_string(),
            message: format!("parameter must be in 'key: value' format, got: '{}'", part),
        })?;
        let key = part[..colon_pos].trim().to_string();
        let value_str = part[colon_pos + 1..].trim();
        let expr = parse_expression(value_str, line_number)?;
        params.push((key, expr));
    }

    Ok(params)
}

/// 按逗号分割参数，但不拆分引号内的逗号
fn split_params(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;

    for (i, c) in s.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}
