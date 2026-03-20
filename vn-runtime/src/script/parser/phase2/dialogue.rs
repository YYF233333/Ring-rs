//! 对话与文本指令解析：chapter、extend

use crate::error::ParseError;
use crate::script::ast::ScriptNode;

use super::super::helpers::extract_quoted_content;
use super::super::inline_tags::parse_inline_tags;
use super::Phase2Parser;

impl Phase2Parser {
    /// 解析章节标记
    pub(super) fn parse_chapter(&self, line: &str) -> Result<Option<ScriptNode>, ParseError> {
        let level = line.chars().take_while(|&c| c == '#').count() as u8;
        if level == 0 || level > 6 {
            return Ok(None);
        }

        let title = line[level as usize..].trim();
        if title.is_empty() {
            return Ok(None);
        }

        Ok(Some(ScriptNode::Chapter {
            title: title.to_string(),
            level,
        }))
    }

    /// 解析 extend 指令
    ///
    /// 语法: `extend "追加文本"`
    pub(super) fn parse_extend(
        &self,
        line: &str,
        line_number: usize,
        no_wait: bool,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content_part = line
            .get("extend".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "extend".to_string(),
                param: "quoted text".to_string(),
            })?;

        let raw_content =
            extract_quoted_content(content_part).ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "extend".to_string(),
                param: "quoted text (use \"...\")".to_string(),
            })?;

        let (content, inline_effects) = parse_inline_tags(raw_content);

        Ok(Some(ScriptNode::Extend {
            content,
            inline_effects,
            no_wait,
        }))
    }
}
