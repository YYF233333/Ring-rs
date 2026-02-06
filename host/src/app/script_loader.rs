//! 脚本扫描与加载

use crate::resources::ResourceManager;
use std::path::PathBuf;
use tracing::{error, info, warn};
use vn_runtime::{Parser, VNRuntime};

use super::AppState;

/// 扫描脚本目录，返回脚本路径列表
pub fn scan_scripts(assets_root: &PathBuf) -> Vec<PathBuf> {
    let scripts_dir = assets_root.join("scripts");
    let mut scripts = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                scripts.push(path);
            }
        }
    }

    // 按路径排序，确保顺序稳定
    scripts.sort();
    scripts
}

/// 从 ZIP 扫描脚本文件
pub fn scan_scripts_from_zip(resource_manager: &ResourceManager) -> Vec<PathBuf> {
    let mut scripts = Vec::new();

    // 通过 ResourceManager 列出 scripts 目录下的文件
    let files = resource_manager.list_files("scripts");

    for file_path in files {
        // 只处理 .md 文件
        if file_path.ends_with(".md") {
            // ZIP 模式下路径已经是相对于 assets_root 的
            scripts.push(PathBuf::from(&file_path));
        }
    }

    // 按路径排序，确保顺序稳定
    scripts.sort();
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

    info!(script_id = %script_id, path = %normalized_path, base_dir = %base_dir, "加载脚本");

    // 通过 ResourceManager 读取（统一处理 FS 和 ZIP 模式）
    let script_text = match app_state.resource_manager.read_text(&normalized_path) {
        Ok(text) => text,
        Err(e) => {
            error!(path = %normalized_path, error = %e, "脚本文件加载失败");
            return false;
        }
    };

    let mut parser = Parser::new();
    match parser.parse_with_base_path(&script_id, &script_text, &base_dir) {
        Ok(script) => {
            info!(node_count = script.len(), "脚本解析成功");

            // 打印警告
            for warning in parser.warnings() {
                warn!(warning = %warning, "解析警告");
            }

            // 创建 VNRuntime 并设置脚本路径
            let mut runtime = VNRuntime::new(script);
            runtime.state_mut().position.set_path(&normalized_path);
            app_state.vn_runtime = Some(runtime);
            true
        }
        Err(e) => {
            error!(error = %e, "脚本解析失败");
            false
        }
    }
}

/// 根据脚本路径加载脚本（用于存档恢复）
///
/// 约定：新版本存档总是携带 `script_path`；因此不再支持通过 `script_id`
/// 推断路径的历史兼容逻辑。
pub fn load_script_from_save_path(app_state: &mut AppState, script_path: &str) -> bool {
    if script_path.is_empty() {
        error!("存档缺少 script_path，无法加载脚本");
        return false;
    }

    info!(path = %script_path, "从存档 script_path 加载脚本");
    load_script_from_logical_path(app_state, script_path)
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
                if let Some(trans) = transition
                    && let Some(TransitionArg::String(mask)) = trans.get_named("mask")
                {
                    paths.push(mask.clone());
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
