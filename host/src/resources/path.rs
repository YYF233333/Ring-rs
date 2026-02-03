//! # 路径规范化模块
//!
//! 提供统一的路径规范化逻辑，所有资源加载都使用此模块。
//!
//! ## 设计原则
//!
//! - 程序内部统一使用**相对于 assets_root 的逻辑路径**
//! - 使用 `/` 作为路径分隔符（跨平台统一）
//! - 路径不包含 `assets/` 前缀
//! - 加载时根据 ResourceSource 类型决定如何解析到实际路径

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
    let result = components.join("/");

    // 移除 assets/ 前缀（如果存在）
    if result.starts_with("assets/") {
        result[7..].to_string()
    } else {
        result
    }
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
}
