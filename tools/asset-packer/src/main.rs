//! # Asset Packer
//!
//! 资源打包工具 - 将 assets 目录打包为 ZIP 文件，用于发布。
//!
//! ## 用法
//!
//! ```bash
//! # 在项目根目录使用 cargo 运行
//! cargo run -p asset-packer
//! cargo run -p asset-packer -- --input assets --output game.zip
//! cargo run -p asset-packer -- list game.zip
//! cargo run -p asset-packer -- verify game.zip --input assets
//! cargo run -p asset-packer -- release
//! cargo run -p asset-packer -- release --output-dir dist --zip
//!
//! # 或安装后直接使用
//! cargo install --path tools/asset-packer
//! packer
//! packer --input assets --output game.zip
//! packer list game.zip
//! packer verify game.zip --input assets
//! packer release
//! packer release --output-dir dist --zip
//! ```

use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Parser)]
#[command(name = "packer")]
#[command(about = "资源打包工具 - 将 assets 目录打包为 ZIP 文件")]
#[command(version, author)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 输入目录（默认：assets）
    #[arg(short, long, default_value = "assets", global = true)]
    input: PathBuf,

    /// 输出 ZIP 文件（默认：game.zip）
    #[arg(short, long, default_value = "game.zip", global = true)]
    output: PathBuf,

    /// 压缩级别 (0-9)（默认：6）
    #[arg(short, long, default_value = "6", global = true)]
    level: u32,
}

#[derive(Subcommand)]
enum Commands {
    /// 列出 ZIP 内容
    List {
        /// ZIP 文件路径
        zip_file: PathBuf,
    },

    /// 验证 ZIP 完整性
    Verify {
        /// ZIP 文件路径
        zip_file: PathBuf,

        /// 原始目录（用于对比）
        #[arg(short, long)]
        input: Option<PathBuf>,
    },

    /// 创建完整发行版
    /// 
    /// 将 assets 打包成 ZIP，编译 release 版本的 host 二进制，
    /// 并将所有文件打包到发行版目录。
    Release {
        /// 发行版输出目录（默认：dist）
        #[arg(long, default_value = "dist")]
        output_dir: PathBuf,

        /// 是否将发行版目录打包为 ZIP
        #[arg(short = 'z', long)]
        zip: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // 默认行为：打包资源
            if let Err(e) = pack_assets(&cli.input, &cli.output, cli.level) {
                eprintln!("❌ 打包失败: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::List { zip_file }) => {
            if let Err(e) = list_zip(&zip_file) {
                eprintln!("❌ 列出失败: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Verify { zip_file, input }) => {
            if let Err(e) = verify_zip(&zip_file, input.as_deref()) {
                eprintln!("❌ 验证失败: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Release { output_dir, zip }) => {
            if let Err(e) = create_release(&cli.input, &cli.output, cli.level, &output_dir, zip) {
                eprintln!("❌ 创建发行版失败: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// 打包资源目录到 ZIP 文件
fn pack_assets(input: &Path, output: &Path, level: u32) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 打包资源目录: {:?} -> {:?}", input, output);

    if !input.exists() {
        return Err(format!("输入目录不存在: {:?}", input).into());
    }

    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(level as i64));

    let mut file_count = 0;
    let mut total_size = 0u64;

    for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // 跳过目录
        if path.is_dir() {
            continue;
        }

        // 计算相对路径
        let relative_path = path.strip_prefix(input)?;
        let name = relative_path.to_string_lossy().replace('\\', "/");

        // 读取文件内容
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let size = buffer.len() as u64;
        total_size += size;

        // 添加到 ZIP
        zip.start_file(&name, options)?;
        zip.write_all(&buffer)?;

        file_count += 1;
        println!("  + {} ({} bytes)", name, size);
    }

    zip.finish()?;

    println!();
    println!("✅ 打包完成！");
    println!("   文件数: {}", file_count);
    println!("   原始大小: {:.2} MB", total_size as f64 / 1024.0 / 1024.0);
    println!("   输出文件: {:?}", output);

    // 显示压缩后大小
    if let Ok(metadata) = std::fs::metadata(output) {
        let compressed_size = metadata.len();
        // 压缩率 = 压缩后大小 / 原始大小 * 100%
        // 例如：原始 100MB，压缩后 50MB，压缩率 = 50%（表示压缩后是原始的 50%）
        let ratio = if total_size > 0 {
            compressed_size as f64 / total_size as f64 * 100.0
        } else {
            0.0
        };
        println!(
            "   压缩后: {:.2} MB (压缩率: {:.1}%)",
            compressed_size as f64 / 1024.0 / 1024.0,
            ratio
        );
    }

    Ok(())
}

/// 列出 ZIP 内容
fn list_zip(zip_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 ZIP 内容: {:?}", zip_path);
    println!();

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut total_size = 0u64;
    let mut compressed_size = 0u64;

    println!("{:<60} {:>12} {:>12}", "文件名", "原始大小", "压缩大小");
    println!("{}", "-".repeat(86));

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let name = file.name();
        let size = file.size();
        let comp_size = file.compressed_size();

        total_size += size;
        compressed_size += comp_size;

        println!("{:<60} {:>12} {:>12}", name, format_size(size), format_size(comp_size));
    }

    println!("{}", "-".repeat(86));
    println!(
        "{:<60} {:>12} {:>12}",
        format!("共 {} 个文件", archive.len()),
        format_size(total_size),
        format_size(compressed_size)
    );

    Ok(())
}

/// 验证 ZIP 完整性
fn verify_zip(zip_path: &Path, input: Option<&Path>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 验证 ZIP: {:?}", zip_path);

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut errors = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // 尝试读取内容（验证解压）
        let mut buffer = Vec::new();
        if let Err(e) = file.read_to_end(&mut buffer) {
            errors.push(format!("{}: 读取失败 - {}", name, e));
            continue;
        }

        // 如果提供了输入目录，对比内容
        if let Some(input_dir) = input {
            let source_path = input_dir.join(&name);
            if source_path.exists() {
                let mut source_file = File::open(&source_path)?;
                let mut source_buffer = Vec::new();
                source_file.read_to_end(&mut source_buffer)?;

                if buffer != source_buffer {
                    errors.push(format!("{}: 内容不一致", name));
                }
            } else {
                // 不算错误，ZIP 可能包含额外文件
            }
        }
    }

    if errors.is_empty() {
        println!("✅ 验证通过！共 {} 个文件", archive.len());
        Ok(())
    } else {
        println!("❌ 验证失败！发现 {} 个问题:", errors.len());
        for error in &errors {
            println!("   - {}", error);
        }
        Err(format!("{} 个文件有问题", errors.len()).into())
    }
}

/// 创建完整发行版
fn create_release(
    assets_dir: &Path,
    zip_output: &Path,
    compression_level: u32,
    release_dir: &Path,
    create_zip: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 创建发行版...");
    println!();

    // 提前读取游戏名称，因为后面需要用到
    let config_path = PathBuf::from("config.json");
    let game_name = if config_path.exists() {
        get_game_name(&config_path)?
    } else {
        "Ring".to_string()
    };

    // 1. 打包资源
    println!("📦 步骤 1/4: 打包资源...");
    pack_assets(assets_dir, zip_output, compression_level)?;
    println!();

    // 2. 编译 release 版本的 host
    println!("🔨 步骤 2/4: 编译 release 版本的 host...");
    let build_result = Command::new("cargo")
        .args(&["build", "--release", "--package", "host"])
        .status()?;

    if !build_result.success() {
        return Err("编译 host 失败".into());
    }

    // 查找编译后的二进制文件
    let host_binary = if cfg!(target_os = "windows") {
        PathBuf::from("target/release/host.exe")
    } else {
        PathBuf::from("target/release/host")
    };

    if !host_binary.exists() {
        return Err(format!("找不到编译后的二进制文件: {:?}", host_binary).into());
    }

    println!("✅ 编译完成: {:?}", host_binary);
    println!();

    // 3. 检查 config.json
    println!("📋 步骤 3/4: 检查配置文件...");
    if !config_path.exists() {
        return Err("找不到 config.json 文件".into());
    }
    println!("✅ 找到配置文件: {:?}", config_path);
    println!("✅ 游戏名称: {}", game_name);
    println!();

    // 4. 创建发行版目录并复制文件
    println!("📁 步骤 4/4: 创建发行版目录...");
    
    // 创建输出目录
    if release_dir.exists() {
        println!("⚠️  发行版目录已存在，将清空: {:?}", release_dir);
        std::fs::remove_dir_all(release_dir)?;
    }
    std::fs::create_dir_all(release_dir)?;

    // 复制文件
    let zip_dest = release_dir.join(zip_output.file_name().unwrap());
    std::fs::copy(zip_output, &zip_dest)?;
    println!("  ✅ 复制资源包: {:?} -> {:?}", zip_output, zip_dest);

    // 根据游戏名称重命名二进制文件
    let binary_filename = if cfg!(target_os = "windows") {
        format!("{}.exe", game_name)
    } else {
        game_name.clone()
    };
    let binary_dest = release_dir.join(&binary_filename);
    std::fs::copy(&host_binary, &binary_dest)?;
    println!("  ✅ 复制二进制: {:?} -> {:?} (重命名为: {})", host_binary, binary_dest, binary_filename);

    let config_dest = release_dir.join("config.json");
    std::fs::copy(&config_path, &config_dest)?;
    println!("  ✅ 复制配置: {:?} -> {:?}", config_path, config_dest);

    // 更新 config.json 以使用 ZIP 模式
    update_config_for_release(&config_dest, zip_output.file_name().unwrap().to_string_lossy().as_ref())?;
    println!("  ✅ 更新配置以使用 ZIP 模式");

    println!();
    println!("✅ 发行版创建完成！");
    println!("   发行版目录: {:?}", release_dir);
    println!("   包含文件:");
    println!("     - {}", zip_dest.file_name().unwrap().to_string_lossy());
    println!("     - {}", binary_filename);
    println!("     - config.json");

    // 可选：打包整个发行版
    if create_zip {
        println!();
        println!("📦 打包发行版为 ZIP...");
        // 使用游戏名称作为 ZIP 文件名
        let release_zip_name = format!("{}.zip", game_name);
        let release_zip = release_dir.parent().unwrap_or(Path::new(".")).join(&release_zip_name);
        pack_directory(release_dir, &release_zip, compression_level)?;
        println!("✅ 发行版 ZIP 创建完成: {:?}", release_zip);
    }

    Ok(())
}

/// 从 config.json 获取游戏名称
/// 如果存在 "name" 字段则使用它，否则使用默认名称 "Ring"
/// 返回的名称会被清理，移除不适合作为文件名的字符
fn get_game_name(config_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut content = String::new();
    let mut file = File::open(config_path)?;
    file.read_to_string(&mut content)?;

    // 解析 JSON
    let config: serde_json::Value = serde_json::from_str(&content)?;

    // 检查是否有 "name" 字段
    let name = if let Some(name) = config.get("name") {
        if let Some(name_str) = name.as_str() {
            if !name_str.is_empty() {
                name_str
            } else {
                "Ring"
            }
        } else {
            "Ring"
        }
    } else {
        "Ring"
    };

    // 清理文件名：移除不适合作为文件名的字符
    let sanitized = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect::<String>();

    // 如果清理后为空，使用默认名称
    if sanitized.trim().is_empty() {
        Ok("Ring".to_string())
    } else {
        Ok(sanitized.trim().to_string())
    }
}

/// 更新配置文件以使用 ZIP 模式
fn update_config_for_release(config_path: &Path, zip_filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = String::new();
    let mut file = File::open(config_path)?;
    file.read_to_string(&mut content)?;

    // 解析 JSON
    let mut config: serde_json::Value = serde_json::from_str(&content)?;

    // 更新 asset_source 为 "zip"
    if let Some(obj) = config.as_object_mut() {
        obj.insert("asset_source".to_string(), serde_json::Value::String("zip".to_string()));
        obj.insert("zip_path".to_string(), serde_json::Value::String(zip_filename.to_string()));
    }

    // 写回文件
    let updated_content = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(config_path)?;
    file.write_all(updated_content.as_bytes())?;

    Ok(())
}

/// 打包目录到 ZIP 文件
fn pack_directory(input: &Path, output: &Path, level: u32) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(level as i64));

    for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let relative_path = path.strip_prefix(input)?;
        let name = relative_path.to_string_lossy().replace('\\', "/");

        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        zip.start_file(&name, options)?;
        zip.write_all(&buffer)?;
    }

    zip.finish()?;
    Ok(())
}

/// 格式化文件大小
fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
    }
}
