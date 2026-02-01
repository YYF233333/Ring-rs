//! # Resources 模块
//!
//! 资源管理系统，负责图片和音频资源的加载、缓存和管理。

use macroquad::prelude::*;
use macroquad::audio::{Sound, load_sound};
use std::collections::HashMap;

mod error;

pub use error::ResourceError;

/// 使用 image crate 加载图片并转换为 Texture2D
/// 支持 JPEG、PNG 等格式
async fn load_texture_with_image_crate(path: &str) -> Result<Texture2D, String> {
    // 读取文件
    let bytes = std::fs::read(path)
        .map_err(|e| format!("无法读取文件 {}: {}", path, e))?;
    
    // 使用 image crate 解码
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("无法解码图片 {}: {}", path, e))?;
    
    // 转换为 RGBA8
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    
    // 创建 macroquad Texture2D
    let texture = Texture2D::from_rgba8(width as u16, height as u16, &rgba);
    
    Ok(texture)
}

/// 资源管理器
///
/// 负责加载和缓存所有游戏资源（图片、音频等）。
#[derive(Debug)]
pub struct ResourceManager {
    /// 图片资源缓存（路径 -> Texture2D）
    textures: HashMap<String, Texture2D>,
    /// 音频资源缓存（路径 -> Sound）
    sounds: HashMap<String, Sound>,
    /// 资源基础路径
    base_path: String,
}

impl ResourceManager {
    /// 创建新的资源管理器
    ///
    /// # 参数
    ///
    /// - `base_path`: 资源文件的基础路径（如 "assets"）
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            textures: HashMap::new(),
            sounds: HashMap::new(),
            base_path: base_path.into(),
        }
    }

    /// 解析资源路径（将相对路径转换为完整路径）
    pub(crate) fn resolve_path(&self, path: &str) -> String {
        // 如果路径已经是绝对路径或包含 "assets"，直接返回
        if path.starts_with('/') || path.contains("assets") {
            return path.to_string();
        }

        // 否则拼接基础路径
        format!("{}/{}", self.base_path, path)
    }

    /// 加载图片资源
    ///
    /// 如果资源已缓存，直接返回缓存的资源。
    /// 否则从文件系统加载并缓存。
    ///
    /// 支持 PNG、JPEG 等格式。
    ///
    /// # 参数
    ///
    /// - `path`: 图片文件路径（相对于 base_path 或绝对路径）
    ///
    /// # 返回
    ///
    /// 加载的 Texture2D，或加载错误
    pub async fn load_texture(&mut self, path: &str) -> Result<Texture2D, ResourceError> {
        // 检查缓存
        if let Some(texture) = self.textures.get(path) {
            return Ok(texture.clone());
        }

        // 解析路径
        let full_path = self.resolve_path(path);

        // 检查文件扩展名，决定使用哪种加载方式
        let lower_path = full_path.to_lowercase();
        let texture = if lower_path.ends_with(".jpg") || lower_path.ends_with(".jpeg") {
            // JPEG 文件使用 image crate 加载
            load_texture_with_image_crate(&full_path)
                .await
                .map_err(|e| ResourceError::LoadFailed {
                    path: full_path.clone(),
                    kind: "texture".to_string(),
                    message: e,
                })?
        } else {
            // 其他格式（PNG 等）使用 macroquad 原生加载
            load_texture(&full_path)
                .await
                .map_err(|e| ResourceError::LoadFailed {
                    path: full_path.clone(),
                    kind: "texture".to_string(),
                    message: e.to_string(),
                })?
        };

        // 设置纹理过滤模式（平滑缩放）
        texture.set_filter(FilterMode::Linear);

        // 缓存资源
        self.textures.insert(path.to_string(), texture.clone());

        // 返回缓存的资源
        Ok(texture)
    }

    /// 加载音频资源
    ///
    /// 如果资源已缓存，直接返回缓存的资源。
    /// 否则从文件系统加载并缓存。
    ///
    /// # 参数
    ///
    /// - `path`: 音频文件路径（相对于 base_path 或绝对路径）
    ///
    /// # 返回
    ///
    /// 加载的 Sound，或加载错误
    pub async fn load_sound(&mut self, path: &str) -> Result<Sound, ResourceError> {
        // 检查缓存
        if let Some(sound) = self.sounds.get(path) {
            return Ok(sound.clone());
        }

        // 解析路径
        let full_path = self.resolve_path(path);

        // 加载音频
        let sound = load_sound(&full_path)
            .await
            .map_err(|e: macroquad::Error| ResourceError::LoadFailed {
                path: full_path.clone(),
                kind: "sound".to_string(),
                message: e.to_string(),
            })?;

        // 缓存资源
        self.sounds.insert(path.to_string(), sound.clone());

        // 返回缓存的资源
        Ok(sound)
    }

    /// 获取已缓存的图片资源（不加载）
    ///
    /// 如果资源未缓存，返回 None。
    pub fn get_texture(&self, path: &str) -> Option<Texture2D> {
        self.textures.get(path).cloned()
    }

    /// 获取已缓存的音频资源（不加载）
    ///
    /// 如果资源未缓存，返回 None。
    pub fn get_sound(&self, path: &str) -> Option<Sound> {
        self.sounds.get(path).cloned()
    }

    /// 检查图片资源是否已加载
    pub fn has_texture(&self, path: &str) -> bool {
        self.textures.contains_key(path)
    }

    /// 检查音频资源是否已加载
    pub fn has_sound(&self, path: &str) -> bool {
        self.sounds.contains_key(path)
    }

    /// 预加载多个图片资源
    ///
    /// # 参数
    ///
    /// - `paths`: 图片路径列表
    pub async fn preload_textures(&mut self, paths: &[&str]) -> Result<(), ResourceError> {
        for path in paths {
            self.load_texture(path).await?;
        }
        Ok(())
    }

    /// 预加载多个音频资源
    ///
    /// # 参数
    ///
    /// - `paths`: 音频路径列表
    pub async fn preload_sounds(&mut self, paths: &[&str]) -> Result<(), ResourceError> {
        for path in paths {
            self.load_sound(path).await?;
        }
        Ok(())
    }

    /// 释放指定的图片资源
    pub fn unload_texture(&mut self, path: &str) {
        self.textures.remove(path);
    }

    /// 释放指定的音频资源
    pub fn unload_sound(&mut self, path: &str) {
        self.sounds.remove(path);
    }

    /// 释放所有资源
    pub fn clear(&mut self) {
        self.textures.clear();
        self.sounds.clear();
    }

    /// 获取已加载的图片资源数量
    pub fn texture_count(&self) -> usize {
        self.textures.len()
    }

    /// 获取已加载的音频资源数量
    pub fn sound_count(&self) -> usize {
        self.sounds.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_manager_creation() {
        let manager = ResourceManager::new("assets");
        assert_eq!(manager.texture_count(), 0);
        assert_eq!(manager.sound_count(), 0);
    }

    #[test]
    fn test_resolve_path() {
        let manager = ResourceManager::new("assets");
        
        // 相对路径
        let path = manager.resolve_path("bg.png");
        assert_eq!(path, "assets/bg.png");
        
        // 绝对路径（包含 assets）
        let path = manager.resolve_path("assets/bg.png");
        assert_eq!(path, "assets/bg.png");
    }
}
