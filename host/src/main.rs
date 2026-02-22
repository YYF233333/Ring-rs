//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。
//!
//! 本文件只保留 macroquad 入口、窗口配置与主循环胶水代码。
//! 业务逻辑位于 `host::app` 模块。

use host::AppConfig;
use host::app::{AppState, draw, ensure_render_resources, load_resources, save_continue, update};
use macroquad::prelude::*;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

/// 配置文件路径
const CONFIG_PATH: &str = "config.json";

/// 窗口配置
fn window_conf() -> Conf {
    // 在窗口创建前读取配置（此函数在 main 之前被 macroquad 调用）
    let config = AppConfig::load(CONFIG_PATH);

    Conf {
        window_title: config.window.title,
        window_width: config.window.width as i32,
        window_height: config.window.height as i32,
        window_resizable: false,
        fullscreen: config.window.fullscreen,
        ..Default::default()
    }
}

/// 主函数
#[macroquad::main(window_conf)]
async fn main() {
    // 先加载配置文件（用来决定 log level）
    // 注意：这一步发生在 tracing 初始化之前，因此 AppConfig::load 内部的日志会被丢弃（这是预期行为）。
    let config = AppConfig::load(CONFIG_PATH);

    // 初始化日志系统
    // 来源：config.debug.log_level（默认 info）
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
            eprintln!(
                "Invalid config debug.log_level: '{other}'. Allowed: trace/debug/info/warn/error/off. Fallback to info."
            );
            LevelFilter::INFO
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        // 更简洁的输出：不显示时间戳，使用紧凑格式，隐藏 target（模块路径）
        // 输出示例：`INFO 配置加载完成 path="config.json"`
        .without_time()
        .compact()
        .with_target(false)
        .init();

    info!(path = ?CONFIG_PATH, "配置加载完成");
    info!(assets_root = %config.assets_root.display(), "资源根目录");
    info!(saves_dir = %config.saves_dir.display(), "存档目录");
    info!(start_script_path = %config.start_script_path, "启动脚本");

    // **验证配置（必须配置 start_script_path）**
    if let Err(e) = config.validate() {
        panic!("❌ 配置验证失败: {}", e);
    }

    // 初始化应用状态
    let mut app_state = AppState::new(config);

    // 加载资源
    load_resources(&mut app_state).await;

    // 主循环
    while app_state.host_state.running {
        // 更新逻辑
        update(&mut app_state);

        // 确保渲染所需资源已加载（按需加载）
        ensure_render_resources(&mut app_state).await;

        // 渲染
        draw(&mut app_state);

        // 等待下一帧
        next_frame().await;
    }

    // 退出前保存 Continue 存档
    save_continue(&mut app_state);
}
