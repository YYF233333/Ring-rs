//! GPU 纹理类型
//!
//! 封装 wgpu 纹理 + 视图 + 绑定组，替代 macroquad 的 `Texture2D`。

use std::sync::Arc;

/// GPU 纹理
///
/// 包含 wgpu 纹理及其关联的视图和绑定组。
/// 通过 `Arc<GpuTexture>` 在缓存和渲染系统间共享。
pub struct GpuTexture {
    #[allow(dead_code)]
    pub(crate) texture: wgpu::Texture,
    #[allow(dead_code)]
    pub(crate) view: wgpu::TextureView,
    pub(crate) bind_group: wgpu::BindGroup,
    width: u32,
    height: u32,
}

impl std::fmt::Debug for GpuTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuTexture")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish_non_exhaustive()
    }
}

impl GpuTexture {
    /// 纹理宽度（像素）
    pub fn width(&self) -> f32 {
        self.width as f32
    }

    /// 纹理高度（像素）
    pub fn height(&self) -> f32 {
        self.height as f32
    }

    /// 纹理宽度（整数）
    pub fn width_u32(&self) -> u32 {
        self.width
    }

    /// 纹理高度（整数）
    pub fn height_u32(&self) -> u32 {
        self.height
    }

    /// 估算显存占用（字节）：width * height * 4 (RGBA8)
    pub fn size_bytes(&self) -> usize {
        (self.width as usize) * (self.height as usize) * 4
    }
}

/// 从 RGBA 字节数据创建 GPU 纹理
#[allow(clippy::too_many_arguments)]
pub(crate) fn create_gpu_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bind_group_layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    width: u32,
    height: u32,
    rgba_data: &[u8],
    label: Option<&str>,
) -> Arc<GpuTexture> {
    use wgpu::util::DeviceExt;

    let texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label,
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        rgba_data,
    );
    let view = texture.create_view(&Default::default());

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });

    Arc::new(GpuTexture {
        texture,
        view,
        bind_group,
        width,
        height,
    })
}
