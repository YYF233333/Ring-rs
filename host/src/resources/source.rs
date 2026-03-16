//! # Resource Source 模块
//!
//! 资源来源抽象层，支持从不同来源（文件系统、ZIP 包等）读取资源。
//!
//! ## 设计原则
//!
//! - 所有资源路径使用 [`LogicalPath`] 类型（编译期防止与文件系统路径混用）
//! - 加载时由具体实现决定如何解析到实际路径
//! - `FsSource` / `ZipSource` 为 `pub(crate)` 可见性，外部只通过 trait 交互

use super::ResourceError;
use super::path::LogicalPath;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::Mutex;

/// 资源来源 trait
///
/// 抽象资源读取接口，允许从不同来源加载资源。
///
/// ## 路径约定
///
/// 所有路径参数使用 [`LogicalPath`]，构造时已完成规范化，实现者无需再次 normalize。
pub trait ResourceSource: Send + Sync {
    /// 读取资源字节
    fn read(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError>;

    /// 检查资源是否存在
    fn exists(&self, path: &LogicalPath) -> bool;

    /// 获取资源的完整路径（用于调试/日志）
    fn full_path(&self, path: &LogicalPath) -> String;

    /// 列出指定目录下的所有文件路径
    fn list_files(&self, dir_path: &LogicalPath) -> Vec<LogicalPath>;

    /// 如果底层是真实文件系统，返回该资源的文件系统路径。
    ///
    /// 非文件系统来源（ZIP/网络等）返回 `None`。
    fn backing_path(&self, path: &LogicalPath) -> Option<PathBuf> {
        let _ = path;
        None
    }
}

/// 文件系统资源来源
///
/// 从本地文件系统读取资源，用于开发模式。
#[derive(Debug, Clone)]
pub(crate) struct FsSource {
    base_path: PathBuf,
}

impl FsSource {
    pub(crate) fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn resolve(&self, path: &LogicalPath) -> PathBuf {
        self.base_path.join(path.as_str())
    }
}

impl ResourceSource for FsSource {
    fn read(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        let full_path = self.resolve(path);

        std::fs::read(&full_path).map_err(|e| ResourceError::LoadFailed {
            path: full_path.to_string_lossy().to_string(),
            kind: "file".to_string(),
            message: e.to_string(),
        })
    }

    fn exists(&self, path: &LogicalPath) -> bool {
        self.resolve(path).exists()
    }

    fn full_path(&self, path: &LogicalPath) -> String {
        self.resolve(path).to_string_lossy().to_string()
    }

    fn list_files(&self, dir_path: &LogicalPath) -> Vec<LogicalPath> {
        let full_dir = self.resolve(dir_path);

        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&full_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Ok(relative) = path.strip_prefix(&self.base_path)
                {
                    let logical = relative.to_string_lossy().replace('\\', "/");
                    files.push(LogicalPath::new(&logical));
                }
            }
        }
        files
    }

    fn backing_path(&self, path: &LogicalPath) -> Option<PathBuf> {
        Some(self.resolve(path))
    }
}

/// ZIP 文件资源来源
///
/// 从 ZIP 文件读取资源，用于发布模式。
/// 内部使用缓存避免重复解压。
pub(crate) struct ZipSource {
    zip_path: PathBuf,
    index_cache: Mutex<Option<HashMap<String, usize>>>,
}

impl ZipSource {
    pub(crate) fn new(zip_path: impl Into<PathBuf>) -> Self {
        Self {
            zip_path: zip_path.into(),
            index_cache: Mutex::new(None),
        }
    }

    fn build_index(&self) -> Result<HashMap<String, usize>, ResourceError> {
        let file = File::open(&self.zip_path).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("Cannot open ZIP file: {}", e),
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("Cannot read ZIP file: {}", e),
        })?;

        let mut index = HashMap::new();
        for i in 0..archive.len() {
            if let Ok(file) = archive.by_index(i)
                && !file.is_dir()
            {
                let name = file.name().replace('\\', "/");
                index.insert(name, i);
            }
        }

        Ok(index)
    }

    fn get_index(&self) -> Result<HashMap<String, usize>, ResourceError> {
        let mut cache = self.index_cache.lock().unwrap();
        if cache.is_none() {
            *cache = Some(self.build_index()?);
        }
        Ok(cache.as_ref().unwrap().clone())
    }
}

impl ResourceSource for ZipSource {
    fn read(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        let key = path.as_str();
        let index = self.get_index()?;

        let file_index = index.get(key).ok_or_else(|| ResourceError::NotFound {
            path: key.to_string(),
        })?;

        let file = File::open(&self.zip_path).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("Cannot open ZIP file: {}", e),
        })?;

        let mut archive = zip::ZipArchive::new(file).map_err(|e| ResourceError::LoadFailed {
            path: self.zip_path.to_string_lossy().to_string(),
            kind: "zip".to_string(),
            message: format!("Cannot read ZIP file: {}", e),
        })?;

        let mut zip_file =
            archive
                .by_index(*file_index)
                .map_err(|e| ResourceError::LoadFailed {
                    path: key.to_string(),
                    kind: "zip_entry".to_string(),
                    message: format!("Cannot read ZIP entry: {}", e),
                })?;

        let mut buffer = Vec::new();
        zip_file
            .read_to_end(&mut buffer)
            .map_err(|e| ResourceError::LoadFailed {
                path: key.to_string(),
                kind: "zip_read".to_string(),
                message: format!("Failed to read ZIP entry: {}", e),
            })?;

        Ok(buffer)
    }

    fn exists(&self, path: &LogicalPath) -> bool {
        self.get_index()
            .map(|index| index.contains_key(path.as_str()))
            .unwrap_or(false)
    }

    fn full_path(&self, path: &LogicalPath) -> String {
        format!("zip://{}#{}", self.zip_path.display(), path)
    }

    fn list_files(&self, dir_path: &LogicalPath) -> Vec<LogicalPath> {
        let dir_str = dir_path.as_str();
        let dir_prefix = if dir_str.ends_with('/') || dir_str.is_empty() {
            dir_str.to_string()
        } else {
            format!("{}/", dir_str)
        };

        let mut files = Vec::new();
        if let Ok(index) = self.get_index() {
            for (path, _) in index.iter() {
                if path.starts_with(&dir_prefix) {
                    let relative = &path[dir_prefix.len()..];
                    if !relative.contains('/') {
                        files.push(LogicalPath::new(path));
                    }
                }
            }
        }
        files
    }
}

// Mutex guarantees thread safety
unsafe impl Send for ZipSource {}
unsafe impl Sync for ZipSource {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir_with_files(files: &[(&str, &[u8])]) -> (tempfile::TempDir, FsSource) {
        let dir = tempfile::tempdir().expect("tempdir");
        for (name, content) in files {
            let path = dir.path().join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let mut f = std::fs::File::create(&path).unwrap();
            f.write_all(content).unwrap();
        }
        let source = FsSource::new(dir.path().to_path_buf());
        (dir, source)
    }

    #[test]
    fn test_fs_source_resolve() {
        let source = FsSource::new("assets");

        let p = LogicalPath::new("bg.png");
        assert_eq!(source.resolve(&p), PathBuf::from("assets/bg.png"));

        let p2 = LogicalPath::new("backgrounds/bg.png");
        assert_eq!(
            source.resolve(&p2),
            PathBuf::from("assets/backgrounds/bg.png")
        );

        // LogicalPath::new normalizes, so `assets/` prefix is already stripped
        let p3 = LogicalPath::new("assets/bg.png");
        assert_eq!(source.resolve(&p3), PathBuf::from("assets/bg.png"));
    }

    #[test]
    fn test_fs_source_backing_path() {
        let source = FsSource::new("assets");
        let p = LogicalPath::new("backgrounds/bg.png");
        assert_eq!(
            source.backing_path(&p),
            Some(PathBuf::from("assets/backgrounds/bg.png"))
        );
    }

    #[test]
    fn test_fs_source_exists_and_read() {
        let (_dir, source) = temp_dir_with_files(&[("hello.txt", b"world")]);

        let p = LogicalPath::new("hello.txt");
        assert!(source.exists(&p));
        assert_eq!(source.read(&p).unwrap(), b"world");
    }

    #[test]
    fn test_fs_source_not_exists() {
        let (_dir, source) = temp_dir_with_files(&[]);

        let p = LogicalPath::new("ghost.txt");
        assert!(!source.exists(&p));
        assert!(source.read(&p).is_err());
    }

    #[test]
    fn test_fs_source_full_path_contains_name() {
        let source = FsSource::new("/base");
        let p = LogicalPath::new("img/bg.png");
        let full = source.full_path(&p);
        assert!(full.contains("img") && full.contains("bg.png"));
    }

    #[test]
    fn test_fs_source_list_files() {
        let (_dir, source) = temp_dir_with_files(&[
            ("scripts/a.json", b"{}"),
            ("scripts/b.json", b"{}"),
            ("other/c.txt", b"hi"),
        ]);

        let dir_path = LogicalPath::new("scripts");
        let mut files = source.list_files(&dir_path);
        files.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        assert_eq!(files.len(), 2);
        assert!(files[0].as_str().contains("a.json"));
        assert!(files[1].as_str().contains("b.json"));
    }

    #[test]
    fn test_fs_source_list_files_empty_dir() {
        let (_dir, source) = temp_dir_with_files(&[]);

        let p = LogicalPath::new("nonexistent");
        let files = source.list_files(&p);
        assert!(files.is_empty());
    }

    // ── ZipSource ──────────────────────────────────────────────────────────────

    fn create_test_zip(files: &[(&str, &[u8])]) -> (tempfile::TempDir, PathBuf) {
        use std::io::Write as _;
        let dir = tempfile::tempdir().expect("tempdir");
        let zip_path = dir.path().join("test.zip");
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default();
        for (name, content) in files {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(content).unwrap();
        }
        zip.finish().unwrap();
        (dir, zip_path)
    }

    #[test]
    fn test_zip_source_exists_and_read() {
        let (_dir, zip_path) = create_test_zip(&[("bg/sky.png", b"fake-png")]);
        let source = ZipSource::new(&zip_path);

        let p = LogicalPath::new("bg/sky.png");
        assert!(source.exists(&p));
        assert_eq!(source.read(&p).unwrap(), b"fake-png");
    }

    #[test]
    fn test_zip_source_not_exists() {
        let (_dir, zip_path) = create_test_zip(&[("a.txt", b"hi")]);
        let source = ZipSource::new(&zip_path);

        let p = LogicalPath::new("ghost.txt");
        assert!(!source.exists(&p));
        assert!(matches!(
            source.read(&p),
            Err(ResourceError::NotFound { .. })
        ));
    }

    #[test]
    fn test_zip_source_full_path_format() {
        let source = ZipSource::new("/data/pack.zip");
        let p = LogicalPath::new("images/bg.png");
        let fp = source.full_path(&p);
        assert!(fp.starts_with("zip://"));
        assert!(fp.contains("images/bg.png"));
    }

    #[test]
    fn test_zip_source_list_files() {
        let (_dir, zip_path) = create_test_zip(&[
            ("scripts/a.json", b"{}"),
            ("scripts/b.json", b"{}"),
            ("other/c.txt", b"hi"),
        ]);
        let source = ZipSource::new(&zip_path);

        let dir_p = LogicalPath::new("scripts");
        let mut files = source.list_files(&dir_p);
        files.sort_by(|a, b| a.as_str().cmp(b.as_str()));

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.as_str().contains("a.json")));
        assert!(files.iter().any(|f| f.as_str().contains("b.json")));
    }

    #[test]
    fn test_zip_source_backing_path_is_none() {
        let (_dir, zip_path) = create_test_zip(&[("a.txt", b"hi")]);
        let source = ZipSource::new(&zip_path);
        let p = LogicalPath::new("a.txt");
        assert!(source.backing_path(&p).is_none());
    }

    #[test]
    fn test_zip_source_invalid_zip_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let bad_path = dir.path().join("bad.zip");
        std::fs::write(&bad_path, b"not a zip file").unwrap();
        let source = ZipSource::new(&bad_path);

        let p = LogicalPath::new("any.txt");
        assert!(!source.exists(&p));
    }
}
