use std::process::{Command, ExitCode};

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
            // - workspace 覆盖率不作为主目标，主要用于“趋势观察”
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
        "help" | "-h" | "--help" => {
            eprintln!(
                "xtask\n\nUSAGE:\n  cargo check-all\n  cargo cov-runtime\n  cargo cov-workspace\n\nALIASES:\n  cargo check-all     -> runs xtask check-all\n  cargo cov-runtime   -> runs xtask cov-runtime\n  cargo cov-workspace -> runs xtask cov-workspace\n"
            );
        }
        other => anyhow::bail!("unknown xtask subcommand: {other}"),
    }

    Ok(())
}
