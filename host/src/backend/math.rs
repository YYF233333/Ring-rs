//! 渲染管线公共数学工具
//!
//! 投影矩阵、顶点类型和 quad 生成，供 SpriteRenderer 和 DissolveRenderer 共享。

/// 通用 2D quad 顶点（pos + uv + color）
///
/// SpriteRenderer 使用 color 做 tint/alpha，DissolveRenderer 不使用 color 但
/// 保持相同的顶点布局以共享 pipeline layout 兼容性。
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl QuadVertex {
    pub const ATTRS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4];

    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &Self::ATTRS,
    };
}

/// 正交投影矩阵：像素坐标 -> NDC（左上角原点）
pub fn orthographic_projection(width: f32, height: f32) -> [f32; 16] {
    #[rustfmt::skip]
    let m = [
        2.0 / width,   0.0,            0.0,  0.0,
        0.0,          -2.0 / height,    0.0,  0.0,
        0.0,           0.0,            1.0,  0.0,
       -1.0,           1.0,            0.0,  1.0,
    ];
    m
}

/// 生成一个 quad 的 6 个顶点（两个三角形：TL-TR-BR, TL-BR-BL）
pub fn quad_vertices(x: f32, y: f32, w: f32, h: f32, uv: bool, color: [f32; 4]) -> [QuadVertex; 6] {
    let (u0, v0, u1, v1) = if uv {
        (0.0f32, 0.0, 1.0, 1.0)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let tl = QuadVertex {
        pos: [x, y],
        uv: [u0, v0],
        color,
    };
    let tr = QuadVertex {
        pos: [x + w, y],
        uv: [u1, v0],
        color,
    };
    let bl = QuadVertex {
        pos: [x, y + h],
        uv: [u0, v1],
        color,
    };
    let br = QuadVertex {
        pos: [x + w, y + h],
        uv: [u1, v1],
        color,
    };

    [tl, tr, br, tl, br, bl]
}
