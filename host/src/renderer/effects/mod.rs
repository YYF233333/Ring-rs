//! # Effects 模块（统一动画/过渡效果解析）
//!
//! 阶段 25 核心模块：把"过渡效果/动画效果"的**解析**收敛到一个统一单元。
//! 背景/立绘/场景遮罩共享同一套效果定义与参数解析。
//!
//! ## 核心组件
//!
//! - [`EffectKind`]：效果类型枚举
//! - [`ResolvedEffect`]：解析后的效果（参数已提取、已校验）
//! - [`resolve`]：将 `vn_runtime::command::Transition` 解析为 `ResolvedEffect`
//!
//! ## 使用流程
//!
//! ```text
//! Transition (from Runtime)
//!   → resolve() → ResolvedEffect
//!   → CommandExecutor 根据 ResolvedEffect.kind 分发到对应处理路径
//!   → AnimationSystem / SceneTransitionManager / TransitionManager
//! ```
//!
//! ## 设计原则
//!
//! - **唯一来源**：效果名到 EffectKind 的映射、默认参数值，只在本模块定义
//! - **上下文无关**：resolver 不知道效果应用在哪个对象上；上下文判断由调用方负责
//! - **默认值分层**：效果级默认值在 `defaults` 模块；上下文级默认值由调用方通过
//!   `duration_or()` 提供

mod registry;
mod resolver;

pub use registry::{EffectKind, defaults};
pub use resolver::{ResolvedEffect, resolve};
