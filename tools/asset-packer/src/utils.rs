//! 通用工具函数

use anyhow::{Context, Result};
use std::path::Path;
use xshell::Shell;

/// 运行外部命令，失败时附带描述信息
pub fn run_command(description: &str, program: &str, args: &[&str]) -> Result<()> {
    let sh = Shell::new()?;
    println!("{description}");
    sh.cmd(program)
        .args(args)
        .run()
        .with_context(|| format!("{description} 失败"))
}

/// 从路径中提取文件名，失败时附带上下文信息
pub fn required_file_name<'a>(path: &'a Path, context: &str) -> Result<&'a std::ffi::OsStr> {
    path.file_name()
        .with_context(|| format!("{context}: {:?}", path))
}

/// 格式化字节数为人类可读的大小字符串
pub fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{:.2} MB", size as f64 / 1024.0 / 1024.0)
    }
}
