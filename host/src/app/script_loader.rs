//! 脚本扫描与加载

use crate::resources::ResourceManager;
use std::path::PathBuf;
use vn_runtime::{Parser, VNRuntime};

use super::AppState;

/// 扫描脚本目录，返回 (script_id, script_path) 列表
pub fn scan_scripts(assets_root: &PathBuf) -> Vec<(String, PathBuf)> {
    let scripts_dir = assets_root.join("scripts");
    let mut scripts = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(stem) = path.file_stem() {
                    let script_id = stem.to_string_lossy().to_string();
                    scripts.push((script_id, path));
                }
            }
        }
    }

    // 按文件名排序，确保顺序稳定
    scripts.sort_by(|a, b| a.0.cmp(&b.0));
    scripts
}

/// 从 ZIP 扫描脚本文件
pub fn scan_scripts_from_zip(resource_manager: &ResourceManager) -> Vec<(String, PathBuf)> {
    let mut scripts = Vec::new();

    // 通过 ResourceManager 列出 scripts 目录下的文件
    let files = resource_manager.list_files("scripts");

    for file_path in files {
        // 只处理 .md 文件
        if file_path.ends_with(".md") {
            if let Some(stem) = PathBuf::from(&file_path).file_stem() {
                let script_id = stem.to_string_lossy().to_string();
                // 使用完整路径作为 PathBuf（ZIP 模式下路径已经是相对于 assets_root 的）
                let full_path = PathBuf::from(&file_path);
                scripts.push((script_id, full_path));
            }
        }
    }

    // 按文件名排序，确保顺序稳定
    scripts.sort_by(|a, b| a.0.cmp(&b.0));
    scripts
}

/// 从逻辑路径加载脚本
///
/// # 参数
/// - `logical_path`: 逻辑路径（相对于 assets_root，如 `scripts/test.md`）
///
/// # 返回
/// 是否加载成功
pub fn load_script_from_logical_path(app_state: &mut AppState, logical_path: &str) -> bool {
    use crate::resources::{extract_base_dir, extract_script_id, normalize_logical_path};

    // 规范化路径
    let normalized_path = normalize_logical_path(logical_path);
    let script_id = extract_script_id(&normalized_path);
    let base_dir = extract_base_dir(&normalized_path);

    println!("📜 加载脚本: {} (路径: {})", script_id, normalized_path);
    println!("📁 脚本目录: {}", base_dir);

    // 通过 ResourceManager 读取（统一处理 FS 和 ZIP 模式）
    let script_text = match app_state.resource_manager.read_text(&normalized_path) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("❌ 脚本文件加载失败: {} - {}", normalized_path, e);
            return false;
        }
    };

    let mut parser = Parser::new();
    match parser.parse_with_base_path(&script_id, &script_text, &base_dir) {
        Ok(script) => {
            println!("✅ 脚本解析成功！节点数: {}", script.len());

            // 打印警告
            for warning in parser.warnings() {
                println!("⚠️ 解析警告: {}", warning);
            }

            // 创建 VNRuntime 并设置脚本路径
            let mut runtime = VNRuntime::new(script);
            runtime.state_mut().position.set_path(&normalized_path);
            app_state.vn_runtime = Some(runtime);
            true
        }
        Err(e) => {
            eprintln!("❌ 脚本解析失败: {}", e);
            false
        }
    }
}

/// 从 PathBuf 加载脚本（兼容旧接口）
pub fn load_script_from_path(app_state: &mut AppState, script_path: &PathBuf) -> bool {
    use crate::resources::normalize_logical_path;

    // 将 PathBuf 转换为逻辑路径
    let path_str = script_path.to_string_lossy().replace('\\', "/");
    let logical_path = normalize_logical_path(&path_str);

    load_script_from_logical_path(app_state, &logical_path)
}

/// 根据脚本路径或 ID 加载脚本（用于存档恢复）
///
/// 优先使用 script_path（如果非空），否则回退到 script_id。
pub fn load_script_by_path_or_id(
    app_state: &mut AppState,
    script_path: &str,
    script_id: &str,
) -> bool {
    // 如果有脚本路径，直接使用
    if !script_path.is_empty() {
        println!("📜 从路径加载脚本: {}", script_path);
        return load_script_from_logical_path(app_state, script_path);
    }

    // 否则从 ID 推断路径
    load_script_by_id(app_state, script_id)
}

/// 根据脚本 ID 加载脚本（兼容旧存档）
pub fn load_script_by_id(app_state: &mut AppState, script_id: &str) -> bool {
    println!("📜 从 ID 推断脚本路径: {}", script_id);

    // 在 scripts 列表中查找
    if let Some((_, path)) = app_state.scripts.iter().find(|(id, _)| id == script_id) {
        let path = path.clone();
        return load_script_from_path(app_state, &path);
    }

    // 尝试常见的脚本位置
    let possible_paths = [
        format!("scripts/{}.md", script_id),
        format!("{}.md", script_id),
    ];

    for path in &possible_paths {
        if app_state.resource_manager.resource_exists(path) {
            return load_script_from_logical_path(app_state, path);
        }
    }

    eprintln!(
        "❌ 找不到脚本: {} (尝试过: {:?})",
        script_id, possible_paths
    );
    false
}

/// 从命令列表中收集需要预取的资源路径
pub fn collect_prefetch_paths(commands: &[vn_runtime::Command]) -> Vec<String> {
    use vn_runtime::Command;
    use vn_runtime::command::TransitionArg;

    let mut paths = Vec::new();

    for command in commands {
        match command {
            Command::ShowBackground { path, .. } => {
                paths.push(path.clone());
            }
            Command::ChangeScene { path, transition } => {
                paths.push(path.clone());
                // Rule 过渡还需要遮罩纹理
                if let Some(trans) = transition {
                    if let Some(TransitionArg::String(mask)) = trans.get_named("mask") {
                        paths.push(mask.clone());
                    }
                }
            }
            Command::ShowCharacter { path, .. } => {
                paths.push(path.clone());
            }
            _ => {}
        }
    }

    paths
}
