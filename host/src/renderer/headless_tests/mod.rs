//! 无头渲染测试：组合绘制链路（high_value）与几何/getter/枚举映射（low_value）分层。

use super::*;
use crate::rendering_types::NullTexture;
use crate::test_harness;

mod high_value;
mod low_value;

/// build_draw_commands 测试用：1280×720 的 renderer、默认 resource_manager 与 manifest。
fn build_draw_deps() -> (Renderer, ResourceManager, Manifest) {
    (
        Renderer::new(1280.0, 720.0),
        test_harness::null_resource_manager(),
        test_harness::default_manifest(),
    )
}
