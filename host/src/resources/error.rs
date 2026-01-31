//! # Resource Error 模块
//!
//! 定义资源管理相关的错误类型。

use thiserror::Error;

/// 资源管理错误
#[derive(Error, Debug)]
pub enum ResourceError {
    /// 资源加载失败
    #[error("加载 {kind} 资源失败: {path} - {message}")]
    LoadFailed {
        /// 资源路径
        path: String,
        /// 资源类型（texture, sound 等）
        kind: String,
        /// 错误消息
        message: String,
    },

    /// 资源未找到
    #[error("资源未找到: {path}")]
    NotFound {
        /// 资源路径
        path: String,
    },

    /// 无效的资源格式
    #[error("无效的资源格式: {path} - {message}")]
    InvalidFormat {
        /// 资源路径
        path: String,
        /// 错误消息
        message: String,
    },
}
