//! # xtask - 开发辅助工具
//!
//! 提供本地质量门禁与开发辅助命令。
//!
//! ## 命令
//!
//! - `check-all`: fmt（直接应用）、clippy（自动 fix）、test（仅输出最终结果）
//! - `cov-runtime`: 运行 vn-runtime 覆盖率
//! - `cov-workspace`: 运行 workspace 覆盖率
//! - `script-check`: 检查脚本文件（语法、label、资源引用）

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};
use vn_runtime::{
    DiagnosticResult, Parser as ScriptParser, analyze_script, extract_resource_references,
};
use walkdir::WalkDir;
use xshell::Shell;

#[derive(Parser, Debug)]
#[command(
    name = "xtask",
    about = "开发辅助工具（本地门禁/覆盖率/脚本静态检查）",
    arg_required_else_help = true,
    after_help = r#"ALIASES (in .cargo/config.toml):
  cargo check-all     -> cargo run -p xtask -- check-all
  cargo cov-runtime   -> cargo run -p xtask -- cov-runtime
  cargo cov-workspace -> cargo run -p xtask -- cov-workspace
  cargo script-check  -> cargo run -p xtask -- script-check
"#
)]
struct Cli {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Subcommand, Debug)]
enum XtaskCommand {
    /// 运行 fmt（直接应用）、clippy（自动 fix）、test（--quiet，仅最终结果）
    CheckAll,

    /// 运行 vn-runtime 覆盖率报告（HTML）
    CovRuntime,

    /// 运行 workspace 覆盖率报告（HTML；排除工具 crate）
    CovWorkspace,

    /// 检查脚本文件（语法、label、资源引用）
    ScriptCheck(ScriptCheckArgs),
}

#[derive(Args, Debug)]
#[command(after_help = r#"说明：
  - 不带 path：检查 scripts_dir 下所有 .md
  - 带 path：检查指定文件或目录

检查内容：
  - 脚本语法错误
  - 未定义的跳转目标（goto/choice 引用的 label）
  - 资源文件是否存在（背景/立绘/音频）
"#)]
struct ScriptCheckArgs {
    /// 脚本文件或目录路径（可选）
    path: Option<PathBuf>,

    /// 默认脚本目录（当未提供 path 时使用）
    #[arg(long, default_value = "assets/scripts")]
    scripts_dir: PathBuf,

    /// 资源根目录（用于验证资源引用是否存在）
    #[arg(long, default_value = "assets")]
    assets_root: PathBuf,
}

fn run(step: &str, sh: &Shell, program: &str, args: &[&str]) -> anyhow::Result<()> {
    eprintln!("\n==> {step}");
    sh.cmd(program)
        .args(args)
        .run()
        .with_context(|| format!("{step} failed"))?;
    Ok(())
}

/// 覆盖率排除规则：vn-runtime 主口径
///
/// 排除纯声明/derive 文件，不含可测试业务逻辑：
/// - `lib.rs`：模块声明与重导出
/// - `error.rs`：thiserror derive，无手写逻辑
const COV_RUNTIME_IGNORE: &str = r"vn-runtime[/\\]src[/\\](lib|error)\.rs$";

/// 覆盖率排除规则：workspace 次口径
///
/// 在 vn-runtime 排除基础上，额外排除不可单元测试的平台/框架胶水代码：
/// - `src/lib.rs`：两个 crate 的模块声明与重导出
/// - `main.rs` / `host_app.rs` / `egui_actions.rs`：平台入口与事件循环
/// - `egui_screens/*`：egui UI 构建逻辑（三方框架行为）
/// - `backend/*`（除 `math.rs`）：GPU 渲染管线（需真实设备）
/// - `audio/playback.rs`：硬件依赖的音频播放
/// - `app/draw.rs`：渲染管线薄包装
const COV_WORKSPACE_IGNORE: &str = concat!(
    r"src[/\\]lib\.rs$|",
    r"vn-runtime[/\\]src[/\\]error\.rs$|",
    r"host[/\\]src[/\\](main|host_app|egui_actions)\.rs$|",
    r"host[/\\]src[/\\]egui_screens[/\\]|",
    r"host[/\\]src[/\\]backend[/\\](mod|sprite_renderer|gpu_texture|dissolve_renderer)\.rs$|",
    r"host[/\\]src[/\\]audio[/\\]playback\.rs$|",
    r"host[/\\]src[/\\]app[/\\]draw\.rs$",
);

fn ensure_cargo_llvm_cov_available(sh: &Shell) -> anyhow::Result<()> {
    if sh
        .cmd("cargo")
        .args(["llvm-cov", "--version"])
        .run()
        .is_ok()
    {
        return Ok(());
    }
    anyhow::bail!(
        "cargo llvm-cov 不可用。\n\
请先安装：\n\
  - cargo install cargo-llvm-cov\n\
  - rustup component add llvm-tools-preview\n\
然后重试。"
    )
}

fn main() -> ExitCode {
    if let Err(e) = real_main() {
        eprintln!("xtask error: {e:#}");
        return ExitCode::from(1);
    }
    ExitCode::from(0)
}

fn real_main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let sh = Shell::new()?;

    match cli.command {
        XtaskCommand::CheckAll => {
            run("cargo fmt --all", &sh, "cargo", &["fmt", "--all"])?;
            run(
                "cargo clippy --workspace --all-targets --fix --allow-dirty",
                &sh,
                "cargo",
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--fix",
                    "--allow-dirty",
                ],
            )?;
            run(
                "cargo test --workspace --quiet",
                &sh,
                "cargo",
                &["test", "--workspace", "--quiet"],
            )?;
        }
        XtaskCommand::CovRuntime => {
            ensure_cargo_llvm_cov_available(&sh)?;

            run(
                "cargo llvm-cov -p vn-runtime (with exclusions)",
                &sh,
                "cargo",
                &[
                    "llvm-cov",
                    "--quiet",
                    "-p",
                    "vn-runtime",
                    "--all-features",
                    "--html",
                    "--ignore-filename-regex",
                    COV_RUNTIME_IGNORE,
                ],
            )?;

            eprintln!("\nCoverage HTML: target/llvm-cov/html/index.html");
        }
        XtaskCommand::CovWorkspace => {
            ensure_cargo_llvm_cov_available(&sh)?;

            run(
                "cargo llvm-cov --workspace (with exclusions)",
                &sh,
                "cargo",
                &[
                    "llvm-cov",
                    "--quiet",
                    "--workspace",
                    "--exclude",
                    "xtask",
                    "--exclude",
                    "asset-packer",
                    "--all-features",
                    "--html",
                    "--ignore-filename-regex",
                    COV_WORKSPACE_IGNORE,
                ],
            )?;

            eprintln!("\nCoverage HTML: target/llvm-cov/html/index.html");
        }
        XtaskCommand::ScriptCheck(args) => {
            script_check(args)?;
        }
    }

    Ok(())
}

//=============================================================================
// script-check 命令实现
//=============================================================================

/// 脚本检查配置
struct ScriptCheckConfig {
    /// 脚本目录（相对于 workspace root）
    scripts_dir: PathBuf,
    /// 资源根目录（相对于 workspace root）
    assets_root: PathBuf,
}

/// 脚本检查结果
struct ScriptCheckResult {
    /// 检查的脚本数量
    scripts_checked: usize,
    /// 解析错误数量
    parse_errors: usize,
    /// 诊断结果
    diagnostics: DiagnosticResult,
    /// 缺失的资源文件
    missing_resources: Vec<MissingResource>,
}

/// 缺失的资源信息
struct MissingResource {
    script_id: String,
    resource_type: String,
    path: String,
}

/// 执行脚本检查
fn script_check(args: ScriptCheckArgs) -> anyhow::Result<()> {
    let config = ScriptCheckConfig {
        scripts_dir: args.scripts_dir,
        assets_root: args.assets_root,
    };

    // 确定要检查的文件
    let files = match args.path {
        Some(p) => {
            if p.is_file() {
                if is_markdown_file(&p) {
                    vec![p]
                } else {
                    anyhow::bail!("仅支持 .md 脚本文件: {}", p.display());
                }
            } else if p.is_dir() {
                collect_script_files(&p)?
            } else {
                anyhow::bail!("路径不存在: {}", p.display());
            }
        }
        None => {
            if !config.scripts_dir.exists() {
                anyhow::bail!(
                    "默认脚本目录不存在: {}\n请在 workspace 根目录运行，或指定脚本路径",
                    config.scripts_dir.display()
                );
            }
            collect_script_files(&config.scripts_dir)?
        }
    };

    if files.is_empty() {
        eprintln!("未找到脚本文件（.md）");
        return Ok(());
    }

    eprintln!("==> 检查 {} 个脚本文件...\n", files.len());

    let mut result = ScriptCheckResult {
        scripts_checked: 0,
        parse_errors: 0,
        diagnostics: DiagnosticResult::new(),
        missing_resources: Vec::new(),
    };

    // 检查每个脚本
    for file in &files {
        check_script_file(file, &config, &mut result)?;
    }

    // 输出结果
    print_check_result(&result);

    // 如果有错误则返回失败
    if result.parse_errors > 0 || result.diagnostics.has_errors() {
        anyhow::bail!("脚本检查发现错误");
    }

    Ok(())
}

/// 收集目录下的所有脚本文件
fn collect_script_files(dir: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(dir).follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() && is_markdown_file(entry.path()) {
            files.push(entry.path().to_path_buf());
        }
    }
    files.sort();
    Ok(files)
}

/// 检查单个脚本文件
fn check_script_file(
    file: &Path,
    config: &ScriptCheckConfig,
    result: &mut ScriptCheckResult,
) -> anyhow::Result<()> {
    let script_id = file.display().to_string();
    result.scripts_checked += 1;

    // 读取文件
    let content = match std::fs::read_to_string(file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[ERROR] {}: 无法读取文件 - {}", script_id, e);
            result.parse_errors += 1;
            return Ok(());
        }
    };

    // 计算 base_path（脚本所在目录，相对于 assets_root）
    let base_path = compute_base_path(file, &config.assets_root);

    // 解析脚本
    let mut parser = ScriptParser::new();
    let script = match parser.parse_with_base_path(&script_id, &content, &base_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[ERROR] {}: {}", script_id, e);
            result.parse_errors += 1;
            return Ok(());
        }
    };

    // 输出解析警告
    for warning in parser.warnings() {
        eprintln!("[WARN] {}: {}", script_id, warning);
    }

    // 运行诊断分析
    let diag = analyze_script(&script);
    result.diagnostics.merge(diag);

    // 检查资源引用
    let refs = extract_resource_references(&script);
    for r in refs {
        let resource_path = config.assets_root.join(&r.resolved_path);
        if !resource_path.exists() {
            result.missing_resources.push(MissingResource {
                script_id: script_id.clone(),
                resource_type: format!("{}", r.resource_type),
                path: r.resolved_path,
            });
        }
    }

    Ok(())
}

/// 计算脚本的 base_path（相对于 assets_root）
fn compute_base_path(file: &Path, assets_root: &Path) -> String {
    // 尝试获取相对路径
    if let Ok(relative) = file.strip_prefix(assets_root)
        && let Some(parent) = relative.parent()
    {
        return normalize_path(parent.to_string_lossy().as_ref());
    }

    // 如果无法获取相对路径，使用文件所在目录
    file.parent()
        .map(|p| normalize_path(p.to_string_lossy().as_ref()))
        .unwrap_or_default()
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches('/').to_string()
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// 输出检查结果
fn print_check_result(result: &ScriptCheckResult) {
    eprintln!("─────────────────────────────────────────────────────");
    eprintln!("检查完成: {} 个脚本", result.scripts_checked);
    eprintln!();

    // 输出诊断
    for diag in &result.diagnostics.diagnostics {
        eprintln!("{}", diag);
    }

    // 输出缺失资源
    for mr in &result.missing_resources {
        eprintln!(
            "[WARN] {}: 资源不存在 [{}] {}",
            mr.script_id, mr.resource_type, mr.path
        );
    }

    // 汇总
    let error_count = result.parse_errors + result.diagnostics.error_count();
    let warn_count = result.diagnostics.warn_count() + result.missing_resources.len();

    eprintln!();
    if error_count > 0 {
        eprintln!("❌ {} 个错误, {} 个警告", error_count, warn_count);
    } else if warn_count > 0 {
        eprintln!("⚠️  0 个错误, {} 个警告", warn_count);
    } else {
        eprintln!("✅ 检查通过，无错误");
    }
}
