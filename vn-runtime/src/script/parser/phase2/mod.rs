//! # 阶段 2：块解析
//!
//! 将块转换为 ScriptNode。按指令域拆分为子模块：
//! - `display`: 显示指令（changeBG/changeScene/show/hide + 过渡效果）
//! - `control`: 控制流（goto/callScript/conditional/set/wait/choice table）
//! - `dialogue`: 对话与文本（chapter/extend）
//! - `misc`: 杂项（audio/sceneEffect/titleCard/cutscene）

mod control;
mod dialogue;
mod display;
mod misc;

use crate::error::ParseError;
use crate::script::ast::ScriptNode;

use super::helpers::{parse_dialogue, starts_with_ignore_case};
use super::inline_tags::parse_inline_tags;
use super::phase1::Block;

/// 阶段 2 解析器
pub struct Phase2Parser {
    /// 解析警告（非致命错误）
    pub warnings: Vec<String>,
}

impl Phase2Parser {
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    pub fn reset_state(&mut self) {
        self.warnings.clear();
    }

    /// 解析单个块
    pub fn parse_block(&mut self, block: Block) -> Result<Option<ScriptNode>, ParseError> {
        match block {
            Block::SingleLine { line, line_number } => self.parse_single_line(&line, line_number),
            Block::Table { lines, start_line } => self.parse_table(&lines, start_line),
            Block::Conditional { lines, start_line } => self.parse_conditional(&lines, start_line),
        }
    }

    /// 解析单行内容 — 按前缀分发到各子模块
    pub fn parse_single_line(
        &mut self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let (line, no_wait) = strip_arrow_suffix(line.trim());
        let line = line.trim();

        // Markdown 注释块
        if line.starts_with('>') {
            return Ok(None);
        }

        // 章节标记 (# Title)
        if line.starts_with('#') {
            return self.parse_chapter(line);
        }

        // 标签定义 (**label_name**)
        if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
            let name = &line[2..line.len() - 2];
            if !name.contains('*') {
                return Ok(Some(ScriptNode::Label {
                    name: name.trim().to_string(),
                }));
            }
        }

        // --- 指令分发（大小写不敏感）---

        // 显示指令
        if starts_with_ignore_case(line, "changebg") {
            return self.parse_change_bg(line, line_number);
        }
        if starts_with_ignore_case(line, "changescene") {
            return self.parse_change_scene(line, line_number);
        }
        if starts_with_command(line, "show") {
            return self.parse_show(line, line_number);
        }
        if starts_with_command(line, "hide") {
            return self.parse_hide(line, line_number);
        }

        // 控制流指令
        if starts_with_ignore_case(line, "goto") {
            return self.parse_goto(line, line_number);
        }
        if starts_with_ignore_case(line, "callscript") {
            return self.parse_call_script(line, line_number);
        }
        if starts_with_ignore_case(line, "returnfromscript") {
            return Ok(Some(ScriptNode::ReturnFromScript));
        }
        if starts_with_ignore_case(line, "fullrestart") {
            return Ok(Some(ScriptNode::FullRestart));
        }
        if starts_with_ignore_case(line, "set ") {
            return self.parse_set_var(line, line_number);
        }
        if starts_with_command(line, "wait") {
            return self.parse_wait(line, line_number);
        }
        if starts_with_ignore_case(line, "pause")
            && (line.len() == 5
                || line
                    .as_bytes()
                    .get(5)
                    .is_none_or(|b| b.is_ascii_whitespace()))
        {
            return Ok(Some(ScriptNode::Pause));
        }
        if starts_with_ignore_case(line, "clearcharacters") {
            return Ok(Some(ScriptNode::ClearCharacters));
        }

        // 音频指令（简单变体内联，复杂解析委托到 misc）
        if starts_with_ignore_case(line, "stopbgm") {
            return Ok(Some(ScriptNode::StopBgm));
        }
        if starts_with_ignore_case(line, "bgmunduck") {
            return Ok(Some(ScriptNode::BgmUnduck));
        }
        if starts_with_ignore_case(line, "bgmduck") {
            return Ok(Some(ScriptNode::BgmDuck));
        }

        // UI 指令
        if starts_with_ignore_case(line, "textboxhide") {
            return Ok(Some(ScriptNode::TextBoxHide));
        }
        if starts_with_ignore_case(line, "textboxshow") {
            return Ok(Some(ScriptNode::TextBoxShow));
        }
        if starts_with_ignore_case(line, "textboxclear") {
            return Ok(Some(ScriptNode::TextBoxClear));
        }

        // 杂项指令
        if starts_with_ignore_case(line, "sceneeffect") {
            return self.parse_scene_effect(line, line_number);
        }
        if starts_with_ignore_case(line, "titlecard") {
            return self.parse_title_card(line, line_number);
        }
        if starts_with_ignore_case(line, "cutscene") {
            return self.parse_cutscene(line, line_number);
        }
        if starts_with_command(line, "extend") {
            return self.parse_extend(line, line_number, no_wait);
        }

        // HTML audio 标签
        if line.starts_with("<audio") {
            return self.parse_audio(line, line_number);
        }

        // 对话/旁白
        if let Some((speaker, raw_content)) = parse_dialogue(line) {
            let (content, inline_effects) = parse_inline_tags(&raw_content);
            return Ok(Some(ScriptNode::Dialogue {
                speaker,
                content,
                inline_effects,
                no_wait,
            }));
        }

        // 未知行
        self.warnings.push(format!(
            "第 {} 行：无法识别的内容，已跳过: {}",
            line_number, line
        ));
        Ok(None)
    }
}

/// 检测并剥离行尾 `-->` 修饰符
fn strip_arrow_suffix(line: &str) -> (&str, bool) {
    let trimmed = line.trim_end();
    if let Some(before) = trimmed.strip_suffix("-->") {
        (before, true)
    } else {
        (line, false)
    }
}

fn starts_with_command(line: &str, command: &str) -> bool {
    let line = line.trim_start();
    if line.len() < command.len() || !starts_with_ignore_case(line, command) {
        return false;
    }
    let next = line[command.len()..].chars().next();
    match next {
        None => true,
        Some(ch) => ch.is_whitespace() || ch == '<',
    }
}

pub(super) fn parse_markdown_link(input: &str) -> Option<(&str, &str, &str)> {
    let trimmed = input.trim_start();
    if !trimmed.starts_with('[') {
        return None;
    }

    let label_end = trimmed.find(']')?;
    let label = &trimmed[1..label_end];
    if label.trim().is_empty() {
        return None;
    }

    let after_label = &trimmed[label_end + 1..];
    if !after_label.starts_with('(') {
        return None;
    }
    let path_end = after_label.find(')')?;
    let path = &after_label[1..path_end];
    if path.trim().is_empty() {
        return None;
    }

    let rest = &after_label[path_end + 1..];
    Some((label.trim(), path.trim(), rest))
}
