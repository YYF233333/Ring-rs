//! # Resource Source 模块
//!
//! 资源来源抽象层，支持从不同来源（文件系统、ZIP 包等）读取资源。

use super::ResourceError;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// 资源来源 trait
///
/// 抽象资源读取接口，允许从不同来源加载资源：
/// - `FsSource`：从文件系统读取（开发模式）
/// - `ZipSource`：从 ZIP 包读取（发布模式，阶段 18.2）
pub trait ResourceSource: Send + Sync {
    /// 读取资源字节
    ///
    /// # 参数
    /// - `path`: 逻辑路径（相对于 assets_root，如 `backgrounds/bg.png`）
    ///
    /// # 返回
    /// 资源字节内容，或错误
    fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError>;

    /// 检查资源是否存在
    fn exists(&self, path: &str) -> bool;

    /// 获取资源的完整路径（用于调试/日志）
    fn full_path(&self, path: &str) -> String;
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

    /// 规范化逻辑路径
    ///
    /// 处理 `..`、`.` 等相对路径组件，返回规范化后的路径字符串。
    /// 使用 `/` 作为路径分隔符（跨平台统一）。
    fn normalize_path(path: &Path) -> String {
        use std::path::Component;

        let mut components: Vec<String> = Vec::new();

        for component in path.components() {
            match component {
                Component::Prefix(p) => {
                    // Windows 盘符，如 C:
                    components.push(p.as_os_str().to_string_lossy().to_string());
                }
                Component::RootDir => {
                    // 根目录 /
                    if components.is_empty() {
                        components.push(String::new());
                    }
                }
                Component::CurDir => {
                    // . 当前目录，跳过
                }
                Component::ParentDir => {
                    // .. 父目录，弹出上一级（保留盘符）
                    if components.len() > 1
                        || (components.len() == 1 && !components[0].contains(':'))
                    {
                        components.pop();
                    }
                }
                Component::Normal(name) => {
                    components.push(name.to_string_lossy().to_string());
                }
            }
        }

        if components.is_empty() {
            return String::new();
        }

        // 处理 Windows 盘符
        if components.len() >= 2 && components[0].contains(':') {
            let drive = &components[0];
            let rest = components[1..].join("/");
            format!("{}/{}", drive, rest)
        } else {
            components.join("/")
        }
    }

    /// 解析逻辑路径到完整文件系统路径
    fn resolve(&self, logical_path: &str) -> String {
        let path = Path::new(logical_path);

        // 绝对路径直接规范化
        if path.is_absolute() {
            return Self::normalize_path(path);
        }

        // 检查是否已包含 base_path
        let base_str = self.base_path.to_string_lossy();
        if logical_path.contains(base_str.as_ref()) {
            return Self::normalize_path(path);
        }

        // 拼接 base_path
        let full_path = self.base_path.join(logical_path);
        Self::normalize_path(&full_path)
    }
}

impl ResourceSource for FsSource {
    fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        let full_path = self.resolve(path);

        std::fs::read(&full_path).map_err(|e| ResourceError::LoadFailed {
            path: full_path.clone(),
            kind: "file".to_string(),
            message: e.to_string(),
        })
    }

    fn exists(&self, path: &str) -> bool {
        let full_path = self.resolve(path);
        Path::new(&full_path).exists()
    }

    fn full_path(&self, path: &str) -> String {
        self.resolve(path)
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
            if let Ok(file) = archive.by_index(i) {
                if !file.is_dir() {
                    // 规范化路径（统一使用 /）
                    let name = file.name().replace('\\', "/");
                    index.insert(name, i);
                }
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

    /// 规范化逻辑路径
    /// 
    /// 处理路径组件，包括：
    /// - 统一使用 / 分隔符
    /// - 移除开头的 ./
    /// - 处理 .. 组件（向上级目录）
    /// - 处理 . 组件（当前目录）
    fn normalize_path(path: &str) -> String {
        // 统一使用 / 分隔符
        let normalized = path.replace('\\', "/");
        
        // 移除开头的 ./
        let path = if normalized.starts_with("./") {
            &normalized[2..]
        } else {
            &normalized
        };
        
        // 处理路径组件
        let mut components = Vec::new();
        for component in path.split('/') {
            match component {
                "" | "." => {
                    // 空组件或当前目录，跳过
                }
                ".." => {
                    // 上级目录，移除最后一个组件（如果存在）
                    if !components.is_empty() {
                        components.pop();
                    }
                }
                _ => {
                    // 正常路径组件
                    components.push(component);
                }
            }
        }
        
        // 重新组合路径
        components.join("/")
    }
}

impl ResourceSource for ZipSource {
    fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        let normalized = Self::normalize_path(path);
        
        // 移除开头的 assets/ 前缀（如果存在）
        // ZIP 文件中的路径不包含 assets 前缀
        let zip_path = if normalized.starts_with("assets/") {
            &normalized[7..] // 移除 "assets/" 前缀
        } else {
            &normalized
        };
        
        let index = self.get_index()?;

        let file_index = index.get(zip_path).ok_or_else(|| ResourceError::NotFound {
            path: zip_path.to_string(),
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

        let mut zip_file = archive.by_index(*file_index).map_err(|e| {
            ResourceError::LoadFailed {
                path: zip_path.to_string(),
                kind: "zip_entry".to_string(),
                message: format!("无法读取 ZIP 条目: {}", e),
            }
        })?;

        let mut buffer = Vec::new();
        zip_file.read_to_end(&mut buffer).map_err(|e| {
            ResourceError::LoadFailed {
                path: zip_path.to_string(),
                kind: "zip_read".to_string(),
                message: format!("读取 ZIP 条目失败: {}", e),
            }
        })?;

        Ok(buffer)
    }

    fn exists(&self, path: &str) -> bool {
        let normalized = Self::normalize_path(path);
        
        // 移除开头的 assets/ 前缀（如果存在）
        let zip_path = if normalized.starts_with("assets/") {
            &normalized[7..]
        } else {
            &normalized
        };
        
        self.get_index()
            .map(|index| index.contains_key(zip_path))
            .unwrap_or(false)
    }

    fn full_path(&self, path: &str) -> String {
        let normalized = Self::normalize_path(path);
        let zip_path = if normalized.starts_with("assets/") {
            &normalized[7..]
        } else {
            &normalized
        };
        format!("zip://{}#{}", self.zip_path.display(), zip_path)
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
    fn test_fs_source_resolve_relative() {
        let source = FsSource::new("assets");
        assert_eq!(source.resolve("bg.png"), "assets/bg.png");
        assert_eq!(
            source.resolve("backgrounds/bg.png"),
            "assets/backgrounds/bg.png"
        );
    }

    #[test]
    fn test_fs_source_resolve_with_dotdot() {
        let source = FsSource::new("assets");
        // scripts/../backgrounds/bg.png -> assets/backgrounds/bg.png
        assert_eq!(
            source.resolve("scripts/../backgrounds/bg.png"),
            "assets/backgrounds/bg.png"
        );
    }

    #[test]
    fn test_fs_source_resolve_already_contains_base() {
        let source = FsSource::new("assets");
        // 已经包含 assets 的路径不应重复拼接
        assert_eq!(source.resolve("assets/bg.png"), "assets/bg.png");
    }

    #[test]
    fn test_normalize_path_handles_dot() {
        let source = FsSource::new("assets");
        assert_eq!(source.resolve("./backgrounds/bg.png"), "assets/backgrounds/bg.png");
    }

    #[test]
    fn test_zip_normalize_path() {
        assert_eq!(ZipSource::normalize_path("./bg.png"), "bg.png");
        assert_eq!(ZipSource::normalize_path("backgrounds\\bg.png"), "backgrounds/bg.png");
        assert_eq!(ZipSource::normalize_path("backgrounds/bg.png"), "backgrounds/bg.png");
        // 测试 .. 组件处理
        assert_eq!(ZipSource::normalize_path("scripts/../backgrounds/bg.png"), "backgrounds/bg.png");
        assert_eq!(ZipSource::normalize_path("scripts/../backgrounds/../bgm/music.mp3"), "bgm/music.mp3");
        assert_eq!(ZipSource::normalize_path("../backgrounds/bg.png"), "backgrounds/bg.png");
        // 测试 . 组件处理
        assert_eq!(ZipSource::normalize_path("scripts/./images/char.png"), "scripts/images/char.png");
        // 测试多个 .. 组件
        assert_eq!(ZipSource::normalize_path("a/b/../../c/d.png"), "c/d.png");
    }
}
