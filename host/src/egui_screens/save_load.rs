//! SaveLoad 页面 UI

use host::SaveLoadTab;
use host::save_manager::SaveInfo;
use host::ui::asset_cache::UiAssetCache;
use host::ui::layout::{ScaleContext, UiLayoutConfig};
use host::ui::nine_patch::{Borders, NinePatch};

use crate::egui_actions::EguiAction;

/// 存读档页面内容区（由 `game_menu_frame` 包裹）
pub fn build_save_load_content(
    ui: &mut egui::Ui,
    tab: SaveLoadTab,
    save_infos: &[Option<SaveInfo>],
    can_save: bool,
    layout: &UiLayoutConfig,
    assets: Option<&UiAssetCache>,
    scale: &ScaleContext,
) -> EguiAction {
    let mut action = EguiAction::None;

    let label_size = scale.uniform(layout.fonts.label_text_size);
    let accent_color = layout.colors.accent.to_egui();
    let idle_color = layout.colors.idle.to_egui();
    let interface_color = layout.colors.interface_text.to_egui();

    let cols = layout.save_load.cols as usize;
    let slot_w = scale.x(layout.save_load.slot_width);
    let slot_h = scale.y(layout.save_load.slot_height);
    let slot_spacing = scale.uniform(layout.save_load.slot_spacing);

    // Tab switcher
    ui.horizontal(|ui| {
        let save_label = if tab == SaveLoadTab::Save {
            egui::RichText::new("[ 保存 ]")
                .size(label_size)
                .strong()
                .color(accent_color)
        } else {
            egui::RichText::new("  保存  ")
                .size(label_size)
                .color(idle_color)
        };
        let load_label = if tab == SaveLoadTab::Load {
            egui::RichText::new("[ 读取 ]")
                .size(label_size)
                .strong()
                .color(accent_color)
        } else {
            egui::RichText::new("  读取  ")
                .size(label_size)
                .color(idle_color)
        };

        if ui.selectable_label(false, save_label).clicked() && can_save {
            action = EguiAction::OpenSave;
        }
        if ui.selectable_label(false, load_label).clicked() {
            action = EguiAction::OpenLoad;
        }
    });
    ui.add_space(scale.y(8.0));
    ui.separator();
    ui.add_space(scale.y(8.0));

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            for (chunk_idx, row) in save_infos.chunks(cols).enumerate() {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = slot_spacing;
                    for (col_idx, info) in row.iter().enumerate() {
                        let actual_slot = (chunk_idx * cols + col_idx + 1) as u32;

                        ui.push_id(actual_slot, |ui| {
                            let (rect, resp) = ui.allocate_exact_size(
                                egui::vec2(slot_w, slot_h),
                                egui::Sense::click(),
                            );
                            let is_hover = resp.hovered();
                            let small_size = scale.uniform(layout.fonts.notify_text_size);

                            // Delete button (allocate before painter borrow)
                            let del_resp = if info.is_some() {
                                let del_size = small_size + 4.0;
                                let del_rect = egui::Rect::from_min_size(
                                    egui::pos2(
                                        rect.right() - del_size - scale.x(6.0),
                                        rect.top() + scale.y(6.0),
                                    ),
                                    egui::vec2(del_size, del_size),
                                );
                                Some(ui.allocate_rect(del_rect, egui::Sense::click()))
                            } else {
                                None
                            };

                            let painter = ui.painter();

                            if let Some(assets) = assets {
                                let key = if is_hover { "slot_hover" } else { "slot_idle" };
                                if let Some(tex) = assets.get(key) {
                                    let borders = Borders::new(15.0, 15.0, 15.0, 15.0);
                                    let np = NinePatch::new(tex, borders);
                                    np.paint(painter, rect, egui::Color32::WHITE);
                                }
                            } else {
                                let bg = if is_hover {
                                    egui::Color32::from_rgb(40, 40, 70)
                                } else {
                                    egui::Color32::from_rgb(30, 30, 55)
                                };
                                painter.rect_filled(rect, 4.0, bg);
                            }

                            let thumb_w = scale.x(layout.save_load.thumbnail_width);
                            let thumb_h = scale.y(layout.save_load.thumbnail_height);
                            let thumb_rect = egui::Rect::from_min_size(
                                egui::pos2(
                                    rect.center().x - thumb_w / 2.0,
                                    rect.top() + scale.y(15.0),
                                ),
                                egui::vec2(thumb_w, thumb_h),
                            );
                            painter.rect_filled(
                                thumb_rect,
                                2.0,
                                egui::Color32::from_rgb(20, 20, 30),
                            );

                            if let Some(si) = info {
                                let chapter = si.chapter_title.as_deref().unwrap_or("---");
                                painter.text(
                                    egui::pos2(rect.center().x, thumb_rect.bottom() + scale.y(8.0)),
                                    egui::Align2::CENTER_TOP,
                                    chapter,
                                    egui::FontId::proportional(small_size),
                                    interface_color,
                                );
                                painter.text(
                                    egui::pos2(
                                        rect.center().x,
                                        thumb_rect.bottom() + scale.y(28.0),
                                    ),
                                    egui::Align2::CENTER_TOP,
                                    si.formatted_timestamp(),
                                    egui::FontId::proportional(small_size * 0.85),
                                    idle_color,
                                );
                            } else {
                                painter.text(
                                    egui::pos2(rect.center().x, thumb_rect.bottom() + scale.y(8.0)),
                                    egui::Align2::CENTER_TOP,
                                    "-- 空 --",
                                    egui::FontId::proportional(small_size),
                                    idle_color,
                                );
                            }

                            // Draw delete button
                            let mut delete_clicked = false;
                            if let Some(del_resp) = &del_resp {
                                let del_color = if del_resp.hovered() {
                                    egui::Color32::from_rgb(255, 80, 80)
                                } else {
                                    idle_color
                                };
                                painter.text(
                                    del_resp.rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "×",
                                    egui::FontId::proportional(small_size),
                                    del_color,
                                );
                                if del_resp.clicked() {
                                    delete_clicked = true;
                                    action = EguiAction::ShowConfirm {
                                        message: "确定删除此存档？".into(),
                                        on_confirm: Box::new(EguiAction::DeleteSlot(actual_slot)),
                                    };
                                }
                            }

                            if resp.clicked() && !delete_clicked {
                                match tab {
                                    SaveLoadTab::Save if can_save => {
                                        action = if info.is_some() {
                                            EguiAction::ShowConfirm {
                                                message: "覆盖已有存档？".into(),
                                                on_confirm: Box::new(EguiAction::SaveToSlot(
                                                    actual_slot,
                                                )),
                                            }
                                        } else {
                                            EguiAction::SaveToSlot(actual_slot)
                                        };
                                    }
                                    SaveLoadTab::Load if info.is_some() => {
                                        action = EguiAction::LoadFromSlot(actual_slot);
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                });
                ui.add_space(slot_spacing);
            }
        });

    action
}
