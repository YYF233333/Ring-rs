//! # 渲染抽象类型
//!
//! 定义 [`Texture`] 和 [`TextureFactory`] trait，将 `renderer/` 和 `resources/`
//! 与具体后端（wgpu）解耦。
//!
//! 同时提供 [`NullTexture`] / [`NullTextureFactory`] 用于 headless 测试。

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

/// 纹理抽象接口
///
/// 由各后端实现（[`crate::backend::GpuTexture`] for wgpu,
/// [`NullTexture`] for headless）。
/// 通过 `Arc<dyn Texture>` 在 renderer / resources 间共享。
pub trait Texture: Send + Sync + Debug + 'static {
    /// 纹理宽度（像素，浮点）
    fn width(&self) -> f32;
    /// 纹理高度（像素，浮点）
    fn height(&self) -> f32;
    /// 纹理宽度（像素，整数）
    fn width_u32(&self) -> u32;
    /// 纹理高度（像素，整数）
    fn height_u32(&self) -> u32;
    /// 估算显存 / 内存占用（字节）
    fn size_bytes(&self) -> usize;
    /// 向下转型到具体类型，供 backend 内部使用
    fn as_any(&self) -> &dyn Any;
}

/// 纹理创建工厂接口
///
/// 由 backend 实现，注入到 [`crate::resources::ResourceManager`]。
/// 将纹理创建从 wgpu 具体类型解耦。
pub trait TextureFactory: Send + Sync {
    /// 从 RGBA 字节数据创建纹理
    fn create_texture(
        &self,
        width: u32,
        height: u32,
        rgba_data: &[u8],
        label: Option<&str>,
    ) -> Arc<dyn Texture>;
}

/// 纹理上下文
///
/// 持有 [`TextureFactory`]，注入到 [`crate::resources::ResourceManager`]。
/// 替代原来的 `GpuResourceContext`（内含 wgpu 具体类型）。
pub struct TextureContext {
    factory: Arc<dyn TextureFactory>,
}

impl TextureContext {
    /// 使用给定工厂创建上下文
    pub fn new(factory: Arc<dyn TextureFactory>) -> Self {
        Self { factory }
    }

    /// 从 RGBA 字节数据创建纹理
    pub fn create_texture(
        &self,
        width: u32,
        height: u32,
        rgba_data: &[u8],
        label: Option<&str>,
    ) -> Arc<dyn Texture> {
        self.factory.create_texture(width, height, rgba_data, label)
    }
}

// ---------------------------------------------------------------------------
// DrawCommand
// ---------------------------------------------------------------------------

/// 绘制命令
///
/// 由 [`crate::renderer::Renderer`] 生成，由 backend 消费。
/// 使用 `Arc<dyn Texture>` 引用纹理，与具体后端解耦。
pub enum DrawCommand {
    /// 绘制纹理 sprite
    Sprite {
        texture: Arc<dyn Texture>,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    },
    /// 绘制纯色矩形
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    },
    /// 遮罩溶解叠加（由 DissolveRenderer 处理，SpriteRenderer 跳过）
    Dissolve {
        mask_texture: Arc<dyn Texture>,
        progress: f32,
        ramp: f32,
        reversed: bool,
        overlay_color: [f32; 4],
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
}

impl DrawCommand {
    /// 提取 sprite/rect 的绘制参数 (x, y, w, h, color, has_uv)
    pub(crate) fn sprite_params(&self) -> Option<(f32, f32, f32, f32, [f32; 4], bool)> {
        match self {
            DrawCommand::Sprite {
                x,
                y,
                width,
                height,
                color,
                ..
            } => Some((*x, *y, *width, *height, *color, true)),
            DrawCommand::Rect {
                x,
                y,
                width,
                height,
                color,
            } => Some((*x, *y, *width, *height, *color, false)),
            DrawCommand::Dissolve { .. } => None,
        }
    }
}

// ---------------------------------------------------------------------------
// NullBackend（headless）
// ---------------------------------------------------------------------------

/// Headless 纹理（仅存储尺寸，无 GPU 资源）
#[derive(Debug, Clone)]
pub struct NullTexture {
    width: u32,
    height: u32,
}

impl NullTexture {
    /// 创建指定尺寸的空纹理
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl Texture for NullTexture {
    fn width(&self) -> f32 {
        self.width as f32
    }
    fn height(&self) -> f32 {
        self.height as f32
    }
    fn width_u32(&self) -> u32 {
        self.width
    }
    fn height_u32(&self) -> u32 {
        self.height
    }
    fn size_bytes(&self) -> usize {
        (self.width as usize) * (self.height as usize) * 4
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Headless 纹理工厂（创建 [`NullTexture`]，无 GPU 操作）
#[derive(Debug, Clone)]
pub struct NullTextureFactory;

impl TextureFactory for NullTextureFactory {
    fn create_texture(
        &self,
        width: u32,
        height: u32,
        _rgba_data: &[u8],
        _label: Option<&str>,
    ) -> Arc<dyn Texture> {
        Arc::new(NullTexture::new(width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_texture_dimensions() {
        let tex = NullTexture::new(1920, 1080);
        assert_eq!(tex.width(), 1920.0);
        assert_eq!(tex.height(), 1080.0);
        assert_eq!(tex.width_u32(), 1920);
        assert_eq!(tex.height_u32(), 1080);
        assert_eq!(tex.size_bytes(), 1920 * 1080 * 4);
    }

    #[test]
    fn null_texture_downcast() {
        let tex: Arc<dyn Texture> = Arc::new(NullTexture::new(64, 64));
        let concrete = tex.as_any().downcast_ref::<NullTexture>();
        assert!(concrete.is_some());
        assert_eq!(concrete.unwrap().width_u32(), 64);
    }

    #[test]
    fn null_factory_creates_correct_size() {
        let factory = NullTextureFactory;
        let tex = factory.create_texture(800, 600, &[], None);
        assert_eq!(tex.width_u32(), 800);
        assert_eq!(tex.height_u32(), 600);
    }
}
