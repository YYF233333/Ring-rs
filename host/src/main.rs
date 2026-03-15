//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod egui_actions;
mod egui_screens;
mod host_app;

use host::{AppConfig, LogicalPath};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use winit::event_loop::{ControlFlow, EventLoop};

const CONFIG_PATH: &str = "config.json";

fn main() {
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

    let source = host::app::init::create_resource_source(&config);
    let font_path = LogicalPath::new(&config.default_font);
    let font_data = match source.read(&font_path) {
        Ok(data) => {
            info!(font = %font_path, "CJK font loaded");
            Some(data)
        }
        Err(e) => {
            tracing::warn!(font = %font_path, error = %e, "Cannot load CJK font");
            None
        }
    };

    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut host_app::HostApp::new(config, font_data))
        .unwrap();
}
