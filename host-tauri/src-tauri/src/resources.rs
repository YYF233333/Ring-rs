//! 资源管理系统
//!
//! 提供 [`LogicalPath`] 路径规范化、[`ResourceSource`] 后端抽象和 [`ResourceManager`] 统一入口。
//!
//! 所有资源（脚本、JSON、图片、音频、视频）统一通过 [`ResourceSource`] trait 读取，
//! 支持文件系统和 ZIP 两种来源。前端通过 `ring-asset` 自定义协议访问资源，
//! 协议 handler 内部使用 [`ResourceManager`] 透明处理 FS/ZIP 差异。

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

// ── ResourceSource trait ─────────────────────────────────────────────────────

/// 资源来源抽象——后端通过此 trait 读取脚本、JSON 等资源，
/// 屏蔽文件系统 vs ZIP 的差异。
pub trait ResourceSource: Send + Sync {
    /// 读取文本资源
    fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError>;
    /// 读取二进制资源
    fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError>;
    /// 检查资源是否存在
    fn exists(&self, path: &LogicalPath) -> bool;
}

// ── FsSource ─────────────────────────────────────────────────────────────────

/// 文件系统资源来源
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

    /// ZIP 文件资源来源
    ///
    /// 通过预建路径索引加速 `exists()` 查询，读取时加锁访问 `ZipArchive`。
    pub struct ZipSource {
        archive: Mutex<zip::ZipArchive<std::fs::File>>,
        index: HashSet<String>,
    }

    impl ZipSource {
        /// 打开 ZIP 文件并构建路径索引
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

/// 统一资源管理器
///
/// 后端读取通过 [`ResourceSource`] trait 代理（支持 FS / ZIP）。
/// 前端通过 `ring-asset` 自定义协议请求资源，handler 内部委托到此管理器。
pub struct ResourceManager {
    source: Box<dyn ResourceSource>,
    /// 文件系统根路径（开发期 debug server 静态文件服务仍需要）
    base_path: PathBuf,
}

impl ResourceManager {
    /// 创建文件系统模式的资源管理器
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        let base = base_path.into();
        Self {
            source: Box::new(FsSource::new(&base)),
            base_path: base,
        }
    }

    /// 使用指定的 ResourceSource 创建资源管理器
    ///
    /// `base_path` 仍需提供，用于前端 asset protocol URL 解析。
    pub fn with_source(source: Box<dyn ResourceSource>, base_path: impl Into<PathBuf>) -> Self {
        Self {
            source,
            base_path: base_path.into(),
        }
    }

    /// 读取文本资源
    pub fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError> {
        self.source.read_text(path)
    }

    /// 读取二进制资源
    pub fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        self.source.read_bytes(path)
    }

    /// 返回逻辑路径对应的文件系统绝对路径（用于 asset 协议）
    #[allow(dead_code)]
    pub fn resolve_fs_path(&self, path: &LogicalPath) -> PathBuf {
        self.base_path.join(path.as_str())
    }

    /// 检查资源是否存在
    #[allow(dead_code)]
    pub fn resource_exists(&self, path: &LogicalPath) -> bool {
        self.source.exists(path)
    }

    /// 获取 assets 根目录（文件系统路径，供 asset protocol 使用）
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

// ── MIME 推断 ────────────────────────────────────────────────────────────────

/// 根据文件扩展名推断 MIME type（用于自定义协议 handler 的 Content-Type）。
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

// ── 测试 ─────────────────────────────────────────────────────────────────────

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
        let dir = std::env::temp_dir().join("ring_test_fs_source");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(dir.join("hello.txt"), "world").unwrap();

        let source = FsSource::new(&dir);
        let path = LogicalPath::new("hello.txt");
        assert!(source.exists(&path));
        assert_eq!(source.read_text(&path).unwrap(), "world");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fs_source_not_found() {
        let source = FsSource::new("/nonexistent_dir_ring_test");
        let path = LogicalPath::new("nope.txt");
        assert!(!source.exists(&path));
        assert!(source.read_text(&path).is_err());
    }

    #[test]
    fn guess_mime_common_types() {
        assert_eq!(guess_mime_type("bg/sky.png"), "image/png");
        assert_eq!(guess_mime_type("bg/sky.jpg"), "image/jpeg");
        assert_eq!(guess_mime_type("bg/sky.jpeg"), "image/jpeg");
        assert_eq!(guess_mime_type("bg/sky.webp"), "image/webp");
        assert_eq!(guess_mime_type("bgm/track.mp3"), "audio/mpeg");
        assert_eq!(guess_mime_type("bgm/track.ogg"), "audio/ogg");
        assert_eq!(guess_mime_type("video/op.mp4"), "video/mp4");
        assert_eq!(guess_mime_type("ui/layout.json"), "application/json");
        assert_eq!(guess_mime_type("gui/theme.css"), "text/css");
        assert_eq!(guess_mime_type("fonts/noto.woff2"), "font/woff2");
        assert_eq!(guess_mime_type("unknown.xyz"), "application/octet-stream");
    }

    #[test]
    fn guess_mime_case_insensitive() {
        assert_eq!(guess_mime_type("BG/SKY.PNG"), "image/png");
        assert_eq!(guess_mime_type("track.MP3"), "audio/mpeg");
    }

    mod zip_tests {
        use super::*;
        use std::io::Write;
        use std::time::{SystemTime, UNIX_EPOCH};

        fn create_test_zip(entries: &[(&str, &[u8])]) -> std::path::PathBuf {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let dir = std::env::temp_dir().join(format!("ring_test_zip_{unique}"));
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
            let zip_path = create_test_zip(&[
                ("scripts/main.rks", b"Hello from ZIP"),
                ("ui/screens.json", b"{}"),
            ]);

            let source = ZipSource::open(&zip_path).unwrap();
            let path = LogicalPath::new("scripts/main.rks");
            assert!(source.exists(&path));
            assert_eq!(source.read_text(&path).unwrap(), "Hello from ZIP");

            let ui_path = LogicalPath::new("ui/screens.json");
            assert_eq!(source.read_text(&ui_path).unwrap(), "{}");

            let missing = LogicalPath::new("nope.txt");
            assert!(!source.exists(&missing));
            assert!(source.read_text(&missing).is_err());

            std::fs::remove_file(&zip_path).ok();
        }

        #[test]
        fn zip_source_read_bytes() {
            let data: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0];
            let zip_path = create_test_zip(&[("img/test.jpg", data)]);

            let source = ZipSource::open(&zip_path).unwrap();
            let path = LogicalPath::new("img/test.jpg");
            assert_eq!(source.read_bytes(&path).unwrap(), data);

            std::fs::remove_file(&zip_path).ok();
        }

        #[test]
        fn resource_manager_with_zip_source() {
            let zip_path = create_test_zip(&[("test.txt", b"via manager")]);

            let source = ZipSource::open(&zip_path).unwrap();
            let rm = ResourceManager::with_source(Box::new(source), "/fake/base");

            let path = LogicalPath::new("test.txt");
            assert_eq!(rm.read_text(&path).unwrap(), "via manager");
            assert_eq!(rm.base_path(), Path::new("/fake/base"));

            std::fs::remove_file(&zip_path).ok();
        }

        #[test]
        fn zip_manifest_via_resource_manager() {
            let manifest_json = br#"{"characters":{},"presets":{},"defaults":{"anchor":{"x":0.5,"y":1.0},"pre_scale":1.0}}"#;
            let zip_path = create_test_zip(&[("manifest.json", manifest_json)]);

            let source = ZipSource::open(&zip_path).unwrap();
            let rm = ResourceManager::with_source(Box::new(source), "/fake/base");

            let path = LogicalPath::new("manifest.json");
            let content = rm.read_text(&path).unwrap();
            let manifest: serde_json::Value = serde_json::from_str(&content).unwrap();
            assert!(manifest.get("defaults").is_some());

            std::fs::remove_file(&zip_path).ok();
        }
    }
}
