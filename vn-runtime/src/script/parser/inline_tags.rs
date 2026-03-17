//! # 内联标签解析
//!
//! 解析对话文本中的节奏控制标签（`{wait}`, `{speed}`, `{/speed}`）。
//!
//! 输入为引号内的原始文本，输出为纯文本 + 效果列表。
//! `-->` 行修饰符由 Phase2 在行级处理，不在本模块范围内。

use crate::command::{InlineEffect, InlineEffectKind};

/// 解析对话文本中的内联标签
///
/// 返回 `(纯文本, 效果列表)`。
/// 标签从文本中剥离，效果列表中的 `position` 指向纯文本的字符索引。
pub fn parse_inline_tags(raw: &str) -> (String, Vec<InlineEffect>) {
    let mut plain = String::new();
    let mut effects = Vec::new();
    let mut chars = raw.char_indices().peekable();

    while let Some(&(byte_idx, ch)) = chars.peek() {
        if ch == '{'
            && let Some((tag_byte_end, kind)) = try_parse_tag(raw, byte_idx)
        {
            let mut advanced = false;
            // skip past the closing '}'
            while let Some(&(bi, _)) = chars.peek() {
                if bi >= tag_byte_end {
                    break;
                }
                advanced = true;
                chars.next();
            }
            if !advanced {
                // 只有确认游标跨过了整个 tag，才把它当作控制标签消费；
                // 否则退回普通文本路径，避免同一个 byte_idx 被无限重复解析。
                plain.push(ch);
                chars.next();
                continue;
            }
            let position = plain.chars().count();
            effects.push(InlineEffect { position, kind });
            continue;
        }
        plain.push(ch);
        chars.next();
    }

    (plain, effects)
}

/// Try to parse a tag starting at `start` (which points to `{`).
/// Returns `(byte_index_after_closing_brace, InlineEffectKind)` on success.
fn try_parse_tag(raw: &str, start: usize) -> Option<(usize, InlineEffectKind)> {
    let rest = &raw[start..];
    let close = rest.find('}')?;
    let inner = rest[1..close].trim();
    let tag_end = start + close + 1; // byte index after '}'

    // {/speed}
    if inner.eq_ignore_ascii_case("/speed") {
        return Some((tag_end, InlineEffectKind::ResetCps));
    }

    // {wait} or {wait Ns} or {wait N}
    if inner.eq_ignore_ascii_case("wait") {
        return Some((tag_end, InlineEffectKind::Wait(None)));
    }
    if let Some(arg) = strip_prefix_ignore_case(inner, "wait ") {
        let arg = arg.trim();
        let num_str = arg.strip_suffix('s').unwrap_or(arg);
        let seconds: f64 = num_str.parse().ok()?;
        if seconds > 0.0 {
            return Some((tag_end, InlineEffectKind::Wait(Some(seconds))));
        }
        return None;
    }

    // {speed Nx} or {speed N}
    if let Some(arg) = strip_prefix_ignore_case(inner, "speed ") {
        let arg = arg.trim();
        if let Some(multiplier_str) = arg.strip_suffix('x').or_else(|| arg.strip_suffix('X')) {
            let multiplier: f64 = multiplier_str.parse().ok()?;
            if multiplier > 0.0 {
                return Some((tag_end, InlineEffectKind::SetCpsRelative(multiplier)));
            }
            return None;
        }
        let cps: f64 = arg.parse().ok()?;
        if cps > 0.0 {
            return Some((tag_end, InlineEffectKind::SetCpsAbsolute(cps)));
        }
        return None;
    }

    None
}

fn strip_prefix_ignore_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_no_tags() {
        let (text, effects) = parse_inline_tags("你好世界");
        assert_eq!(text, "你好世界");
        assert!(effects.is_empty());
    }

    #[test]
    fn wait_click_at_end() {
        let (text, effects) = parse_inline_tags("差不多吧。{wait}");
        assert_eq!(text, "差不多吧。");
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].position, 5);
        assert_eq!(effects[0].kind, InlineEffectKind::Wait(None));
    }

    #[test]
    fn wait_timed() {
        let (text, effects) = parse_inline_tags("差不多吧。{wait 1.0s}");
        assert_eq!(text, "差不多吧。");
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].position, 5);
        assert_eq!(effects[0].kind, InlineEffectKind::Wait(Some(1.0)));
    }

    #[test]
    fn wait_timed_no_suffix() {
        let (text, effects) = parse_inline_tags("文本{wait 0.5}继续");
        assert_eq!(text, "文本继续");
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].position, 2);
        assert_eq!(effects[0].kind, InlineEffectKind::Wait(Some(0.5)));
    }

    #[test]
    fn speed_absolute() {
        let (text, effects) = parse_inline_tags("{speed 20}慢速文本{/speed}");
        assert_eq!(text, "慢速文本");
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].position, 0);
        assert_eq!(effects[0].kind, InlineEffectKind::SetCpsAbsolute(20.0));
        assert_eq!(effects[1].position, 4);
        assert_eq!(effects[1].kind, InlineEffectKind::ResetCps);
    }

    #[test]
    fn speed_relative() {
        let (text, effects) = parse_inline_tags("{speed 2x}快速{/speed}");
        assert_eq!(text, "快速");
        assert_eq!(effects.len(), 2);
        assert_eq!(effects[0].position, 0);
        assert_eq!(effects[0].kind, InlineEffectKind::SetCpsRelative(2.0));
        assert_eq!(effects[1].position, 2);
        assert_eq!(effects[1].kind, InlineEffectKind::ResetCps);
    }

    #[test]
    fn combined_wait_and_speed() {
        let (text, effects) =
            parse_inline_tags("子文，{wait 0.5s}你的作品{speed 3}...{/speed}{wait}怎么样了？");
        assert_eq!(text, "子文，你的作品...怎么样了？");
        assert_eq!(effects.len(), 4);
        assert_eq!(
            effects[0],
            InlineEffect {
                position: 3,
                kind: InlineEffectKind::Wait(Some(0.5))
            }
        );
        assert_eq!(
            effects[1],
            InlineEffect {
                position: 7,
                kind: InlineEffectKind::SetCpsAbsolute(3.0)
            }
        );
        assert_eq!(
            effects[2],
            InlineEffect {
                position: 10,
                kind: InlineEffectKind::ResetCps
            }
        );
        assert_eq!(
            effects[3],
            InlineEffect {
                position: 10,
                kind: InlineEffectKind::Wait(None)
            }
        );
    }

    #[test]
    fn literal_braces_preserved_if_not_a_tag() {
        let (text, effects) = parse_inline_tags("这是 {unknown} 标签");
        assert_eq!(text, "这是 {unknown} 标签");
        assert!(effects.is_empty());
    }

    #[test]
    fn case_insensitive() {
        let (text, effects) = parse_inline_tags("{Wait 1s}{Speed 2X}快{/Speed}");
        assert_eq!(text, "快");
        assert_eq!(effects.len(), 3);
        assert_eq!(effects[0].kind, InlineEffectKind::Wait(Some(1.0)));
        assert_eq!(effects[1].kind, InlineEffectKind::SetCpsRelative(2.0));
        assert_eq!(effects[2].kind, InlineEffectKind::ResetCps);
    }

    #[test]
    fn empty_input() {
        let (text, effects) = parse_inline_tags("");
        assert_eq!(text, "");
        assert!(effects.is_empty());
    }

    #[test]
    fn speed_half() {
        let (_, effects) = parse_inline_tags("{speed 0.5x}慢");
        assert_eq!(effects[0].kind, InlineEffectKind::SetCpsRelative(0.5));
    }
}
