//! 无头渲染测试：组合绘制链路（high_value）与几何/getter/枚举映射（low_value）分层。

use super::*;
use crate::rendering_types::{NullTexture, NullTextureFactory, TextureContext};
use crate::resources::ResourceManager;
use std::sync::Arc;

mod high_value;
mod low_value;

fn make_test_resource_manager() -> ResourceManager {
    let mut manager = ResourceManager::new("assets", 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));
    manager
}

/// build_draw_commands 测试用：1280×720 的 renderer、默认 resource_manager 与 manifest。
fn build_draw_deps() -> (Renderer, ResourceManager, Manifest) {
    (
        Renderer::new(1280.0, 720.0),
        make_test_resource_manager(),
        Manifest::with_defaults(),
    )
}
