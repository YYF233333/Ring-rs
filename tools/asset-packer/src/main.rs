//! # Asset Packer
//!
//! 资源打包工具 - 将 assets 目录打包为 ZIP 文件，用于发布。
//!
//! ## 用法
//!
//! ```bash
//! # 在项目根目录使用 cargo 运行
//! cargo run --bin packer
//! cargo run --bin packer -- --input assets --output game.zip
//! cargo run --bin packer -- list game.zip
//! cargo run --bin packer -- verify game.zip --input assets
//!
//! # 或安装后直接使用
//! cargo install --path tools/asset-packer
//! packer
//! packer --input assets --output game.zip
//! packer list game.zip
//! packer verify game.zip --input assets
//! ```

use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
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
