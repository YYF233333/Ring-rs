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
//!
//! ## 模块结构
//!
//! - `helpers`: 辅助解析函数
//! - `expr_parser`: 表达式解析器
//! - `phase1`: 块识别
//! - `phase2`: 块解析

mod expr_parser;
mod helpers;
mod phase1;
mod phase2;

#[cfg(test)]
mod tests;

use crate::error::ParseError;
use crate::script::ast::Script;

use phase1::recognize_blocks;
use phase2::Phase2Parser;

// 重新导出辅助函数供测试使用
pub use helpers::{
    extract_audio_src, extract_img_src, extract_keyword_value, is_table_separator, parse_arg_value,
    parse_dialogue, parse_transition, parse_transition_args, starts_with_ignore_case,
};

// 重新导出表达式解析函数
pub use expr_parser::parse_expression;

/// 脚本解析器
pub struct Parser {
    /// 阶段2解析器
    phase2: Phase2Parser,
}

impl Parser {
    /// 创建新的解析器
    pub fn new() -> Self {
        Self {
            phase2: Phase2Parser::new(),
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
        self.phase2.warnings.clear();

        // 阶段 1：块识别
        let blocks = recognize_blocks(text);

        // 阶段 2：块解析（同时收集行号）
        let mut nodes = Vec::new();
        let mut source_map = Vec::new();
        for block in blocks {
            let line_number = block.start_line();
            match self.phase2.parse_block(block) {
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
        &self.phase2.warnings
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}
