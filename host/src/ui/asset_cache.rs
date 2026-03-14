//! # UI 素材缓存
//!
//! 将 GUI 图片素材从 [`ResourceSource`] 加载为 [`egui::TextureHandle`]，
//! 供 egui 渲染直接使用。

use std::collections::HashMap;

use crate::resources::{LogicalPath, ResourceSource};

use super::layout::UiAssetPaths;

/// 缓存 egui 纹理句柄的容器。
///
/// 生命周期说明：`TextureHandle` 内部是 `Arc` 引用计数，
/// 只要 `UiAssetCache` 持有引用，纹理就不会被 egui 释放。
pub struct UiAssetCache {
    textures: HashMap<String, egui::TextureHandle>,
}

impl UiAssetCache {
    /// 从 `UiAssetPaths` 定义的路径中加载所有 UI 素材。
    ///
    /// 加载失败的素材会记录警告并跳过（不 panic），
    /// 渲染时通过 `get()` 返回 `None` 降级为纯色/无背景。
    pub fn load(paths: &UiAssetPaths, source: &dyn ResourceSource, ctx: &egui::Context) -> Self {
        let mut textures = HashMap::new();

        for (key, path_str) in paths.all_entries() {
            if path_str.is_empty() {
                continue;
            }
            let logical = LogicalPath::new(path_str);
            match source.read(&logical) {
                Ok(bytes) => match load_image_to_texture(ctx, key, &bytes) {
                    Some(handle) => {
                        textures.insert(key.to_string(), handle);
                    }
                    None => {
                        tracing::warn!(key, path = path_str, "Failed to decode UI asset image");
                    }
                },
                Err(e) => {
                    tracing::warn!(key, path = path_str, error = %e, "Failed to read UI asset");
                }
            }
        }

        tracing::info!(count = textures.len(), "UI asset cache loaded");
        Self { textures }
    }

    /// 获取指定 key 的纹理句柄
    pub fn get(&self, key: &str) -> Option<&egui::TextureHandle> {
        self.textures.get(key)
    }

    /// 构造指定大小的 egui Image widget
    pub fn image(&self, key: &str, size: egui::Vec2) -> Option<egui::Image<'_>> {
        self.get(key)
            .map(|handle| egui::Image::new(handle).fit_to_exact_size(size))
    }

    /// 获取纹理的原始像素尺寸
    pub fn texture_size(&self, key: &str) -> Option<[usize; 2]> {
        self.get(key).map(|h| h.size())
    }
}

fn load_image_to_texture(
    ctx: &egui::Context,
    name: &str,
    bytes: &[u8],
) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let pixels = rgba.into_raw();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    Some(ctx.load_texture(
        format!("ui_{name}"),
        color_image,
        egui::TextureOptions::LINEAR,
    ))
}
