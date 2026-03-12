//! ZIP 内容查看与完整性验证

use crate::utils::format_size;
use anyhow::{Context, Result, bail};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// 列出 ZIP 文件的所有条目及大小信息
pub fn list_zip(zip_path: &Path) -> Result<()> {
    println!("ZIP 内容: {:?}", zip_path);
    println!();

    let file =
        File::open(zip_path).with_context(|| format!("无法打开 ZIP 文件: {:?}", zip_path))?;
    let mut archive = ZipArchive::new(file)?;

    let mut total_size = 0u64;
    let mut compressed_size = 0u64;

    println!("{:<60} {:>12} {:>12}", "文件名", "原始大小", "压缩大小");
    println!("{}", "-".repeat(86));

    for i in 0..archive.len() {
        let entry = archive.by_index(i)?;
        let size = entry.size();
        let comp = entry.compressed_size();
        total_size += size;
        compressed_size += comp;
        println!(
            "{:<60} {:>12} {:>12}",
            entry.name(),
            format_size(size),
            format_size(comp)
        );
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

/// 验证 ZIP 文件的完整性，可选与原始目录对比内容
pub fn verify_zip(zip_path: &Path, input: Option<&Path>) -> Result<()> {
    println!("验证 ZIP: {:?}", zip_path);

    let file =
        File::open(zip_path).with_context(|| format!("无法打开 ZIP 文件: {:?}", zip_path))?;
    let mut archive = ZipArchive::new(file)?;

    let mut errors: Vec<String> = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        let mut buf = Vec::new();
        if let Err(e) = entry.read_to_end(&mut buf) {
            errors.push(format!("{name}: 读取失败 - {e}"));
            continue;
        }

        if let Some(dir) = input {
            let source = dir.join(&name);
            if source.exists() {
                let mut src_buf = Vec::new();
                File::open(&source)?.read_to_end(&mut src_buf)?;
                if buf != src_buf {
                    errors.push(format!("{name}: 内容与源文件不一致"));
                }
            }
        }
    }

    if errors.is_empty() {
        println!("验证通过！共 {} 个文件", archive.len());
        Ok(())
    } else {
        println!("验证失败！发现 {} 个问题:", errors.len());
        for e in &errors {
            println!("   - {e}");
        }
        bail!("{} 个文件有问题", errors.len())
    }
}
