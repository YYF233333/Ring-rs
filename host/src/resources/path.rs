//! # 路径规范化模块
//!
//! 提供统一的路径规范化逻辑，所有资源加载都使用此模块。
//!
//! ## 设计原则
//!
//! - 程序内部统一使用 [`LogicalPath`] 表示**相对于 assets_root 的逻辑路径**
//! - 使用 `/` 作为路径分隔符（跨平台统一）
//! - 路径不包含 `assets/` 前缀
//! - 加载时根据 `ResourceSource` 类型决定如何解析到实际路径
//!
//! ## LogicalPath newtype
//!
//! [`LogicalPath`] 是规范化逻辑路径的 newtype 包装。所有通过 [`ResourceManager`]
//! 和 [`ResourceSource`] 进行的资源访问都使用此类型，编译期防止逻辑路径与文件系统路径混用。

use std::path::PathBuf;

/// 规范化的逻辑资源路径。
///
/// 不变量：
/// - 相对于 assets_root（不含 `assets/` 前缀）
/// - 使用 `/` 分隔符
/// - 已解析 `..` 和 `.`
///
/// 只能通过 [`LogicalPath::new()`] 构造（内部调用 [`normalize_logical_path`]），
/// 保证所有实例都满足不变量。
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
    pub fn join(&self, relative: &str) -> Self {
        if self.0.is_empty() {
            Self::new(relative)
        } else {
            Self::new(&format!("{}/{}", self.0, relative))
        }
    }

    /// 转换为 [`PathBuf`]（用于需要 std::path 的场合，如日志）。
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.0)
    }
}

impl std::fmt::Display for LogicalPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// 规范化逻辑路径
///
/// 处理路径组件，包括：
/// - 统一使用 `/` 分隔符
/// - 移除开头的 `./`
/// - 处理 `..` 组件（向上级目录）
/// - 处理 `.` 组件（当前目录）
/// - 移除 `assets/` 前缀（如果存在）
///
/// # 示例
///
/// ```
/// use host::resources::path::normalize_logical_path;
///
/// assert_eq!(normalize_logical_path("scripts/../backgrounds/bg.png"), "backgrounds/bg.png");
/// assert_eq!(normalize_logical_path("assets/bgm/music.mp3"), "bgm/music.mp3");
/// assert_eq!(normalize_logical_path("./backgrounds/bg.png"), "backgrounds/bg.png");
/// ```
pub fn normalize_logical_path(path: &str) -> String {
    // 统一使用 / 分隔符
    let normalized = path.replace('\\', "/");

    // 移除开头的 ./
    let path = normalized.strip_prefix("./").unwrap_or(&normalized);

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
    let result = components.join("/");

    // 移除 assets/ 前缀（如果存在）
    result
        .strip_prefix("assets/")
        .unwrap_or(&result)
        .to_string()
}

/// 解析相对路径
///
/// 将相对于某个基础目录的路径解析为相对于 assets_root 的路径。
/// 用于处理脚本中的相对路径引用。
///
/// # 参数
///
/// - `base_dir`: 基础目录路径（相对于 assets_root，如 `scripts/chapter1`）
/// - `relative_path`: 相对路径（如 `../backgrounds/bg.png`）
///
/// # 返回
///
/// 规范化后的逻辑路径（相对于 assets_root）
///
/// # 示例
///
/// ```
/// use host::resources::path::resolve_relative_path;
///
/// assert_eq!(
///     resolve_relative_path("scripts", "../backgrounds/bg.png"),
///     "backgrounds/bg.png"
/// );
/// assert_eq!(
///     resolve_relative_path("scripts/chapter1", "../common/char.png"),
///     "scripts/common/char.png"
/// );
/// ```
pub fn resolve_relative_path(base_dir: &str, relative_path: &str) -> String {
    // 如果相对路径是绝对路径或以 http 开头，直接返回
    if relative_path.starts_with('/') || relative_path.starts_with("http") {
        return normalize_logical_path(relative_path);
    }

    // 规范化基础目录（移除 assets 前缀等）
    let base = normalize_logical_path(base_dir);

    // 如果基础目录为空，直接规范化相对路径
    if base.is_empty() {
        return normalize_logical_path(relative_path);
    }

    // 拼接路径并规范化
    let combined = format!("{}/{}", base, relative_path);
    normalize_logical_path(&combined)
}

/// 从完整路径提取脚本 ID
///
/// 提取文件名（不含扩展名）作为脚本 ID。
///
/// # 示例
///
/// ```
/// use host::resources::path::extract_script_id;
///
/// assert_eq!(extract_script_id("scripts/test.md"), "test");
/// assert_eq!(extract_script_id("scripts/chapter1/intro.md"), "intro");
/// ```
pub fn extract_script_id(path: &str) -> String {
    let normalized = normalize_logical_path(path);

    // 提取文件名
    let filename = normalized.rsplit('/').next().unwrap_or(&normalized);

    // 移除扩展名
    if let Some(dot_pos) = filename.rfind('.') {
        filename[..dot_pos].to_string()
    } else {
        filename.to_string()
    }
}

/// 从脚本路径提取基础目录
///
/// 提取脚本所在的目录路径，用于解析脚本内的相对路径。
///
/// # 示例
///
/// ```
/// use host::resources::path::extract_base_dir;
///
/// assert_eq!(extract_base_dir("scripts/test.md"), "scripts");
/// assert_eq!(extract_base_dir("scripts/chapter1/intro.md"), "scripts/chapter1");
/// assert_eq!(extract_base_dir("test.md"), "");
/// ```
pub fn extract_base_dir(path: &str) -> String {
    let normalized = normalize_logical_path(path);

    if let Some(last_slash) = normalized.rfind('/') {
        normalized[..last_slash].to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic() {
        assert_eq!(
            normalize_logical_path("backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
        assert_eq!(
            normalize_logical_path("./backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
        assert_eq!(
            normalize_logical_path("backgrounds\\bg.png"),
            "backgrounds/bg.png"
        );
    }

    #[test]
    fn test_normalize_with_dotdot() {
        assert_eq!(
            normalize_logical_path("scripts/../backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
        assert_eq!(normalize_logical_path("a/b/../../c/d.png"), "c/d.png");
        assert_eq!(
            normalize_logical_path("../backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
    }

    #[test]
    fn test_normalize_removes_assets_prefix() {
        assert_eq!(
            normalize_logical_path("assets/backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
        assert_eq!(
            normalize_logical_path("assets\\scripts\\test.md"),
            "scripts/test.md"
        );
    }

    #[test]
    fn test_resolve_relative() {
        assert_eq!(
            resolve_relative_path("scripts", "../backgrounds/bg.png"),
            "backgrounds/bg.png"
        );
        assert_eq!(
            resolve_relative_path("scripts/chapter1", "../common/char.png"),
            "scripts/common/char.png"
        );
        assert_eq!(
            resolve_relative_path("scripts", "images/char.png"),
            "scripts/images/char.png"
        );
    }

    #[test]
    fn test_resolve_absolute_path() {
        assert_eq!(
            resolve_relative_path("scripts", "/absolute/path.png"),
            "absolute/path.png"
        );
    }

    #[test]
    fn test_extract_script_id() {
        assert_eq!(extract_script_id("scripts/test.md"), "test");
        assert_eq!(extract_script_id("scripts/chapter1/intro.md"), "intro");
        assert_eq!(extract_script_id("test.md"), "test");
        assert_eq!(extract_script_id("test"), "test");
    }

    #[test]
    fn test_extract_base_dir() {
        assert_eq!(extract_base_dir("scripts/test.md"), "scripts");
        assert_eq!(
            extract_base_dir("scripts/chapter1/intro.md"),
            "scripts/chapter1"
        );
        assert_eq!(extract_base_dir("test.md"), "");
    }

    #[test]
    fn test_logical_path_new_normalizes() {
        let p = LogicalPath::new("assets/backgrounds\\bg.png");
        assert_eq!(p.as_str(), "backgrounds/bg.png");

        let p2 = LogicalPath::new("scripts/../backgrounds/bg.png");
        assert_eq!(p2.as_str(), "backgrounds/bg.png");

        let p3 = LogicalPath::new("./backgrounds/bg.png");
        assert_eq!(p3.as_str(), "backgrounds/bg.png");
    }

    #[test]
    fn test_logical_path_file_stem() {
        let p = LogicalPath::new("scripts/chapter1/intro.md");
        assert_eq!(p.file_stem(), "intro");

        let p2 = LogicalPath::new("test");
        assert_eq!(p2.file_stem(), "test");
    }

    #[test]
    fn test_logical_path_parent_dir() {
        let p = LogicalPath::new("scripts/chapter1/intro.md");
        assert_eq!(p.parent_dir(), "scripts/chapter1");

        let p2 = LogicalPath::new("test.md");
        assert_eq!(p2.parent_dir(), "");
    }

    #[test]
    fn test_logical_path_join() {
        let base = LogicalPath::new("scripts/chapter1");
        let joined = base.join("../common/char.png");
        assert_eq!(joined.as_str(), "scripts/common/char.png");

        let empty = LogicalPath::new("");
        let joined2 = empty.join("images/bg.png");
        assert_eq!(joined2.as_str(), "images/bg.png");
    }

    #[test]
    fn test_logical_path_equality_after_normalization() {
        let a = LogicalPath::new("assets/backgrounds/bg.png");
        let b = LogicalPath::new("backgrounds/bg.png");
        assert_eq!(a, b);
    }

    #[test]
    fn test_logical_path_to_path_buf_preserves_normalized_segments() {
        let p = LogicalPath::new("assets\\scripts/../backgrounds/bg.png");
        assert_eq!(p.to_path_buf(), PathBuf::from("backgrounds/bg.png"));
    }
}
