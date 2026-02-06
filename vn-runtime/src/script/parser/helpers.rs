//! # 辅助解析函数
//!
//! 手写的字符串解析辅助函数，无正则依赖。

use crate::command::{Transition, TransitionArg};

/// 跳过字符串开头的空白字符，返回剩余部分
pub fn skip_whitespace(s: &str) -> &str {
    s.trim_start()
}

/// 检查字符串是否以指定前缀开头（大小写不敏感）
pub fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
    s.len() >= prefix.len()
        && s.chars()
            .zip(prefix.chars())
            .all(|(a, b)| a.eq_ignore_ascii_case(&b))
}

/// 从字符串中提取 HTML img 标签的 src 属性值
///
/// 输入: `<img src="path/to/image.png" alt="desc" />`
/// 输出: `Some("path/to/image.png")`
pub fn extract_img_src(s: &str) -> Option<&str> {
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
pub fn extract_audio_src(s: &str) -> Option<&str> {
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
pub fn extract_keyword_value<'a>(s: &'a str, keyword: &str) -> Option<&'a str> {
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
        if let Some(pos) = lower.find(pattern.as_str())
            && (best_pos.is_none() || pos < best_pos.unwrap())
        {
            best_pos = Some(pos);
            best_pattern_len = pattern.len();
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
        if let Some(p) = remaining_lower.find(term)
            && p < end_pos
        {
            end_pos = p;
        }
    }

    let value = remaining[..end_pos].trim();
    if value.is_empty() { None } else { Some(value) }
}

/// 解析过渡效果表达式
///
/// 输入: `"dissolve"` 或 `"Dissolve(1.5)"` 或 `"Dissolve(duration: 1.5)"`
pub fn parse_transition(s: &str) -> Option<Transition> {
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
pub fn parse_transition_args(s: &str) -> Result<Vec<(Option<String>, TransitionArg)>, String> {
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
pub fn parse_arg_value(s: &str) -> TransitionArg {
    let s = s.trim();

    // 带引号的字符串
    if ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
        && s.len() >= 2
    {
        return TransitionArg::String(s[1..s.len() - 1].to_string());
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
pub fn is_table_separator(s: &str) -> bool {
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
pub fn parse_dialogue(s: &str) -> Option<(Option<String>, String)> {
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
pub fn extract_quoted_content(s: &str) -> Option<&str> {
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
