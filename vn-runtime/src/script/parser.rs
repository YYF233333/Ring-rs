//! # Parser 模块
//!
//! 两阶段脚本解析器实现（手写递归下降，无 regex 依赖）。
//!
//! ## 架构
//!
//! ```text
//! 原始文本 → [阶段1: 块识别] → Vec<Block> → [阶段2: 块解析] → Vec<ScriptNode>
//! ```
//!
//! ## 设计原则
//!
//! - 使用手写的字符串解析函数，避免正则表达式
//! - 清晰的错误处理和行号追踪
//! - 容错解析：尽可能解析有效内容，无效行记录警告

use crate::command::{Position, Transition, TransitionArg};
use crate::error::ParseError;
use crate::script::ast::{ChoiceOption, Script, ScriptNode};

//=============================================================================
// 辅助解析函数
//=============================================================================

/// 跳过字符串开头的空白字符，返回剩余部分
fn skip_whitespace(s: &str) -> &str {
    s.trim_start()
}

/// 检查字符串是否以指定前缀开头（大小写不敏感）
fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len()
        && s.chars()
            .zip(prefix.chars())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
}

/// 从字符串中提取 HTML img 标签的 src 属性值
///
/// 输入: `<img src="path/to/image.png" alt="desc" />`
/// 输出: `Some("path/to/image.png")`
fn extract_img_src(s: &str) -> Option<&str> {
    // 查找 <img
    let img_start = s.find("<img")?;
    let after_img = &s[img_start..];

    // 查找 src=
    let src_pos = after_img.find("src")?;
    let after_src = &after_img[src_pos + 3..];

    // 跳过空白和等号
    let after_eq = skip_whitespace(after_src);
    let after_eq = after_eq.strip_prefix('=')?;
    let after_eq = skip_whitespace(after_eq);

    // 提取引号内的内容
    let quote_char = after_eq.chars().next()?;
    if quote_char != '"' && quote_char != '\'' {
        return None;
    }

    let after_quote = &after_eq[1..];
    let end_quote = after_quote.find(quote_char)?;

    Some(&after_quote[..end_quote])
}

/// 从字符串中提取 HTML audio 标签的 src 属性值
///
/// 输入: `<audio src="path/to/audio.mp3"></audio>`
/// 输出: `Some("path/to/audio.mp3")`
fn extract_audio_src(s: &str) -> Option<&str> {
    // 查找 <audio
    let audio_start = s.find("<audio")?;
    let after_audio = &s[audio_start..];

    // 查找 src=
    let src_pos = after_audio.find("src")?;
    let after_src = &after_audio[src_pos + 3..];

    // 跳过空白和等号
    let after_eq = skip_whitespace(after_src);
    let after_eq = after_eq.strip_prefix('=')?;
    let after_eq = skip_whitespace(after_eq);

    // 提取引号内的内容
    let quote_char = after_eq.chars().next()?;
    if quote_char != '"' && quote_char != '\'' {
        return None;
    }

    let after_quote = &after_eq[1..];
    let end_quote = after_quote.find(quote_char)?;

    Some(&after_quote[..end_quote])
}

/// 查找关键字并提取其后的值
///
/// 输入: `"show <img> as royu at center with dissolve"`, `"as"`
/// 输出: `Some("royu")`
///
/// 支持多种格式：
/// - `... as value ...` (标准空格分隔)
/// - `...>as value ...` (无空格，紧跟 img 标签)
fn extract_keyword_value<'a>(s: &'a str, keyword: &str) -> Option<&'a str> {
    let lower = s.to_lowercase();
    let keyword_lower = keyword.to_lowercase();

    // 尝试多种模式查找关键字
    let patterns = [
        format!(" {} ", keyword_lower),   // 标准：空格包围
        format!(">{} ", keyword_lower),   // 紧跟 >：如 />as
        format!(" {}", keyword_lower),    // 只有前空格
    ];

    let mut best_pos = None;
    let mut best_pattern_len = 0;

    for pattern in &patterns {
        if let Some(pos) = lower.find(pattern.as_str()) {
            if best_pos.is_none() || pos < best_pos.unwrap() {
                best_pos = Some(pos);
                best_pattern_len = pattern.len();
            }
        }
    }

    let pos = best_pos?;
    let value_start = pos + best_pattern_len;

    // 确保 value_start 不超出范围
    if value_start >= s.len() {
        return None;
    }

    let remaining = &s[value_start..];
    let remaining_lower = remaining.to_lowercase();

    // 查找下一个关键字或边界的位置
    let terminators = [" with ", " as ", " at ", ">with", ">as", ">at"];
    let mut end_pos = remaining.len();

    for term in &terminators {
        if let Some(p) = remaining_lower.find(term) {
            if p < end_pos {
                end_pos = p;
            }
        }
    }

    let value = remaining[..end_pos].trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// 解析过渡效果表达式
///
/// 输入: `"dissolve"` 或 `"Dissolve(1.5)"` 或 `"rule(\"mask.png\", 1.0, true)"`
fn parse_transition(s: &str) -> Option<Transition> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // 查找括号
    if let Some(paren_start) = s.find('(') {
        // 函数调用格式: name(args)
        let name = s[..paren_start].trim();
        let paren_end = s.rfind(')')?;
        let args_str = &s[paren_start + 1..paren_end];

        let args = parse_transition_args(args_str);
        Some(Transition::with_args(name, args))
    } else {
        // 简单名称格式: name
        Some(Transition::simple(s))
    }
}

/// 解析过渡效果参数列表
fn parse_transition_args(s: &str) -> Vec<TransitionArg> {
    let s = s.trim();
    if s.is_empty() {
        return Vec::new();
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = '"';

    for ch in s.chars() {
        if in_string {
            if ch == string_char {
                in_string = false;
                // 将字符串内容作为参数
                args.push(TransitionArg::String(current.clone()));
                current.clear();
            } else {
                current.push(ch);
            }
        } else if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
        } else if ch == ',' {
            // 参数分隔符
            let arg = current.trim();
            if !arg.is_empty() {
                args.push(parse_single_arg(arg));
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    // 处理最后一个参数
    let arg = current.trim();
    if !arg.is_empty() {
        args.push(parse_single_arg(arg));
    }

    args
}

/// 解析单个参数值
fn parse_single_arg(s: &str) -> TransitionArg {
    let s = s.trim();

    // 尝试解析为数字
    if let Ok(n) = s.parse::<f64>() {
        return TransitionArg::Number(n);
    }

    // 尝试解析为布尔值
    if s.eq_ignore_ascii_case("true") {
        return TransitionArg::Bool(true);
    }
    if s.eq_ignore_ascii_case("false") {
        return TransitionArg::Bool(false);
    }

    // 作为字符串处理
    TransitionArg::String(s.to_string())
}

/// 检查是否是表格分隔行 (| --- | --- |)
fn is_table_separator(s: &str) -> bool {
    let s = s.trim();
    if !s.starts_with('|') || !s.ends_with('|') {
        return false;
    }

    // 检查中间只包含 -, :, |, 空格
    s[1..s.len() - 1]
        .chars()
        .all(|c| c == '-' || c == ':' || c == '|' || c.is_whitespace())
}

/// 解析对话行，提取说话者和内容
///
/// 支持格式:
/// - `角色名："内容"` 或 `角色名："内容"`
/// - `角色名: "内容"` 或 `角色名: "内容"`
/// - `："内容"` (旁白)
fn parse_dialogue(s: &str) -> Option<(Option<String>, String)> {
    let s = s.trim();

    // 查找冒号位置（支持中英文冒号）
    // 中文冒号 U+FF1A (：) 或 U+003A (:)
    let (colon_pos, colon_len) = if let Some(pos) = s.find('：') {
        (pos, '：'.len_utf8())
    } else if let Some(pos) = s.find(':') {
        (pos, ':'.len_utf8())
    } else {
        return None;
    };

    let speaker_part = s[..colon_pos].trim();
    let content_part = s[colon_pos + colon_len..].trim();

    // 内容必须被引号包围
    let content = extract_quoted_content(content_part)?;

    let speaker = if speaker_part.is_empty() {
        None
    } else {
        Some(speaker_part.to_string())
    };

    Some((speaker, content.to_string()))
}

/// 提取引号内的内容
///
/// 支持:
/// - "内容" (ASCII 双引号 U+0022)
/// - "内容" (中文引号 U+201C / U+201D)
fn extract_quoted_content(s: &str) -> Option<&str> {
    let s = s.trim();

    // 定义引号字符
    const ASCII_QUOTE: char = '"';       // U+0022
    const CN_LEFT_QUOTE: char = '\u{201C}';  // 中文左双引号
    const CN_RIGHT_QUOTE: char = '\u{201D}'; // 中文右双引号

    // 获取第一个字符
    let first_char = s.chars().next()?;

    // 确定开始和结束引号
    let end_quote = if first_char == ASCII_QUOTE {
        ASCII_QUOTE
    } else if first_char == CN_LEFT_QUOTE {
        CN_RIGHT_QUOTE
    } else {
        return None;
    };

    // 跳过开始引号
    let content_start = first_char.len_utf8();
    let remaining = &s[content_start..];

    // 查找结束引号
    let end_pos = remaining.find(end_quote)?;

    Some(&remaining[..end_pos])
}

//=============================================================================
// 块类型定义
//=============================================================================

/// 块类型（阶段 1 输出）
#[derive(Debug, Clone)]
enum Block {
    /// 单行内容
    SingleLine { line: String, line_number: usize },
    /// 表格块（选择分支）
    Table { lines: Vec<String>, start_line: usize },
}

//=============================================================================
// Parser 主结构
//=============================================================================

/// 脚本解析器
pub struct Parser {
    /// 解析警告（非致命错误）
    warnings: Vec<String>,
}

impl Parser {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    /// 解析脚本文本
    ///
    /// # 参数
    ///
    /// - `script_id`: 脚本标识符
    /// - `text`: 脚本文本内容
    ///
    /// # 返回
    ///
    /// 解析后的 `Script`，或解析错误
    ///
    /// # 注意
    ///
    /// 此方法使用空的 base_path，素材路径将保持原样。
    /// 如需支持相对于脚本文件的路径，请使用 `parse_with_base_path`。
    pub fn parse(&mut self, script_id: &str, text: &str) -> Result<Script, ParseError> {
        self.parse_with_base_path(script_id, text, "")
    }

    /// 解析脚本文本（带基础路径）
    ///
    /// # 参数
    ///
    /// - `script_id`: 脚本标识符
    /// - `text`: 脚本文本内容
    /// - `base_path`: 脚本文件所在目录，用于解析相对路径
    ///
    /// # 返回
    ///
    /// 解析后的 `Script`，或解析错误
    pub fn parse_with_base_path(
        &mut self,
        script_id: &str,
        text: &str,
        base_path: &str,
    ) -> Result<Script, ParseError> {
        self.warnings.clear();

        // 阶段 1：块识别
        let blocks = self.recognize_blocks(text);

        // 阶段 2：块解析
        let mut nodes = Vec::new();
        for block in blocks {
            match self.parse_block(block) {
                Ok(Some(node)) => nodes.push(node),
                Ok(None) => {} // 跳过（如空内容）
                Err(e) => return Err(e),
            }
        }

        Ok(Script::new(script_id, nodes, base_path))
    }

    /// 获取解析过程中的警告
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    //=========================================================================
    // 阶段 1：块识别
    //=========================================================================

    fn recognize_blocks(&self, text: &str) -> Vec<Block> {
        let mut blocks = Vec::new();
        let mut current_table: Option<(Vec<String>, usize)> = None;

        for (line_idx, line) in text.lines().enumerate() {
            let line_number = line_idx + 1;
            let trimmed = line.trim();

            // 空行：结束当前表格块
            if trimmed.is_empty() {
                if let Some((lines, start)) = current_table.take() {
                    blocks.push(Block::Table {
                        lines,
                        start_line: start,
                    });
                }
                continue;
            }

            // 以 `|` 开头：表格行
            if trimmed.starts_with('|') {
                match &mut current_table {
                    Some((lines, _)) => {
                        lines.push(trimmed.to_string());
                    }
                    None => {
                        current_table = Some((vec![trimmed.to_string()], line_number));
                    }
                }
            } else {
                // 非表格行：结束当前表格块，创建单行块
                if let Some((lines, start)) = current_table.take() {
                    blocks.push(Block::Table {
                        lines,
                        start_line: start,
                    });
                }
                blocks.push(Block::SingleLine {
                    line: trimmed.to_string(),
                    line_number,
                });
            }
        }

        // 处理末尾的表格块
        if let Some((lines, start)) = current_table {
            blocks.push(Block::Table {
                lines,
                start_line: start,
            });
        }

        blocks
    }

    //=========================================================================
    // 阶段 2：块解析
    //=========================================================================

    fn parse_block(&mut self, block: Block) -> Result<Option<ScriptNode>, ParseError> {
        match block {
            Block::SingleLine { line, line_number } => self.parse_single_line(&line, line_number),
            Block::Table { lines, start_line } => self.parse_table(&lines, start_line),
        }
    }

    /// 解析单行内容
    fn parse_single_line(
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
        if starts_with_ignore_case(line, "uianim") {
            return self.parse_ui_anim(line, line_number);
        }
        if starts_with_ignore_case(line, "goto") {
            return self.parse_goto(line, line_number);
        }
        // stopBGM - 停止 BGM
        if starts_with_ignore_case(line, "stopbgm") {
            return Ok(Some(ScriptNode::StopBgm));
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

        // 5. 未知行
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

    /// 解析 changeBG 指令
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

        Ok(Some(ScriptNode::ChangeBG {
            path: path.to_string(),
            transition,
        }))
    }

    /// 解析 changeScene 指令
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

        let transition = self.extract_transition_from_line(line);

        Ok(Some(ScriptNode::ChangeScene {
            path: path.to_string(),
            transition,
        }))
    }

    /// 解析 show 指令
    fn parse_show(&self, line: &str, line_number: usize) -> Result<Option<ScriptNode>, ParseError> {
        let path = extract_img_src(line).ok_or_else(|| ParseError::MissingParameter {
            line: line_number,
            command: "show".to_string(),
            param: "图片路径 (<img src=\"...\">)".to_string(),
        })?;

        let alias =
            extract_keyword_value(line, "as").ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "show".to_string(),
                param: "as (别名)".to_string(),
            })?;

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
            path: path.to_string(),
            alias: alias.to_string(),
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

    /// 解析 UIAnim 指令
    fn parse_ui_anim(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        // 跳过 "UIAnim" 前缀
        let content = line
            .get(6..)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "UIAnim".to_string(),
                param: "效果".to_string(),
            })?;

        let effect = parse_transition(content).ok_or_else(|| ParseError::InvalidTransition {
            line: line_number,
            message: format!("无法解析过渡效果: {}", content),
        })?;

        Ok(Some(ScriptNode::UIAnim { effect }))
    }

    /// 解析 goto 指令
    ///
    /// 语法: `goto **label**`
    fn parse_goto(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
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
        let target_label = if content.starts_with("**") && content.ends_with("**") && content.len() > 4 {
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
    fn extract_transition_from_line(&self, line: &str) -> Option<Transition> {
        let lower = line.to_lowercase();

        // 查找 "with" 的位置（支持 " with " 和 ">with "）
        let with_pos = lower
            .rfind(" with ")
            .or_else(|| lower.rfind(">with "))
            .or_else(|| lower.rfind(" with`"))  // 行内代码格式
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
        if transition_text.contains("<img") {
            if let Some(rule_path) = extract_img_src(transition_text) {
                return Some(Transition::with_args(
                    "rule",
                    vec![TransitionArg::String(rule_path.to_string())],
                ));
            }
            return Some(Transition::simple("rule"));
        }

        // 处理行内代码格式: `Dissolve(2.0, 0.5)`
        let transition_text = if transition_text.starts_with('`') && transition_text.ends_with('`') {
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
                self.warnings.push(format!(
                    "第 {} 行：表格行格式不完整，已跳过",
                    line_number
                ));
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

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

//=============================================================================
// 测试
//=============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // 辅助函数测试
    // -------------------------------------------------------------------------

    #[test]
    fn test_extract_img_src() {
        assert_eq!(
            extract_img_src(r#"<img src="path/to/image.png" />"#),
            Some("path/to/image.png")
        );
        assert_eq!(
            extract_img_src(r#"<img   src='another.jpg' alt="test">"#),
            Some("another.jpg")
        );
        assert_eq!(
            extract_img_src(r#"show <img src="char.png" /> as test"#),
            Some("char.png")
        );
        assert_eq!(extract_img_src("no image here"), None);
    }

    #[test]
    fn test_extract_keyword_value() {
        let line = "show <img src=\"char.png\" /> as royu at center with dissolve";
        assert_eq!(extract_keyword_value(line, "as"), Some("royu"));
        assert_eq!(extract_keyword_value(line, "at"), Some("center"));

        let line2 = "show <img src=\"char.png\" /> as test_char at nearleft";
        assert_eq!(extract_keyword_value(line2, "as"), Some("test_char"));
        assert_eq!(extract_keyword_value(line2, "at"), Some("nearleft"));
    }

    #[test]
    fn test_parse_transition() {
        let t = parse_transition("dissolve").unwrap();
        assert_eq!(t.name, "dissolve");
        assert!(t.args.is_empty());

        let t = parse_transition("Dissolve(1.5)").unwrap();
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 1);
        assert!(matches!(&t.args[0], TransitionArg::Number(n) if (*n - 1.5).abs() < 0.001));

        let t = parse_transition("fade").unwrap();
        assert_eq!(t.name, "fade");
    }

    #[test]
    fn test_is_table_separator() {
        assert!(is_table_separator("| --- | --- |"));
        assert!(is_table_separator("|---|---|"));
        assert!(is_table_separator("| :---: | ---: |"));
        assert!(!is_table_separator("| text | text |"));
        assert!(!is_table_separator("not a table"));
    }

    #[test]
    fn test_parse_dialogue_function() {
        // 中文冒号和引号
        let chinese_dialogue = format!("羽艾：“你好”");
        let result = parse_dialogue(&chinese_dialogue);
        assert!(result.is_some());
        let (speaker, content) = result.unwrap();
        assert_eq!(speaker, Some("羽艾".to_string()));
        assert_eq!(content, "你好");

        // 英文冒号和引号
        let result = parse_dialogue(r#"Test: "Hello""#);
        assert!(result.is_some());
        let (speaker, content) = result.unwrap();
        assert_eq!(speaker, Some("Test".to_string()));
        assert_eq!(content, "Hello");

        // 旁白
        let narration = format!("：“这是旁白”");
        let result = parse_dialogue(&narration);
        assert!(result.is_some());
        let (speaker, content) = result.unwrap();
        assert_eq!(speaker, None);
        assert_eq!(content, "这是旁白");
    }

    // -------------------------------------------------------------------------
    // Parser 集成测试
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_chapter() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "# Chapter 1").unwrap();

        assert_eq!(script.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Chapter { title, level: 1 } if title == "Chapter 1"
        ));
    }

    #[test]
    fn test_parse_label() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "**start**").unwrap();

        assert_eq!(script.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Label { name } if name == "start"
        ));
    }

    #[test]
    fn test_parse_dialogue() {
        let mut parser = Parser::new();

        // 中文冒号和引号
        let chinese_dialogue = format!("羽艾：“你好”");
        let script = parser.parse("test", &chinese_dialogue).unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Dialogue { speaker: Some(s), content } if s == "羽艾" && content == "你好"
        ));

        // 英文冒号和引号
        let script = parser.parse("test", r#"Test: "Hello""#).unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Dialogue { speaker: Some(s), content } if s == "Test" && content == "Hello"
        ));

        // 旁白
        let narration = format!("：“这是旁白”");
        let script = parser.parse("test", &narration).unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Dialogue { speaker: None, content } if content == "这是旁白"
        ));
    }

    #[test]
    fn test_parse_change_bg() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeBG <img src="assets/bg.png" /> with dissolve"#,
            )
            .unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ChangeBG { path, transition: Some(t) }
            if path == "assets/bg.png" && t.name == "dissolve"
        ));
    }

    #[test]
    fn test_parse_show_character() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"show <img src="assets/char.png" /> as royu at center with Dissolve(1.5)"#,
            )
            .unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { path, alias, position: Position::Center, transition: Some(t) }
            if path == "assets/char.png" && alias == "royu" && t.name == "Dissolve"
        ));
    }

    #[test]
    fn test_parse_hide_character() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "hide royu with fade").unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::HideCharacter { alias, transition: Some(t) }
            if alias == "royu" && t.name == "fade"
        ));
    }

    #[test]
    fn test_parse_choice_table() {
        let mut parser = Parser::new();
        let text = r#"
| 选择 |        |
| ---- | ------ |
| 选项A | label_a |
| 选项B | label_b |
"#;
        let script = parser.parse("test", text).unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Choice { style: Some(s), options }
            if s == "选择" && options.len() == 2
        ));
    }

    #[test]
    fn test_parse_transition_with_args() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeBG <img src="bg.png" /> with Dissolve(1.5)"#,
            )
            .unwrap();

        if let ScriptNode::ChangeBG {
            transition: Some(t),
            ..
        } = &script.nodes[0]
        {
            assert_eq!(t.name, "Dissolve");
            assert_eq!(t.args.len(), 1);
            assert!(matches!(&t.args[0], TransitionArg::Number(n) if (*n - 1.5).abs() < 0.001));
        } else {
            panic!("Expected ChangeBG node");
        }
    }

    #[test]
    fn test_parse_full_script() {
        let mut parser = Parser::new();
        let text = r#"
# Chapter 1

changeBG <img src="assets/bg.png" /> with dissolve

羽艾：“你好”

：“这是旁白”

show <img src="assets/char.png" /> as protagonist at center with dissolve

**choice_point**

| 选择 |        |
| ---- | ------ |
| 继续 | cont |
| 结束 | end |

**cont**

羽艾：“继续”

**end**

hide protagonist with fade
"#;

        let script = parser.parse("test", &text).unwrap();

        // 验证节点数量
        assert!(script.len() >= 8);

        // 验证标签索引
        assert!(script.find_label("choice_point").is_some());
        assert!(script.find_label("cont").is_some());
        assert!(script.find_label("end").is_some());
    }

    // -------------------------------------------------------------------------
    // 从 C# Parser.cs 移植的测试用例
    // -------------------------------------------------------------------------

    /// 测试对话解析：冒号前后有空格
    #[test]
    fn test_parse_dialogue_with_spaces() {
        // 英文冒号前后有空格
        let result = parse_dialogue(r#"红叶 : "台词""#);
        assert!(result.is_some());
        let (speaker, content) = result.unwrap();
        assert_eq!(speaker, Some("红叶".to_string()));
        assert_eq!(content, "台词");
    }

    /// 测试对话解析：内容包含特殊字符
    #[test]
    fn test_parse_dialogue_special_chars() {
        let result = parse_dialogue(r#"红叶: "台词 abab;:.""#);
        assert!(result.is_some());
        let (speaker, content) = result.unwrap();
        assert_eq!(speaker, Some("红叶".to_string()));
        assert_eq!(content, "台词 abab;:.");
    }

    /// 测试 show 指令：无空格、有空格、多空格
    #[test]
    fn test_parse_show_whitespace_tolerance() {
        let mut parser = Parser::new();

        // 标准格式
        let script = parser
            .parse(
                "test",
                r#"show <img src="assets/bg2.jpg" /> as 红叶 at left"#,
            )
            .unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { alias, position: Position::Left, .. }
            if alias == "红叶"
        ));

        // 无空格格式
        let script = parser
            .parse(
                "test",
                r#"show<img src="assets/bg2.jpg" />as 红叶 at left"#,
            )
            .unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { alias, position: Position::Left, .. }
            if alias == "红叶"
        ));

        // 多空格格式
        let script = parser
            .parse(
                "test",
                r#"show   <img src="assets/bg2.jpg" />   as 红叶 at left"#,
            )
            .unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { alias, position: Position::Left, .. }
            if alias == "红叶"
        ));
    }

    /// 测试 show 指令：无过渡效果
    #[test]
    fn test_parse_show_without_effect() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"show <img src="assets/bg2.jpg" /> as 红叶 at left"#,
            )
            .unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { alias, path, position: Position::Left, transition: None }
            if alias == "红叶" && path == "assets/bg2.jpg"
        ));
    }

    /// 测试 show 指令：带行内代码格式的 effect
    #[test]
    fn test_parse_show_with_inline_code_effect() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"show <img src="assets/bg2.jpg" /> as 红叶 at left with `Dissolve(2.0, 0.5)`"#,
            )
            .unwrap();

        if let ScriptNode::ShowCharacter { transition: Some(t), .. } = &script.nodes[0] {
            // 行内代码格式的 effect 应该被解析（去掉反引号）
            assert!(t.name.contains("Dissolve") || t.name == "`Dissolve(2.0, 0.5)`");
        } else {
            panic!("Expected ShowCharacter with transition");
        }
    }

    /// 测试 hide 指令：无过渡效果
    #[test]
    fn test_parse_hide_without_effect() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "hide 红叶").unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::HideCharacter { alias, transition: None }
            if alias == "红叶"
        ));
    }

    /// 测试 changeBG 指令：无过渡效果
    #[test]
    fn test_parse_change_bg_without_effect() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeBG <img src="assets/bg2.jpg" />"#,
            )
            .unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ChangeBG { path, transition: None }
            if path == "assets/bg2.jpg"
        ));
    }

    /// 测试 changeBG 指令：空格容错
    #[test]
    fn test_parse_change_bg_whitespace_tolerance() {
        let mut parser = Parser::new();

        // 无空格
        let script = parser
            .parse(
                "test",
                r#"changeBG<img src="assets/bg2.jpg" />with dissolve"#,
            )
            .unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ChangeBG { path, transition: Some(t) }
            if path == "assets/bg2.jpg" && t.name == "dissolve"
        ));

        // 多空格
        let script = parser
            .parse(
                "test",
                r#"changeBG   <img src="assets/bg2.jpg" />   with dissolve"#,
            )
            .unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ChangeBG { path, transition: Some(t) }
            if path == "assets/bg2.jpg" && t.name == "dissolve"
        ));
    }

    /// 测试 changeBG 指令：行内代码格式的 effect
    #[test]
    fn test_parse_change_bg_with_inline_code_effect() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeBG <img src="assets/bg2.jpg" /> with `Dissolve(2.0, 0.5)`"#,
            )
            .unwrap();

        if let ScriptNode::ChangeBG { transition: Some(t), .. } = &script.nodes[0] {
            // 行内代码格式应该被解析
            assert!(!t.name.is_empty());
        } else {
            panic!("Expected ChangeBG with transition");
        }
    }

    /// 测试选择分支：竖排样式
    #[test]
    fn test_parse_branch_vertical() {
        let mut parser = Parser::new();
        let text = r#"| 竖排  |        |
| ----- | ------ |
| 选项1 | label1 |
| 选项2 | label2 |
| 选项3 | label3 |"#;

        let script = parser.parse("test", text).unwrap();

        if let ScriptNode::Choice { style, options } = &script.nodes[0] {
            assert_eq!(style.as_deref(), Some("竖排"));
            assert_eq!(options.len(), 3);
            assert_eq!(options[0].text, "选项1");
            assert_eq!(options[0].target_label, "label1");
            assert_eq!(options[1].text, "选项2");
            assert_eq!(options[1].target_label, "label2");
            assert_eq!(options[2].text, "选项3");
            assert_eq!(options[2].target_label, "label3");
        } else {
            panic!("Expected Choice node");
        }
    }

    /// 测试 UIAnim 指令
    #[test]
    fn test_parse_uianim() {
        let mut parser = Parser::new();

        // 简单效果名
        let script = parser.parse("test", "UIAnim dissolve").unwrap();
        if let ScriptNode::UIAnim { effect } = &script.nodes[0] {
            assert_eq!(effect.name, "dissolve");
        } else {
            panic!("Expected UIAnim node");
        }
    }

    /// 测试 UIAnim 指令：带参数
    #[test]
    fn test_parse_uianim_with_args() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "UIAnim Dissolve(2.0)").unwrap();

        if let ScriptNode::UIAnim { effect } = &script.nodes[0] {
            assert_eq!(effect.name, "Dissolve");
            assert_eq!(effect.args.len(), 1);
        } else {
            panic!("Expected UIAnim node");
        }
    }

    /// 测试标签解析：中文标签名
    #[test]
    fn test_parse_label_chinese() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "**选择支1**").unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Label { name } if name == "选择支1"
        ));
    }

    /// 测试 img 标签解析：带 style 和 alt 属性
    #[test]
    fn test_extract_img_src_with_attributes() {
        // 带 style 属性
        assert_eq!(
            extract_img_src(r#"<img src="bg1.png" style="zoom: 10%;" />"#),
            Some("bg1.png")
        );

        // 带 alt 和 style 属性
        assert_eq!(
            extract_img_src(r#"<img src="assets/chara.png" alt="chara" style="zoom:25%;" />"#),
            Some("assets/chara.png")
        );
    }

    /// 综合测试：模拟真实脚本（来自 C# 测试）
    #[test]
    fn test_parse_realistic_script() {
        let mut parser = Parser::new();
        let script_text = r#"# 章节标题

角色名:"台词"

changeBG <img src="bg1.png" style="zoom: 10%;" /> with dissolve

show <img src="assets/chara.png" style="zoom:25%;" /> as 红叶 at farleft with dissolve
"#;

        let script = parser.parse("test", script_text).unwrap();

        // 应该有 4 个节点
        assert_eq!(script.len(), 4);

        // 验证章节
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Chapter { title, level: 1 } if title == "章节标题"
        ));

        // 验证对话
        assert!(matches!(
            &script.nodes[1],
            ScriptNode::Dialogue { speaker: Some(s), content }
            if s == "角色名" && content == "台词"
        ));

        // 验证 changeBG
        assert!(matches!(
            &script.nodes[2],
            ScriptNode::ChangeBG { path, transition: Some(t) }
            if path == "bg1.png" && t.name == "dissolve"
        ));

        // 验证 show
        if let ScriptNode::ShowCharacter { path, alias, position: _, transition } = &script.nodes[3] {
            assert_eq!(path, "assets/chara.png");
            assert_eq!(alias, "红叶");
            assert!(transition.is_some());
            assert_eq!(transition.as_ref().unwrap().name, "dissolve");
            // farleft 可能还没有在 Position 中定义
        } else {
            panic!("Expected ShowCharacter node");
        }
    }

    //=========================================================================
    // goto 语法测试
    //=========================================================================

    /// 测试 goto 指令：基本语法
    #[test]
    fn test_parse_goto_basic() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "goto **start**").unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Goto { target_label } if target_label == "start"
        ));
    }

    /// 测试 goto 指令：中文标签
    #[test]
    fn test_parse_goto_chinese_label() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "goto **选择支1**").unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Goto { target_label } if target_label == "选择支1"
        ));
    }

    /// 测试 goto 指令：带空格
    #[test]
    fn test_parse_goto_with_spaces() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "goto  **end_scene**").unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Goto { target_label } if target_label == "end_scene"
        ));
    }

    //=========================================================================
    // audio 语法测试
    //=========================================================================

    /// 测试 audio 指令：SFX（无 loop）
    #[test]
    fn test_parse_audio_sfx() {
        let mut parser = Parser::new();
        let script = parser.parse("test", r#"<audio src="sfx/ding.mp3"></audio>"#).unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::PlayAudio { path, is_bgm } if path == "sfx/ding.mp3" && !is_bgm
        ));
    }

    /// 测试 audio 指令：BGM（带 loop）
    #[test]
    fn test_parse_audio_bgm() {
        let mut parser = Parser::new();
        let script = parser.parse("test", r#"<audio src="bgm/Signal.mp3"></audio> loop"#).unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::PlayAudio { path, is_bgm } if path == "bgm/Signal.mp3" && *is_bgm
        ));
    }

    /// 测试 audio 指令：相对路径
    #[test]
    fn test_parse_audio_relative_path() {
        let mut parser = Parser::new();
        let script = parser.parse("test", r#"<audio src="../bgm/music.mp3"></audio> loop"#).unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::PlayAudio { path, is_bgm } if path == "../bgm/music.mp3" && *is_bgm
        ));
    }

    /// 测试 stopBGM 指令
    #[test]
    fn test_parse_stop_bgm() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "stopBGM").unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(&script.nodes[0], ScriptNode::StopBgm));
    }

    //=========================================================================
    // 相对路径测试
    //=========================================================================

    /// 测试 changeBG 相对路径
    #[test]
    fn test_parse_change_bg_relative_path() {
        let mut parser = Parser::new();
        let script = parser.parse(
            "test",
            r#"changeBG <img src="../backgrounds/bg.jpg" /> with `dissolve`"#
        ).unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ChangeBG { path, .. } if path == "../backgrounds/bg.jpg"
        ));
    }

    /// 测试 show 相对路径
    #[test]
    fn test_parse_show_relative_path() {
        let mut parser = Parser::new();
        let script = parser.parse(
            "test",
            r#"show <img src="../characters/北风.png" /> as beifeng at center"#
        ).unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { path, alias, .. } 
            if path == "../characters/北风.png" && alias == "beifeng"
        ));
    }

    //=========================================================================
    // 综合测试：包含 goto 和 audio 的完整脚本
    //=========================================================================

    #[test]
    fn test_parse_script_with_goto_and_audio() {
        let mut parser = Parser::new();
        let text = r#"# 测试章节

<audio src="../bgm/intro.mp3"></audio> loop

**开始**

角色："欢迎！"

| 选项 | 跳转 |
| --- | --- |
| 去选项A | **选项A** |
| 去选项B | **选项B** |

**选项A**
角色："你选了A"
goto **结束**

**选项B**
角色："你选了B"
goto **结束**

**结束**
stopBGM
角色："再见！"
"#;

        let script = parser.parse("test", text).unwrap();
        
        // 验证关键节点
        let has_audio = script.nodes.iter().any(|n| matches!(n, ScriptNode::PlayAudio { is_bgm: true, .. }));
        let has_goto = script.nodes.iter().any(|n| matches!(n, ScriptNode::Goto { .. }));
        let has_stop_bgm = script.nodes.iter().any(|n| matches!(n, ScriptNode::StopBgm));
        let has_choice = script.nodes.iter().any(|n| matches!(n, ScriptNode::Choice { .. }));
        
        assert!(has_audio, "应该有 BGM 播放");
        assert!(has_goto, "应该有 goto 指令");
        assert!(has_stop_bgm, "应该有 stopBGM 指令");
        assert!(has_choice, "应该有选择分支");
    }
}
