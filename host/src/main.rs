//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod egui_actions;
mod egui_screens;
mod host_app;

use std::path::PathBuf;

use clap::Parser;
use host::AppConfig;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use winit::event_loop::{ControlFlow, EventLoop};

const CONFIG_PATH: &str = "config.json";

/// Ring VN Engine CLI
#[derive(Parser)]
pub struct Cli {
    /// 以无窗口 headless 模式运行
    #[arg(long)]
    pub headless: bool,

    /// 输入录制文件路径（headless 必须）
    #[arg(long)]
    pub replay_input: Option<PathBuf>,

    /// 事件流输出文件路径
    #[arg(long)]
    pub event_stream: Option<PathBuf>,

    /// 退出条件（replay-end / script-finished）
    #[arg(long, default_value = "replay-end")]
    pub exit_on: String,

    /// 最大帧数限制
    #[arg(long)]
    pub max_frames: Option<u64>,

    /// 超时（秒）
    #[arg(long)]
    pub timeout_sec: Option<u64>,
}

fn main() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!(
            "[Ring VN] Panic detected. If recording was enabled, check the recordings/ directory."
        );
        default_hook(info);
    }));

    let cli = Cli::parse();

    if cli.headless && cli.replay_input.is_none() {
        eprintln!("--headless 必须搭配 --replay-input");
        std::process::exit(1);
    }

    let config = AppConfig::load(CONFIG_PATH).unwrap_or_else(|e| panic!("{}", e));

    let configured = config
        .debug
        .log_level
        .as_deref()
        .unwrap_or("info")
        .trim()
        .to_ascii_lowercase();

    let level = match configured.as_str() {
        "trace" => LevelFilter::TRACE,
        "debug" => LevelFilter::DEBUG,
        "info" => LevelFilter::INFO,
        "warn" | "warning" => LevelFilter::WARN,
        "error" => LevelFilter::ERROR,
        "off" => LevelFilter::OFF,
        other => {
            // logger 未初始化前的唯一合法 stderr 输出（CLAUDE.md eprintln 禁令的例外）
            eprintln!("Invalid log_level: '{other}', fallback to info.");
            LevelFilter::INFO
        }
    };

    let log_file_writer =
        config
            .debug
            .log_file
            .as_ref()
            .and_then(|path| match std::fs::File::create(path) {
                Ok(file) => Some(file),
                Err(e) => {
                    // logger 未初始化前的唯一合法 stderr 输出（CLAUDE.md eprintln 禁令的例外）
                    eprintln!("Failed to create log file '{path}': {e}");
                    None
                }
            });

    if let Some(file) = log_file_writer {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .without_time()
            .compact()
            .with_target(false)
            .with_ansi(false)
            .with_writer(std::sync::Mutex::new(file))
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(level)
            .without_time()
            .compact()
            .with_target(false)
            .init();
    }

    info!(path = ?CONFIG_PATH, "Config loaded");

    if let Err(e) = config.validate() {
        panic!("Config validation failed: {}", e);
    }

    if cli.headless {
        let headless_cli = host::headless::HeadlessCli {
            replay_input: cli.replay_input.unwrap(),
            event_stream: cli.event_stream,
            exit_on: cli.exit_on,
            max_frames: cli.max_frames,
            timeout_sec: cli.timeout_sec,
        };
        match host::headless::run(config, &headless_cli) {
            Ok(()) => std::process::exit(0),
            Err(e) => {
                eprintln!("Headless 执行失败: {e}");
                std::process::exit(1);
            }
        }
    } else {
        let el = EventLoop::new().unwrap();
        el.set_control_flow(ControlFlow::Poll);
        el.run_app(&mut host_app::HostApp::new(config, cli.event_stream))
            .unwrap();
    }
}
