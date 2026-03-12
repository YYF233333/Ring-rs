//! ZIP 打包操作

use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

/// 打包过程统计信息
#[derive(Default)]
pub struct PackStats {
    pub file_count: usize,
    pub total_size: u64,
}

/// 将资源目录打包为 ZIP 文件，并打印统计信息
pub fn pack_assets(input: &Path, output: &Path) -> Result<()> {
    println!("打包资源目录: {:?} -> {:?}", input, output);

    if !input.exists() {
        bail!("输入目录不存在: {:?}", input);
    }

    let file = File::create(output).with_context(|| format!("无法创建输出 ZIP: {:?}", output))?;
    let mut zip = ZipWriter::new(file);

    let mut stats = PackStats::default();
    add_dir_to_zip(input, input, &mut zip, Some(&mut stats))?;
    zip.finish()?;

    let zip_size = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);

    println!();
    println!("打包完成！");
    println!("   文件数: {}", stats.file_count);
    println!(
        "   原始大小: {:.2} MB",
        stats.total_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "   ZIP 大小: {:.2} MB（无压缩）",
        zip_size as f64 / 1024.0 / 1024.0,
    );
    println!("   输出文件: {:?}", output);

    Ok(())
}

/// 将目录打包为 ZIP 文件，不打印统计信息（用于内部调用）
pub fn pack_directory(input: &Path, output: &Path) -> Result<()> {
    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);
    add_dir_to_zip(input, input, &mut zip, None)?;
    zip.finish()?;
    Ok(())
}

fn stored_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(CompressionMethod::Stored)
}

fn add_dir_to_zip(
    root: &Path,
    input: &Path,
    zip: &mut ZipWriter<File>,
    mut stats: Option<&mut PackStats>,
) -> Result<()> {
    for entry in WalkDir::new(input).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            continue;
        }

        let relative = path.strip_prefix(root)?;
        let name = relative.to_string_lossy().replace('\\', "/");

        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        let size = buf.len() as u64;

        zip.start_file(&name, stored_options())?;
        zip.write_all(&buf)?;

        if let Some(s) = stats.as_deref_mut() {
            s.file_count += 1;
            s.total_size += size;
        }
    }
    Ok(())
}
