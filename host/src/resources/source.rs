//! # Resource Source 模块
//!
//! 资源来源抽象层，支持从不同来源（文件系统、ZIP 包等）读取资源。
//!
//! ## 设计原则
//!
//! - 所有资源路径在内部使用**逻辑路径**（相对于 assets_root，使用 `/` 分隔符）
//! - 加载时由具体实现决定如何解析到实际路径
//! - 使用 `super::path` 模块进行统一的路径规范化

use super::ResourceError;
use super::path::normalize_logical_path;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;

/// 资源来源 trait
///
/// 抽象资源读取接口，允许从不同来源加载资源：
/// - `FsSource`：从文件系统读取（开发模式）
/// - `ZipSource`：从 ZIP 包读取（发布模式）
///
/// ## 路径约定
///
/// 所有路径参数都是**逻辑路径**，即：
/// - 相对于 assets_root（不包含 `assets/` 前缀）
/// - 使用 `/` 作为路径分隔符
/// - 已经过 `normalize_logical_path()` 规范化
pub trait ResourceSource: Send + Sync {
    /// 读取资源字节
    ///
    /// # 参数
    /// - `path`: 逻辑路径（如 `backgrounds/bg.png`，不含 `assets/` 前缀）
    ///
    /// # 返回
    /// 资源字节内容，或错误
    fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError>;

    /// 检查资源是否存在
    fn exists(&self, path: &str) -> bool;

    /// 获取资源的完整路径（用于调试/日志）
    fn full_path(&self, path: &str) -> String;

    /// 列出指定目录下的所有文件路径
    ///
    /// # 参数
    /// - `dir_path`: 目录路径（如 `scripts`）
    ///
    /// # 返回
    /// 文件路径列表（逻辑路径）
    fn list_files(&self, dir_path: &str) -> Vec<String>;
}

/// 文件系统资源来源
///
/// 从本地文件系统读取资源，用于开发模式。
#[derive(Debug, Clone)]
pub struct FsSource {
    /// 资源根目录
    base_path: PathBuf,
}

impl FsSource {
    /// 创建文件系统资源来源
    ///
    /// # 参数
    /// - `base_path`: 资源根目录（如 `assets`）
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    /// 解析逻辑路径到完整文件系统路径
    ///
    /// 将规范化的逻辑路径转换为实际的文件系统路径。
    fn resolve(&self, logical_path: &str) -> PathBuf {
        // 先规范化逻辑路径
        let normalized = normalize_logical_path(logical_path);

        // 拼接 base_path
        self.base_path.join(&normalized)
    }
}

impl ResourceSource for FsSource {
    fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        let full_path = self.resolve(path);

        std::fs::read(&full_path).map_err(|e| ResourceError::LoadFailed {
            path: full_path.to_string_lossy().to_string(),
            kind: "file".to_string(),
            message: e.to_string(),
        })
    }

    fn exists(&self, path: &str) -> bool {
        let full_path = self.resolve(path);
        full_path.exists()
    }

    fn full_path(&self, path: &str) -> String {
        self.resolve(path).to_string_lossy().to_string()
    }

    fn list_files(&self, dir_path: &str) -> Vec<String> {
        let full_dir = self.resolve(dir_path);

        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&full_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    // 转换为相对于 base_path 的逻辑路径
                    if let Ok(relative) = path.strip_prefix(&self.base_path) {
                        files.push(relative.to_string_lossy().replace('\\', "/"));
                    }
                }
            }
        }
        files
    }
}

/// ZIP 文件资源来源
///
/// 从 ZIP 文件读取资源，用于发布模式。
/// 内部使用缓存避免重复解压。
pub struct ZipSource {
    /// ZIP 文件路径
    zip_path: PathBuf,
    /// 文件索引缓存（逻辑路径 -> ZIP 内索引）
    index_cache: Mutex<Option<HashMap<String, usize>>>,
}

impl ZipSource {
    /// 创建 ZIP 资源来源
    ///
    /// # 参数
    /// - `zip_path`: ZIP 文件路径
    pub fn new(zip_path: impl Into<PathBuf>) -> Self {
        Self {
            zip_path: zip_path.into(),
            index_cache: Mutex::new(None),
        }
    }

    /// 构建文件索引
    fn build_index(&self) -> Result<HashMap<String, usize>, ResourceError> {
        let file = File::open(&self.zip_path).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("无法打开 ZIP 文件: {}", e),
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("无法读取 ZIP 文件: {}", e),
        })?;

        let mut index = HashMap::new();
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i)
                && !file.is_dir()
            {
                // 规范化路径（统一使用 /）
                let name = file.name().replace('\\', "/");
                index.insert(name, i);
            }
        }

        Ok(index)
    }

    /// 获取或构建索引
    fn get_index(&self) -> Result<HashMap<String, usize>, ResourceError> {
        let mut cache = self.index_cache.lock().unwrap();
        if cache.is_none() {
            *cache = Some(self.build_index()?);
        }
        Ok(cache.as_ref().unwrap().clone())
    }
}

impl ResourceSource for ZipSource {
    fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        // 使用统一的路径规范化（会移除 assets/ 前缀）
        let zip_path = normalize_logical_path(path);

        let index = self.get_index()?;

        let file_index = index
            .get(&zip_path)
            .ok_or_else(|| ResourceError::NotFound {
                path: zip_path.clone(),
            })?;

        let file = File::open(&self.zip_path).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("无法打开 ZIP 文件: {}", e),
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("无法读取 ZIP 文件: {}", e),
        })?;

        let mut zip_file =
            archive
                .by_index(*file_index)
                .map_err(|e| ResourceError::LoadFailed {
                    path: zip_path.clone(),
                    kind: "zip_entry".to_string(),
                    message: format!("无法读取 ZIP 条目: {}", e),
                })?;

        let mut buffer = Vec::new();
        zip_file
            .read_to_end(&mut buffer)
            .map_err(|e| ResourceError::LoadFailed {
                path: zip_path.clone(),
                kind: "zip_read".to_string(),
                message: format!("读取 ZIP 条目失败: {}", e),
            })?;

        Ok(buffer)
    }

    fn exists(&self, path: &str) -> bool {
        let zip_path = normalize_logical_path(path);

        self.get_index()
            .map(|index| index.contains_key(&zip_path))
            .unwrap_or(false)
    }

    fn full_path(&self, path: &str) -> String {
        let zip_path = normalize_logical_path(path);
        format!("zip://{}#{}", self.zip_path.display(), zip_path)
    }

    fn list_files(&self, dir_path: &str) -> Vec<String> {
        // 规范化目录路径
        let zip_dir = normalize_logical_path(dir_path);

        // 确保目录路径以 / 结尾（用于前缀匹配）
        let dir_prefix = if zip_dir.ends_with('/') || zip_dir.is_empty() {
            zip_dir.clone()
        } else {
            format!("{}/", zip_dir)
        };

        let mut files = Vec::new();
        if let Ok(index) = self.get_index() {
            for (path, _) in index.iter() {
                // 检查路径是否在指定目录下
                if path.starts_with(&dir_prefix) {
                    // 移除目录前缀，只保留文件名和子路径
                    let relative = &path[dir_prefix.len()..];
                    // 如果不在子目录中（直接文件），添加到列表
                    if !relative.contains('/') {
                        // 返回完整的逻辑路径
                        files.push(path.clone());
                    }
                }
            }
        }
        files
    }
}

// ZipSource 需要实现 Send + Sync
// Mutex 保证了线程安全
unsafe impl Send for ZipSource {}
unsafe impl Sync for ZipSource {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_source_resolve() {
        let source = FsSource::new("assets");

        // 相对路径
        assert_eq!(source.resolve("bg.png"), PathBuf::from("assets/bg.png"));
        assert_eq!(
            source.resolve("backgrounds/bg.png"),
            PathBuf::from("assets/backgrounds/bg.png")
        );

        // 带 .. 的路径
        assert_eq!(
            source.resolve("scripts/../backgrounds/bg.png"),
            PathBuf::from("assets/backgrounds/bg.png")
        );

        // 已包含 assets 前缀的路径（会被规范化移除再重新加上）
        assert_eq!(
            source.resolve("assets/bg.png"),
            PathBuf::from("assets/bg.png")
        );
    }

    #[test]
    fn test_normalize_logical_path() {
        // 基本规范化
        assert_eq!(normalize_logical_path("bg.png"), "bg.png");
        assert_eq!(normalize_logical_path("./bg.png"), "bg.png");
        assert_eq!(
            normalize_logical_path("backgrounds\\bg.png"),
            "backgrounds/bg.png"
        );

        // 处理 ..
        assert_eq!(
            normalize_logical_path("scripts/../backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
        assert_eq!(normalize_logical_path("a/b/../../c/d.png"), "c/d.png");

        // 移除 assets/ 前缀
        assert_eq!(
            normalize_logical_path("assets/backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
    }
}
