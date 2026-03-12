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
pub fn pack_assets(input: &Path, output: &Path, level: u32) -> Result<()> {
    println!("打包资源目录: {:?} -> {:?}", input, output);

    if !input.exists() {
        bail!("输入目录不存在: {:?}", input);
    }

    let file = File::create(output).with_context(|| format!("无法创建输出 ZIP: {:?}", output))?;
    let mut zip = ZipWriter::new(file);
    let options = make_options(level);

    let mut stats = PackStats::default();
    add_dir_to_zip(input, input, &mut zip, options, Some(&mut stats))?;
    zip.finish()?;

    let compressed = std::fs::metadata(output).map(|m| m.len()).unwrap_or(0);
    let ratio = if stats.total_size > 0 {
        compressed as f64 / stats.total_size as f64 * 100.0
    } else {
        0.0
    };

    println!();
    println!("打包完成！");
    println!("   文件数: {}", stats.file_count);
    println!(
        "   原始大小: {:.2} MB",
        stats.total_size as f64 / 1024.0 / 1024.0
    );
    println!(
        "   压缩后: {:.2} MB (压缩率: {:.1}%)",
        compressed as f64 / 1024.0 / 1024.0,
        ratio
    );
    println!("   输出文件: {:?}", output);

    Ok(())
}

/// 将目录打包为 ZIP 文件，不打印统计信息（用于内部调用）
pub fn pack_directory(input: &Path, output: &Path, level: u32) -> Result<()> {
    let file = File::create(output)?;
    let mut zip = ZipWriter::new(file);
    let options = make_options(level);
    add_dir_to_zip(input, input, &mut zip, options, None)?;
    zip.finish()?;
    Ok(())
}

fn make_options(level: u32) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(level as i64))
}

fn add_dir_to_zip(
    root: &Path,
    input: &Path,
    zip: &mut ZipWriter<File>,
    options: SimpleFileOptions,
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

        zip.start_file(&name, options)?;
        zip.write_all(&buf)?;

        if let Some(s) = stats.as_deref_mut() {
            s.file_count += 1;
            s.total_size += size;
        }
    }
    Ok(())
}
