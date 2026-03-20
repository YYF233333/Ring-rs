//! 显示相关指令解析：changeBG、changeScene、show、hide + 过渡效果提取

use crate::command::{Position, Transition, TransitionArg};
use crate::error::ParseError;
use crate::script::ast::ScriptNode;

use super::super::helpers::{
    extract_img_src, extract_keyword_value, parse_transition, parse_transition_args,
};

use super::Phase2Parser;

impl Phase2Parser {
    /// 解析 changeBG 指令
    ///
    /// changeBG 只支持简单效果：无过渡、dissolve、Dissolve(duration)
    /// fade/fadewhite/Fade/FadeWhite 已废弃，请使用 changeScene
    pub(super) fn parse_change_bg(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let path = extract_img_src(line).ok_or_else(|| ParseError::MissingParameter {
            line: line_number,
            command: "changeBG".to_string(),
            param: "图片路径 (<img src=\"...\">)".to_string(),
        })?;

        let transition = self.extract_transition_from_line(line);

        if let Some(ref t) = transition {
            let name_lower = t.name.to_lowercase();
            if name_lower == "fade" || name_lower == "fadewhite" {
                return Err(ParseError::InvalidTransition {
                    line: line_number,
                    message: format!(
                        "changeBG 不再支持 '{}' 效果。请使用 changeScene with {}(...) 替代",
                        t.name,
                        if name_lower == "fade" {
                            "Fade"
                        } else {
                            "FadeWhite"
                        }
                    ),
                });
            }
            if name_lower != "dissolve" {
                return Err(ParseError::InvalidTransition {
                    line: line_number,
                    message: format!(
                        "changeBG 只支持 dissolve 效果，不支持 '{}'。如需复杂过渡，请使用 changeScene",
                        t.name
                    ),
                });
            }
        }

        Ok(Some(ScriptNode::ChangeBG {
            path: path.to_string(),
            transition,
        }))
    }

    /// 解析 changeScene 指令
    ///
    /// changeScene 必须带 with 子句，支持：
    /// - Dissolve(duration)
    /// - Fade(duration) / FadeWhite(duration)
    /// - <img src="rule.png"/> (duration: N, reversed: bool)
    pub(super) fn parse_change_scene(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let path = extract_img_src(line).ok_or_else(|| ParseError::MissingParameter {
            line: line_number,
            command: "changeScene".to_string(),
            param: "图片路径 (<img src=\"...\">)".to_string(),
        })?;

        let lower = line.to_lowercase();
        if !lower.contains(" with ") && !lower.contains(">with ") && !lower.contains(" with`") {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "changeScene".to_string(),
                param: "with 子句（changeScene 必须指定过渡效果）".to_string(),
            });
        }

        let transition = self.extract_transition_from_line(line).ok_or_else(|| {
            ParseError::InvalidTransition {
                line: line_number,
                message: "无法解析 changeScene 的过渡效果".to_string(),
            }
        })?;

        Ok(Some(ScriptNode::ChangeScene {
            path: path.to_string(),
            transition: Some(transition),
        }))
    }

    /// 解析 show 指令
    ///
    /// 支持两种格式：
    /// - `show <img src="..."> as alias at position` - 显示新立绘并绑定别名
    /// - `show alias at position` - 使用已绑定的别名改变位置
    pub(super) fn parse_show(
        &mut self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let path = extract_img_src(line).map(|s| s.to_string());

        let alias: String = if path.is_some() {
            extract_keyword_value(line, "as")
                .ok_or_else(|| ParseError::MissingParameter {
                    line: line_number,
                    command: "show".to_string(),
                    param: "as (别名)".to_string(),
                })?
                .to_string()
        } else {
            let line_lower = line.to_lowercase();
            let show_pos = line_lower
                .find("show")
                .ok_or_else(|| ParseError::InvalidLine {
                    line: line_number,
                    message: "无法找到 'show' 关键字".to_string(),
                })?;
            let after_show = &line[show_pos + 4..].trim_start();

            let at_pos = after_show.to_lowercase().find(" at ").ok_or_else(|| {
                ParseError::MissingParameter {
                    line: line_number,
                    command: "show".to_string(),
                    param: "at (位置)".to_string(),
                }
            })?;

            after_show[..at_pos].trim().to_string()
        };

        let position_str =
            extract_keyword_value(line, "at").ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "show".to_string(),
                param: "at (位置)".to_string(),
            })?;

        let position: Position =
            position_str
                .parse()
                .map_err(|_| ParseError::InvalidParameter {
                    line: line_number,
                    param: "position".to_string(),
                    message: format!("未知位置 '{}'", position_str),
                })?;

        let transition = self.extract_transition_from_line(line);

        Ok(Some(ScriptNode::ShowCharacter {
            path,
            alias,
            position,
            transition,
        }))
    }

    /// 解析 hide 指令
    pub(super) fn parse_hide(
        &mut self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() < 2 {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "hide".to_string(),
                param: "别名".to_string(),
            });
        }

        let alias = parts[1].to_string();
        let transition = self.extract_transition_from_line(line);

        Ok(Some(ScriptNode::HideCharacter { alias, transition }))
    }

    /// 从行中提取 with 子句的过渡效果
    ///
    /// 支持多种格式：
    /// - `... with dissolve` (标准空格分隔)
    /// - `...>with dissolve` (无空格)
    /// - `... with `Dissolve(2.0)`` (行内代码格式)
    /// - `... with <img src="rule.png"/> (duration: 1, reversed: true)` (rule-based effect)
    pub(in crate::script::parser) fn extract_transition_from_line(
        &self,
        line: &str,
    ) -> Option<Transition> {
        let lower = line.to_lowercase();

        let with_pos = lower
            .rfind(" with ")
            .or_else(|| lower.rfind(">with "))
            .or_else(|| lower.rfind(" with`"))
            .or_else(|| lower.rfind(">with`"))?;

        let with_slice = &lower[with_pos..];
        let text_after_with =
            if with_slice.starts_with(" with ") || with_slice.starts_with(">with ") {
                &line[with_pos + 6..]
            } else {
                &line[with_pos + 5..]
            };

        let transition_text = text_after_with.trim();

        if transition_text.contains("<img") {
            if let Some(rule_path) = extract_img_src(transition_text) {
                let args = self.extract_rule_args(transition_text, rule_path);
                return Some(Transition::with_named_args("rule", args));
            }
            return Some(Transition::simple("rule"));
        }

        let transition_text = if let Some(stripped) = transition_text.strip_prefix('`') {
            if let Some(stripped) = stripped.strip_suffix('`') {
                stripped
            } else if let Some(end) = stripped.find('`') {
                &stripped[..end]
            } else {
                stripped
            }
        } else {
            transition_text
        };

        parse_transition(transition_text.trim())
    }

    /// 从 rule-based effect 文本中提取参数
    fn extract_rule_args(
        &self,
        text: &str,
        mask_path: &str,
    ) -> Vec<(Option<String>, TransitionArg)> {
        let mut args = vec![(
            Some("mask".to_string()),
            TransitionArg::String(mask_path.to_string()),
        )];

        if let Some(img_end) = text.find("/>") {
            let after_img = &text[img_end + 2..];
            if let Some(paren_start) = after_img.find('(')
                && let Some(paren_end) = after_img.rfind(')')
            {
                let params_str = &after_img[paren_start + 1..paren_end];
                if let Ok(parsed_args) = parse_transition_args(params_str) {
                    args.extend(parsed_args);
                }
            }
        }

        args
    }
}
