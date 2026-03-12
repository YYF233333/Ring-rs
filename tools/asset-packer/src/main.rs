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

mod inspect;
mod pack;
mod release;
mod utils;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "packer")]
#[command(about = "资源打包工具 - 将 assets 目录打包为 ZIP 文件（不压缩）")]
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
    /// 并将所有文件组装到发行版目录。
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
    if let Err(e) = run() {
        eprintln!("错误: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => pack::pack_assets(&cli.input, &cli.output),
        Some(Commands::List { zip_file }) => inspect::list_zip(&zip_file),
        Some(Commands::Verify { zip_file, input }) => {
            inspect::verify_zip(&zip_file, input.as_deref())
        }
        Some(Commands::Release { output_dir, zip }) => {
            release::create_release(&cli.input, &cli.output, &output_dir, zip)
        }
    }
}
