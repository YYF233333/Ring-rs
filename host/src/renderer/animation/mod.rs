//! # Animation 模块
//!
//! 通用动画系统，负责管理所有动画效果。
//!
//! ## 核心设计理念
//!
//! 动画系统只负责 **时间轴管理**：
//! - 知道某个属性从 A 到 B 需要在 duration 内变化
//! - 维护当前值，通过 PropertyKey 查询
//! - **不假设对象类型**，对象自己决定如何使用这些值
//!
//! ## 核心概念
//!
//! - `PropertyKey`: 属性键，唯一标识一个可动画的属性
//! - `Animation`: 单个动画实例，管理 f32 值的时间变化
//! - `AnimationSystem`: 动画系统管理器
//! - `EasingFunction`: 缓动函数
//!
//! ## 使用示例
//!
//! ```rust,ignore
//! let mut system = AnimationSystem::new();
//!
//! // 角色淡入
//! system.character_fade_in("alice", 0.3);
//!
//! // 自定义属性动画
//! system.animate(
//!     PropertyKey::custom("my_object.progress"),
//!     0.0, 1.0, 2.0
//! );
//!
//! // 查询当前值
//! let alpha = system.get_character_alpha("alice");
//! let progress = system.get_value_or(&PropertyKey::custom("my_object.progress"), 0.0);
//! ```

mod animation;
mod easing;
mod system;
mod target;
mod traits;
mod transform;

// 核心类型
pub use animation::{Animation, AnimationId, AnimationState};
pub use easing::EasingFunction;
pub use system::AnimationSystem;

// Trait-based 动画系统 API
pub use traits::{Animatable, AnimPropertyKey, ObjectId, PropertyAccessor, SimplePropertyAccessor};

// 辅助类型（保留以便需要时使用）
pub use target::AnimationTarget;
pub use transform::{Transform, Vec2};

/// 动画事件
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationEvent {
    /// 动画开始
    Started(AnimationId),
    /// 动画完成
    Completed(AnimationId),
    /// 动画被跳过
    Skipped(AnimationId),
}
