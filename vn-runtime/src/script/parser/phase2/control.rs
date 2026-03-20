//! 控制流指令解析：goto、callScript、conditional、set、wait、choice table

use crate::error::ParseError;
use crate::script::Expr;
use crate::script::ast::{ChoiceOption, ConditionalBranch, ScriptNode};

use super::super::expr_parser::parse_expression;
use super::super::helpers::{is_table_separator, starts_with_ignore_case};
use super::Phase2Parser;

impl Phase2Parser {
    /// 解析 set 指令
    ///
    /// 语法: `set $var = value`
    pub(super) fn parse_set_var(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line[4..].trim();

        let eq_pos = content
            .find('=')
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "set".to_string(),
                param: "赋值符号 '='".to_string(),
            })?;

        let var_part = content[..eq_pos].trim();
        let value_part = content[eq_pos + 1..].trim();

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

        let is_valid = if let Some(bare) = var_name.strip_prefix("persistent.") {
            !bare.is_empty() && bare.chars().all(|c| c.is_alphanumeric() || c == '_')
        } else {
            var_name.chars().all(|c| c.is_alphanumeric() || c == '_')
        };
        if !is_valid {
            return Err(ParseError::InvalidLine {
                line: line_number,
                message: format!(
                    "变量名格式无效。普通变量名只能含字母、数字和下划线；持久变量须为 persistent.<name> 格式，实际: '{}'",
                    var_name
                ),
            });
        }

        let value = parse_expression(value_part, line_number)?;

        Ok(Some(ScriptNode::SetVar {
            name: var_name.to_string(),
            value,
        }))
    }

    /// 解析条件块
    pub(super) fn parse_conditional(
        &mut self,
        lines: &[(String, usize)],
        start_line: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        if lines.is_empty() {
            return Ok(None);
        }

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

            if starts_with_ignore_case(trimmed, "elseif ") {
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

            if trimmed.eq_ignore_ascii_case("else") {
                let body = self.parse_body_lines(&current_body_lines)?;
                branches.push(ConditionalBranch {
                    condition: current_condition.take(),
                    body,
                });
                current_body_lines.clear();
                current_condition = None;
                continue;
            }

            if trimmed.eq_ignore_ascii_case("endif") {
                let body = self.parse_body_lines(&current_body_lines)?;
                branches.push(ConditionalBranch {
                    condition: current_condition.take(),
                    body,
                });
                break;
            }

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
    pub(super) fn parse_body_lines(
        &mut self,
        lines: &[(String, usize)],
    ) -> Result<Vec<ScriptNode>, ParseError> {
        let mut nodes = Vec::new();

        for (line, line_number) in lines {
            if line.trim().is_empty() {
                continue;
            }
            if let Some(node) = self.parse_single_line(line, *line_number)? {
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    /// 解析 goto 指令
    ///
    /// 语法: `goto **label**`
    pub(super) fn parse_goto(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get(4..)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "goto".to_string(),
                param: "目标标签".to_string(),
            })?;

        let target_label =
            if content.starts_with("**") && content.ends_with("**") && content.len() > 4 {
                content[2..content.len() - 2].trim().to_string()
            } else {
                content.to_string()
            };

        if target_label.is_empty() {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "goto".to_string(),
                param: "目标标签".to_string(),
            });
        }

        if target_label.contains("::") {
            return Err(ParseError::InvalidLine {
                line: line_number,
                message: "暂不支持跨文件 goto，请使用 callScript/returnFromScript 组织流程"
                    .to_string(),
            });
        }

        Ok(Some(ScriptNode::Goto { target_label }))
    }

    /// 解析 callScript 指令
    ///
    /// 语法: `callScript [label](path/to/script.md)`
    pub(super) fn parse_call_script(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let content = line
            .get("callScript".len()..)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ParseError::MissingParameter {
                line: line_number,
                command: "callScript".to_string(),
                param: "目标脚本路径".to_string(),
            })?;

        let (display_label, path, after_link) =
            super::parse_markdown_link(content).ok_or_else(|| ParseError::InvalidLine {
                line: line_number,
                message:
                    "callScript 必须使用 Markdown 链接格式，例如 callScript [chapter1](ring/summer/1-1.md)"
                        .to_string(),
            })?;

        if !after_link.trim().is_empty() {
            return Err(ParseError::InvalidLine {
                line: line_number,
                message: "callScript 链接后不允许额外参数".to_string(),
            });
        }

        Ok(Some(ScriptNode::CallScript {
            path: path.to_string(),
            display_label: Some(display_label.to_string()),
        }))
    }

    /// 解析 wait 指令
    ///
    /// 语法: `wait <duration>`，duration 为秒数（正数）
    pub(super) fn parse_wait(
        &self,
        line: &str,
        line_number: usize,
    ) -> Result<Option<ScriptNode>, ParseError> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(ParseError::MissingParameter {
                line: line_number,
                command: "wait".to_string(),
                param: "等待时长（秒）".to_string(),
            });
        }

        let duration: f64 = parts[1].parse().map_err(|_| ParseError::InvalidParameter {
            line: line_number,
            param: "duration".to_string(),
            message: format!("无法解析为数字: '{}'", parts[1]),
        })?;

        if duration <= 0.0 {
            return Err(ParseError::InvalidParameter {
                line: line_number,
                param: "duration".to_string(),
                message: format!("等待时长必须为正数，实际: {}", duration),
            });
        }

        Ok(Some(ScriptNode::Wait { duration }))
    }

    /// 解析表格块（选择分支）
    pub(super) fn parse_table(
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

            if is_table_separator(line) {
                continue;
            }

            let cells: Vec<&str> = line
                .split('|')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if !header_parsed {
                if !cells.is_empty() {
                    style = Some(cells[0].to_string());
                }
                header_parsed = true;
                continue;
            }

            if cells.len() < 2 {
                self.warnings
                    .push(format!("第 {} 行：表格行格式不完整，已跳过", line_number));
                continue;
            }

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
