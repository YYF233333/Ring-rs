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
use crate::script::Expr;
use crate::script::ast::{ChoiceOption, ConditionalBranch, Script, ScriptNode};

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
        format!(" {} ", keyword_lower), // 标准：空格包围
        format!(">{} ", keyword_lower), // 紧跟 >：如 />as
        format!(" {}", keyword_lower),  // 只有前空格
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
    if value.is_empty() { None } else { Some(value) }
}

/// 解析过渡效果表达式
///
/// 输入: `"dissolve"` 或 `"Dissolve(1.5)"` 或 `"Dissolve(duration: 1.5)"`
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

        match parse_transition_args(args_str) {
            Ok(args) => Some(Transition::with_named_args(name, args)),
            Err(_) => None, // 参数解析失败
        }
    } else {
        // 简单名称格式: name
        Some(Transition::simple(s))
    }
}

/// 解析过渡效果参数列表（支持位置参数和命名参数，不允许混用）
///
/// - 位置参数: `1.0, 0.5, true, "x"`
/// - 命名参数: `duration: 1.0, reversed: true`
/// - 混用会返回 Err
fn parse_transition_args(s: &str) -> Result<Vec<(Option<String>, TransitionArg)>, String> {
    let s = s.trim();
    if s.is_empty() {
        return Ok(Vec::new());
    }

    // 先分割参数（考虑字符串内的逗号）
    let raw_args = split_args(s);
    if raw_args.is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    let mut has_named = false;
    let mut has_positional = false;
    let mut seen_keys = std::collections::HashSet::new();

    for raw in &raw_args {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }

        // 检测是否是命名参数（包含 key: value 模式）
        // 注意：字符串里的冒号不算
        if let Some((key, value)) = parse_named_arg(raw) {
            // 命名参数
            if has_positional {
                return Err("不允许混用位置参数和命名参数".to_string());
            }
            has_named = true;

            // 检查重复 key
            if seen_keys.contains(&key) {
                return Err(format!("重复的命名参数: {}", key));
            }
            seen_keys.insert(key.clone());

            args.push((Some(key), value));
        } else {
            // 位置参数
            if has_named {
                return Err("不允许混用位置参数和命名参数".to_string());
            }
            has_positional = true;

            args.push((None, parse_arg_value(raw)));
        }
    }

    Ok(args)
}

/// 分割参数列表（考虑字符串内的逗号）
fn split_args(s: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut string_char = '"';

    for ch in s.chars() {
        if in_string {
            current.push(ch);
            if ch == string_char {
                in_string = false;
            }
        } else if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
            current.push(ch);
        } else if ch == ',' {
            result.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }

    // 最后一个参数
    let last = current.trim();
    if !last.is_empty() {
        result.push(last.to_string());
    }

    result
}

/// 尝试解析命名参数 "key: value"
/// 返回 Some((key, value)) 或 None（如果不是命名参数）
fn parse_named_arg(s: &str) -> Option<(String, TransitionArg)> {
    // 查找第一个不在字符串内的冒号
    let mut in_string = false;
    let mut string_char = '"';
    let mut colon_pos = None;

    for (i, ch) in s.char_indices() {
        if in_string {
            if ch == string_char {
                in_string = false;
            }
        } else if ch == '"' || ch == '\'' {
            in_string = true;
            string_char = ch;
        } else if ch == ':' {
            colon_pos = Some(i);
            break;
        }
    }

    let colon_pos = colon_pos?;
    let key = s[..colon_pos].trim();
    let value_str = s[colon_pos + 1..].trim();

    // key 必须是有效标识符 [a-zA-Z_][a-zA-Z0-9_]*
    if !is_valid_identifier(key) {
        return None;
    }

    // 解析 value
    let value = parse_arg_value(value_str);
    Some((key.to_string(), value))
}

/// 检查是否是有效标识符
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// 解析参数值（可能带引号的字符串、数字、布尔值）
fn parse_arg_value(s: &str) -> TransitionArg {
    let s = s.trim();

    // 带引号的字符串
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 {
            return TransitionArg::String(s[1..s.len() - 1].to_string());
        }
    }

    parse_single_arg(s)
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
    const ASCII_QUOTE: char = '"'; // U+0022
    const CN_LEFT_QUOTE: char = '\u{201C}'; // 中文左双引号
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
// 表达式解析
//=============================================================================

/// 解析表达式字符串
///
/// 支持的语法:
/// - 字面量: `"string"`, `true`, `false`
/// - 变量: `$var_name`
/// - 比较: `$var == "value"`, `$var != "value"`
/// - 逻辑: `expr and expr`, `expr or expr`, `not expr`
/// - 括号: `(expr)`
fn parse_expression(input: &str, line_number: usize) -> Result<Expr, ParseError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ParseError::InvalidLine {
            line: line_number,
            message: "空表达式".to_string(),
        });
    }

    // 使用简单的递归下降解析器
    let mut parser = ExprParser::new(input, line_number);
    let expr = parser.parse_or()?;
    parser.skip_whitespace();
    if !parser.remaining().is_empty() {
        return Err(ParseError::InvalidLine {
            line: line_number,
            message: format!("表达式末尾存在无法解析的内容: '{}'", parser.remaining()),
        });
    }
    Ok(expr)
}

/// 表达式解析器
struct ExprParser<'a> {
    input: &'a str,
    pos: usize,
    line_number: usize,
}

impl<'a> ExprParser<'a> {
    fn new(input: &'a str, line_number: usize) -> Self {
        Self {
            input,
            pos: 0,
            line_number,
        }
    }

    fn remaining(&self) -> &str {
        &self.input[self.pos..]
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn starts_with_keyword(&self, keyword: &str) -> bool {
        let remaining = self.remaining().to_lowercase();
        if remaining.starts_with(&keyword.to_lowercase()) {
            // 确保后面是空白或结束
            let after = &self.input[self.pos + keyword.len()..];
            after.is_empty()
                || after.starts_with(char::is_whitespace)
                || after.starts_with('(')
                || after.starts_with(')')
        } else {
            false
        }
    }

    fn consume_keyword(&mut self, keyword: &str) {
        self.pos += keyword.len();
        self.skip_whitespace();
    }

    /// 解析 or 表达式（最低优先级）
    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;

        loop {
            self.skip_whitespace();
            if self.starts_with_keyword("or") {
                self.consume_keyword("or");
                let right = self.parse_and()?;
                left = Expr::or(left, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// 解析 and 表达式
    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not()?;

        loop {
            self.skip_whitespace();
            if self.starts_with_keyword("and") {
                self.consume_keyword("and");
                let right = self.parse_not()?;
                left = Expr::and(left, right);
            } else {
                break;
            }
        }

        Ok(left)
    }

    /// 解析 not 表达式
    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        self.skip_whitespace();
        if self.starts_with_keyword("not") {
            self.consume_keyword("not");
            let expr = self.parse_not()?;
            Ok(Expr::not(expr))
        } else {
            self.parse_comparison()
        }
    }

    /// 解析比较表达式
    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_primary()?;

        self.skip_whitespace();

        // 检查比较运算符
        if self.remaining().starts_with("==") {
            self.pos += 2;
            self.skip_whitespace();
            let right = self.parse_primary()?;
            Ok(Expr::eq(left, right))
        } else if self.remaining().starts_with("!=") {
            self.pos += 2;
            self.skip_whitespace();
            let right = self.parse_primary()?;
            Ok(Expr::not_eq(left, right))
        } else {
            Ok(left)
        }
    }

    /// 解析基本表达式
    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        self.skip_whitespace();

        let c = self.peek_char().ok_or_else(|| ParseError::InvalidLine {
            line: self.line_number,
            message: "表达式意外结束".to_string(),
        })?;

        match c {
            // 括号
            '(' => {
                self.consume_char();
                let expr = self.parse_or()?;
                self.skip_whitespace();
                if self.peek_char() != Some(')') {
                    return Err(ParseError::InvalidLine {
                        line: self.line_number,
                        message: "缺少右括号 ')'".to_string(),
                    });
                }
                self.consume_char();
                Ok(expr)
            }

            // 变量
            '$' => {
                self.consume_char();
                let name = self.parse_identifier()?;
                Ok(Expr::var(name))
            }

            // 字符串字面量
            '"' => {
                let s = self.parse_string_literal('"')?;
                Ok(Expr::string(s))
            }
            '\'' => {
                let s = self.parse_string_literal('\'')?;
                Ok(Expr::string(s))
            }

            // 布尔字面量或数字
            _ => {
                if self.starts_with_keyword("true") {
                    self.consume_keyword("true");
                    Ok(Expr::bool(true))
                } else if self.starts_with_keyword("false") {
                    self.consume_keyword("false");
                    Ok(Expr::bool(false))
                } else if c.is_ascii_digit() || c == '-' {
                    // 尝试解析数字
                    let num = self.parse_number()?;
                    Ok(Expr::int(num))
                } else {
                    Err(ParseError::InvalidLine {
                        line: self.line_number,
                        message: format!("无法解析表达式，意外字符: '{}'", c),
                    })
                }
            }
        }
    }

    /// 解析标识符
    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c.is_alphanumeric() || c == '_' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }

        if self.pos == start {
            return Err(ParseError::InvalidLine {
                line: self.line_number,
                message: "期望标识符".to_string(),
            });
        }

        Ok(self.input[start..self.pos].to_string())
    }

    /// 解析字符串字面量
    fn parse_string_literal(&mut self, quote: char) -> Result<String, ParseError> {
        self.consume_char(); // 消费开始引号
        let start = self.pos;

        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c == quote {
                let s = self.input[start..self.pos].to_string();
                self.consume_char(); // 消费结束引号
                return Ok(s);
            }
            self.pos += c.len_utf8();
        }

        Err(ParseError::InvalidLine {
            line: self.line_number,
            message: format!("字符串字面量未闭合，缺少 '{}'", quote),
        })
    }

    /// 解析数字
    fn parse_number(&mut self) -> Result<i64, ParseError> {
        let start = self.pos;

        // 处理负号
        if self.peek_char() == Some('-') {
            self.consume_char();
        }

        while self.pos < self.input.len() {
            let c = self.input[self.pos..].chars().next().unwrap();
            if c.is_ascii_digit() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }

        let num_str = &self.input[start..self.pos];
        num_str.parse::<i64>().map_err(|_| ParseError::InvalidLine {
            line: self.line_number,
            message: format!("无法解析数字: '{}'", num_str),
        })
    }
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
    Table {
        lines: Vec<String>,
        start_line: usize,
    },
    /// 条件块（if/elseif/else/endif）
    Conditional {
        /// 原始行列表 (line, line_number)
        lines: Vec<(String, usize)>,
        start_line: usize,
    },
}

impl Block {
    /// 获取块的起始行号
    fn start_line(&self) -> usize {
        match self {
            Block::SingleLine { line_number, .. } => *line_number,
            Block::Table { start_line, .. } => *start_line,
            Block::Conditional { start_line, .. } => *start_line,
        }
    }
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

        // 阶段 2：块解析（同时收集行号）
        let mut nodes = Vec::new();
        let mut source_map = Vec::new();
        for block in blocks {
            let line_number = block.start_line();
            match self.parse_block(block) {
                Ok(Some(node)) => {
                    nodes.push(node);
                    source_map.push(line_number);
                }
                Ok(None) => {} // 跳过（如空内容）
                Err(e) => return Err(e),
            }
        }

        Ok(Script::with_source_map(
            script_id, nodes, base_path, source_map,
        ))
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
        let mut current_conditional: Option<(Vec<(String, usize)>, usize, usize)> = None; // (lines, start_line, depth)

        for (line_idx, line) in text.lines().enumerate() {
            let line_number = line_idx + 1;
            let trimmed = line.trim();

            // 检查是否是条件语句
            let is_if = starts_with_ignore_case(trimmed, "if ");
            let is_endif = trimmed.eq_ignore_ascii_case("endif");

            // 处理条件块
            if let Some((ref mut lines, _start, ref mut depth)) = current_conditional {
                // 嵌套 if
                if is_if {
                    *depth += 1;
                }

                lines.push((trimmed.to_string(), line_number));

                // endif
                if is_endif {
                    if *depth > 0 {
                        *depth -= 1;
                    } else {
                        // 条件块结束
                        let (lines, start, _) = current_conditional.take().unwrap();
                        blocks.push(Block::Conditional {
                            lines,
                            start_line: start,
                        });
                    }
                }
                continue;
            }

            // 开始新的条件块
            if is_if {
                // 先结束任何打开的表格块
                if let Some((tbl_lines, tbl_start)) = current_table.take() {
                    blocks.push(Block::Table {
                        lines: tbl_lines,
                        start_line: tbl_start,
                    });
                }

                current_conditional =
                    Some((vec![(trimmed.to_string(), line_number)], line_number, 0));
                continue;
            }

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

        // 处理未闭合的条件块（添加到 blocks 以便在 parse_block 阶段报错）
        if let Some((lines, start, _depth)) = current_conditional {
            blocks.push(Block::Conditional {
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
            Block::Conditional { lines, start_line } => self.parse_conditional(&lines, start_line),
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
    fn extract_transition_from_line(&self, line: &str) -> Option<Transition> {
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
                let args = self.extract_rule_args(transition_text, &rule_path);
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
            if let Some(paren_start) = after_img.find('(') {
                if let Some(paren_end) = after_img.rfind(')') {
                    let params_str = &after_img[paren_start + 1..paren_end];
                    // 解析参数（命名参数格式）
                    if let Ok(parsed_args) = parse_transition_args(params_str) {
                        args.extend(parsed_args);
                    }
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
    fn test_extract_img_src_requires_quoted_value() {
        // 覆盖：quote_char 不是 ' 或 " 时返回 None
        assert_eq!(extract_img_src(r#"<img src=path/to/image.png />"#), None);
        assert_eq!(extract_img_src(r#"<img src= path/to/image.png />"#), None);
    }

    #[test]
    fn test_extract_audio_src_and_requires_quoted_value() {
        assert_eq!(
            extract_audio_src(r#"<audio src="bgm.mp3"></audio>"#),
            Some("bgm.mp3")
        );
        // 覆盖：quote_char 校验失败分支
        assert_eq!(extract_audio_src(r#"<audio src=bgm.mp3></audio>"#), None);
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
    fn test_extract_keyword_value_edge_cases() {
        // 覆盖：value_start 越界 / value 为空
        assert_eq!(
            extract_keyword_value("show <img src=\"x\" /> as", "as"),
            None
        );

        // 覆盖：">as " 模式（无空格紧跟 img 标签结束）
        let line = "show <img src=\"char.png\" />as royu at center";
        assert_eq!(extract_keyword_value(line, "as"), Some("royu"));
    }

    #[test]
    fn test_parse_transition() {
        let t = parse_transition("dissolve").unwrap();
        assert_eq!(t.name, "dissolve");
        assert!(t.args.is_empty());

        // 位置参数
        let t = parse_transition("Dissolve(1.5)").unwrap();
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 1);
        assert!(matches!(&t.args[0], (None, TransitionArg::Number(n)) if (*n - 1.5).abs() < 0.001));

        let t = parse_transition("fade").unwrap();
        assert_eq!(t.name, "fade");
    }

    #[test]
    fn test_parse_transition_invalid_format_returns_none() {
        // 缺失右括号
        assert_eq!(parse_transition("Dissolve(1.0"), None);
        // 参数解析失败（混用位置/命名）-> parse_transition 内部吞掉 Err，返回 None
        assert_eq!(parse_transition("Dissolve(1.0, duration: 2.0)"), None);
    }

    #[test]
    fn test_parse_transition_named_args() {
        // 命名参数
        let t = parse_transition("Dissolve(duration: 1.5)").unwrap();
        assert_eq!(t.name, "Dissolve");
        assert_eq!(t.args.len(), 1);
        assert!(
            matches!(&t.args[0], (Some(key), TransitionArg::Number(n)) if key == "duration" && (*n - 1.5).abs() < 0.001)
        );
        assert!(t.is_all_named());

        // 多个命名参数
        let t = parse_transition("Fade(duration: 2.0, reversed: true)").unwrap();
        assert_eq!(t.name, "Fade");
        assert_eq!(t.args.len(), 2);
        assert!(
            matches!(&t.args[0], (Some(key), TransitionArg::Number(n)) if key == "duration" && (*n - 2.0).abs() < 0.001)
        );
        assert!(matches!(&t.args[1], (Some(key), TransitionArg::Bool(true)) if key == "reversed"));

        // 辅助方法测试
        assert_eq!(t.get_duration(), Some(2.0));
        assert_eq!(t.get_reversed(), Some(true));
    }

    #[test]
    fn test_parse_transition_positional_args() {
        // 多个位置参数
        let t = parse_transition("Effect(1.0, 0.5, true, \"test\")").unwrap();
        assert_eq!(t.name, "Effect");
        assert_eq!(t.args.len(), 4);
        assert!(t.is_all_positional());
        assert!(matches!(&t.args[0], (None, TransitionArg::Number(n)) if (*n - 1.0).abs() < 0.001));
        assert!(matches!(&t.args[1], (None, TransitionArg::Number(n)) if (*n - 0.5).abs() < 0.001));
        assert!(matches!(&t.args[2], (None, TransitionArg::Bool(true))));
        assert!(matches!(&t.args[3], (None, TransitionArg::String(s)) if s == "test"));

        // 辅助方法测试（位置参数回退）
        assert_eq!(t.get_positional(0), Some(&TransitionArg::Number(1.0)));
    }

    #[test]
    fn test_parse_transition_mixed_args_error() {
        // 混用位置参数和命名参数应该失败
        let result = parse_transition_args("1.0, duration: 2.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不允许混用"));

        let result = parse_transition_args("duration: 2.0, 1.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不允许混用"));
    }

    #[test]
    fn test_parse_transition_duplicate_key_error() {
        // 重复的命名参数 key 应该失败
        let result = parse_transition_args("duration: 1.0, duration: 2.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("重复的命名参数"));
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
    fn test_parse_chapter_invalid_is_ignored() {
        let mut parser = Parser::new();
        // 7 个 #（超过 6）应被忽略而不是报错
        let script = parser.parse("test", "####### too deep").unwrap();
        assert_eq!(script.len(), 0);
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
            ScriptNode::ShowCharacter { path: Some(path), alias, position: Position::Center, transition: Some(t) }
            if path.as_str() == "assets/char.png" && alias == "royu" && t.name == "Dissolve"
        ));
    }

    #[test]
    fn test_parse_show_character_without_path() {
        let mut parser = Parser::new();
        let script = parser.parse("test", r#"show beifeng at left"#).unwrap();

        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { path: None, alias, position: Position::Left, transition: None }
            if alias == "beifeng"
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
            assert!(
                matches!(&t.args[0], (None, TransitionArg::Number(n)) if (*n - 1.5).abs() < 0.001)
            );
        } else {
            panic!("Expected ChangeBG node");
        }
    }

    #[test]
    fn test_parse_transition_with_named_args() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeBG <img src="bg.png" /> with Dissolve(duration: 2.0)"#,
            )
            .unwrap();

        if let ScriptNode::ChangeBG {
            transition: Some(t),
            ..
        } = &script.nodes[0]
        {
            assert_eq!(t.name, "Dissolve");
            assert_eq!(t.args.len(), 1);
            assert!(
                matches!(&t.args[0], (Some(key), TransitionArg::Number(n)) if key == "duration" && (*n - 2.0).abs() < 0.001)
            );
            assert_eq!(t.get_duration(), Some(2.0));
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
            .parse("test", r#"show<img src="assets/bg2.jpg" />as 红叶 at left"#)
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
            ScriptNode::ShowCharacter { alias, path: Some(path), position: Position::Left, transition: None }
            if alias == "红叶" && path.as_str() == "assets/bg2.jpg"
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

        if let ScriptNode::ShowCharacter {
            transition: Some(t),
            ..
        } = &script.nodes[0]
        {
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
            .parse("test", r#"changeBG <img src="assets/bg2.jpg" />"#)
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

        if let ScriptNode::ChangeBG {
            transition: Some(t),
            ..
        } = &script.nodes[0]
        {
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
        if let ScriptNode::ShowCharacter {
            path: Some(path),
            alias,
            position: _,
            transition,
        } = &script.nodes[3]
        {
            assert_eq!(path.as_str(), "assets/chara.png");
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
        let script = parser
            .parse("test", r#"<audio src="sfx/ding.mp3"></audio>"#)
            .unwrap();
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
        let script = parser
            .parse("test", r#"<audio src="bgm/Signal.mp3"></audio> loop"#)
            .unwrap();
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
        let script = parser
            .parse("test", r#"<audio src="../bgm/music.mp3"></audio> loop"#)
            .unwrap();
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
        let script = parser
            .parse(
                "test",
                r#"changeBG <img src="../backgrounds/bg.jpg" /> with `dissolve`"#,
            )
            .unwrap();
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
        let script = parser
            .parse(
                "test",
                r#"show <img src="../characters/北风.png" /> as beifeng at center"#,
            )
            .unwrap();
        assert_eq!(script.nodes.len(), 1);
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::ShowCharacter { path: Some(path), alias, .. }
            if path.as_str() == "../characters/北风.png" && alias == "beifeng"
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
        let has_audio = script
            .nodes
            .iter()
            .any(|n| matches!(n, ScriptNode::PlayAudio { is_bgm: true, .. }));
        let has_goto = script
            .nodes
            .iter()
            .any(|n| matches!(n, ScriptNode::Goto { .. }));
        let has_stop_bgm = script
            .nodes
            .iter()
            .any(|n| matches!(n, ScriptNode::StopBgm));
        let has_choice = script
            .nodes
            .iter()
            .any(|n| matches!(n, ScriptNode::Choice { .. }));

        assert!(has_audio, "应该有 BGM 播放");
        assert!(has_goto, "应该有 goto 指令");
        assert!(has_stop_bgm, "应该有 stopBGM 指令");
        assert!(has_choice, "应该有选择分支");
    }

    //=========================================================================
    // changeScene / changeBG 职责分离测试
    //=========================================================================

    /// 测试 changeBG 不允许 fade 效果
    #[test]
    fn test_parse_change_bg_fade_deprecated() {
        let mut parser = Parser::new();
        let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with fade"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{:?}", err).contains("fade") || format!("{:?}", err).contains("Fade"));
    }

    /// 测试 changeBG 不允许 fadewhite 效果
    #[test]
    fn test_parse_change_bg_fadewhite_deprecated() {
        let mut parser = Parser::new();
        let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with fadewhite"#);
        assert!(result.is_err());
    }

    /// 测试 changeBG 不允许其他非 dissolve 效果
    #[test]
    fn test_parse_change_bg_only_dissolve() {
        let mut parser = Parser::new();

        // dissolve 允许
        let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with dissolve"#);
        assert!(result.is_ok());

        // Dissolve(duration) 允许
        let result = parser.parse(
            "test",
            r#"changeBG <img src="bg.jpg" /> with Dissolve(1.5)"#,
        );
        assert!(result.is_ok());

        // 其他效果不允许
        let result = parser.parse("test", r#"changeBG <img src="bg.jpg" /> with rule"#);
        assert!(result.is_err());
    }

    /// 测试 changeScene 必须带 with 子句
    #[test]
    fn test_parse_change_scene_requires_with() {
        let mut parser = Parser::new();
        let result = parser.parse("test", r#"changeScene <img src="bg.jpg" />"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(format!("{:?}", err).contains("with"));
    }

    /// 测试 changeScene with Dissolve
    #[test]
    fn test_parse_change_scene_dissolve() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeScene <img src="bg.jpg" /> with Dissolve(duration: 1)"#,
            )
            .unwrap();

        if let ScriptNode::ChangeScene {
            path,
            transition: Some(t),
        } = &script.nodes[0]
        {
            assert_eq!(path, "bg.jpg");
            assert_eq!(t.name, "Dissolve");
            assert_eq!(t.get_duration(), Some(1.0));
        } else {
            panic!("Expected ChangeScene node");
        }
    }

    /// 测试 changeScene with Fade
    #[test]
    fn test_parse_change_scene_fade() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeScene <img src="bg.jpg" /> with Fade(duration: 1.5)"#,
            )
            .unwrap();

        if let ScriptNode::ChangeScene {
            path,
            transition: Some(t),
        } = &script.nodes[0]
        {
            assert_eq!(path, "bg.jpg");
            assert_eq!(t.name, "Fade");
            assert_eq!(t.get_duration(), Some(1.5));
        } else {
            panic!("Expected ChangeScene node");
        }
    }

    /// 测试 changeScene with FadeWhite
    #[test]
    fn test_parse_change_scene_fade_white() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeScene <img src="bg.jpg" /> with FadeWhite(duration: 2)"#,
            )
            .unwrap();

        if let ScriptNode::ChangeScene {
            path,
            transition: Some(t),
        } = &script.nodes[0]
        {
            assert_eq!(path, "bg.jpg");
            assert_eq!(t.name, "FadeWhite");
            assert_eq!(t.get_duration(), Some(2.0));
        } else {
            panic!("Expected ChangeScene node");
        }
    }

    /// 测试 changeScene with rule-based effect
    #[test]
    fn test_parse_change_scene_rule() {
        let mut parser = Parser::new();
        let script = parser.parse(
            "test",
            r#"changeScene <img src="bg.jpg" /> with <img src="rule_10.png" /> (duration: 1, reversed: true)"#
        ).unwrap();

        if let ScriptNode::ChangeScene {
            path,
            transition: Some(t),
        } = &script.nodes[0]
        {
            assert_eq!(path, "bg.jpg");
            assert_eq!(t.name, "rule");
            // 检查参数
            assert_eq!(
                t.get_named("mask"),
                Some(&TransitionArg::String("rule_10.png".to_string()))
            );
            assert_eq!(t.get_duration(), Some(1.0));
            assert_eq!(t.get_reversed(), Some(true));
        } else {
            panic!("Expected ChangeScene node");
        }
    }

    /// 测试 changeScene with rule-based effect（无参数）
    #[test]
    fn test_parse_change_scene_rule_no_params() {
        let mut parser = Parser::new();
        let script = parser
            .parse(
                "test",
                r#"changeScene <img src="bg.jpg" /> with <img src="mask.png" />"#,
            )
            .unwrap();

        if let ScriptNode::ChangeScene {
            path,
            transition: Some(t),
        } = &script.nodes[0]
        {
            assert_eq!(path, "bg.jpg");
            assert_eq!(t.name, "rule");
            assert_eq!(
                t.get_named("mask"),
                Some(&TransitionArg::String("mask.png".to_string()))
            );
        } else {
            panic!("Expected ChangeScene node");
        }
    }

    #[test]
    fn test_parse_change_scene_requires_with_clause() {
        let mut parser = Parser::new();
        let err = parser
            .parse("test", r#"changeScene <img src="assets/bg.png" />"#)
            .unwrap_err();
        assert!(matches!(err, ParseError::MissingParameter { .. }));
    }

    #[test]
    fn test_parse_change_scene_invalid_transition() {
        let mut parser = Parser::new();
        // 过渡表达式解析失败（混用参数），应报 InvalidTransition
        let err = parser
            .parse(
                "test",
                r#"changeScene <img src="assets/bg.png" /> with Dissolve(1.0, duration: 2.0)"#,
            )
            .unwrap_err();
        assert!(matches!(err, ParseError::InvalidTransition { .. }));
    }

    #[test]
    fn test_parse_show_simplified_requires_at() {
        let mut parser = Parser::new();
        let err = parser.parse("test", "show alice").unwrap_err();
        assert!(matches!(err, ParseError::MissingParameter { .. }));
    }

    #[test]
    fn test_parse_hide_missing_alias() {
        let mut parser = Parser::new();
        let err = parser.parse("test", "hide").unwrap_err();
        assert!(matches!(err, ParseError::MissingParameter { .. }));
    }

    #[test]
    fn test_parse_goto_label_extraction_and_empty_label_error() {
        let mut parser = Parser::new();
        // 支持 **label**
        let script = parser.parse("test", "goto **end**").unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::Goto { target_label } if target_label == "end"
        ));

        // "**  **" 会被 trim 成空字符串，应报 MissingParameter
        let err = parser.parse("test", "goto **  **").unwrap_err();
        assert!(matches!(err, ParseError::MissingParameter { .. }));
    }

    #[test]
    fn test_parse_audio_loop_and_no_close_tag() {
        let mut parser = Parser::new();

        // 有 </audio> 且带 loop -> BGM
        let script = parser
            .parse("test", r#"<audio src="bgm.mp3"></audio> loop"#)
            .unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::PlayAudio { path, is_bgm: true } if path == "bgm.mp3"
        ));

        // 没有 </audio> -> is_bgm = false
        let script = parser.parse("test", r#"<audio src="sfx.mp3">"#).unwrap();
        assert!(matches!(
            &script.nodes[0],
            ScriptNode::PlayAudio { path, is_bgm: false } if path == "sfx.mp3"
        ));
    }

    #[test]
    fn test_unknown_line_produces_warning() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "??? what is this").unwrap();
        assert_eq!(script.len(), 0);
        assert_eq!(parser.warnings().len(), 1);
        assert!(parser.warnings()[0].contains("无法识别"));
    }

    #[test]
    fn test_parse_table_incomplete_rows_and_empty_options_errors() {
        let mut parser = Parser::new();

        // 1) 不完整行应产生 warning，但仍能解析出 Choice
        let text = r#"
| 横排 |
| --- |
| 只有一列 |
| 选项1 | label1 |
"#
        .trim();
        let script = parser.parse("test", text).unwrap();
        assert_eq!(script.len(), 1);
        assert!(matches!(&script.nodes[0], ScriptNode::Choice { .. }));
        assert!(!parser.warnings().is_empty());

        // 2) 没有任何有效选项 -> InvalidTable
        let mut parser = Parser::new();
        let text = r#"
| 横排 |
| --- |
| 只有一列 |
"#
        .trim();
        let err = parser.parse("test", text).unwrap_err();
        assert!(matches!(err, ParseError::InvalidTable { .. }));
    }

    #[test]
    fn test_extract_transition_from_line_rule_without_src_and_with_invalid_args() {
        let parser = Parser::new();

        // rule: 有 <img 但没有 src -> 返回 simple("rule")
        let t = parser
            .extract_transition_from_line(
                r#"changeScene <img src="bg.png" /> with <img alt="x" />"#,
            )
            .unwrap();
        assert_eq!(t.name, "rule");
        assert!(t.args.is_empty());

        // rule: 有 src，但括号里参数非法（混用）-> extract_rule_args 不扩展参数，只保留 mask
        let t = parser
            .extract_transition_from_line(
                r#"changeScene <img src="bg.png" /> with <img src="masks/rule.png" /> (1.0, duration: 2.0)"#,
            )
            .unwrap();
        assert_eq!(t.name, "rule");
        assert!(t.get_named("mask").is_some());
        assert!(t.get_named("duration").is_none());
    }

    //=========================================================================
    // set 指令测试
    //=========================================================================

    #[test]
    fn test_parse_set_var_string() {
        let mut parser = Parser::new();
        let script = parser.parse("test", r#"set $name = "Alice""#).unwrap();

        assert_eq!(script.len(), 1);
        if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
            assert_eq!(name, "name");
            assert!(
                matches!(value, crate::script::Expr::Literal(crate::state::VarValue::String(s)) if s == "Alice")
            );
        } else {
            panic!("Expected SetVar node");
        }
    }

    #[test]
    fn test_parse_set_var_bool() {
        let mut parser = Parser::new();

        let script = parser.parse("test", "set $is_active = true").unwrap();
        if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
            assert_eq!(name, "is_active");
            assert!(matches!(
                value,
                crate::script::Expr::Literal(crate::state::VarValue::Bool(true))
            ));
        } else {
            panic!("Expected SetVar node");
        }

        let script = parser.parse("test", "set $is_done = false").unwrap();
        if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
            assert_eq!(name, "is_done");
            assert!(matches!(
                value,
                crate::script::Expr::Literal(crate::state::VarValue::Bool(false))
            ));
        } else {
            panic!("Expected SetVar node");
        }
    }

    #[test]
    fn test_parse_set_var_int() {
        let mut parser = Parser::new();
        let script = parser.parse("test", "set $count = 42").unwrap();

        if let ScriptNode::SetVar { name, value } = &script.nodes[0] {
            assert_eq!(name, "count");
            assert!(matches!(
                value,
                crate::script::Expr::Literal(crate::state::VarValue::Int(42))
            ));
        } else {
            panic!("Expected SetVar node");
        }
    }

    #[test]
    fn test_parse_set_var_missing_dollar() {
        let mut parser = Parser::new();
        let err = parser.parse("test", "set name = 123").unwrap_err();
        assert!(matches!(err, ParseError::InvalidLine { .. }));
    }

    #[test]
    fn test_parse_set_var_missing_equals() {
        let mut parser = Parser::new();
        let err = parser.parse("test", "set $name 123").unwrap_err();
        assert!(matches!(err, ParseError::MissingParameter { .. }));
    }

    //=========================================================================
    // 条件分支测试
    //=========================================================================

    #[test]
    fn test_parse_simple_if() {
        let mut parser = Parser::new();
        let text = r#"
if $flag == true
  ："条件为真"
endif
"#;
        let script = parser.parse("test", text).unwrap();

        assert_eq!(script.len(), 1);
        if let ScriptNode::Conditional { branches } = &script.nodes[0] {
            assert_eq!(branches.len(), 1);
            assert!(branches[0].condition.is_some());
            assert_eq!(branches[0].body.len(), 1);
        } else {
            panic!("Expected Conditional node");
        }
    }

    #[test]
    fn test_parse_if_else() {
        let mut parser = Parser::new();
        let text = r#"
if $name == "Alice"
  ："你好，Alice"
else
  ："你好，陌生人"
endif
"#;
        let script = parser.parse("test", text).unwrap();

        if let ScriptNode::Conditional { branches } = &script.nodes[0] {
            assert_eq!(branches.len(), 2);
            assert!(branches[0].condition.is_some()); // if 分支
            assert!(branches[1].condition.is_none()); // else 分支
        } else {
            panic!("Expected Conditional node");
        }
    }

    #[test]
    fn test_parse_if_elseif_else() {
        let mut parser = Parser::new();
        let text = r#"
if $role == "admin"
  ："欢迎管理员"
elseif $role == "user"
  ："欢迎用户"
else
  ："欢迎访客"
endif
"#;
        let script = parser.parse("test", text).unwrap();

        if let ScriptNode::Conditional { branches } = &script.nodes[0] {
            assert_eq!(branches.len(), 3);
            assert!(branches[0].condition.is_some()); // if
            assert!(branches[1].condition.is_some()); // elseif
            assert!(branches[2].condition.is_none()); // else
        } else {
            panic!("Expected Conditional node");
        }
    }

    #[test]
    fn test_parse_if_with_logical_ops() {
        let mut parser = Parser::new();
        let text = r#"
if $a == true and $b == false
  ："复合条件"
endif
"#;
        let script = parser.parse("test", text).unwrap();

        if let ScriptNode::Conditional { branches } = &script.nodes[0] {
            assert_eq!(branches.len(), 1);
            // 条件应该是 And 表达式
            if let Some(crate::script::Expr::And(_, _)) = &branches[0].condition {
                // OK
            } else {
                panic!("Expected And expression");
            }
        } else {
            panic!("Expected Conditional node");
        }
    }

    #[test]
    fn test_parse_if_missing_endif() {
        let mut parser = Parser::new();
        let text = r#"
if $flag == true
  ："没有 endif"
"#;
        // 未闭合的条件块会在 parse_conditional 阶段报错
        let result = parser.parse("test", text);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ParseError::InvalidLine { .. }
        ));
    }

    //=========================================================================
    // 表达式解析测试
    //=========================================================================

    #[test]
    fn test_parse_expression_variable() {
        let expr = parse_expression("$foo == \"bar\"", 1).unwrap();
        assert!(matches!(expr, crate::script::Expr::Eq(_, _)));
    }

    #[test]
    fn test_parse_expression_bool_literal() {
        let expr = parse_expression("true", 1).unwrap();
        assert!(matches!(
            expr,
            crate::script::Expr::Literal(crate::state::VarValue::Bool(true))
        ));

        let expr = parse_expression("false", 1).unwrap();
        assert!(matches!(
            expr,
            crate::script::Expr::Literal(crate::state::VarValue::Bool(false))
        ));
    }

    #[test]
    fn test_parse_expression_not() {
        let expr = parse_expression("not $flag", 1).unwrap();
        assert!(matches!(expr, crate::script::Expr::Not(_)));
    }

    #[test]
    fn test_parse_expression_and_or() {
        let expr = parse_expression("$a == true and $b == false", 1).unwrap();
        assert!(matches!(expr, crate::script::Expr::And(_, _)));

        let expr = parse_expression("$a == true or $b == false", 1).unwrap();
        assert!(matches!(expr, crate::script::Expr::Or(_, _)));
    }

    #[test]
    fn test_parse_expression_parentheses() {
        let expr = parse_expression("($a == true)", 1).unwrap();
        assert!(matches!(expr, crate::script::Expr::Eq(_, _)));

        let expr = parse_expression("($a == true) and ($b == false)", 1).unwrap();
        assert!(matches!(expr, crate::script::Expr::And(_, _)));
    }

    #[test]
    fn test_parse_expression_not_equal() {
        let expr = parse_expression("$name != \"Bob\"", 1).unwrap();
        assert!(matches!(expr, crate::script::Expr::NotEq(_, _)));
    }

    #[test]
    fn test_parse_expression_empty_error() {
        let err = parse_expression("", 1).unwrap_err();
        assert!(matches!(err, ParseError::InvalidLine { .. }));
    }

    #[test]
    fn test_parse_expression_unclosed_paren() {
        let err = parse_expression("($a == true", 1).unwrap_err();
        assert!(matches!(err, ParseError::InvalidLine { .. }));
    }
}
