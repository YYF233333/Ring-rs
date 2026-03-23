//! # 地图 UI 模式处理器
//!
//! 实现 `UiModeHandler`，支持背景图渲染和颜色掩码命中检测。
//! 无 hit_mask 时回退到基于坐标的矩形命中检测（向后兼容）。

use std::collections::HashMap;

use tracing::{debug, warn};
use vn_runtime::state::VarValue;

use crate::resources::{LogicalPath, ResourceManager};
use crate::ui::layout::ScaleContext;
use crate::ui::map::MapDefinition;

use super::{UiModeError, UiModeHandler, UiModeStatus};

/// 地图 UI 模式处理器
#[derive(Debug)]
pub struct MapModeHandler {
    state: Option<MapActiveState>,
}

struct MapActiveState {
    definition: MapDefinition,
    request_key: String,
    /// 位置可用性缓存
    availability: Vec<bool>,
    /// 背景图原始像素（RGBA，延迟加载为 egui 纹理）
    background_image: Option<LoadedImage>,
    /// 背景图 egui 纹理（首次 render 时创建）
    background_texture: Option<egui::TextureHandle>,
    /// 掩码图原始像素（CPU 侧，用于命中检测）
    mask_data: Option<MaskData>,
}

impl std::fmt::Debug for MapActiveState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapActiveState")
            .field("definition", &self.definition)
            .field("request_key", &self.request_key)
            .field("availability", &self.availability)
            .field("has_background", &self.background_texture.is_some())
            .field("has_mask", &self.mask_data.is_some())
            .finish()
    }
}

#[derive(Debug)]
struct LoadedImage {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Debug)]
struct MaskData {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    /// RGB → location 索引映射表
    color_to_location: HashMap<[u8; 3], usize>,
}

/// 解析 "#RRGGBB" 格式的颜色字符串
fn parse_hex_color(s: &str) -> Option<[u8; 3]> {
    let s = s.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

impl Default for MapModeHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl MapModeHandler {
    pub fn new() -> Self {
        Self { state: None }
    }

    fn hit_test_mask(mask: &MaskData, base_x: f32, base_y: f32) -> Option<usize> {
        let px = base_x as u32;
        let py = base_y as u32;
        if px >= mask.width || py >= mask.height {
            return None;
        }
        let offset = ((py * mask.width + px) * 4) as usize;
        if offset + 3 >= mask.pixels.len() {
            return None;
        }
        let r = mask.pixels[offset];
        let g = mask.pixels[offset + 1];
        let b = mask.pixels[offset + 2];
        let a = mask.pixels[offset + 3];
        if a < 128 {
            return None;
        }
        mask.color_to_location.get(&[r, g, b]).copied()
    }
}

impl UiModeHandler for MapModeHandler {
    fn mode_id(&self) -> &str {
        "show_map"
    }

    fn activate(
        &mut self,
        key: String,
        params: &HashMap<String, VarValue>,
        resources: &ResourceManager,
    ) -> Result<(), UiModeError> {
        let map_id = params
            .get("map_id")
            .and_then(|v| {
                if let VarValue::String(s) = v {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .ok_or_else(|| UiModeError::InvalidParams("missing map_id parameter".into()))?;

        let map_path = format!("maps/{}.json", map_id);
        let logical = LogicalPath::new(&map_path);
        let json_text = resources
            .read_text(&logical)
            .map_err(|e| UiModeError::ResourceLoadFailed(format!("map file: {}", e)))?;

        let definition: MapDefinition = serde_json::from_str(&json_text)
            .map_err(|e| UiModeError::ResourceLoadFailed(format!("map JSON parse: {}", e)))?;

        let availability: Vec<bool> = definition.locations.iter().map(|loc| loc.enabled).collect();

        let background_image = if let Some(ref bg_path) = definition.background {
            let logical = LogicalPath::new(bg_path);
            match resources.read_bytes(&logical) {
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let (w, h) = rgba.dimensions();
                        Some(LoadedImage {
                            pixels: rgba.into_raw(),
                            width: w,
                            height: h,
                        })
                    }
                    Err(e) => {
                        warn!(path = bg_path, error = %e, "地图背景图解码失败");
                        None
                    }
                },
                Err(e) => {
                    warn!(path = bg_path, error = %e, "地图背景图加载失败");
                    None
                }
            }
        } else {
            None
        };

        let mask_data = if let Some(ref mask_path) = definition.hit_mask {
            let logical = LogicalPath::new(mask_path);
            match resources.read_bytes(&logical) {
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgba = img.to_rgba8();
                        let (w, h) = rgba.dimensions();
                        let mut color_map = HashMap::new();
                        for (i, loc) in definition.locations.iter().enumerate() {
                            if let Some(ref color_str) = loc.mask_color {
                                if let Some(rgb) = parse_hex_color(color_str) {
                                    color_map.insert(rgb, i);
                                } else {
                                    warn!(
                                        location = loc.id,
                                        color = color_str,
                                        "无效的 mask_color 格式"
                                    );
                                }
                            }
                        }
                        Some(MaskData {
                            pixels: rgba.into_raw(),
                            width: w,
                            height: h,
                            color_to_location: color_map,
                        })
                    }
                    Err(e) => {
                        warn!(path = mask_path, error = %e, "地图掩码图解码失败");
                        None
                    }
                },
                Err(e) => {
                    warn!(path = mask_path, error = %e, "地图掩码图加载失败");
                    None
                }
            }
        } else {
            None
        };

        debug!(map_id, "MapModeHandler 激活");
        self.state = Some(MapActiveState {
            definition,
            request_key: key,
            availability,
            background_image,
            background_texture: None,
            mask_data,
        });
        Ok(())
    }

    fn render(&mut self, ctx: &egui::Context, scale: &ScaleContext) -> UiModeStatus {
        let state = match self.state.as_mut() {
            Some(s) => s,
            None => return UiModeStatus::Cancelled,
        };

        // 延迟创建背景纹理
        if state.background_texture.is_none()
            && let Some(ref img) = state.background_image
        {
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [img.width as usize, img.height as usize],
                &img.pixels,
            );
            state.background_texture =
                Some(ctx.load_texture("map_background", color_image, egui::TextureOptions::LINEAR));
        }

        let screen_rect = egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(scale.actual_w, scale.actual_h),
        );

        let button_w = scale.x(200.0);
        let button_h = scale.y(50.0);

        let mut result_status = UiModeStatus::Active;

        egui::Area::new(egui::Id::new("ui_mode_map_overlay"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                ui.set_min_size(screen_rect.size());

                // 先 allocate 所有按钮区域（需要 &mut ui）
                let mut responses: Vec<(usize, egui::Response, egui::Rect)> = Vec::new();
                for (i, loc) in state.definition.locations.iter().enumerate() {
                    let cx = scale.x(loc.x);
                    let cy = scale.y(loc.y);
                    let btn_rect = egui::Rect::from_center_size(
                        egui::pos2(cx, cy),
                        egui::vec2(button_w, button_h),
                    );
                    let response = ui.allocate_rect(btn_rect, egui::Sense::click());
                    responses.push((i, response, btn_rect));
                }

                // allocate 完成后获取 painter（&ui 不可变借用）
                let painter = ui.painter();

                // 绘制背景
                if let Some(ref tex) = state.background_texture {
                    painter.image(
                        tex.id(),
                        screen_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                } else {
                    painter.rect_filled(
                        screen_rect,
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 200),
                    );
                }

                // 标题
                let title_size = scale.uniform(36.0);
                painter.text(
                    egui::pos2(scale.actual_w / 2.0, scale.y(40.0)),
                    egui::Align2::CENTER_TOP,
                    &state.definition.title,
                    egui::FontId::proportional(title_size),
                    egui::Color32::WHITE,
                );

                // 检测鼠标位置（用于掩码命中检测）
                let pointer_pos = ctx.input(|i| i.pointer.hover_pos());
                let mut mask_hovered: Option<usize> = None;

                if let (Some(pos), Some(mask)) = (pointer_pos, &state.mask_data) {
                    let base_x = pos.x / scale.actual_w * mask.width as f32;
                    let base_y = pos.y / scale.actual_h * mask.height as f32;
                    mask_hovered = Self::hit_test_mask(mask, base_x, base_y);
                    if let Some(idx) = mask_hovered
                        && !state.availability.get(idx).copied().unwrap_or(false)
                    {
                        mask_hovered = None;
                    }
                }

                let accent = egui::Color32::from_rgb(100, 149, 237);
                let idle = egui::Color32::from_gray(200);
                let hover_color = egui::Color32::WHITE;

                for (i, response, btn_rect) in &responses {
                    let loc = &state.definition.locations[*i];
                    let enabled = state.availability.get(*i).copied().unwrap_or(true);
                    let is_mask_hover = mask_hovered == Some(*i);
                    let is_hover = response.hovered() || is_mask_hover;

                    let (bg_color, text_color) = if !enabled {
                        (
                            egui::Color32::from_rgba_unmultiplied(60, 60, 60, 180),
                            egui::Color32::from_gray(100),
                        )
                    } else if is_hover {
                        (
                            egui::Color32::from_rgba_unmultiplied(80, 80, 120, 220),
                            hover_color,
                        )
                    } else {
                        (egui::Color32::from_rgba_unmultiplied(40, 40, 60, 200), idle)
                    };

                    painter.rect_filled(*btn_rect, 8.0, bg_color);
                    painter.rect_stroke(
                        *btn_rect,
                        8.0,
                        egui::Stroke::new(
                            1.5,
                            if enabled {
                                accent
                            } else {
                                egui::Color32::from_gray(80)
                            },
                        ),
                        egui::StrokeKind::Inside,
                    );

                    let text_size = scale.uniform(20.0);
                    painter.text(
                        btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &loc.label,
                        egui::FontId::proportional(text_size),
                        text_color,
                    );

                    let clicked = if enabled {
                        response.clicked()
                            || (is_mask_hover
                                && ctx.input(|i| {
                                    i.pointer.button_clicked(egui::PointerButton::Primary)
                                }))
                    } else {
                        false
                    };

                    if clicked {
                        result_status = UiModeStatus::Completed(VarValue::String(loc.id.clone()));
                    }
                }

                // Esc 取消
                if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    result_status = UiModeStatus::Cancelled;
                }
            });

        result_status
    }

    fn deactivate(&mut self) {
        self.state = None;
    }
}
