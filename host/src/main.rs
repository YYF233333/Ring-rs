//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。
//!
//! 本文件只保留 macroquad 入口、窗口配置与主循环胶水代码。
//! 业务逻辑位于 `host::app` 模块。

use host::AppConfig;
use host::app::{AppState, draw, ensure_render_resources, load_resources, save_continue, update};
use macroquad::prelude::*;

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
    // 加载配置文件
    let config = AppConfig::load(CONFIG_PATH);
    println!("✅ 配置加载完成: {:?}", CONFIG_PATH);
    println!("   assets_root: {:?}", config.assets_root);
    println!("   saves_dir: {:?}", config.saves_dir);
    println!("   start_script_path: {:?}", config.start_script_path);

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
