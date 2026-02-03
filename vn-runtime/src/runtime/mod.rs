//! # Runtime 模块
//!
//! VN 执行引擎核心，负责脚本执行和状态管理。
//!
//! ## 模块结构
//!
//! - [`engine`]：核心执行引擎
//! - [`executor`]：AST 节点到 Command 的转换

pub mod engine;
pub mod executor;

pub use engine::VNRuntime;
