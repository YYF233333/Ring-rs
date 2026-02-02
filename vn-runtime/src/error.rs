//! # Error 模块
//!
//! 定义 vn-runtime 中使用的错误类型。

use thiserror::Error;

/// 解析错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    /// 无效的行格式
    #[error("第 {line} 行：无效的格式 - {message}")]
    InvalidLine { line: usize, message: String },

    /// 无效的指令
    #[error("第 {line} 行：未知指令 '{command}'")]
    UnknownCommand { line: usize, command: String },

    /// 缺少必需参数
    #[error("第 {line} 行：指令 '{command}' 缺少参数 '{param}'")]
    MissingParameter {
        line: usize,
        command: String,
        param: String,
    },

    /// 无效的参数值
    #[error("第 {line} 行：参数 '{param}' 的值无效 - {message}")]
    InvalidParameter {
        line: usize,
        param: String,
        message: String,
    },

    /// 无效的表格格式
    #[error("第 {line} 行：无效的表格格式 - {message}")]
    InvalidTable { line: usize, message: String },

    /// 无效的过渡效果语法
    #[error("第 {line} 行：无效的过渡效果语法 - {message}")]
    InvalidTransition { line: usize, message: String },
}

/// 运行时错误
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RuntimeError {
    /// 标签未找到
    #[error("标签 '{label}' 未找到")]
    LabelNotFound { label: String },

    /// 无效的选择索引
    #[error("无效的选择索引 {index}，有效范围是 0..{max}")]
    InvalidChoiceIndex { index: usize, max: usize },

    /// 状态不匹配
    #[error("当前状态不允许此操作：期望 {expected}，实际 {actual}")]
    StateMismatch { expected: String, actual: String },

    /// 脚本执行结束
    #[error("脚本已执行完毕")]
    ScriptEnded,

    /// 无效的状态操作
    #[error("无效的状态操作: {message}")]
    InvalidState { message: String },
}

/// vn-runtime 统一错误类型
#[derive(Error, Debug, Clone, PartialEq)]
pub enum VnError {
    /// 解析错误
    #[error("解析错误: {0}")]
    Parse(#[from] ParseError),

    /// 运行时错误
    #[error("运行时错误: {0}")]
    Runtime(#[from] RuntimeError),
}

/// Result 类型别名
pub type VnResult<T> = Result<T, VnError>;

