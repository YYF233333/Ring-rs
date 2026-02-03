//! AppState 初始化拆分
//!
//! 目标：把 `AppState::new` 中"资源/音频/manifest/脚本/用户设置"等初始化逻辑按职责拆开，
//! 让 `app/mod.rs` 保持可读，后续扩展更容易定位修改点。

use crate::manifest::Manifest;
use crate::resources::ResourceManager;
use crate::resources::path::{extract_base_dir, normalize_logical_path};
use crate::save_manager::SaveManager;
use crate::{AppConfig, AssetSourceType, AudioManager, UserSettings, ZipSource};
use std::path::PathBuf;
use std::sync::Arc;
use vn_runtime::{Parser, analyze_script, extract_resource_references};

use super::script_loader::{scan_scripts, scan_scripts_from_zip};

pub fn assets_root_string(config: &AppConfig) -> String {
    config.assets_root.to_string_lossy().to_string()
}

pub fn saves_dir_string(config: &AppConfig) -> String {
    config.saves_dir.to_string_lossy().to_string()
}

pub fn window_size(config: &AppConfig) -> (f32, f32) {
    (config.window.width as f32, config.window.height as f32)
}

pub fn create_resource_manager(config: &AppConfig) -> ResourceManager {
    let assets_root = assets_root_string(config);

    match config.asset_source {
        AssetSourceType::Fs => {
            println!("📂 资源来源: 文件系统 ({})", assets_root);
            ResourceManager::new(&assets_root, config.resources.texture_cache_size_mb)
        }
        AssetSourceType::Zip => {
            let zip_path = config.zip_path.as_ref().expect("Zip 模式必须配置 zip_path");
            println!("📦 资源来源: ZIP 文件 ({})", zip_path);
            ResourceManager::with_source(
                &assets_root,
                Arc::new(ZipSource::new(zip_path)),
                config.resources.texture_cache_size_mb,
            )
        }
    }
}

pub fn create_audio_manager(config: &AppConfig) -> Option<AudioManager> {
    let assets_root = assets_root_string(config);

    match config.asset_source {
        AssetSourceType::Fs => match AudioManager::new(&assets_root) {
            Ok(am) => {
                println!("✅ 音频系统初始化成功");
                Some(am)
            }
            Err(e) => {
                eprintln!("⚠️ 音频系统初始化失败: {}", e);
                None
            }
        },
        AssetSourceType::Zip => match AudioManager::new_zip_mode(&assets_root) {
            Ok(am) => {
                println!("✅ 音频系统初始化成功 (ZIP 模式)");
                Some(am)
            }
            Err(e) => {
                eprintln!("⚠️ 音频系统初始化失败: {}", e);
                None
            }
        },
    }
}

pub fn load_manifest(config: &AppConfig, resource_manager: &ResourceManager) -> Manifest {
    match config.asset_source {
        AssetSourceType::Fs => {
            let manifest_path = config.manifest_full_path();
            match Manifest::load(&manifest_path.to_string_lossy()) {
                Ok(m) => {
                    println!("✅ 资源清单加载成功: {:?}", manifest_path);
                    m
                }
                Err(e) => {
                    eprintln!("⚠️ 资源清单加载失败，使用默认配置: {}", e);
                    Manifest::with_defaults()
                }
            }
        }
        AssetSourceType::Zip => {
            // ZIP 模式：通过 ResourceManager 读取
            let manifest_path = &config.manifest_path;
            match resource_manager.read_text(manifest_path) {
                Ok(content) => match Manifest::load_from_bytes(content.as_bytes()) {
                    Ok(m) => {
                        println!("✅ 资源清单加载成功: {}", manifest_path);
                        m
                    }
                    Err(e) => {
                        eprintln!("⚠️ 资源清单解析失败，使用默认配置: {}", e);
                        Manifest::with_defaults()
                    }
                },
                Err(e) => {
                    eprintln!("⚠️ 资源清单加载失败，使用默认配置: {}", e);
                    Manifest::with_defaults()
                }
            }
        }
    }
}

pub fn create_save_manager(config: &AppConfig) -> SaveManager {
    let saves_dir = saves_dir_string(config);
    let save_manager = SaveManager::new(&saves_dir);
    println!("✅ 存档管理器初始化成功: {}", saves_dir);
    save_manager
}

pub fn scan_script_list(
    config: &AppConfig,
    resource_manager: &ResourceManager,
) -> Vec<(String, PathBuf)> {
    let scripts = match config.asset_source {
        AssetSourceType::Fs => scan_scripts(&config.assets_root),
        AssetSourceType::Zip => scan_scripts_from_zip(resource_manager),
    };
    println!("📜 发现 {} 个脚本文件", scripts.len());
    scripts
}

pub fn load_user_settings(settings_path: &str) -> UserSettings {
    let settings = UserSettings::load(settings_path);
    println!("✅ 用户设置加载完成");
    settings
}

/// 运行脚本检查（Dev Mode 自动诊断）
///
/// 在 `debug.script_check = true` 时运行（debug build 默认开启），检查所有脚本的：
/// - 语法错误
/// - 未定义的跳转目标
/// - 资源文件是否存在
///
/// 只输出警告，不阻塞启动。
pub fn run_script_check(
    config: &AppConfig,
    scripts: &[(String, PathBuf)],
    resource_manager: &ResourceManager,
) {
    // 检查是否需要运行
    if !config.debug.script_check {
        return;
    }

    println!("\n🔍 Dev Mode: 运行脚本检查...");

    let mut total_errors = 0;
    let mut total_warnings = 0;
    let mut scripts_checked = 0;

    for (script_id, script_path) in scripts {
        // 读取脚本内容
        let logical_path = normalize_logical_path(&script_path.to_string_lossy());
        let content = match resource_manager.read_text(&logical_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  [WARN] {}: 无法读取 - {}", script_id, e);
                total_warnings += 1;
                continue;
            }
        };

        // 计算 base_path
        let base_path = extract_base_dir(&logical_path);

        // 解析脚本
        let mut parser = Parser::new();
        let script = match parser.parse_with_base_path(script_id, &content, &base_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  [ERROR] {}: {}", script_id, e);
                total_errors += 1;
                continue;
            }
        };

        // 输出解析警告
        for warning in parser.warnings() {
            eprintln!("  [WARN] {}: {}", script_id, warning);
            total_warnings += 1;
        }

        // 运行诊断分析
        let diag = analyze_script(&script);
        for d in &diag.diagnostics {
            let level = if d.level == vn_runtime::DiagnosticLevel::Error {
                total_errors += 1;
                "ERROR"
            } else {
                total_warnings += 1;
                "WARN"
            };

            if let Some(line) = d.line {
                eprintln!("  [{}] {}:{}: {}", level, script_id, line, d.message);
            } else {
                eprintln!("  [{}] {}: {}", level, script_id, d.message);
            }
        }

        // 检查资源引用
        let refs = extract_resource_references(&script);
        for r in refs {
            let resource_path = config.assets_root.join(&r.resolved_path);
            if !resource_path.exists() {
                eprintln!(
                    "  [WARN] {}: 资源不存在 [{}] {}",
                    script_id, r.resource_type, r.resolved_path
                );
                total_warnings += 1;
            }
        }

        scripts_checked += 1;
    }

    // 输出汇总
    if total_errors > 0 || total_warnings > 0 {
        eprintln!(
            "🔍 脚本检查完成: {} 个脚本, {} 个错误, {} 个警告",
            scripts_checked, total_errors, total_warnings
        );
        if total_errors > 0 {
            eprintln!("⚠️  发现错误，建议修复后再继续");
        }
    } else {
        println!("✅ 脚本检查通过: {} 个脚本", scripts_checked);
    }
    println!();
}
