//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。

use macroquad::prelude::*;
use host::HostState;
use host::resources::ResourceManager;

/// 窗口配置
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_TITLE: &str = "Visual Novel Engine";

/// 应用状态
struct AppState {
    host_state: HostState,
    resource_manager: ResourceManager,
    background_texture: Option<Texture2D>,
    loading_error: Option<String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            host_state: HostState::new(),
            resource_manager: ResourceManager::new("assets"),
            background_texture: None,
            loading_error: None,
        }
    }
}

/// 主函数
#[macroquad::main(window_conf)]
async fn main() {
    // 初始化应用状态
    let mut app_state = AppState::new();

    // 加载测试背景
    match app_state
        .resource_manager
        .load_texture("backgrounds/rule_10.png")
        .await
    {
        Ok(texture) => {
            app_state.background_texture = Some(texture);
            println!("✅ 成功加载背景图片: backgrounds/rule_10.png");
        }
        Err(e) => {
            app_state.loading_error = Some(format!("加载失败: {}", e));
            eprintln!("❌ 加载背景图片失败: {}", e);
        }
    }

    // 主循环
    while app_state.host_state.running {
        // 更新逻辑
        update(&mut app_state);

        // 渲染
        draw(&app_state);

        // 等待下一帧
        next_frame().await;
    }
}

/// 窗口配置
fn window_conf() -> Conf {
    Conf {
        window_title: WINDOW_TITLE.to_string(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: false,
        fullscreen: false,
        ..Default::default()
    }
}

/// 更新逻辑
fn update(app_state: &mut AppState) {
    // 检查窗口关闭
    if is_key_pressed(KeyCode::Escape) {
        app_state.host_state.stop();
    }

    // 切换调试模式
    if is_key_pressed(KeyCode::F1) {
        app_state.host_state.debug_mode = !app_state.host_state.debug_mode;
    }

    // 按 R 键重新加载背景（测试用）
    if is_key_pressed(KeyCode::R) {
        // 异步加载需要在 async 上下文中，这里只是演示
        println!("按 R 键：重新加载背景（需要在 async 上下文中）");
    }
}

/// 渲染函数
fn draw(app_state: &AppState) {
    // 清空屏幕
    clear_background(BLACK);

    // 绘制背景图片
    if let Some(ref texture) = app_state.background_texture {
        // 计算缩放比例以适应窗口
        let screen_width = screen_width();
        let screen_height = screen_height();
        let texture_width = texture.width();
        let texture_height = texture.height();

        // 计算缩放比例，保持宽高比
        let scale_x = screen_width / texture_width;
        let scale_y = screen_height / texture_height;
        let scale = scale_x.min(scale_y);

        let scaled_width = texture_width * scale;
        let scaled_height = texture_height * scale;

        // 居中绘制
        let x = (screen_width - scaled_width) / 2.0;
        let y = (screen_height - scaled_height) / 2.0;

        draw_texture_ex(
            &texture,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(scaled_width, scaled_height)),
                ..Default::default()
            },
        );
    } else if let Some(ref error) = app_state.loading_error {
        // 显示加载错误
        draw_text(
            &format!("加载错误: {}", error),
            screen_width() / 2.0 - 200.0,
            screen_height() / 2.0,
            30.0,
            RED,
        );
    } else {
        // 显示加载中
        draw_text(
            "加载中...",
            screen_width() / 2.0 - 50.0,
            screen_height() / 2.0,
            30.0,
            WHITE,
        );
    }

    // 显示调试信息
    if app_state.host_state.debug_mode {
        draw_debug_info(app_state);
    }

    // 显示操作提示
    draw_text(
        "按 ESC 退出 | 按 F1 切换调试模式",
        10.0,
        screen_height() - 30.0,
        20.0,
        WHITE,
    );
}

/// 绘制调试信息
fn draw_debug_info(app_state: &AppState) {
    let fps = get_fps();
    let texture_count = app_state.resource_manager.texture_count();
    let sound_count = app_state.resource_manager.sound_count();
    let has_bg = app_state.background_texture.is_some();

    let debug_text = format!(
        "FPS: {}\nDebug Mode: ON\nRunning: {}\nTextures: {}\nSounds: {}\nBackground Loaded: {}",
        fps, app_state.host_state.running, texture_count, sound_count, has_bg
    );

    // 绘制调试文本（白色，左上角）
    draw_text(&debug_text, 10.0, 30.0, 20.0, WHITE);
}
