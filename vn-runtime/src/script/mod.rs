//! # Script 模块
//!
//! 脚本解析相关功能，包括 AST 定义和解析器实现。
//!
//! ## 模块结构
//!
//! - [`ast`]：脚本抽象语法树定义
//! - [`parser`]：两阶段解析器实现

pub mod ast;
pub mod parser;

pub use ast::*;
pub use parser::Parser;

