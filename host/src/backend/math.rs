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

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_proj(m: &[f32; 16], x: f32, y: f32) -> (f32, f32) {
        let out_x = m[0] * x + m[4] * y + m[12];
        let out_y = m[1] * x + m[5] * y + m[13];
        (out_x, out_y)
    }

    #[test]
    fn ortho_top_left_maps_to_neg1_pos1() {
        let m = orthographic_projection(800.0, 600.0);
        let (x, y) = apply_proj(&m, 0.0, 0.0);
        assert!((x - (-1.0)).abs() < 1e-6);
        assert!((y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn ortho_bottom_right_maps_to_pos1_neg1() {
        let m = orthographic_projection(800.0, 600.0);
        let (x, y) = apply_proj(&m, 800.0, 600.0);
        assert!((x - 1.0).abs() < 1e-6);
        assert!((y - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn ortho_center_maps_to_origin() {
        let m = orthographic_projection(1920.0, 1080.0);
        let (x, y) = apply_proj(&m, 960.0, 540.0);
        assert!(x.abs() < 1e-6);
        assert!(y.abs() < 1e-6);
    }

    #[test]
    fn quad_vertices_produces_six_vertices() {
        let verts = quad_vertices(10.0, 20.0, 100.0, 50.0, true, [1.0; 4]);
        assert_eq!(verts.len(), 6);
    }

    #[test]
    fn quad_vertices_triangle_winding() {
        let v = quad_vertices(0.0, 0.0, 100.0, 50.0, true, [1.0; 4]);
        // Triangle 1: TL(0,0) - TR(100,0) - BR(100,50)
        assert_eq!(v[0].pos, [0.0, 0.0]);
        assert_eq!(v[1].pos, [100.0, 0.0]);
        assert_eq!(v[2].pos, [100.0, 50.0]);
        // Triangle 2: TL(0,0) - BR(100,50) - BL(0,50)
        assert_eq!(v[3].pos, [0.0, 0.0]);
        assert_eq!(v[4].pos, [100.0, 50.0]);
        assert_eq!(v[5].pos, [0.0, 50.0]);
    }

    #[test]
    fn quad_vertices_uv_enabled() {
        let v = quad_vertices(0.0, 0.0, 1.0, 1.0, true, [1.0; 4]);
        assert_eq!(v[0].uv, [0.0, 0.0]); // TL
        assert_eq!(v[1].uv, [1.0, 0.0]); // TR
        assert_eq!(v[2].uv, [1.0, 1.0]); // BR
        assert_eq!(v[5].uv, [0.0, 1.0]); // BL
    }

    #[test]
    fn quad_vertices_uv_disabled() {
        let v = quad_vertices(0.0, 0.0, 1.0, 1.0, false, [1.0; 4]);
        for vert in &v {
            assert_eq!(vert.uv, [0.0, 0.0]);
        }
    }

    #[test]
    fn quad_vertices_color_passthrough() {
        let color = [0.5, 0.6, 0.7, 0.8];
        let v = quad_vertices(0.0, 0.0, 1.0, 1.0, true, color);
        for vert in &v {
            assert_eq!(vert.color, color);
        }
    }

    #[test]
    fn quad_vertices_offset_position() {
        let v = quad_vertices(50.0, 30.0, 200.0, 100.0, false, [1.0; 4]);
        assert_eq!(v[0].pos, [50.0, 30.0]); // TL
        assert_eq!(v[1].pos, [250.0, 30.0]); // TR
        assert_eq!(v[2].pos, [250.0, 130.0]); // BR
        assert_eq!(v[5].pos, [50.0, 130.0]); // BL
    }
}
