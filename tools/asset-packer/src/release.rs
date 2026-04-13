//! 发行版构建流程
//!
//! 将资源打包、编译宿主应用、组装发行版目录。

use crate::pack::{pack_assets, pack_directory};
use crate::utils::{required_file_name, run_command};
use anyhow::{Result, bail};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

const DEFAULT_GAME_NAME: &str = "Ring";

/// 创建完整发行版
///
/// 步骤：
/// 1. 打包 assets -> game.zip
/// 2. `cargo build --release -p host-dioxus` 编译宿主应用
/// 3. 检查 config.json
/// 4. 组装发行版目录（并可选打包为 ZIP）
pub fn create_release(
    assets_dir: &Path,
    zip_output: &Path,
    release_dir: &Path,
    create_zip: bool,
) -> Result<()> {
    println!("创建发行版...");
    println!();

    let config_path = PathBuf::from("config.json");
    let game_name = if config_path.exists() {
        read_game_name(&config_path)?
    } else {
        DEFAULT_GAME_NAME.to_string()
    };

    println!("步骤 1/4: 打包资源...");
    pack_assets(assets_dir, zip_output)?;
    println!();

    println!("步骤 2/4: 编译宿主应用（release）...");
    run_command(
        "执行 cargo build --release -p host-dioxus",
        "cargo",
        &["build", "--release", "-p", "host-dioxus"],
    )?;

    let host_binary = host_binary_path();
    if !host_binary.exists() {
        bail!("找不到编译后的二进制文件: {:?}", host_binary);
    }
    println!("编译完成: {:?}", host_binary);
    println!();

    println!("步骤 3/4: 检查配置文件...");
    if !config_path.exists() {
        bail!("找不到 config.json 文件");
    }
    println!("找到配置文件: {:?}", config_path);
    println!("游戏名称: {}", game_name);
    println!();

    println!("步骤 4/4: 创建发行版目录...");
    assemble_release_dir(
        release_dir,
        zip_output,
        &host_binary,
        &config_path,
        &game_name,
    )?;

    if create_zip {
        println!();
        println!("打包发行版为 ZIP...");
        let release_zip = release_dir
            .parent()
            .unwrap_or(Path::new("."))
            .join(format!("{}.zip", game_name));
        pack_directory(release_dir, &release_zip)?;
        println!("发行版 ZIP 创建完成: {:?}", release_zip);
    }

    Ok(())
}

fn assemble_release_dir(
    release_dir: &Path,
    zip_output: &Path,
    host_binary: &Path,
    config_path: &Path,
    game_name: &str,
) -> Result<()> {
    if release_dir.exists() {
        println!("发行版目录已存在，将清空: {:?}", release_dir);
        std::fs::remove_dir_all(release_dir)?;
    }
    std::fs::create_dir_all(release_dir)?;

    let zip_name = required_file_name(zip_output, "资源 ZIP 输出路径必须是文件")?;
    let zip_dest = release_dir.join(zip_name);
    std::fs::rename(zip_output, &zip_dest)?;
    println!("  移动资源包: {:?} -> {:?}", zip_output, zip_dest);

    let binary_filename = binary_name(game_name);
    let binary_dest = release_dir.join(&binary_filename);
    std::fs::copy(host_binary, &binary_dest)?;
    println!(
        "  复制二进制: {:?} -> {:?} (重命名为: {})",
        host_binary, binary_dest, binary_filename
    );

    let config_dest = release_dir.join("config.json");
    std::fs::copy(config_path, &config_dest)?;
    update_config_for_release(&config_dest, &zip_name.to_string_lossy())?;
    println!("  复制配置并更新为 ZIP 模式");

    println!();
    println!("发行版创建完成！");
    println!("   发行版目录: {:?}", release_dir);
    println!("   包含文件:");
    println!("     - {}", zip_name.to_string_lossy());
    println!("     - {}", binary_filename);
    println!("     - config.json");

    Ok(())
}

/// 从 config.json 读取游戏名称；缺失或无效则返回默认名称。
/// 返回的名称已清理掉不适合用于文件名的字符。
fn read_game_name(config_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(config_path)?;
    let config: serde_json::Value = serde_json::from_str(&content)?;

    let raw = config
        .get("name")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_GAME_NAME);

    let sanitized: String = raw
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();

    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        Ok(DEFAULT_GAME_NAME.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

/// 更新发行版 config.json：设置 ZIP 模式 + release 调试配置
fn update_config_for_release(config_path: &Path, zip_filename: &str) -> Result<()> {
    let content = std::fs::read_to_string(config_path)?;
    let mut config: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = config.as_object_mut() {
        obj.insert("asset_source".into(), "zip".into());
        obj.insert("zip_path".into(), zip_filename.into());

        if let Some(debug) = obj.get_mut("debug").and_then(|v| v.as_object_mut()) {
            debug.insert("script_check".into(), false.into());
            debug.insert("log_file".into(), "game.log".into());
        }
    }

    let updated = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(config_path)?;
    file.write_all(updated.as_bytes())?;

    Ok(())
}

/// 宿主二进制产物路径（workspace 共享 target 目录）
fn host_binary_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        PathBuf::from("target/release/host-dioxus.exe")
    } else {
        PathBuf::from("target/release/host-dioxus")
    }
}

fn binary_name(game_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.exe", game_name)
    } else {
        game_name.to_string()
    }
}
