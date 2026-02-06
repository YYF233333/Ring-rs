//! # 阶段 1：块识别
//!
//! 将原始文本按行分组为块（单行、表格、条件块）。

use super::helpers::starts_with_ignore_case;

type ConditionalState = (Vec<(String, usize)>, usize, usize); // (lines, start_line, depth)

/// 块类型（阶段 1 输出）
#[derive(Debug, Clone)]
pub enum Block {
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
    pub fn start_line(&self) -> usize {
        match self {
            Block::SingleLine { line_number, .. } => *line_number,
            Block::Table { start_line, .. } => *start_line,
            Block::Conditional { start_line, .. } => *start_line,
        }
    }
}

/// 识别文本中的块
pub fn recognize_blocks(text: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current_table: Option<(Vec<String>, usize)> = None;
    let mut current_conditional: Option<ConditionalState> = None;

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

            current_conditional = Some((vec![(trimmed.to_string(), line_number)], line_number, 0));
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
