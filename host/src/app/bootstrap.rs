//! 启动与资源引导（bootstrap）
//!
//! 目标：让 `host/src/main.rs` 只保留 macroquad 入口与主循环胶水。

use super::AppState;
use crate::AssetSourceType;
use tracing::{debug, error, info, warn};

/// 加载启动阶段必需资源（字体、基础纹理等）
pub async fn load_resources(app_state: &mut AppState) {
    info!("开始加载资源...");

    // 加载字体（使用配置中的字体路径）
    match app_state.config.asset_source {
        AssetSourceType::Fs => {
            let font_path = app_state
                .config
                .assets_root
                .join(&app_state.config.default_font);
            info!(path = ?font_path, "加载字体");
            if let Err(e) = app_state.renderer.init(&font_path.to_string_lossy()).await {
                warn!(
                    error = %e,
                    "字体加载失败，回退到 macroquad 默认字体（仅支持 ASCII）"
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
                    warn!(
                        font = %app_state.config.default_font,
                        error = %e,
                        "无法从 ZIP 读取字体文件，回退到 macroquad 默认字体（仅支持 ASCII）"
                    );
                    return;
                }
            };

            // 创建临时文件
            let temp_dir = std::env::temp_dir();
            let temp_font_path = temp_dir.join(format!("ring_font_{}.ttf", std::process::id()));

            if let Err(e) = std::fs::write(&temp_font_path, &font_bytes) {
                warn!(
                    path = %temp_font_path.display(),
                    error = %e,
                    "无法写入临时字体文件，回退到 macroquad 默认字体（仅支持 ASCII）"
                );
                return;
            }

            info!(
                font = %app_state.config.default_font,
                temp_path = ?temp_font_path,
                "加载字体"
            );
            if let Err(e) = app_state
                .renderer
                .init(&temp_font_path.to_string_lossy())
                .await
            {
                warn!(
                    error = %e,
                    "字体加载失败，回退到 macroquad 默认字体（仅支持 ASCII）"
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
            Ok(_) => debug!(path = %path, "预加载纹理"),
            Err(e) => warn!(path = %path, error = %e, "预加载失败"),
        }
    }

    app_state.loading_complete = true;
    let stats = app_state.resource_manager.texture_cache_stats();
    info!(stats = %stats.format(), "资源加载完成");
}

/// 确保渲染所需资源已加载（按需加载）
///
/// 检查 RenderState / 过渡状态中引用的资源，如果尚未缓存则加载。
pub async fn ensure_render_resources(app_state: &mut AppState) {
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
    if let Some(crate::renderer::SceneTransitionType::Rule { mask_path, .. }) =
        app_state.renderer.scene_transition.transition_type()
    {
        if !app_state.resource_manager.has_texture(mask_path) {
            paths_to_load.push(mask_path.clone());
        }
    }

    // 加载缺失的资源
    for path in paths_to_load {
        match app_state.resource_manager.load_texture(&path).await {
            Ok(_) => debug!(path = %path, "按需加载纹理"),
            Err(e) => error!(path = %path, error = %e, "加载失败"),
        }
    }
}
