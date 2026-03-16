//! # Transform 模块
//!
//! 变换状态，表示一个对象的位置、缩放、旋转和透明度。

/// 二维向量
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    /// 创建新的向量
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// 零向量
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// 单位向量 (1, 1)
    pub const fn one() -> Self {
        Self { x: 1.0, y: 1.0 }
    }

    /// 线性插值
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl From<Vec2> for (f32, f32) {
    fn from(v: Vec2) -> Self {
        (v.x, v.y)
    }
}

/// 变换状态
///
/// 表示一个可动画对象的完整变换状态。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// 位置偏移（相对于默认位置）
    pub position: Vec2,
    /// 缩放因子
    pub scale: Vec2,
    /// 旋转角度（弧度）
    pub rotation: f32,
    /// 透明度 (0.0 - 1.0)
    pub alpha: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::zero(),
            scale: Vec2::one(),
            rotation: 0.0,
            alpha: 1.0,
        }
    }
}

impl Transform {
    /// 创建默认变换（无偏移、无缩放、无旋转、完全不透明）
    pub fn identity() -> Self {
        Self::default()
    }

    /// 创建只有透明度的变换
    pub fn with_alpha(alpha: f32) -> Self {
        Self {
            alpha,
            ..Self::default()
        }
    }

    /// 创建只有位置偏移的变换
    pub fn with_position(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            ..Self::default()
        }
    }

    /// 创建只有缩放的变换
    pub fn with_scale(x: f32, y: f32) -> Self {
        Self {
            scale: Vec2::new(x, y),
            ..Self::default()
        }
    }

    /// 创建均匀缩放的变换
    pub fn with_uniform_scale(s: f32) -> Self {
        Self::with_scale(s, s)
    }

    /// 线性插值到另一个变换
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            position: self.position.lerp(other.position, t),
            scale: self.scale.lerp(other.scale, t),
            rotation: self.rotation + (other.rotation - self.rotation) * t,
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }

    /// 设置透明度
    pub fn set_alpha(&mut self, alpha: f32) {
        self.alpha = alpha.clamp(0.0, 1.0);
    }

    /// 设置位置
    pub fn set_position(&mut self, x: f32, y: f32) {
        self.position = Vec2::new(x, y);
    }

    /// 设置缩放
    pub fn set_scale(&mut self, x: f32, y: f32) {
        self.scale = Vec2::new(x, y);
    }

    /// 设置旋转
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_default() {
        let t = Transform::default();
        assert_eq!(t.position, Vec2::zero());
        assert_eq!(t.scale, Vec2::one());
        assert_eq!(t.rotation, 0.0);
        assert_eq!(t.alpha, 1.0);
    }

    #[test]
    fn test_transform_lerp() {
        let t1 = Transform::with_alpha(0.0);
        let t2 = Transform::with_alpha(1.0);
        let mid = t1.lerp(&t2, 0.5);
        assert!((mid.alpha - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_vec2_lerp() {
        let v1 = Vec2::new(0.0, 0.0);
        let v2 = Vec2::new(10.0, 20.0);
        let mid = v1.lerp(v2, 0.5);
        assert_eq!(mid.x, 5.0);
        assert_eq!(mid.y, 10.0);
    }

    #[test]
    fn test_vec2_constructors() {
        let z = Vec2::zero();
        assert_eq!(z.x, 0.0);
        assert_eq!(z.y, 0.0);

        let o = Vec2::one();
        assert_eq!(o.x, 1.0);
        assert_eq!(o.y, 1.0);
    }

    #[test]
    fn test_vec2_from_tuple() {
        let v: Vec2 = (3.0, 4.0).into();
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);

        let t: (f32, f32) = v.into();
        assert_eq!(t, (3.0, 4.0));
    }

    #[test]
    fn test_vec2_lerp_boundaries() {
        let a = Vec2::new(1.0, 2.0);
        let b = Vec2::new(5.0, 8.0);
        let start = a.lerp(b, 0.0);
        assert_eq!(start.x, 1.0);
        assert_eq!(start.y, 2.0);

        let end = a.lerp(b, 1.0);
        assert_eq!(end.x, 5.0);
        assert_eq!(end.y, 8.0);
    }

    #[test]
    fn test_transform_identity() {
        let t = Transform::identity();
        assert_eq!(t.alpha, 1.0);
        assert_eq!(t.position, Vec2::zero());
        assert_eq!(t.scale, Vec2::one());
        assert_eq!(t.rotation, 0.0);
    }

    #[test]
    fn test_transform_with_alpha() {
        let t = Transform::with_alpha(0.5);
        assert!((t.alpha - 0.5).abs() < 0.001);
        assert_eq!(t.position, Vec2::zero());
    }

    #[test]
    fn test_transform_with_position() {
        let t = Transform::with_position(10.0, 20.0);
        assert_eq!(t.position.x, 10.0);
        assert_eq!(t.position.y, 20.0);
        assert_eq!(t.alpha, 1.0);
    }

    #[test]
    fn test_transform_with_scale() {
        let t = Transform::with_scale(2.0, 3.0);
        assert_eq!(t.scale.x, 2.0);
        assert_eq!(t.scale.y, 3.0);
    }

    #[test]
    fn test_transform_with_uniform_scale() {
        let t = Transform::with_uniform_scale(1.5);
        assert_eq!(t.scale.x, 1.5);
        assert_eq!(t.scale.y, 1.5);
    }

    #[test]
    fn test_transform_set_methods() {
        let mut t = Transform::identity();

        t.set_alpha(0.3);
        assert!((t.alpha - 0.3).abs() < 0.001);

        t.set_alpha(1.5);
        assert_eq!(t.alpha, 1.0);
        t.set_alpha(-0.5);
        assert_eq!(t.alpha, 0.0);

        t.set_position(5.0, 10.0);
        assert_eq!(t.position, Vec2::new(5.0, 10.0));

        t.set_scale(2.0, 3.0);
        assert_eq!(t.scale, Vec2::new(2.0, 3.0));

        t.set_rotation(1.57);
        assert!((t.rotation - 1.57).abs() < 0.001);
    }

    #[test]
    fn test_transform_lerp_full() {
        let a = Transform {
            position: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            alpha: 0.0,
        };
        let b = Transform {
            position: Vec2::new(10.0, 20.0),
            scale: Vec2::new(3.0, 3.0),
            rotation: std::f32::consts::PI,
            alpha: 1.0,
        };

        let mid = a.lerp(&b, 0.5);
        assert!((mid.position.x - 5.0).abs() < 0.001);
        assert!((mid.position.y - 10.0).abs() < 0.001);
        assert!((mid.scale.x - 2.0).abs() < 0.001);
        assert!((mid.rotation - std::f32::consts::FRAC_PI_2).abs() < 0.001);
        assert!((mid.alpha - 0.5).abs() < 0.001);
    }
}
