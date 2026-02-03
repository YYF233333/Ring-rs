//! # Easing 模块
//!
//! 缓动函数库，用于动画的时间插值。

use std::f32::consts::PI;

/// 缓动函数类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EasingFunction {
    /// 线性（匀速）
    Linear,
    /// 缓入（先慢后快）
    EaseIn,
    /// 缓出（先快后慢）
    EaseOut,
    /// 缓入缓出（两头慢中间快）
    #[default]
    EaseInOut,
    /// 二次缓入
    EaseInQuad,
    /// 二次缓出
    EaseOutQuad,
    /// 二次缓入缓出
    EaseInOutQuad,
    /// 三次缓入
    EaseInCubic,
    /// 三次缓出
    EaseOutCubic,
    /// 三次缓入缓出
    EaseInOutCubic,
    /// 正弦缓入
    EaseInSine,
    /// 正弦缓出
    EaseOutSine,
    /// 正弦缓入缓出
    EaseInOutSine,
    /// 弹性缓出
    EaseOutElastic,
    /// 弹跳缓出
    EaseOutBounce,
}

impl EasingFunction {
    /// 计算缓动值
    ///
    /// # 参数
    /// - `t`: 时间进度 (0.0 - 1.0)
    ///
    /// # 返回
    /// - 缓动后的进度值 (0.0 - 1.0)
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);

        match self {
            EasingFunction::Linear => t,
            EasingFunction::EaseIn => ease_in(t),
            EasingFunction::EaseOut => ease_out(t),
            EasingFunction::EaseInOut => ease_in_out(t),
            EasingFunction::EaseInQuad => t * t,
            EasingFunction::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
            EasingFunction::EaseInOutQuad => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            EasingFunction::EaseInCubic => t * t * t,
            EasingFunction::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            EasingFunction::EaseInOutCubic => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            EasingFunction::EaseInSine => 1.0 - (t * PI / 2.0).cos(),
            EasingFunction::EaseOutSine => (t * PI / 2.0).sin(),
            EasingFunction::EaseInOutSine => -((PI * t).cos() - 1.0) / 2.0,
            EasingFunction::EaseOutElastic => ease_out_elastic(t),
            EasingFunction::EaseOutBounce => ease_out_bounce(t),
        }
    }
}

/// 缓入（Cubic）
fn ease_in(t: f32) -> f32 {
    t * t * t
}

/// 缓出（Cubic）
fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

/// 缓入缓出（Cubic）
fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// 弹性缓出
fn ease_out_elastic(t: f32) -> f32 {
    if t == 0.0 {
        0.0
    } else if t == 1.0 {
        1.0
    } else {
        let c4 = (2.0 * PI) / 3.0;
        2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c4).sin() + 1.0
    }
}

/// 弹跳缓出
fn ease_out_bounce(t: f32) -> f32 {
    let n1 = 7.5625;
    let d1 = 2.75;

    if t < 1.0 / d1 {
        n1 * t * t
    } else if t < 2.0 / d1 {
        let t = t - 1.5 / d1;
        n1 * t * t + 0.75
    } else if t < 2.5 / d1 {
        let t = t - 2.25 / d1;
        n1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / d1;
        n1 * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        let easing = EasingFunction::Linear;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(0.5), 0.5);
        assert_eq!(easing.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_in_out() {
        let easing = EasingFunction::EaseInOut;
        assert_eq!(easing.apply(0.0), 0.0);
        assert_eq!(easing.apply(1.0), 1.0);
        // 中点应该是 0.5
        let mid = easing.apply(0.5);
        assert!((mid - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_clamp() {
        let easing = EasingFunction::Linear;
        // 超出范围应该被限制
        assert_eq!(easing.apply(-0.5), 0.0);
        assert_eq!(easing.apply(1.5), 1.0);
    }

    #[test]
    fn test_ease_out_bounce() {
        let easing = EasingFunction::EaseOutBounce;
        assert_eq!(easing.apply(0.0), 0.0);
        assert!((easing.apply(1.0) - 1.0).abs() < 0.001);
    }
}
