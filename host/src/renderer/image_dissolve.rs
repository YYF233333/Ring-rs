//! # ImageDissolve 模块
//!
//! 保留 ramp 参数管理；实际渲染已迁移到 wgpu DissolveRenderer。

/// ImageDissolve 效果参数（渲染由 DissolveRenderer 处理）
pub struct ImageDissolve {
    /// 渐变带宽（默认 0.0，即硬边）
    ramp: f32,
}

impl ImageDissolve {
    pub fn new() -> Self {
        Self { ramp: 0.0 }
    }

    /// 设置渐变带宽
    pub fn set_ramp(&mut self, ramp: f32) {
        self.ramp = ramp.clamp(0.0, 1.0);
    }

    /// 获取当前渐变带宽
    pub fn ramp(&self) -> f32 {
        self.ramp
    }
}

impl Default for ImageDissolve {
    fn default() -> Self {
        Self::new()
    }
}
