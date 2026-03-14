//! 游戏菜单通用框架
//!
//! 提供统一的「左侧导航 + 右侧内容」布局，
//! 供 SaveLoad、Settings、History 等页面复用。

use host::AppMode;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};

use crate::egui_actions::EguiAction;

/// 游戏菜单框架的构建入口。
///
/// `title`: 页面标题（如 "保存"、"设置"）
/// `content_builder`: 在右侧内容区执行的闭包，返回可能的 EguiAction
pub fn build_game_menu_frame(
    ctx: &egui::Context,
    title: &str,
    layout: &UiLayoutConfig,
    assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
    content_builder: impl FnOnce(&mut egui::Ui) -> EguiAction,
) -> EguiAction {
    let mut action = EguiAction::None;
    let nav_width = scale.x(layout.game_menu.nav_width);

    egui::CentralPanel::default()
        .frame(
            egui::Frame::new()
                .fill(egui::Color32::TRANSPARENT)
                .inner_margin(0.0),
        )
        .show(ctx, |ui| {
            let screen_rect = ui.max_rect();

            // Background image
            if let Some(assets) = assets {
                let bg_key = "game_menu_bg";
                if let Some(tex) = assets.get(bg_key) {
                    ui.painter().image(
                        tex.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                if let Some(overlay) = assets.get("game_menu_overlay") {
                    ui.painter().image(
                        overlay.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }

            // Left navigation panel
            let nav_rect = egui::Rect::from_min_size(
                screen_rect.left_top(),
                egui::vec2(nav_width, screen_rect.height()),
            );

            let nav_action = build_nav_panel(ui, nav_rect, layout, scale);
            if !matches!(nav_action, EguiAction::None) {
                action = nav_action;
            }

            // Right content area
            let content_rect = egui::Rect::from_min_max(
                egui::pos2(
                    nav_rect.right() + scale.x(20.0),
                    screen_rect.top() + scale.y(40.0),
                ),
                egui::pos2(
                    screen_rect.right() - scale.x(40.0),
                    screen_rect.bottom() - scale.y(40.0),
                ),
            );

            // Title
            let title_size = scale.uniform(layout.fonts.title_text_size * 0.6);
            let interface_color = layout.colors.interface_text.to_egui();
            ui.painter().text(
                egui::pos2(content_rect.left(), content_rect.top()),
                egui::Align2::LEFT_TOP,
                title,
                egui::FontId::proportional(title_size),
                interface_color,
            );

            let content_start_y = content_rect.top() + title_size + scale.y(16.0);
            let inner_rect = egui::Rect::from_min_max(
                egui::pos2(content_rect.left(), content_start_y),
                content_rect.max,
            );

            let mut content_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(inner_rect)
                    .layout(egui::Layout::top_down(egui::Align::LEFT)),
            );

            let content_action = content_builder(&mut content_ui);
            if !matches!(content_action, EguiAction::None) {
                action = content_action;
            }
        });

    action
}

fn build_nav_panel(
    ui: &mut egui::Ui,
    nav_rect: egui::Rect,
    layout: &UiLayoutConfig,
    scale: &ScaleContext,
) -> EguiAction {
    let mut action = EguiAction::None;
    let text_size = scale.uniform(layout.fonts.interface_text_size);
    let btn_h = text_size + 16.0;
    let spacing = scale.y(layout.game_menu.navigation_spacing);

    let entries: &[(&str, EguiAction)] = &[
        ("历史", EguiAction::ReplaceTo(AppMode::History)),
        ("保存", EguiAction::OpenSave),
        ("读取", EguiAction::OpenLoad),
        ("设置", EguiAction::ReplaceTo(AppMode::Settings)),
        (
            "返回标题",
            EguiAction::ShowConfirm {
                message: "确定返回标题画面？".into(),
                on_confirm: Box::new(EguiAction::ReturnToTitle),
            },
        ),
        (
            "退出",
            EguiAction::ShowConfirm {
                message: "确定退出游戏？".into(),
                on_confirm: Box::new(EguiAction::Exit),
            },
        ),
    ];

    let total_h = entries.len() as f32 * (btn_h + spacing);
    let start_y = nav_rect.center().y - total_h / 2.0;
    let btn_x = nav_rect.left() + scale.x(layout.title.navigation_xpos);
    let btn_w = nav_rect.width() - scale.x(layout.title.navigation_xpos) - scale.x(20.0);

    let idle_color = layout.colors.idle.to_egui();
    let hover_color = layout.colors.hover.to_egui();

    let mut y = start_y;
    for (label, btn_action) in entries {
        let btn_rect = egui::Rect::from_min_size(egui::pos2(btn_x, y), egui::vec2(btn_w, btn_h));
        let resp = ui.allocate_rect(btn_rect, egui::Sense::click());
        let is_hover = resp.hovered();

        let text_color = if is_hover { hover_color } else { idle_color };
        ui.painter().text(
            egui::pos2(btn_rect.left() + 10.0, btn_rect.center().y),
            egui::Align2::LEFT_CENTER,
            *label,
            egui::FontId::proportional(text_size),
            text_color,
        );

        if resp.clicked() {
            action = btn_action.clone();
        }
        y += btn_h + spacing;
    }

    // Return button at bottom
    let return_rect = egui::Rect::from_min_size(
        egui::pos2(btn_x, nav_rect.bottom() - btn_h - scale.y(40.0)),
        egui::vec2(btn_w, btn_h),
    );
    let return_resp = ui.allocate_rect(return_rect, egui::Sense::click());
    let ret_color = if return_resp.hovered() {
        hover_color
    } else {
        idle_color
    };
    ui.painter().text(
        egui::pos2(return_rect.left() + 10.0, return_rect.center().y),
        egui::Align2::LEFT_CENTER,
        "返回",
        egui::FontId::proportional(text_size),
        ret_color,
    );
    if return_resp.clicked() {
        action = EguiAction::ReturnToGame;
    }

    action
}
