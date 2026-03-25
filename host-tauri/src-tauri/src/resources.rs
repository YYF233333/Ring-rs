//! 资源管理系统（简化版，仅文件系统模式）
//!
//! 提供 [`LogicalPath`] 路径规范化和 [`ResourceManager`] 资源读取。

use std::path::{Path, PathBuf};
use thiserror::Error;

// ── LogicalPath ──────────────────────────────────────────────────────────────

/// 规范化的逻辑资源路径。
///
/// 不变量：
/// - 相对于 assets_root（不含 `assets/` 前缀）
/// - 使用 `/` 分隔符
/// - 已解析 `..` 和 `.`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalPath(String);

impl LogicalPath {
    /// 从原始字符串构造，内部自动规范化。
    pub fn new(raw: &str) -> Self {
        Self(normalize_logical_path(raw))
    }

    /// 获取内部字符串切片。
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// 提取文件名（不含扩展名）。
    pub fn file_stem(&self) -> &str {
        let filename = self.0.rsplit('/').next().unwrap_or(&self.0);
        if let Some(dot_pos) = filename.rfind('.') {
            &filename[..dot_pos]
        } else {
            filename
        }
    }

    /// 提取父目录路径。
    pub fn parent_dir(&self) -> &str {
        if let Some(last_slash) = self.0.rfind('/') {
            &self.0[..last_slash]
        } else {
            ""
        }
    }

    /// 拼接子路径并规范化。
    #[allow(dead_code)]
    pub fn join(&self, relative: &str) -> Self {
        if self.0.is_empty() {
            Self::new(relative)
        } else {
            Self::new(&format!("{}/{}", self.0, relative))
        }
    }

    /// 转换为 [`PathBuf`]。
    #[allow(dead_code)]
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.0)
    }
}

impl std::fmt::Display for LogicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ── 路径规范化 ───────────────────────────────────────────────────────────────

/// 规范化逻辑路径：统一分隔符、处理 `.` `..`、去除 `assets/` 前缀。
pub fn normalize_logical_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let path = normalized.strip_prefix("./").unwrap_or(&normalized);

    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            _ => {
                components.push(component);
            }
        }
    }

    let result = components.join("/");
    result
        .strip_prefix("assets/")
        .unwrap_or(&result)
        .to_string()
}

// ── ResourceError ────────────────────────────────────────────────────────────

/// 资源管理错误
#[derive(Error, Debug)]
pub enum ResourceError {
    /// 资源加载失败
    #[error("加载 {kind} 资源失败: {path} - {message}")]
    LoadFailed {
        path: String,
        kind: String,
        message: String,
    },
    /// 资源未找到
    #[allow(dead_code)]
    #[error("资源未找到: {path}")]
    NotFound { path: String },
}

// ── ResourceManager ──────────────────────────────────────────────────────────

/// 简化版资源管理器（仅文件系统模式）
pub struct ResourceManager {
    base_path: PathBuf,
}

impl ResourceManager {
    /// 创建资源管理器
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// 将逻辑路径解析为文件系统绝对路径
    fn resolve(&self, path: &LogicalPath) -> PathBuf {
        self.base_path.join(path.as_str())
    }

    /// 读取文本资源
    pub fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError> {
        let full = self.resolve(path);
        std::fs::read_to_string(&full).map_err(|e| ResourceError::LoadFailed {
            path: full.to_string_lossy().to_string(),
            kind: "text".to_string(),
            message: e.to_string(),
        })
    }

    /// 读取二进制资源
    #[allow(dead_code)]
    pub fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        let full = self.resolve(path);
        std::fs::read(&full).map_err(|e| ResourceError::LoadFailed {
            path: full.to_string_lossy().to_string(),
            kind: "file".to_string(),
            message: e.to_string(),
        })
    }

    /// 返回逻辑路径对应的文件系统绝对路径（用于 asset 协议）
    #[allow(dead_code)]
    pub fn resolve_fs_path(&self, path: &LogicalPath) -> PathBuf {
        self.resolve(path)
    }

    /// 检查资源是否存在
    #[allow(dead_code)]
    pub fn resource_exists(&self, path: &LogicalPath) -> bool {
        self.resolve(path).exists()
    }

    /// 获取 assets 根目录
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}
