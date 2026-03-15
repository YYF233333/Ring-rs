//! # Resources 模块
//!
//! 资源管理系统，负责图片资源的加载、缓存和管理。
//!
//! 所有资源访问通过 [`ResourceManager`] 进行，路径使用 [`LogicalPath`] 类型。

use crate::rendering_types::{Texture, TextureContext};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

mod cache;
mod error;
pub mod path;
pub(crate) mod source;

pub use cache::{CacheStats, DEFAULT_TEXTURE_BUDGET_MB, TextureCache};
pub use error::ResourceError;
pub use path::{
    LogicalPath, extract_base_dir, extract_script_id, normalize_logical_path, resolve_relative_path,
};
pub use source::ResourceSource;
pub(crate) use source::{FsSource, ZipSource};

fn load_texture_from_bytes(
    bytes: &[u8],
    label: &str,
    ctx: &TextureContext,
) -> Result<Arc<dyn Texture>, String> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| format!("Cannot decode image {}: {}", label, e))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(ctx.create_texture(width, height, &rgba, Some(label)))
}

/// 资源管理器
///
/// 负责加载和缓存所有游戏资源。
/// 所有资源访问使用 [`LogicalPath`]，内部通过 [`ResourceSource`] 读取。
pub struct ResourceManager {
    texture_cache: TextureCache,
    failed_textures: HashSet<String>,
    source: Arc<dyn ResourceSource>,
    texture_ctx: Option<TextureContext>,
}

impl ResourceManager {
    /// 创建新的资源管理器（使用文件系统来源）
    pub fn new(base_path: impl Into<String>, texture_cache_size_mb: usize) -> Self {
        let base = base_path.into();
        Self {
            texture_cache: TextureCache::new(texture_cache_size_mb),
            failed_textures: HashSet::new(),
            source: Arc::new(FsSource::new(&base)),
            texture_ctx: None,
        }
    }

    /// 创建使用自定义资源来源的资源管理器
    pub fn with_source(source: Arc<dyn ResourceSource>, texture_cache_size_mb: usize) -> Self {
        Self {
            texture_cache: TextureCache::new(texture_cache_size_mb),
            failed_textures: HashSet::new(),
            source,
            texture_ctx: None,
        }
    }

    /// 获取底层资源来源的引用
    pub fn source(&self) -> &dyn ResourceSource {
        &*self.source
    }

    /// 注入纹理上下文
    ///
    /// 在 WgpuBackend 创建后调用，使 load_texture 能够创建纹理。
    pub fn set_texture_context(&mut self, ctx: TextureContext) {
        self.texture_ctx = Some(ctx);
    }

    /// 加载图片资源（同步）
    ///
    /// 缓存键使用 [`LogicalPath`] 的规范化字符串，保证跨平台一致。
    pub fn load_texture(&mut self, path: &LogicalPath) -> Result<Arc<dyn Texture>, ResourceError> {
        let key = path.as_str();

        if let Some(texture) = self.texture_cache.get(key) {
            return Ok(texture);
        }

        if self.failed_textures.contains(key) {
            return Err(ResourceError::LoadFailed {
                path: key.to_string(),
                kind: "texture".to_string(),
                message: "Previously failed, skipping retry".to_string(),
            });
        }

        let bytes = match self.source.read(path) {
            Ok(bytes) => bytes,
            Err(err) => {
                self.failed_textures.insert(key.to_string());
                return Err(err);
            }
        };

        let ctx = self
            .texture_ctx
            .as_ref()
            .ok_or_else(|| ResourceError::LoadFailed {
                path: key.to_string(),
                kind: "texture".to_string(),
                message: "Texture context not set. Call set_texture_context() after backend init."
                    .to_string(),
            })?;

        let texture = load_texture_from_bytes(&bytes, key, ctx).map_err(|e| {
            self.failed_textures.insert(key.to_string());
            ResourceError::LoadFailed {
                path: key.to_string(),
                kind: "texture".to_string(),
                message: e,
            }
        })?;

        self.failed_textures.remove(key);
        self.texture_cache.insert(key.to_string(), texture.clone());

        Ok(texture)
    }

    /// 只读获取已缓存的图片资源（不加载）
    pub fn peek_texture(&self, path: &LogicalPath) -> Option<Arc<dyn Texture>> {
        self.texture_cache.get(path.as_str())
    }

    /// 检查图片资源是否已加载
    pub fn has_texture(&self, path: &LogicalPath) -> bool {
        self.texture_cache.contains(path.as_str())
    }

    /// 检查纹理是否已被标记为加载失败
    pub fn has_failed_texture(&self, path: &LogicalPath) -> bool {
        self.failed_textures.contains(path.as_str())
    }

    /// 预加载多个图片资源
    pub fn preload_textures(&mut self, paths: &[&LogicalPath]) -> Result<(), ResourceError> {
        for path in paths {
            self.load_texture(path)?;
        }
        Ok(())
    }

    /// 释放指定的图片资源
    pub fn unload_texture(&mut self, path: &LogicalPath) {
        let key = path.as_str();
        self.texture_cache.remove(key);
        self.failed_textures.remove(key);
    }

    /// 释放所有资源
    pub fn clear(&mut self) {
        self.texture_cache.clear();
        self.failed_textures.clear();
    }

    pub fn texture_count(&self) -> usize {
        self.texture_cache.len()
    }

    #[cfg(test)]
    pub fn texture_cache_mut(&mut self) -> &mut TextureCache {
        &mut self.texture_cache
    }

    pub fn texture_cache_stats(&self) -> CacheStats {
        self.texture_cache.stats()
    }

    /// 读取文本资源
    pub fn read_text(&self, path: &LogicalPath) -> Result<String, ResourceError> {
        let bytes = self.source.read(path)?;
        String::from_utf8(bytes).map_err(|e| ResourceError::LoadFailed {
            path: path.to_string(),
            kind: "text".to_string(),
            message: format!("Cannot convert bytes to UTF-8: {}", e),
        })
    }

    /// 读取文本资源，不存在时返回 None（不报错）。
    pub fn read_text_optional(&self, path: &LogicalPath) -> Option<String> {
        if !self.source.exists(path) {
            return None;
        }
        self.read_text(path).ok()
    }

    /// 检查资源是否存在
    pub fn resource_exists(&self, path: &LogicalPath) -> bool {
        self.source.exists(path)
    }

    /// 读取原始字节资源
    pub fn read_bytes(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        self.source.read(path)
    }

    /// 列出指定目录下的所有文件
    pub fn list_files(&self, dir_path: &LogicalPath) -> Vec<LogicalPath> {
        self.source.list_files(dir_path)
    }

    /// 将逻辑路径物化为真实文件系统路径。
    ///
    /// - FS 来源：直接返回 backing path，无需清理。
    /// - ZIP 来源：提取到 `temp_dir` 下的临时文件，调用方负责清理。
    ///
    /// 仅用于必须要真实文件路径的子系统（FFmpeg 等）。
    pub fn materialize_to_fs(
        &self,
        path: &LogicalPath,
        temp_dir: &Path,
    ) -> Result<(PathBuf, Option<PathBuf>), ResourceError> {
        if let Some(fs_path) = self.source.backing_path(path) {
            return Ok((fs_path, None));
        }

        let bytes = self.source.read(path)?;

        std::fs::create_dir_all(temp_dir).map_err(|e| ResourceError::LoadFailed {
            path: path.to_string(),
            kind: "materialize".to_string(),
            message: format!("Cannot create temp dir: {}", e),
        })?;

        let filename = path.as_str().rsplit('/').next().unwrap_or("resource");
        let temp_path = temp_dir.join(filename);

        std::fs::write(&temp_path, &bytes).map_err(|e| ResourceError::LoadFailed {
            path: path.to_string(),
            kind: "materialize".to_string(),
            message: format!("Cannot write temp file: {}", e),
        })?;

        Ok((temp_path.clone(), Some(temp_path)))
    }
}

#[cfg(test)]
mod tests;
