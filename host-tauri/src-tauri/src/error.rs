//! host-tauri 统一错误类型
//!
//! `HostError` 是后端所有业务方法的错误类型。
//! IPC 命令层（`commands.rs`）在边界处统一转换为 `String`
//! 以满足 Tauri 序列化要求。

use thiserror::Error;

use crate::config::ConfigError;
use crate::resources::ResourceError;

/// host-tauri 后端统一错误类型。
///
/// 变体按失败域划分，跨域调用保留内层错误以维护错误链。
#[derive(Debug, Error)]
pub enum HostError {
    /// 会话/Owner 校验失败
    #[error("{0}")]
    Session(String),

    /// vn-runtime 脚本解析或执行错误
    #[error("{0}")]
    Script(#[from] vn_runtime::VnError),

    /// 存档读写错误
    #[error("{0}")]
    Save(#[from] vn_runtime::SaveError),

    /// 配置加载/校验错误
    #[error("{0}")]
    Config(#[from] ConfigError),

    /// 资源读取错误
    #[error("{0}")]
    Resource(#[from] ResourceError),

    /// 输入参数非法
    #[error("{0}")]
    InvalidInput(String),

    /// IO 错误（持久化变量等非资源 IO）
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// 内部逻辑错误（不应出现的状态）
    #[error("{0}")]
    Internal(String),
}

impl From<vn_runtime::ParseError> for HostError {
    fn from(e: vn_runtime::ParseError) -> Self {
        Self::Script(vn_runtime::VnError::from(e))
    }
}

impl From<vn_runtime::RuntimeError> for HostError {
    fn from(e: vn_runtime::RuntimeError) -> Self {
        Self::Script(vn_runtime::VnError::from(e))
    }
}

pub type HostResult<T> = Result<T, HostError>;
