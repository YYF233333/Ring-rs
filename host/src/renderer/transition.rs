//! # Transition 模块
//!
//! 过渡效果系统，负责管理各种过渡动画。
//!
//! ## 支持的过渡效果
//!
//! - `dissolve` / `Dissolve(duration)`: 淡入淡出
//! - `fade` / `Fade(duration)`: 渐隐渐显（先全黑再显示新内容）
//! - `none`: 无过渡，立即切换

use macroquad::prelude::*;

/// 过渡效果类型
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionType {
    /// 无过渡
    None,
    /// 淡入淡出（交叉溶解）
    Dissolve,
    /// 渐隐渐显（通过黑色）
    Fade,
    /// 渐隐渐显（通过白色）
    FadeWhite,
}

/// 过渡效果状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionPhase {
    /// 空闲状态
    Idle,
    /// 淡出阶段（旧内容消失）
    FadeOut,
    /// 淡入阶段（新内容出现）
    FadeIn,
}

/// 过渡效果管理器
#[derive(Debug)]
pub struct TransitionManager {
    /// 当前过渡类型
    transition_type: TransitionType,
    /// 当前阶段
    phase: TransitionPhase,
    /// 过渡时长（秒）
    duration: f32,
    /// 当前进度（0.0 - 1.0）
    progress: f32,
    /// 是否可以跳过
    skippable: bool,
}

impl TransitionManager {
    /// 创建新的过渡效果管理器
    pub fn new() -> Self {
        Self {
            transition_type: TransitionType::None,
            phase: TransitionPhase::Idle,
            duration: 0.3,
            progress: 0.0,
            skippable: true,
        }
    }

    /// 开始过渡效果
    ///
    /// # 参数
    ///
    /// - `transition_type`: 过渡类型
    /// - `duration`: 过渡时长（秒）
    pub fn start(&mut self, transition_type: TransitionType, duration: f32) {
        if transition_type == TransitionType::None {
            self.phase = TransitionPhase::Idle;
            self.progress = 1.0;
            return;
        }

        self.transition_type = transition_type;
        self.duration = duration.max(0.01); // 避免除零
        self.progress = 0.0;
        
        // Dissolve 只有 FadeIn 阶段，Fade 有 FadeOut + FadeIn
        match self.transition_type {
            TransitionType::Dissolve => {
                self.phase = TransitionPhase::FadeIn;
            }
            TransitionType::Fade | TransitionType::FadeWhite => {
                self.phase = TransitionPhase::FadeOut;
            }
            TransitionType::None => {
                self.phase = TransitionPhase::Idle;
            }
        }
    }

    /// 从 vn-runtime 的 Transition 解析
    pub fn start_from_command(&mut self, transition: &vn_runtime::command::Transition) {
        let name = transition.name.to_lowercase();
        let duration = transition
            .args
            .first()
            .and_then(|arg| {
                if let vn_runtime::command::TransitionArg::Number(n) = arg {
                    Some(*n as f32)
                } else {
                    None
                }
            })
            .unwrap_or(0.3);

        let transition_type = match name.as_str() {
            "dissolve" => TransitionType::Dissolve,
            "fade" => TransitionType::Fade,
            "fadewhite" | "fade_white" => TransitionType::FadeWhite,
            "none" => TransitionType::None,
            _ => {
                println!("⚠️ 未知过渡效果: {}, 使用 dissolve", name);
                TransitionType::Dissolve
            }
        };

        self.start(transition_type, duration);
    }

    /// 更新过渡效果
    ///
    /// # 返回
    ///
    /// - `true`: 过渡效果仍在进行中
    /// - `false`: 过渡效果已完成或处于空闲状态
    pub fn update(&mut self, dt: f32) -> bool {
        if self.phase == TransitionPhase::Idle {
            return false;
        }

        // 更新进度
        self.progress += dt / self.duration;

        if self.progress >= 1.0 {
            self.progress = 1.0;

            // 切换到下一阶段
            match (&self.transition_type, &self.phase) {
                (TransitionType::Dissolve, TransitionPhase::FadeIn) => {
                    // Dissolve 完成
                    self.phase = TransitionPhase::Idle;
                }
                (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeOut) => {
                    // FadeOut 完成，进入 FadeIn
                    self.phase = TransitionPhase::FadeIn;
                    self.progress = 0.0;
                }
                (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeIn) => {
                    // Fade 完成
                    self.phase = TransitionPhase::Idle;
                }
                _ => {
                    self.phase = TransitionPhase::Idle;
                }
            }
        }

        self.phase != TransitionPhase::Idle
    }

    /// 跳过过渡效果
    pub fn skip(&mut self) {
        if self.skippable && self.phase != TransitionPhase::Idle {
            self.phase = TransitionPhase::Idle;
            self.progress = 1.0;
        }
    }

    /// 获取当前阶段
    pub fn phase(&self) -> TransitionPhase {
        self.phase
    }

    /// 是否正在过渡中
    pub fn is_active(&self) -> bool {
        self.phase != TransitionPhase::Idle
    }

    /// 获取当前进度（0.0 - 1.0）
    pub fn progress(&self) -> f32 {
        self.progress
    }

    /// 获取用于渲染新内容的 alpha 值
    ///
    /// Dissolve: 新内容从 0 淡入到 1
    /// Fade: FadeOut 阶段为 0，FadeIn 阶段从 0 到 1
    pub fn new_content_alpha(&self) -> f32 {
        match (&self.transition_type, &self.phase) {
            (TransitionType::None, _) | (_, TransitionPhase::Idle) => 1.0,
            (TransitionType::Dissolve, TransitionPhase::FadeIn) => {
                ease_in_out(self.progress)
            }
            (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeOut) => 0.0,
            (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeIn) => {
                ease_in_out(self.progress)
            }
            _ => 1.0,
        }
    }

    /// 获取用于渲染旧内容的 alpha 值
    ///
    /// Dissolve: 旧内容从 1 淡出到 0
    /// Fade: FadeOut 阶段从 1 到 0，FadeIn 阶段为 0
    pub fn old_content_alpha(&self) -> f32 {
        match (&self.transition_type, &self.phase) {
            (TransitionType::None, _) | (_, TransitionPhase::Idle) => 0.0,
            (TransitionType::Dissolve, TransitionPhase::FadeIn) => {
                1.0 - ease_in_out(self.progress)
            }
            (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeOut) => {
                1.0 - ease_in_out(self.progress)
            }
            (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeIn) => 0.0,
            _ => 0.0,
        }
    }

    /// 获取遮罩层 alpha 值（用于 Fade 效果）
    ///
    /// Fade: FadeOut 阶段从 0 到 1，FadeIn 阶段从 1 到 0
    pub fn overlay_alpha(&self) -> f32 {
        match (&self.transition_type, &self.phase) {
            (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeOut) => {
                ease_in_out(self.progress)
            }
            (TransitionType::Fade | TransitionType::FadeWhite, TransitionPhase::FadeIn) => {
                1.0 - ease_in_out(self.progress)
            }
            _ => 0.0,
        }
    }

    /// 获取遮罩层颜色
    pub fn overlay_color(&self) -> Color {
        match self.transition_type {
            TransitionType::FadeWhite => WHITE,
            _ => BLACK,
        }
    }

    /// 渲染过渡效果遮罩层
    ///
    /// 在所有内容渲染完成后调用此方法，绘制 Fade 效果的遮罩。
    pub fn render_overlay(&self) {
        let alpha = self.overlay_alpha();
        if alpha > 0.0 {
            let mut color = self.overlay_color();
            color.a = alpha;
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), color);
        }
    }
}

impl Default for TransitionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓动函数：ease-in-out
fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transition_manager_creation() {
        let manager = TransitionManager::new();
        assert_eq!(manager.phase(), TransitionPhase::Idle);
        assert!(!manager.is_active());
    }

    #[test]
    fn test_dissolve_transition() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionType::Dissolve, 1.0);
        
        assert!(manager.is_active());
        assert_eq!(manager.phase(), TransitionPhase::FadeIn);
        assert_eq!(manager.new_content_alpha(), 0.0);

        // 模拟半程
        manager.update(0.5);
        assert!(manager.new_content_alpha() > 0.0);
        assert!(manager.new_content_alpha() < 1.0);

        // 完成
        manager.update(0.6);
        assert!(!manager.is_active());
        assert_eq!(manager.new_content_alpha(), 1.0);
    }

    #[test]
    fn test_fade_transition() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionType::Fade, 0.5);
        
        assert!(manager.is_active());
        assert_eq!(manager.phase(), TransitionPhase::FadeOut);

        // FadeOut 完成
        manager.update(0.6);
        assert_eq!(manager.phase(), TransitionPhase::FadeIn);

        // FadeIn 完成
        manager.update(0.6);
        assert!(!manager.is_active());
    }

    #[test]
    fn test_skip_transition() {
        let mut manager = TransitionManager::new();
        manager.start(TransitionType::Dissolve, 1.0);
        
        assert!(manager.is_active());
        manager.skip();
        assert!(!manager.is_active());
    }
}
