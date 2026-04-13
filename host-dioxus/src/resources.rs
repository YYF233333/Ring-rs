//! 资源管理系统
//!
//! 提供 [`LogicalPath`] 路径规范化、[`ResourceSource`] 后端抽象和 [`ResourceManager`] 统一入口。

use std::path::{Path, PathBuf};
use thiserror::Error;

// ── LogicalPath ──────────────────────────────────────────────────────────────

/// 规范化的逻辑资源路径。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LogicalPath(String);

impl LogicalPath {
    pub fn new(raw: &str) -> Self {
        Self(normalize_logical_path(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn file_stem(&self) -> &str {
        let filename = self.0.rsplit('/').next().unwrap_or(&self.0);
        if let Some(dot_pos) = filename.rfind('.') {
            &filename[..dot_pos]
        } else {
            filename
        }
    }

    pub fn parent_dir(&self) -> &str {
        if let Some(last_slash) = self.0.rfind('/') {
            &self.0[..last_slash]
        } else {
            ""
        }
    }
}

impl std::fmt::Display for LogicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

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
            _ => components.push(component),
        }
    }

    let result = components.join("/");
    result
        .strip_prefix("assets/")
        .unwrap_or(&result)
        .to_string()
}

// ── ResourceError ────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("加载 {kind} 资源失败: {path} - {message}")]
    LoadFailed {
        path: String,
        kind: String,
        message: String,
    },
    #[error("资源未找到: {path}")]
    NotFound { path: String },
}

// ── ResourceSource trait ─────────────────────────────────────────────────────

pub trait ResourceSource: Send + Sync {
    fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError>;
    fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError>;
    fn exists(&self, path: &LogicalPath) -> bool;
}

// ── FsSource ─────────────────────────────────────────────────────────────────

pub struct FsSource {
    base_path: PathBuf,
}

impl FsSource {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }

    fn resolve(&self, path: &LogicalPath) -> PathBuf {
        self.base_path.join(path.as_str())
    }
}

impl ResourceSource for FsSource {
    fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError> {
        let full = self.resolve(path);
        std::fs::read_to_string(&full).map_err(|e| ResourceError::LoadFailed {
            path: full.to_string_lossy().to_string(),
            kind: "text".to_string(),
            message: e.to_string(),
        })
    }

    fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        let full = self.resolve(path);
        std::fs::read(&full).map_err(|e| ResourceError::LoadFailed {
            path: full.to_string_lossy().to_string(),
            kind: "file".to_string(),
            message: e.to_string(),
        })
    }

    fn exists(&self, path: &LogicalPath) -> bool {
        self.resolve(path).exists()
    }
}

// ── ZipSource ────────────────────────────────────────────────────────────────

mod zip_source {
    use super::*;
    use std::collections::HashSet;
    use std::io::Read;
    use std::sync::Mutex;

    pub struct ZipSource {
        archive: Mutex<zip::ZipArchive<std::fs::File>>,
        index: HashSet<String>,
    }

    impl ZipSource {
        pub fn open(zip_path: impl AsRef<Path>) -> Result<Self, ResourceError> {
            let file =
                std::fs::File::open(zip_path.as_ref()).map_err(|e| ResourceError::LoadFailed {
                    path: zip_path.as_ref().to_string_lossy().to_string(),
                    kind: "zip".to_string(),
                    message: e.to_string(),
                })?;
            let archive = zip::ZipArchive::new(file).map_err(|e| ResourceError::LoadFailed {
                path: zip_path.as_ref().to_string_lossy().to_string(),
                kind: "zip".to_string(),
                message: e.to_string(),
            })?;
            let index: HashSet<String> = (0..archive.len())
                .filter_map(|i| archive.name_for_index(i).map(normalize_logical_path))
                .collect();
            Ok(Self {
                archive: Mutex::new(archive),
                index,
            })
        }
    }

    impl ResourceSource for ZipSource {
        fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError> {
            let bytes = self.read_bytes(path)?;
            String::from_utf8(bytes).map_err(|e| ResourceError::LoadFailed {
                path: path.as_str().to_string(),
                kind: "text".to_string(),
                message: format!("UTF-8 解码失败: {e}"),
            })
        }

        fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
            let mut archive = self.archive.lock().map_err(|e| ResourceError::LoadFailed {
                path: path.as_str().to_string(),
                kind: "zip".to_string(),
                message: format!("锁获取失败: {e}"),
            })?;
            let mut entry =
                archive
                    .by_name(path.as_str())
                    .map_err(|_| ResourceError::NotFound {
                        path: path.as_str().to_string(),
                    })?;
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut buf)
                .map_err(|e| ResourceError::LoadFailed {
                    path: path.as_str().to_string(),
                    kind: "zip".to_string(),
                    message: e.to_string(),
                })?;
            Ok(buf)
        }

        fn exists(&self, path: &LogicalPath) -> bool {
            self.index.contains(path.as_str())
        }
    }
}

pub use zip_source::ZipSource;

// ── ResourceManager ──────────────────────────────────────────────────────────

pub struct ResourceManager {
    source: Box<dyn ResourceSource>,
    base_path: PathBuf,
}

impl ResourceManager {
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let base = base_path.into();
        Self {
            source: Box::new(FsSource::new(&base)),
            base_path: base,
        }
    }

    pub fn with_source(source: Box<dyn ResourceSource>, base_path: impl Into<PathBuf>) -> Self {
        Self {
            source,
            base_path: base_path.into(),
        }
    }

    pub fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError> {
        self.source.read_text(path)
    }

    pub fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        self.source.read_bytes(path)
    }

    pub fn resource_exists(&self, path: &LogicalPath) -> bool {
        self.source.exists(path)
    }

    /// 尝试读取文本资源，不存在时返回 None。
    pub fn read_text_optional(&self, path: &LogicalPath) -> Option<String> {
        if !self.source.exists(path) {
            return None;
        }
        self.read_text(path).ok()
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// ── MIME 推断 ────────────────────────────────────────────────────────────────

pub fn guess_mime_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "json" => "application/json",
        "css" => "text/css",
        "js" => "text/javascript",
        "html" | "htm" => "text/html",
        "txt" | "md" | "rks" => "text/plain",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_strips_assets_prefix() {
        assert_eq!(normalize_logical_path("assets/bg/sky.png"), "bg/sky.png");
    }

    #[test]
    fn normalize_resolves_dotdot() {
        assert_eq!(
            normalize_logical_path("scripts/../bg/sky.png"),
            "bg/sky.png"
        );
    }

    #[test]
    fn fs_source_read_text() {
        let dir = std::env::temp_dir().join("ring_dioxus_test_fs");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("hello.txt"), "world").unwrap();
        let source = FsSource::new(&dir);
        let path = LogicalPath::new("hello.txt");
        assert!(source.exists(&path));
        assert_eq!(source.read_text(&path).unwrap(), "world");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn guess_mime_common_types() {
        assert_eq!(guess_mime_type("bg/sky.png"), "image/png");
        assert_eq!(guess_mime_type("bgm/track.mp3"), "audio/mpeg");
        assert_eq!(guess_mime_type("unknown.xyz"), "application/octet-stream");
    }

    mod zip_tests {
        use super::*;
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};

        fn create_test_zip(entries: &[(&str, &[u8])]) -> PathBuf {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("ring_dioxus_zip_{unique}"));
            let _ = std::fs::create_dir_all(&dir);
            let zip_path = dir.join("test_assets.zip");
            let file = std::fs::File::create(&zip_path).unwrap();
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            for (name, content) in entries {
                writer.start_file(*name, options).unwrap();
                writer.write_all(content).unwrap();
            }
            writer.finish().unwrap();
            zip_path
        }

        #[test]
        fn zip_source_read_text() {
            let zip_path = create_test_zip(&[("scripts/main.rks", b"Hello from ZIP")]);
            let source = ZipSource::open(&zip_path).unwrap();
            let path = LogicalPath::new("scripts/main.rks");
            assert!(source.exists(&path));
            assert_eq!(source.read_text(&path).unwrap(), "Hello from ZIP");
            std::fs::remove_file(&zip_path).ok();
        }

        #[test]
        fn resource_manager_with_zip_source() {
            let zip_path = create_test_zip(&[("test.txt", b"via manager")]);
            let source = ZipSource::open(&zip_path).unwrap();
            let rm = ResourceManager::with_source(Box::new(source), "/fake/base");
            let path = LogicalPath::new("test.txt");
            assert_eq!(rm.read_text(&path).unwrap(), "via manager");
            std::fs::remove_file(&zip_path).ok();
        }
    }
}
