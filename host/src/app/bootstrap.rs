//! 启动与资源引导（bootstrap）

use super::AppState;
use crate::resources::LogicalPath;
use tracing::{debug, error, info};

/// 加载启动阶段必需资源（基础纹理等）
///
/// 纹理加载现在是同步的（wgpu 纹理创建不需要 async）。
pub fn load_resources(app_state: &mut AppState) {
    info!("开始加载资源...");

    // Renderer 初始化（Phase 2 前为空操作）
    app_state.core.renderer.init();

    // 预加载必需的 UI 纹理（用于过渡效果）
    let essential_textures = ["backgrounds/black.png", "backgrounds/white.png"];
    for path in &essential_textures {
        let logical = LogicalPath::new(path);
        match app_state.core.resource_manager.load_texture(&logical) {
            Ok(_) => debug!(path = %path, "预加载纹理"),
            Err(e) => error!(path = %path, error = %e, "预加载失败"),
        }
    }

    app_state.loading_complete = true;
    let stats = app_state.core.resource_manager.texture_cache_stats();
    info!(stats = %stats.format(), "资源加载完成");
}

/// 确保渲染所需资源已加载（按需加载）
///
/// 检查 RenderState / 过渡状态中引用的资源，如果尚未缓存则加载。
pub fn ensure_render_resources(app_state: &mut AppState) {
    let mut paths_to_load: Vec<LogicalPath> = Vec::new();

    // 检查当前背景
    if let Some(ref bg_path) = app_state.core.render_state.current_background {
        let logical = LogicalPath::new(bg_path);
        if !app_state.core.resource_manager.has_texture(&logical)
            && !app_state.core.resource_manager.has_failed_texture(&logical)
        {
            paths_to_load.push(logical);
        }
    }

    // 检查可见角色
    for character in app_state.core.render_state.visible_characters.values() {
        let logical = LogicalPath::new(&character.texture_path);
        if !app_state.core.resource_manager.has_texture(&logical)
            && !app_state.core.resource_manager.has_failed_texture(&logical)
        {
            paths_to_load.push(logical);
        }
    }

    // 检查场景过渡（Rule 效果需要遮罩纹理）
    if let Some(crate::renderer::SceneTransitionType::Rule { mask_path, .. }) =
        app_state.core.renderer.scene_transition.transition_type()
    {
        let logical = LogicalPath::new(mask_path);
        if !app_state.core.resource_manager.has_texture(&logical)
            && !app_state.core.resource_manager.has_failed_texture(&logical)
        {
            paths_to_load.push(logical);
        }
    }

    // 加载缺失的资源
    for path in paths_to_load {
        match app_state.core.resource_manager.load_texture(&path) {
            Ok(_) => debug!(path = %path, "按需加载纹理"),
            Err(e) => error!(path = %path, error = %e, "加载失败"),
        }
    }
}
