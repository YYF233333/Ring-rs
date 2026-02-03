//! # xtask - 开发辅助工具
//!
//! 提供本地质量门禁与开发辅助命令。
//!
//! ## 命令
//!
//! - `check-all`: 运行 fmt、clippy、test
//! - `cov-runtime`: 运行 vn-runtime 覆盖率
//! - `cov-workspace`: 运行 workspace 覆盖率
//! - `script-check`: 检查脚本文件（语法、label、资源引用）

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use vn_runtime::{DiagnosticResult, Parser, analyze_script, extract_resource_references};

fn run(step: &str, cmd: &mut Command) -> anyhow::Result<()> {
    eprintln!("\n==> {step}");
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("{step} failed with {status}");
    }
    Ok(())
}

fn ensure_cargo_llvm_cov_available() -> anyhow::Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.args(["llvm-cov", "--version"]);
    let status = cmd.status();
    match status {
        Ok(s) if s.success() => Ok(()),
        _ => anyhow::bail!(
            "cargo llvm-cov 不可用。\n\
请先安装：\n\
  - cargo install cargo-llvm-cov\n\
  - rustup component add llvm-tools-preview\n\
然后重试。"
        ),
    }
}

fn main() -> ExitCode {
    if let Err(e) = real_main() {
        eprintln!("xtask error: {e:#}");
        return ExitCode::from(1);
    }
    ExitCode::from(0)
}

fn real_main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let sub = args.next().unwrap_or_else(|| "help".to_string());

    match sub.as_str() {
        "check-all" => {
            let mut fmt = Command::new("cargo");
            fmt.args(["fmt", "--all", "--", "--check"]);
            run("cargo fmt --all -- --check", &mut fmt)?;

            let mut clippy = Command::new("cargo");
            clippy.args(["clippy", "--workspace", "--all-targets"]);
            run("cargo clippy --workspace --all-targets", &mut clippy)?;

            let mut test = Command::new("cargo");
            test.args(["test", "--workspace"]);
            run("cargo test --workspace", &mut test)?;
        }
        "cov-runtime" => {
            ensure_cargo_llvm_cov_available()?;

            let mut cov = Command::new("cargo");
            cov.args(["llvm-cov", "-p", "vn-runtime", "--all-features", "--html"]);
            run(
                "cargo llvm-cov -p vn-runtime --all-features --html",
                &mut cov,
            )?;

            eprintln!("\nCoverage HTML: target/llvm-cov/html/index.html");
        }
        "cov-workspace" => {
            ensure_cargo_llvm_cov_available()?;

            // 说明：
            // - workspace 覆盖率不作为主目标，主要用于"趋势观察"
            // - 在口径上排除 tool crates（xtask/asset-packer）以免稀释信号
            let mut cov = Command::new("cargo");
            cov.args([
                "llvm-cov",
                "--workspace",
                "--exclude",
                "xtask",
                "--exclude",
                "asset-packer",
                "--all-features",
                "--html",
            ]);
            run(
                "cargo llvm-cov --workspace --exclude xtask --exclude asset-packer --all-features --html",
                &mut cov,
            )?;

            eprintln!("\nCoverage HTML: target/llvm-cov/html/index.html");
        }
        "script-check" => {
            let path = args.next();
            script_check(path.as_deref())?;
        }
        "help" | "-h" | "--help" => {
            print_help();
        }
        other => anyhow::bail!("unknown xtask subcommand: {other}"),
    }

    Ok(())
}

fn print_help() {
    eprintln!(
        r#"xtask - 开发辅助工具

USAGE:
  cargo xtask <command>

COMMANDS:
  check-all       运行 fmt、clippy、test 门禁检查
  cov-runtime     运行 vn-runtime 覆盖率报告
  cov-workspace   运行 workspace 覆盖率报告
  script-check    检查脚本文件

SCRIPT-CHECK:
  cargo xtask script-check [path]

  不带参数：检查 assets/scripts/ 下所有 .md 文件
  带路径参数：检查指定文件或目录

  检查内容：
    - 脚本语法错误
    - 未定义的跳转目标（goto/choice 引用的 label）
    - 资源文件是否存在（背景/立绘/音频）

ALIASES (in .cargo/config.toml):
  cargo check-all     -> cargo xtask check-all
  cargo cov-runtime   -> cargo xtask cov-runtime
  cargo cov-workspace -> cargo xtask cov-workspace
  cargo script-check  -> cargo xtask script-check
"#
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

impl Default for ScriptCheckConfig {
    fn default() -> Self {
        Self {
            scripts_dir: PathBuf::from("assets/scripts"),
            assets_root: PathBuf::from("assets"),
        }
    }
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
fn script_check(path: Option<&str>) -> anyhow::Result<()> {
    let config = ScriptCheckConfig::default();

    // 确定要检查的文件
    let files = match path {
        Some(p) => {
            let path = PathBuf::from(p);
            if path.is_file() {
                vec![path]
            } else if path.is_dir() {
                collect_script_files(&path)?
            } else {
                anyhow::bail!("路径不存在: {}", p);
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
    collect_script_files_recursive(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_script_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_script_files_recursive(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
    Ok(())
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
    let mut parser = Parser::new();
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
        return parent
            .to_string_lossy()
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
    }

    // 如果无法获取相对路径，使用文件所在目录
    file.parent()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
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
