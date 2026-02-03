//! 渲染逻辑

use crate::AppMode;
use crate::AssetSourceType;
use macroquad::prelude::*;

use super::AppState;

/// 渲染函数
pub fn draw(app_state: &mut AppState) {
    let current_mode = app_state.navigation.current();

    // 根据当前模式绘制
    match current_mode {
        AppMode::Title => {
            app_state
                .title_screen
                .draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::InGame => {
            // 渲染游戏画面
            app_state.renderer.render(
                &app_state.render_state,
                &app_state.resource_manager,
                &app_state.manifest,
            );
        }
        AppMode::InGameMenu => {
            // 先渲染游戏画面，再渲染菜单覆盖层
            app_state.renderer.render(
                &app_state.render_state,
                &app_state.resource_manager,
                &app_state.manifest,
            );
            app_state
                .ingame_menu
                .draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::SaveLoad => {
            // 如果是从游戏内打开，先渲染游戏画面
            if app_state.vn_runtime.is_some() {
                app_state.renderer.render(
                    &app_state.render_state,
                    &app_state.resource_manager,
                    &app_state.manifest,
                );
            }
            app_state
                .save_load_screen
                .draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::Settings => {
            app_state
                .settings_screen
                .draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::History => {
            // 先渲染游戏画面，再渲染历史覆盖层
            app_state.renderer.render(
                &app_state.render_state,
                &app_state.resource_manager,
                &app_state.manifest,
            );
            app_state
                .history_screen
                .draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
    }

    // 绘制 Toast 提示（所有模式都可显示）
    app_state
        .toast_manager
        .draw(&app_state.ui_context, &app_state.renderer.text_renderer);

    // 显示调试信息
    if app_state.host_state.debug_mode {
        draw_debug_info(app_state);
    }
}

/// 绘制调试信息
pub fn draw_debug_info(app_state: &AppState) {
    let fps = get_fps();
    let char_count = app_state.render_state.visible_characters.len();
    let has_bg = app_state.render_state.current_background.is_some();
    let has_dialogue = app_state.render_state.dialogue.is_some();
    let current_mode = app_state.navigation.current();

    // 获取缓存统计
    let cache_stats = app_state.resource_manager.texture_cache_stats();

    // 绘制半透明背景（加高以容纳更多信息）
    // 注意：使用较高的 alpha 值确保可见性
    draw_rectangle(5.0, 5.0, 320.0, 240.0, Color::new(0.0, 0.0, 0.0, 0.85));

    // 基础信息
    let mut lines: Vec<(String, Color)> = vec![
        (format!("FPS: {}", fps), GREEN),
        (format!("模式: {:?}", current_mode), GREEN),
        (
            format!(
                "角色: {} | 背景: {} | 对话: {}",
                char_count, has_bg, has_dialogue
            ),
            GREEN,
        ),
    ];

    // 缓存统计
    lines.push(("--- 纹理缓存 ---".to_string(), YELLOW));
    lines.push((
        format!(
            "条目: {} | 占用: {:.1}MB / {:.1}MB",
            cache_stats.entries,
            cache_stats.used_bytes as f64 / 1024.0 / 1024.0,
            cache_stats.budget_bytes as f64 / 1024.0 / 1024.0
        ),
        WHITE,
    ));
    lines.push((
        format!(
            "命中率: {:.1}% ({}/{})",
            cache_stats.hit_rate * 100.0,
            cache_stats.hits,
            cache_stats.hits + cache_stats.misses
        ),
        if cache_stats.hit_rate > 0.8_f64 {
            GREEN
        } else if cache_stats.hit_rate > 0.5_f64 {
            YELLOW
        } else {
            RED
        },
    ));
    lines.push((
        format!("驱逐次数: {}", cache_stats.evictions),
        if cache_stats.evictions == 0 {
            GREEN
        } else {
            YELLOW
        },
    ));

    // 资源来源
    let source_info = match app_state.config.asset_source {
        AssetSourceType::Fs => "文件系统".to_string(),
        AssetSourceType::Zip => format!(
            "ZIP: {}",
            app_state.config.zip_path.as_deref().unwrap_or("?")
        ),
    };
    lines.push((
        format!("来源: {}", source_info),
        Color::new(0.7, 0.7, 0.7, 1.0),
    )); // 灰色

    // 绘制所有行
    for (i, (line, color)) in lines.iter().enumerate() {
        let y = 25.0 + i as f32 * 22.0;
        // 使用文本渲染器绘制（支持中文）
        app_state
            .renderer
            .text_renderer
            .draw_ui_text(line, 10.0, y, 16.0, *color);
    }
}
