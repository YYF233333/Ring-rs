//! # 阶段 2：块解析
//!
//! 将块转换为 ScriptNode。

use crate::command::{Position, Transition, TransitionArg};
use crate::error::ParseError;
use crate::script::Expr;
use crate::script::ast::{ChoiceOption, ConditionalBranch, ScriptNode};

use super::expr_parser::parse_expression;
use super::helpers::{
    extract_audio_src, extract_img_src, extract_keyword_value, is_table_separator, parse_dialogue,
    parse_transition, parse_transition_args, starts_with_ignore_case,
};
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

    /// 解析单个块
    pub fn parse_block(&mut self, block: Block) -> Result<Option<ScriptNode>, ParseError> {
        match block {
            Block::SingleLine { line, line_number } => self.parse_single_line(&line, line_number),
            Block::Table { lines, start_line } => self.parse_table(&lines, start_line),
            Block::Conditional { lines, start_line } => self.parse_conditional(&lines, start_line),
        }
    }

    /// 解析单行内容
    pub fn parse_single_line(
        &mut self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let line = line.trim();

        // 1. 章节标记 (# Title, ## Title, etc.)
        if line.starts_with('#') {
            return self.parse_chapter(line);
        }

        // 2. 标签定义 (**label_name**)
        if line.starts_with("**") && line.ends_with("**") && line.len() > 4 {
            let name = &line[2..line.len() - 2];
            if !name.contains('*') {
                return Ok(Some(ScriptNode::Label {
                    name: name.trim().to_string(),
                }));
            }
        }

        // 3. 指令解析（大小写不敏感）
        if starts_with_ignore_case(line, "changebg") {
            return self.parse_change_bg(line, line_number);
        }
        if starts_with_ignore_case(line, "changescene") {
            return self.parse_change_scene(line, line_number);
        }
        if starts_with_ignore_case(line, "show") {
            return self.parse_show(line, line_number);
        }
        if starts_with_ignore_case(line, "hide") {
            return self.parse_hide(line, line_number);
        }
        if starts_with_ignore_case(line, "goto") {
            return self.parse_goto(line, line_number);
        }
        // stopBGM - 停止 BGM
        if starts_with_ignore_case(line, "stopbgm") {
            return Ok(Some(ScriptNode::StopBgm));
        }
        // set - 变量赋值
        if starts_with_ignore_case(line, "set ") {
            return self.parse_set_var(line, line_number);
        }
        // textBoxHide - 隐藏对话框
        if starts_with_ignore_case(line, "textboxhide") {
            return Ok(Some(ScriptNode::TextBoxHide));
        }
        // textBoxShow - 显示对话框
        if starts_with_ignore_case(line, "textboxshow") {
            return Ok(Some(ScriptNode::TextBoxShow));
        }
        // textBoxClear - 清理对话框内容
        if starts_with_ignore_case(line, "textboxclear") {
            return Ok(Some(ScriptNode::TextBoxClear));
        }
        // clearCharacters - 清除所有角色立绘
        if starts_with_ignore_case(line, "clearcharacters") {
            return Ok(Some(ScriptNode::ClearCharacters));
        }

        // 4. HTML 标签解析
        // <audio src="..."></audio> 或 <audio src="..."></audio> loop
        if line.starts_with("<audio") {
            return self.parse_audio(line, line_number);
        }

        // 5. 对话/旁白 (包含冒号和引号)
        if let Some((speaker, content)) = parse_dialogue(line) {
            return Ok(Some(ScriptNode::Dialogue { speaker, content }));
        }

        // 6. 未知行
        self.warnings.push(format!(
            "第 {} 行：无法识别的内容，已跳过: {}",
            line_number, line
        ));
        Ok(None)
    }

    /// 解析章节标记
    fn parse_chapter(&self, line: &str) -> Result<Option<ScriptNode>, ParseError> {
        // 计算 # 的数量
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

    /// 解析 set 指令
    ///
    /// 语法: `set $var = value`
    fn parse_set_var(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        // 跳过 "set "
        let content = line[4..].trim();

        // 查找 = 号
        let eq_pos = content
            .find('=')
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "set".to_string(),
                param: "赋值符号 '='".to_string(),
            })?;

        let var_part = content[..eq_pos].trim();
        let value_part = content[eq_pos + 1..].trim();

        // 变量名必须以 $ 开头
        let var_name = var_part
            .strip_prefix('$')
            .ok_or_else(|| ParseError::InvalidLine {
                line: line_number,
                message: format!("变量名必须以 '$' 开头，实际: '{}'", var_part),
            })?;

        if var_name.is_empty() {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "set".to_string(),
                param: "变量名".to_string(),
            });
        }

        // 验证变量名格式
        if !var_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ParseError::InvalidLine {
                line: line_number,
                message: format!("变量名只能包含字母、数字和下划线，实际: '{}'", var_name),
            });
        }

        // 解析表达式（允许完整表达式：==/!=/and/or/not/括号 等）
        let value = parse_expression(value_part, line_number)?;

        Ok(Some(ScriptNode::SetVar {
            name: var_name.to_string(),
            value,
        }))
    }

    /// 解析条件块
    ///
    /// 条件块结构:
    /// ```text
    /// if <condition>
    ///   <body>
    /// elseif <condition>
    ///   <body>
    /// else
    ///   <body>
    /// endif
    /// ```
    fn parse_conditional(
        &mut self,
        lines: &[(String, usize)],
        start_line: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        if lines.is_empty() {
            return Ok(None);
        }

        // 检查最后一行是否是 endif
        let last_line = lines.last().map(|(l, _)| l.as_str()).unwrap_or("");
        if !last_line.eq_ignore_ascii_case("endif") {
            return Err(ParseError::InvalidLine {
                line: start_line,
                message: "条件块未闭合，缺少 'endif'".to_string(),
            });
        }

        let mut branches = Vec::new();
        let mut current_body_lines: Vec<(String, usize)> = Vec::new();
        let mut current_condition: Option<Expr> = None;
        let mut is_first = true;

        for (line, line_number) in lines.iter() {
            let trimmed = line.trim();

            if is_first {
                // 第一行必须是 if
                if !starts_with_ignore_case(trimmed, "if ") {
                    return Err(ParseError::InvalidLine {
                        line: *line_number,
                        message: "条件块必须以 'if' 开头".to_string(),
                    });
                }

                let condition_str = &trimmed[3..].trim();
                current_condition = Some(parse_expression(condition_str, *line_number)?);
                is_first = false;
                continue;
            }

            // elseif
            if starts_with_ignore_case(trimmed, "elseif ") {
                // 保存前一个分支
                let body = self.parse_body_lines(&current_body_lines)?;
                branches.push(ConditionalBranch {
                    condition: current_condition.take(),
                    body,
                });
                current_body_lines.clear();

                let condition_str = &trimmed[7..].trim();
                current_condition = Some(parse_expression(condition_str, *line_number)?);
                continue;
            }

            // else
            if trimmed.eq_ignore_ascii_case("else") {
                // 保存前一个分支
                let body = self.parse_body_lines(&current_body_lines)?;
                branches.push(ConditionalBranch {
                    condition: current_condition.take(),
                    body,
                });
                current_body_lines.clear();

                // else 分支没有条件
                current_condition = None;
                continue;
            }

            // endif
            if trimmed.eq_ignore_ascii_case("endif") {
                // 保存最后一个分支
                let body = self.parse_body_lines(&current_body_lines)?;
                branches.push(ConditionalBranch {
                    condition: current_condition.take(),
                    body,
                });
                break;
            }

            // 普通内容行，添加到当前分支体
            current_body_lines.push((trimmed.to_string(), *line_number));
        }

        if branches.is_empty() {
            return Err(ParseError::InvalidLine {
                line: start_line,
                message: "条件块没有有效分支".to_string(),
            });
        }

        Ok(Some(ScriptNode::Conditional { branches }))
    }

    /// 解析分支体内的行列表
    fn parse_body_lines(
        &mut self,
        lines: &[(String, usize)],
    ) -> Result<Vec<ScriptNode>, ParseError> {
        let mut nodes = Vec::new();

        for (line, line_number) in lines {
            // 跳过空行
            if line.trim().is_empty() {
                continue;
            }

            // 递归解析每一行
            if let Some(node) = self.parse_single_line(line, *line_number)? {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    /// 解析 changeBG 指令
    ///
    /// changeBG 只支持简单效果：无过渡、dissolve、Dissolve(duration)
    /// fade/fadewhite/Fade/FadeWhite 已废弃，请使用 changeScene
    fn parse_change_bg(
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

        // 检查是否使用了废弃的效果
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
            // 只允许 dissolve/Dissolve
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
    fn parse_change_scene(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let path = extract_img_src(line).ok_or_else(|| ParseError::MissingParameter {
            line: line_number,
            command: "changeScene".to_string(),
            param: "图片路径 (<img src=\"...\">)".to_string(),
        })?;

        // changeScene 强制要求 with 子句
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
    fn parse_show(&self, line: &str, line_number: usize) -> Result<Option<ScriptNode>, ParseError> {
        // 尝试提取图片路径（可选）
        let path = extract_img_src(line).map(|s| s.to_string());

        // 提取别名
        // 如果有 <img src>，别名在 "as" 后面
        // 如果没有 <img src>，别名就是 "show" 后面的第一个词
        let alias: String = if path.is_some() {
            // 标准格式：show <img src="..."> as alias at position
            extract_keyword_value(line, "as")
                .ok_or_else(|| ParseError::MissingParameter {
                    line: line_number,
                    command: "show".to_string(),
                    param: "as (别名)".to_string(),
                })?
                .to_string()
        } else {
            // 简化格式：show alias at position
            // 提取 "show" 后面的第一个词（到 "at" 之前）
            let line_lower = line.to_lowercase();
            let show_pos = line_lower
                .find("show")
                .ok_or_else(|| ParseError::InvalidLine {
                    line: line_number,
                    message: "无法找到 'show' 关键字".to_string(),
                })?;
            let after_show = &line[show_pos + 4..].trim_start();

            // 查找 "at" 的位置
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

        let position =
            Position::from_str(position_str).ok_or_else(|| ParseError::InvalidParameter {
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
    fn parse_hide(&self, line: &str, line_number: usize) -> Result<Option<ScriptNode>, ParseError> {
        // hide alias with transition
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

    /// 解析 goto 指令
    ///
    /// 语法: `goto **label**`
    fn parse_goto(&self, line: &str, line_number: usize) -> Result<Option<ScriptNode>, ParseError> {
        // 跳过 "goto" 前缀
        let content = line
            .get(4..)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "goto".to_string(),
                param: "目标标签".to_string(),
            })?;

        // 提取 **label** 中的 label
        let target_label =
            if content.starts_with("**") && content.ends_with("**") && content.len() > 4 {
                content[2..content.len() - 2].trim().to_string()
            } else {
                // 也支持不带 ** 的格式
                content.to_string()
            };

        if target_label.is_empty() {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "goto".to_string(),
                param: "目标标签".to_string(),
            });
        }

        Ok(Some(ScriptNode::Goto { target_label }))
    }

    /// 解析 audio 标签
    ///
    /// 语法:
    /// - `<audio src="path/to/audio.mp3"></audio>` - SFX（播放一次）
    /// - `<audio src="path/to/audio.mp3"></audio> loop` - BGM（循环播放）
    fn parse_audio(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        // 提取 src 属性
        let path = extract_audio_src(line).ok_or_else(|| ParseError::MissingParameter {
            line: line_number,
            command: "audio".to_string(),
            param: "音频路径 (<audio src=\"...\">)".to_string(),
        })?;

        // 检查是否有 loop 标识（在 </audio> 后面）
        // 查找 </audio> 的位置
        let is_bgm = if let Some(close_tag_pos) = line.to_lowercase().find("</audio>") {
            // 检查 </audio> 后面是否有 "loop"
            let after_tag = &line[close_tag_pos + 8..]; // "</audio>" 长度为 8
            after_tag.to_lowercase().contains("loop")
        } else {
            false
        };

        Ok(Some(ScriptNode::PlayAudio {
            path: path.to_string(),
            is_bgm,
        }))
    }

    /// 从行中提取 with 子句的过渡效果
    ///
    /// 支持多种格式：
    /// - `... with dissolve` (标准空格分隔)
    /// - `...>with dissolve` (无空格)
    /// - `... with `Dissolve(2.0)`` (行内代码格式)
    /// - `... with <img src="rule.png"/> (duration: 1, reversed: true)` (rule-based effect)
    pub(super) fn extract_transition_from_line(&self, line: &str) -> Option<Transition> {
        let lower = line.to_lowercase();

        // 查找 "with" 的位置（支持 " with " 和 ">with "）
        let with_pos = lower
            .rfind(" with ")
            .or_else(|| lower.rfind(">with "))
            .or_else(|| lower.rfind(" with`")) // 行内代码格式
            .or_else(|| lower.rfind(">with`"))?;

        // 计算实际的过渡文本起始位置
        let text_after_with = if lower[with_pos..].starts_with(" with ") {
            &line[with_pos + 6..]
        } else if lower[with_pos..].starts_with(">with ") {
            &line[with_pos + 6..]
        } else if lower[with_pos..].starts_with(" with`") {
            &line[with_pos + 5..]
        } else {
            &line[with_pos + 5..]
        };

        let transition_text = text_after_with.trim();

        // 如果包含 <img，这是 rule 类型的过渡
        // 格式: <img src="rule.png" .../> (duration: 1, reversed: true)
        if transition_text.contains("<img") {
            if let Some(rule_path) = extract_img_src(transition_text) {
                // 尝试提取括号内的参数
                let args = self.extract_rule_args(transition_text, rule_path);
                return Some(Transition::with_named_args("rule", args));
            }
            return Some(Transition::simple("rule"));
        }

        // 处理行内代码格式: `Dissolve(2.0, 0.5)`
        let transition_text = if transition_text.starts_with('`') && transition_text.ends_with('`')
        {
            &transition_text[1..transition_text.len() - 1]
        } else if transition_text.starts_with('`') {
            // 只有开始反引号，查找结束
            if let Some(end) = transition_text[1..].find('`') {
                &transition_text[1..end + 1]
            } else {
                transition_text.trim_start_matches('`')
            }
        } else {
            transition_text
        };

        parse_transition(transition_text.trim())
    }

    /// 从 rule-based effect 文本中提取参数
    ///
    /// 输入格式: `<img src="rule.png" .../> (duration: 1, reversed: true)`
    fn extract_rule_args(
        &self,
        text: &str,
        mask_path: &str,
    ) -> Vec<(Option<String>, TransitionArg)> {
        let mut args = vec![(
            Some("mask".to_string()),
            TransitionArg::String(mask_path.to_string()),
        )];

        // 查找 /> 后面的括号参数
        if let Some(img_end) = text.find("/>") {
            let after_img = &text[img_end + 2..];
            // 查找 (...)
            if let Some(paren_start) = after_img.find('(')
                && let Some(paren_end) = after_img.rfind(')')
            {
                let params_str = &after_img[paren_start + 1..paren_end];
                // 解析参数（命名参数格式）
                if let Ok(parsed_args) = parse_transition_args(params_str) {
                    args.extend(parsed_args);
                }
            }
        }

        args
    }

    /// 解析表格块（选择分支）
    fn parse_table(
        &mut self,
        lines: &[String],
        start_line: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        if lines.is_empty() {
            return Ok(None);
        }

        let mut options = Vec::new();
        let mut style = None;
        let mut header_parsed = false;

        for (idx, line) in lines.iter().enumerate() {
            let line_number = start_line + idx;

            // 跳过分隔行
            if is_table_separator(line) {
                continue;
            }

            // 解析表格行：分割 | 并提取单元格
            let cells: Vec<&str> = line
                .split('|')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            // 第一个非分隔行是表头
            if !header_parsed {
                if !cells.is_empty() {
                    style = Some(cells[0].to_string());
                }
                header_parsed = true;
                continue;
            }

            // 选项行需要至少两个单元格
            if cells.len() < 2 {
                self.warnings
                    .push(format!("第 {} 行：表格行格式不完整，已跳过", line_number));
                continue;
            }

            // 选项行
            options.push(ChoiceOption {
                text: cells[0].to_string(),
                target_label: cells[1].to_string(),
            });
        }

        if options.is_empty() {
            return Err(ParseError::InvalidTable {
                line: start_line,
                message: "表格中没有有效的选项".to_string(),
            });
        }

        Ok(Some(ScriptNode::Choice { style, options }))
    }
}

impl Default for Phase2Parser {
    fn default() -> Self {
        Self::new()
    }
}
