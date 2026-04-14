//! # xtask - 开发辅助工具
//!
//! 提供本地质量门禁与开发辅助命令。
//!
//! ## 命令
//!
//! - `check-all`: fmt（直接应用）、clippy（自动 fix）、test
//! - `cov`: 运行 workspace 覆盖率（排除工具 crate 与平台胶水代码）
//! - `script-check`: 检查脚本文件（语法、label、资源引用）
//! - `mutants`: 运行变异测试（vn-runtime），检测测试质量
//! - `gen-symbols`: 从 rustdoc JSON 生成符号索引（`docs/engine/symbol-index.md`）

mod gen_symbols;

use std::path::{Path, PathBuf};
use std::process::{ExitCode, ExitStatus};

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
  cargo check-all          -> cargo run -p xtask -- check-all
  cargo cov                -> cargo run -p xtask -- cov
  cargo script-check       -> cargo run -p xtask -- script-check
  cargo mutants-check      -> cargo run -p xtask -- mutants
"#
)]
struct Cli {
    #[command(subcommand)]
    command: XtaskCommand,
}

#[derive(Subcommand, Debug)]
enum XtaskCommand {
    /// 运行 fmt、clippy（自动 fix）、test
    CheckAll,

    /// 运行 workspace 覆盖率报告（HTML；排除工具 crate 与平台胶水代码）
    Cov,

    /// 检查脚本文件（语法、label、资源引用）
    ScriptCheck(ScriptCheckArgs),

    /// 运行变异测试（vn-runtime），检测测试质量
    Mutants(MutantsArgs),

    /// 从 rustdoc JSON 生成符号索引（docs/engine/symbol-index.md）
    GenSymbols,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum MutantsPackage {
    #[value(name = "vn-runtime")]
    VnRuntime,
}

impl MutantsPackage {
    fn cargo_package_name(self) -> &'static str {
        match self {
            Self::VnRuntime => "vn-runtime",
        }
    }
}

#[derive(Args, Debug)]
#[command(after_help = r#"说明：
  变异测试通过在源码中注入小变更（mutant），检验测试是否能捕获这些变更。
  "missed" 表示测试未能发现该变异——意味着测试覆盖不足或断言太弱。

  默认测试 vn-runtime 全部可变异函数（排除规则见 .cargo/mutants.toml）。
  可用 --file 缩小范围加速迭代。

示例：
  cargo mutants-check                                          # 全量
  cargo mutants-check -- --file vn-runtime/src/state.rs        # 单文件
"#)]
struct MutantsArgs {
    /// 目标 crate（默认 vn-runtime）
    #[arg(long, value_enum, default_value_t = MutantsPackage::VnRuntime)]
    package: MutantsPackage,

    /// cargo-mutants 并发数；本地默认 3，不可和--in-place一起使用
    #[arg(long, default_value_t = 3, value_parser = parse_jobs)]
    jobs: usize,

    /// 透传给 cargo-mutants 的额外参数（如 --file, --re 等）
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    extra: Vec<String>,
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

fn run_status(step: &str, program: &str, args: &[&str]) -> anyhow::Result<ExitStatus> {
    eprintln!("\n==> {step}");
    eprintln!("$ {program} {}", args.join(" "));
    std::process::Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("{step} failed to start"))
}

/// 覆盖率排除规则
///
/// 排除不可单元测试的平台/框架胶水代码：
/// - `src/lib.rs`：模块声明与重导出
/// - `vn-runtime/src/error.rs`：thiserror derive，无手写逻辑
///
/// Builds the coverage ignore regex from `cov-ignore-regex.txt`.
///
/// The file contains one pattern per line; comments (`#`) and blank lines are skipped.
/// Patterns are joined with `|` at runtime.
fn cov_ignore_regex() -> String {
    include_str!("../cov-ignore-regex.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect::<Vec<_>>()
        .join("|")
}

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
                "cargo clippy --workspace --fix --allow-dirty",
                &sh,
                "cargo",
                &["clippy", "--workspace", "--fix", "--allow-dirty"],
            )?;

            run(
                "cargo test --workspace --quiet",
                &sh,
                "cargo",
                &["test", "--workspace", "--quiet"],
            )?;
        }
        XtaskCommand::Cov => {
            ensure_cargo_llvm_cov_available(&sh)?;

            // Cranelift 不支持 -Cinstrument-coverage，覆盖率须使用 LLVM 后端
            let _llvm_guard = sh.push_env("CARGO_PROFILE_DEV_CODEGEN_BACKEND", "llvm");

            let cov_ignore = cov_ignore_regex();
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
                    &cov_ignore,
                ],
            )?;

            eprintln!("\nCoverage HTML: target/llvm-cov/html/index.html");
        }
        XtaskCommand::ScriptCheck(args) => {
            script_check(args)?;
        }
        XtaskCommand::Mutants(args) => {
            run_mutants(&sh, args)?;
        }
        XtaskCommand::GenSymbols => {
            gen_symbols::gen_symbols(&sh)?;
        }
    }

    Ok(())
}

//=============================================================================
// mutants 命令实现
//=============================================================================

#[derive(Debug, Default)]
struct MutantsReportSummary {
    caught: usize,
    missed: usize,
    timeout: usize,
    unviable: usize,
}

fn ensure_cargo_mutants_available(sh: &Shell) -> anyhow::Result<()> {
    if sh.cmd("cargo").args(["mutants", "--version"]).run().is_ok() {
        return Ok(());
    }
    anyhow::bail!(
        "cargo-mutants 不可用。\n\
         请先安装：cargo install cargo-mutants\n\
         然后重试。"
    )
}

fn mutants_output_dir(extra: &[String]) -> PathBuf {
    let mut iter = extra.iter();
    while let Some(arg) = iter.next() {
        if arg == "--output" {
            if let Some(path) = iter.next() {
                return PathBuf::from(path);
            }
            break;
        }

        if let Some(path) = arg.strip_prefix("--output=") {
            return PathBuf::from(path);
        }
    }

    PathBuf::from("mutants.out")
}

fn mutants_report_dir(extra: &[String]) -> PathBuf {
    let output_dir = mutants_output_dir(extra);
    if output_dir.file_name().and_then(|name| name.to_str()) == Some("mutants.out") {
        output_dir
    } else {
        output_dir.join("mutants.out")
    }
}

fn mutants_generates_report(extra: &[String]) -> bool {
    !extra.iter().any(|arg| arg == "--list")
}

fn count_non_empty_lines(path: &Path) -> usize {
    std::fs::read_to_string(path)
        .ok()
        .map(|content| {
            content
                .lines()
                .filter(|line| !line.trim().is_empty())
                .count()
        })
        .unwrap_or(0)
}

fn collect_mutants_report_summary(report_dir: &Path) -> MutantsReportSummary {
    MutantsReportSummary {
        caught: count_non_empty_lines(&report_dir.join("caught.txt")),
        missed: count_non_empty_lines(&report_dir.join("missed.txt")),
        timeout: count_non_empty_lines(&report_dir.join("timeout.txt")),
        unviable: count_non_empty_lines(&report_dir.join("unviable.txt")),
    }
}

fn print_mutants_report_overview(report_dir: &Path, summary: &MutantsReportSummary) {
    eprintln!("\nFull Report: {}/", report_dir.display());
    eprintln!("  - missed: {}", summary.missed);
    eprintln!("  - caught: {}", summary.caught);
    eprintln!("  - timeout: {}", summary.timeout);
    eprintln!("  - unviable: {}", summary.unviable);
}

fn cargo_mutants_runs_in_place(extra: &[String]) -> bool {
    extra
        .iter()
        .any(|arg| arg == "--in-place" || arg.starts_with("--in-place="))
}

fn build_mutants_command_args(package: &str, jobs: usize, extra: Vec<String>) -> Vec<String> {
    let in_place = cargo_mutants_runs_in_place(&extra);
    let mut cmd_args = vec!["mutants".to_string(), "-p".to_string(), package.to_string()];
    if !in_place {
        cmd_args.push("-j".to_string());
        cmd_args.push(jobs.to_string());
    }
    cmd_args.extend(extra);
    cmd_args
}

fn parse_jobs(raw: &str) -> Result<usize, String> {
    let jobs = raw
        .parse::<usize>()
        .map_err(|_| "jobs 必须是正整数".to_string())?;

    if jobs == 0 {
        return Err("jobs 必须大于 0".to_string());
    }

    Ok(jobs)
}

fn run_mutants(sh: &Shell, args: MutantsArgs) -> anyhow::Result<()> {
    ensure_cargo_mutants_available(sh)?;

    let package = args.package.cargo_package_name();
    let report_dir = mutants_report_dir(&args.extra);
    let expects_report = mutants_generates_report(&args.extra);
    let cmd_args = build_mutants_command_args(package, args.jobs, args.extra);

    let str_args: Vec<&str> = cmd_args.iter().map(String::as_str).collect();
    let status = run_status(&format!("cargo mutants -p {package}"), "cargo", &str_args)?;

    let summary = expects_report.then(|| collect_mutants_report_summary(&report_dir));
    if let Some(summary) = &summary {
        print_mutants_report_overview(&report_dir, summary);
    }

    if status.success() {
        return Ok(());
    }

    if let Some(summary) = &summary
        && summary.missed > 0
    {
        anyhow::bail!(
            "mutation testing found {} missed mutants; see {}/",
            summary.missed,
            report_dir.display()
        );
    }

    let code = status
        .code()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    anyhow::bail!(
        "cargo-mutants exited with code {code}; see {}/",
        report_dir.display()
    );
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

#[cfg(test)]
mod tests {
    use super::{build_mutants_command_args, cargo_mutants_runs_in_place};

    #[test]
    fn build_mutants_command_args_includes_jobs_without_in_place() {
        let args =
            build_mutants_command_args("vn-runtime", 3, vec!["--timeout".into(), "60".into()]);

        assert_eq!(
            args,
            vec!["mutants", "-p", "vn-runtime", "-j", "3", "--timeout", "60"]
        );
    }

    #[test]
    fn build_mutants_command_args_omits_jobs_with_in_place() {
        let args = build_mutants_command_args(
            "vn-runtime",
            3,
            vec!["--in-place".into(), "--timeout".into(), "60".into()],
        );

        assert_eq!(
            args,
            vec![
                "mutants",
                "-p",
                "vn-runtime",
                "--in-place",
                "--timeout",
                "60"
            ]
        );
    }

    #[test]
    fn cargo_mutants_runs_in_place_matches_explicit_bool_value() {
        assert!(cargo_mutants_runs_in_place(&["--in-place=true".into()]));
        assert!(!cargo_mutants_runs_in_place(&[
            "--timeout".into(),
            "60".into()
        ]));
    }
}
