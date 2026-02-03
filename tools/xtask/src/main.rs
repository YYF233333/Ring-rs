use std::process::{Command, ExitCode};

fn run(step: &str, cmd: &mut Command) -> anyhow::Result<()> {
    eprintln!("\n==> {step}");
    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("{step} failed with {status}");
    }
    Ok(())
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
        "help" | "-h" | "--help" => {
            eprintln!(
                "xtask\n\nUSAGE:\n  cargo check-all\n\nALIASES:\n  cargo check-all  -> runs xtask check-all\n"
            );
        }
        other => anyhow::bail!("unknown xtask subcommand: {other}"),
    }

    Ok(())
}
