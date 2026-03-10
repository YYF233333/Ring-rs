//! # Resources 模块
//!
//! 资源管理系统，负责图片资源的加载、缓存和管理。

use crate::backend::GpuTexture;
use crate::backend::sprite_renderer::SpriteRenderer;
use std::collections::HashSet;
use std::sync::Arc;

mod cache;
mod error;
pub mod path;
mod source;

pub use cache::{CacheStats, DEFAULT_TEXTURE_BUDGET_MB, TextureCache};
pub use error::ResourceError;
pub use path::{
    extract_base_dir, extract_script_id, normalize_logical_path, resolve_relative_path,
};
pub use source::{FsSource, ResourceSource, ZipSource};

/// 从字节数据加载图片并转换为 GPU 纹理
fn load_texture_from_bytes(
    bytes: &[u8],
    path: &str,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    sprite_renderer: &SpriteRenderer,
) -> Result<Arc<GpuTexture>, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("Cannot decode image {}: {}", path, e))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(sprite_renderer.create_texture(device, queue, width, height, &rgba, Some(path)))
}

/// GPU 资源上下文
///
/// 持有 GPU 设备引用，供 ResourceManager 创建纹理时使用。
/// 使用 Arc 在 ResourceManager 和 WgpuBackend 之间共享。
pub struct GpuResourceContext {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,
    pub(crate) sprite_renderer: Arc<SpriteRenderer>,
}

/// 资源管理器
///
/// 负责加载和缓存所有游戏资源（图片等）。
/// 纹理使用 LRU 缓存 + 显存预算管理。
pub struct ResourceManager {
    /// 纹理缓存（带 LRU 驱逐）
    texture_cache: TextureCache,
    /// 纹理加载失败缓存（规范化路径）
    failed_textures: HashSet<String>,
    /// 资源基础路径
    base_path: String,
    /// 资源来源（文件系统/ZIP 等）
    source: Arc<dyn ResourceSource>,
    /// GPU 资源上下文（延迟注入，在 WgpuBackend 创建后设置）
    gpu: Option<GpuResourceContext>,
}

impl ResourceManager {
    /// 创建新的资源管理器（使用文件系统来源）
    pub fn new(base_path: impl Into<String>, texture_cache_size_mb: usize) -> Self {
        let base = base_path.into();
        Self {
            texture_cache: TextureCache::new(texture_cache_size_mb),
            failed_textures: HashSet::new(),
            source: Arc::new(FsSource::new(&base)),
            base_path: base,
            gpu: None,
        }
    }

    /// 创建使用自定义资源来源的资源管理器
    pub fn with_source(
        base_path: impl Into<String>,
        source: Arc<dyn ResourceSource>,
        texture_cache_size_mb: usize,
    ) -> Self {
        Self {
            texture_cache: TextureCache::new(texture_cache_size_mb),
            failed_textures: HashSet::new(),
            base_path: base_path.into(),
            source,
            gpu: None,
        }
    }

    /// 创建指定缓存大小的资源管理器
    pub fn with_budget(base_path: impl Into<String>, texture_cache_size_mb: usize) -> Self {
        let base = base_path.into();
        Self {
            texture_cache: TextureCache::new(texture_cache_size_mb),
            failed_textures: HashSet::new(),
            source: Arc::new(FsSource::new(&base)),
            base_path: base,
            gpu: None,
        }
    }

    /// 注入 GPU 资源上下文
    ///
    /// 在 WgpuBackend 创建后调用，使 load_texture 能够创建 GPU 纹理。
    pub fn set_gpu_context(&mut self, gpu: GpuResourceContext) {
        self.gpu = Some(gpu);
    }

    /// 解析资源路径（将相对路径转换为完整路径）
    pub fn resolve_path(&self, path: &str) -> String {
        use std::path::{Path, PathBuf};

        let path_obj = Path::new(path);

        if path_obj.is_absolute() {
            return Self::normalize_path_components(path_obj);
        }

        if path.contains(&self.base_path) {
            return Self::normalize_path_components(path_obj);
        }

        let full_path: PathBuf = [&self.base_path, path].iter().collect();
        Self::normalize_path_components(&full_path)
    }

    fn normalize_path_components(path: &std::path::Path) -> String {
        use std::path::Component;

        let mut components: Vec<String> = Vec::new();

        for component in path.components() {
            match component {
                Component::Prefix(p) => {
                    components.push(p.as_os_str().to_string_lossy().to_string());
                }
                Component::RootDir => {
                    if components.is_empty() {
                        components.push(String::new());
                    }
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    if components.len() > 1
                        || (components.len() == 1 && !components[0].contains(':'))
                    {
                        components.pop();
                    }
                }
                Component::Normal(name) => {
                    components.push(name.to_string_lossy().to_string());
                }
            }
        }

        if components.is_empty() {
            return String::new();
        }

        if components.len() >= 2 && components[0].contains(':') {
            let drive = &components[0];
            let rest = components[1..].join("/");
            format!("{}/{}", drive, rest)
        } else {
            components.join("/")
        }
    }

    /// 加载图片资源（同步）
    ///
    /// 如果资源已缓存，直接返回缓存的资源。
    /// 否则通过 ResourceSource 加载并缓存。
    pub fn load_texture(&mut self, path: &str) -> Result<Arc<GpuTexture>, ResourceError> {
        let full_path = self.resolve_path(path);

        if let Some(texture) = self.texture_cache.get(&full_path) {
            return Ok(texture);
        }

        if self.failed_textures.contains(&full_path) {
            return Err(ResourceError::LoadFailed {
                path: full_path,
                kind: "texture".to_string(),
                message: "Previously failed, skipping retry".to_string(),
            });
        }

        let bytes = match self.source.read(path) {
            Ok(bytes) => bytes,
            Err(err) => {
                self.failed_textures.insert(full_path.clone());
                return Err(err);
            }
        };

        let gpu = self.gpu.as_ref().ok_or_else(|| ResourceError::LoadFailed {
            path: full_path.clone(),
            kind: "texture".to_string(),
            message: "GPU context not set. Call set_gpu_context() after WgpuBackend init."
                .to_string(),
        })?;

        let texture = load_texture_from_bytes(
            &bytes,
            &full_path,
            &gpu.device,
            &gpu.queue,
            &gpu.sprite_renderer,
        )
        .map_err(|e| {
            self.failed_textures.insert(full_path.clone());
            ResourceError::LoadFailed {
                path: full_path.clone(),
                kind: "texture".to_string(),
                message: e,
            }
        })?;

        self.failed_textures.remove(&full_path);
        self.texture_cache.insert(full_path, texture.clone());

        Ok(texture)
    }

    /// 获取已缓存的图片资源（不加载），更新 LRU
    pub fn get_texture(&mut self, path: &str) -> Option<Arc<GpuTexture>> {
        let full_path = self.resolve_path(path);
        self.texture_cache.get(&full_path)
    }

    /// 只读获取已缓存的图片资源（不更新 LRU）
    pub fn peek_texture(&self, path: &str) -> Option<Arc<GpuTexture>> {
        let full_path = self.resolve_path(path);
        self.texture_cache.peek(&full_path)
    }

    /// 检查图片资源是否已加载
    pub fn has_texture(&self, path: &str) -> bool {
        let full_path = self.resolve_path(path);
        self.texture_cache.contains(&full_path)
    }

    /// 检查纹理是否已被标记为加载失败
    pub fn has_failed_texture(&self, path: &str) -> bool {
        let full_path = self.resolve_path(path);
        self.failed_textures.contains(&full_path)
    }

    /// 预加载多个图片资源
    pub fn preload_textures(&mut self, paths: &[&str]) -> Result<(), ResourceError> {
        for path in paths {
            self.load_texture(path)?;
        }
        Ok(())
    }

    /// 释放指定的图片资源
    pub fn unload_texture(&mut self, path: &str) {
        let full_path = self.resolve_path(path);
        self.texture_cache.remove(&full_path);
        self.failed_textures.remove(&full_path);
    }

    /// 释放所有资源
    pub fn clear(&mut self) {
        self.texture_cache.clear();
        self.failed_textures.clear();
    }

    pub fn texture_count(&self) -> usize {
        self.texture_cache.len()
    }

    pub fn texture_cache_stats(&self) -> CacheStats {
        self.texture_cache.stats()
    }

    pub fn pin_texture(&mut self, path: &str) {
        let full_path = self.resolve_path(path);
        self.texture_cache.pin(&full_path);
    }

    pub fn unpin_texture(&mut self, path: &str) {
        let full_path = self.resolve_path(path);
        self.texture_cache.unpin(&full_path);
    }

    pub fn unpin_all_textures(&mut self) {
        self.texture_cache.unpin_all();
    }

    /// 读取文本资源
    pub fn read_text(&self, path: &str) -> Result<String, ResourceError> {
        let bytes = self.source.read(path)?;
        String::from_utf8(bytes).map_err(|e| ResourceError::LoadFailed {
            path: path.to_string(),
            kind: "text".to_string(),
            message: format!("Cannot convert bytes to UTF-8: {}", e),
        })
    }

    /// 检查资源是否存在
    pub fn resource_exists(&self, path: &str) -> bool {
        self.source.exists(path)
    }

    /// 读取原始字节资源
    pub fn read_bytes(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        self.source.read(path)
    }

    /// 列出指定目录下的所有文件
    pub fn list_files(&self, dir_path: &str) -> Vec<String> {
        self.source.list_files(dir_path)
    }
}

#[cfg(test)]
mod tests;
