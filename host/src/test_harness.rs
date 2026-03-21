//! 共享测试基础设施
//!
//! 提供无 GPU / 无 IO 的测试辅助构造，消除各模块 `tests/mod.rs` 中的重复代码。

use crate::event_stream::EventStream;
use crate::manifest::Manifest;
use crate::rendering_types::{NullTexture, NullTextureFactory, Texture, TextureContext};
use crate::resources::ResourceManager;
use std::sync::Arc;

/// 构造注入了 `NullTextureFactory` 的 `ResourceManager`（无 GPU / 无 IO）。
pub(crate) fn null_resource_manager() -> ResourceManager {
    let mut manager = ResourceManager::new("assets", 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));
    manager
}

/// 构造指定尺寸的 `NullTexture`（用于缓存测试等需要占位纹理的场景）。
pub(crate) fn null_texture(w: u32, h: u32) -> Arc<dyn Texture> {
    Arc::new(NullTexture::new(w, h))
}

/// 构造 `NullTextureFactory` 的 `TextureContext`（用于自定义 source 的测试场景）。
pub(crate) fn null_texture_context() -> TextureContext {
    TextureContext::new(Arc::new(NullTextureFactory))
}

/// 默认 `Manifest`（与 `Manifest::with_defaults` 语义一致，提供统一发现入口）。
pub(crate) fn default_manifest() -> Manifest {
    Manifest::with_defaults()
}

/// 内存模式的 `EventStream`（测试专用），事件收集到内存中供断言。
#[allow(dead_code)]
pub(crate) fn test_event_stream() -> EventStream {
    EventStream::in_memory()
}
