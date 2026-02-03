//! # Script 模块
//!
//! 脚本解析相关功能，包括 AST 定义和解析器实现。
//!
//! ## 模块结构
//!
//! - [`ast`]：脚本抽象语法树定义
//! - [`expr`]：表达式 AST 和求值器
//! - [`parser`]：两阶段解析器实现

pub mod ast;
pub mod expr;
pub mod parser;

pub use ast::*;
pub use expr::{EvalContext, EvalError, Expr, evaluate, evaluate_to_bool};
pub use parser::Parser;
