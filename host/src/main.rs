//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。
//!
//! 本文件只保留 macroquad 入口、窗口配置与主循环胶水代码。
//! 业务逻辑位于 `host::app` 模块。

use host::app::{AppState, draw, save_continue, update};
use host::{AppConfig, AssetSourceType};
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

/// 加载所有资源
async fn load_resources(app_state: &mut AppState) {
    println!("📦 开始加载资源...");

    // 加载字体（使用配置中的字体路径）
    match app_state.config.asset_source {
        AssetSourceType::Fs => {
            let font_path = app_state
                .config
                .assets_root
                .join(&app_state.config.default_font);
            println!("✅ 加载字体: {:?}", font_path);
            if let Err(e) = app_state.renderer.init(&font_path.to_string_lossy()).await {
                eprintln!(
                    "⚠️ 字体加载失败，回退到 macroquad 默认字体（仅支持 ASCII）: {}",
                    e
                );
            }
        }
        AssetSourceType::Zip => {
            // ZIP 模式：需要将字体文件写入临时文件
            // 因为 macroquad 的 load_ttf_font 只接受文件路径
            let font_bytes = match app_state
                .resource_manager
                .read_bytes(&app_state.config.default_font)
            {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!(
                        "⚠️ 无法从 ZIP 读取字体文件: {} - {}",
                        app_state.config.default_font, e
                    );
                    eprintln!("⚠️ 回退到 macroquad 默认字体（仅支持 ASCII）");
                    return;
                }
            };

            // 创建临时文件
            let temp_dir = std::env::temp_dir();
            let temp_font_path = temp_dir.join(format!("ring_font_{}.ttf", std::process::id()));

            if let Err(e) = std::fs::write(&temp_font_path, &font_bytes) {
                eprintln!(
                    "⚠️ 无法写入临时字体文件: {} - {}",
                    temp_font_path.display(),
                    e
                );
                eprintln!("⚠️ 回退到 macroquad 默认字体（仅支持 ASCII）");
                return;
            }

            println!(
                "✅ 加载字体: {} (临时文件: {:?})",
                app_state.config.default_font, temp_font_path
            );
            if let Err(e) = app_state
                .renderer
                .init(&temp_font_path.to_string_lossy())
                .await
            {
                eprintln!(
                    "⚠️ 字体加载失败，回退到 macroquad 默认字体（仅支持 ASCII）: {}",
                    e
                );
            }

            // 注意：临时文件会在程序退出时自动清理（操作系统负责）
        }
    }

    // 预加载必需的 UI 纹理（用于过渡效果）
    // 其他资源改为按需加载（由 TextureCache 管理）
    let essential_textures = ["backgrounds/black.png", "backgrounds/white.png"];
    for path in &essential_textures {
        match app_state.resource_manager.load_texture(path).await {
            Ok(_) => println!("✅ 预加载: {}", path),
            Err(e) => eprintln!("⚠️ 预加载失败: {} - {}", path, e),
        }
    }

    app_state.loading_complete = true;
    let stats = app_state.resource_manager.texture_cache_stats();
    println!("📦 资源加载完成！{}", stats.format());
}

/// 确保渲染所需资源已加载（按需加载）
///
/// 检查 RenderState 中引用的资源，如果尚未缓存则加载。
async fn ensure_render_resources(app_state: &mut AppState) {
    // 收集需要加载的资源路径
    let mut paths_to_load: Vec<String> = Vec::new();

    // 检查当前背景
    if let Some(ref bg_path) = app_state.render_state.current_background {
        if !app_state.resource_manager.has_texture(bg_path) {
            paths_to_load.push(bg_path.clone());
        }
    }

    // 检查可见角色
    for character in app_state.render_state.visible_characters.values() {
        if !app_state
            .resource_manager
            .has_texture(&character.texture_path)
        {
            paths_to_load.push(character.texture_path.clone());
        }
    }

    // 检查场景过渡（Rule 效果需要遮罩纹理）
    if let Some(host::renderer::SceneTransitionType::Rule { mask_path, .. }) =
        app_state.renderer.scene_transition.transition_type()
    {
        if !app_state.resource_manager.has_texture(mask_path) {
            paths_to_load.push(mask_path.clone());
        }
    }

    // 加载缺失的资源
    for path in paths_to_load {
        match app_state.resource_manager.load_texture(&path).await {
            Ok(_) => println!("📦 按需加载: {}", path),
            Err(e) => eprintln!("❌ 加载失败: {} - {}", path, e),
        }
    }
}
