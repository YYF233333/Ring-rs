//! InGame 页面 UI（对话框 + 快捷菜单 + 选项 + 章节标记 + 标题字卡）

use host::RenderState;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};
use host::ui::nine_patch::{Borders, NinePatch};

use crate::egui_actions::EguiAction;

pub fn build_ingame_ui(
    ctx: &egui::Context,
    render_state: &RenderState,
    layout: &UiLayoutConfig,
    assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
    let mut action = EguiAction::None;

    // 标题字卡（全屏居中，阻塞式）
    if let Some(ref tc) = render_state.title_card {
        build_title_card(ctx, tc, layout, scale);
    }

    // 章节标记（左上角，非阻塞，带淡入淡出）
    if let Some(ref cm) = render_state.chapter_mark {
        build_chapter_mark(ctx, cm, layout, scale);
    }

    if let Some(ref dialogue) = render_state.dialogue {
        let tb_height = scale.y(layout.dialogue.textbox_height);
        let screen_w = scale.actual_w;
        let area_rect = egui::Rect::from_min_size(
            egui::pos2(0.0, scale.actual_h - tb_height),
            egui::vec2(screen_w, tb_height),
        );

        // 对话框背景+文本：不拦截点击，让鼠标穿透以推进游戏
        egui::Area::new(egui::Id::new("dialogue_area"))
            .anchor(egui::Align2::LEFT_BOTTOM, [0.0, 0.0])
            .order(egui::Order::Middle)
            .interactable(false)
            .show(ctx, |ui| {
                ui.set_min_size(area_rect.size());

                let painter = ui.painter();

                if let Some(assets) = assets {
                    if let Some(tex) = assets.get("textbox") {
                        painter.image(
                            tex.id(),
                            area_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE,
                        );
                    }
                }

                if let Some(ref speaker) = dialogue.speaker {
                    let name_x = scale.x(layout.dialogue.name_xpos);
                    let name_y = scale.y(layout.dialogue.name_ypos);
                    let name_pos = egui::pos2(name_x, area_rect.top() + name_y);

                    if let Some(assets) = assets {
                        if let Some(tex) = assets.get("namebox") {
                            let borders = Borders::from_array(layout.dialogue.namebox_borders);
                            let name_size = scale.uniform(layout.fonts.name_text_size);
                            let nb_w = name_size * speaker.len() as f32 * 0.7
                                + scale.x(borders.left + borders.right)
                                + 20.0;
                            let nb_h = name_size + scale.y(borders.top + borders.bottom) + 8.0;
                            let nb_rect =
                                egui::Rect::from_min_size(name_pos, egui::vec2(nb_w, nb_h));
                            let np = NinePatch::new(tex, borders);
                            np.paint(painter, nb_rect, egui::Color32::WHITE);
                        }
                    }

                    let name_text_size = scale.uniform(layout.fonts.name_text_size);
                    let text_offset_x = scale.x(layout.dialogue.namebox_borders[0]) + 10.0;
                    let text_offset_y = scale.y(layout.dialogue.namebox_borders[1]) + 4.0;
                    painter.text(
                        egui::pos2(name_pos.x + text_offset_x, name_pos.y + text_offset_y),
                        egui::Align2::LEFT_TOP,
                        speaker,
                        egui::FontId::proportional(name_text_size),
                        layout.colors.accent.to_egui(),
                    );
                }

                let dlg_x = scale.x(layout.dialogue.dialogue_xpos);
                let dlg_y = scale.y(layout.dialogue.dialogue_ypos);
                let dlg_w = scale.x(layout.dialogue.dialogue_width);
                let text_size = scale.uniform(layout.fonts.text_size);

                let visible_text: String = dialogue
                    .content
                    .chars()
                    .take(dialogue.visible_chars)
                    .collect();

                let text_rect = egui::Rect::from_min_size(
                    egui::pos2(dlg_x, area_rect.top() + dlg_y),
                    egui::vec2(dlg_w, tb_height - dlg_y),
                );

                let galley = painter.layout(
                    visible_text,
                    egui::FontId::proportional(text_size),
                    layout.colors.text.to_egui(),
                    text_rect.width(),
                );
                painter.galley(text_rect.min, galley, egui::Color32::WHITE);
            });

        // 快捷菜单：独立 Area，拦截点击以避免同时推进游戏
        let qm_action = build_quick_menu(ctx, layout, scale, area_rect);
        if !matches!(qm_action, EguiAction::None) {
            action = qm_action;
        }
    }

    if let Some(ref choices) = render_state.choices {
        let choice_action = build_choices_ui(ctx, choices, layout, assets, scale);
        if !matches!(choice_action, EguiAction::None) {
            action = choice_action;
        }
    }

    action
}

fn build_quick_menu(
    ctx: &egui::Context,
    layout: &UiLayoutConfig,
    scale: &ScaleContext,
    textbox_rect: egui::Rect,
) -> EguiAction {
    let mut action = EguiAction::None;
    let text_size = scale.uniform(layout.quick_menu.text_size);
    let button_h = text_size + 8.0;
    let y = textbox_rect.bottom() - button_h - scale.y(4.0);

    let buttons: &[(&str, EguiAction)] = &[
        ("历史", EguiAction::NavigateTo(host::AppMode::History)),
        ("快进", EguiAction::ToggleSkip),
        ("自动", EguiAction::ToggleAuto),
        ("保存", EguiAction::OpenSave),
        ("快存", EguiAction::QuickSave),
        ("快读", EguiAction::QuickLoad),
        ("设置", EguiAction::NavigateTo(host::AppMode::Settings)),
    ];

    let total_w: f32 = buttons.len() as f32 * scale.x(90.0);
    let start_x = textbox_rect.center().x - total_w / 2.0;

    egui::Area::new(egui::Id::new("quick_menu_area"))
        .fixed_pos(egui::pos2(start_x, y))
        .order(egui::Order::Middle)
        .show(ctx, |ui| {
            ui.set_min_size(egui::vec2(total_w, button_h));
            ui.horizontal(|ui| {
                let idle_color = layout.colors.idle.to_egui();
                let hover_color = layout.colors.hover.to_egui();

                for (label, btn_action) in buttons {
                    let resp = ui.add(
                        egui::Button::new(
                            egui::RichText::new(*label)
                                .size(text_size)
                                .color(idle_color),
                        )
                        .frame(false),
                    );
                    if resp.hovered() {
                        ui.painter().text(
                            resp.rect.center(),
                            egui::Align2::CENTER_CENTER,
                            *label,
                            egui::FontId::proportional(text_size),
                            hover_color,
                        );
                    }
                    if resp.clicked() {
                        action = btn_action.clone();
                    }
                }
            });
        });

    action
}

fn build_choices_ui(
    ctx: &egui::Context,
    choices: &host::renderer::render_state::ChoicesState,
    layout: &UiLayoutConfig,
    assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
    let choice_w = scale.x(layout.choice.button_width);
    let spacing = scale.y(layout.choice.spacing);
    let text_size = scale.uniform(layout.fonts.text_size);

    // 选项不拦截点击——选择由 InputManager 的 choice_rects 驱动
    egui::Area::new(egui::Id::new("choices_area"))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.set_width(choice_w);
                ui.spacing_mut().item_spacing.y = spacing;

                for (i, choice) in choices.choices.iter().enumerate() {
                    let is_hover = choices.hovered_index == Some(i) || i == choices.selected_index;

                    let btn_h = text_size
                        + scale
                            .y(layout.choice.button_borders[1] + layout.choice.button_borders[3])
                        + 16.0;
                    let (rect, _resp) =
                        ui.allocate_exact_size(egui::vec2(choice_w, btn_h), egui::Sense::hover());

                    let painter = ui.painter();

                    if let Some(assets) = assets {
                        let key = if is_hover {
                            "choice_hover"
                        } else {
                            "choice_idle"
                        };
                        if let Some(tex) = assets.get(key) {
                            let borders = Borders::from_array(layout.choice.button_borders);
                            let np = NinePatch::new(tex, borders);
                            np.paint(painter, rect, egui::Color32::WHITE);
                        }
                    } else {
                        let bg = if is_hover {
                            egui::Color32::from_rgba_unmultiplied(60, 60, 80, 200)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(30, 30, 50, 180)
                        };
                        painter.rect_filled(rect, 4.0, bg);
                    }

                    let text_color = if is_hover {
                        egui::Color32::WHITE
                    } else {
                        egui::Color32::from_rgb(204, 204, 204)
                    };
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &choice.text,
                        egui::FontId::proportional(text_size),
                        text_color,
                    );
                }
            });
        });

    EguiAction::None
}

fn build_title_card(
    ctx: &egui::Context,
    tc: &host::renderer::render_state::TitleCardState,
    layout: &UiLayoutConfig,
    scale: &ScaleContext,
) {
    const FADE_DURATION: f32 = 0.3;
    let alpha = if tc.elapsed < FADE_DURATION {
        tc.elapsed / FADE_DURATION
    } else if tc.elapsed > tc.duration - FADE_DURATION {
        ((tc.duration - tc.elapsed) / FADE_DURATION).max(0.0)
    } else {
        1.0
    };
    let a = (alpha * 255.0) as u8;

    egui::Area::new(egui::Id::new("title_card_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            let rect = egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(scale.actual_w, scale.actual_h),
            );
            ui.set_min_size(rect.size());
            let painter = ui.painter();

            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, (a as f32 * 0.6) as u8),
            );

            let text_size = scale.uniform(layout.fonts.title_text_size);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                &tc.text,
                egui::FontId::proportional(text_size),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, a),
            );
        });
}

fn build_chapter_mark(
    ctx: &egui::Context,
    cm: &host::renderer::render_state::ChapterMarkState,
    layout: &UiLayoutConfig,
    scale: &ScaleContext,
) {
    let a = (cm.alpha * 255.0) as u8;
    if a == 0 {
        return;
    }

    let text_size = scale.uniform(layout.fonts.label_text_size);
    let padding_x = scale.x(20.0);
    let padding_y = scale.y(10.0);
    let margin = scale.uniform(30.0);

    egui::Area::new(egui::Id::new("chapter_mark_overlay"))
        .fixed_pos(egui::pos2(margin, margin))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            // 先测量文本尺寸来确定背景大小
            let font_id = egui::FontId::proportional(text_size);
            let galley =
                ctx.fonts(|f| f.layout_no_wrap(cm.title.clone(), font_id, egui::Color32::WHITE));
            let text_w = galley.rect.width();
            let text_h = galley.rect.height();
            let bg_w = text_w + padding_x * 2.0;
            let bg_h = text_h + padding_y * 2.0;

            ui.set_min_size(egui::vec2(bg_w, bg_h));

            let painter = ui.painter();
            let bg_rect =
                egui::Rect::from_min_size(egui::pos2(margin, margin), egui::vec2(bg_w, bg_h));

            painter.rect_filled(
                bg_rect,
                4.0,
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, (a as f32 * 0.5) as u8),
            );
            painter.galley(
                egui::pos2(bg_rect.left() + padding_x, bg_rect.top() + padding_y),
                galley,
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, a),
            );
        });
}
