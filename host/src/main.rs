//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。

mod egui_actions;
mod egui_screens;
mod host_app;

use host::AppConfig;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use winit::event_loop::{ControlFlow, EventLoop};

const CONFIG_PATH: &str = "config.json";

fn main() {
    let config = AppConfig::load(CONFIG_PATH);

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
            eprintln!("Invalid log_level: '{other}', fallback to info.");
            LevelFilter::INFO
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .without_time()
        .compact()
        .with_target(false)
        .init();

    info!(path = ?CONFIG_PATH, "Config loaded");

    if let Err(e) = config.validate() {
        panic!("Config validation failed: {}", e);
    }

    let font_path = config.assets_root.join(&config.default_font);
    let font_data = match std::fs::read(&font_path) {
        Ok(data) => {
            info!(path = ?font_path, "CJK font loaded");
            Some(data)
        }
        Err(e) => {
            tracing::warn!(path = ?font_path, error = %e, "Cannot load CJK font");
            None
        }
    };

    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Poll);
    el.run_app(&mut host_app::HostApp::new(config, font_data))
        .unwrap();
}
