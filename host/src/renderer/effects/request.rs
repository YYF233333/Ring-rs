//! # Effect Request（统一动画请求模型）
//!
//! 定义 `EffectRequest` 和 `EffectTarget`，作为 CommandExecutor → EffectApplier 的统一接口。
//!
//! ## 设计说明
//!
//! - `EffectTarget` 携带**上下文数据**（哪个对象、什么状态）
//! - `ResolvedEffect` 携带**效果数据**（什么效果、时长、缓动）
//! - 两者组合成 `EffectRequest`，完整描述一个动画/过渡请求
//!
//! ## 使用流程
//!
//! ```text
//! CommandExecutor.execute()
//!   → 构造 EffectRequest { target, effect }
//!   → 存入 CommandOutput.effect_requests
//!   → EffectApplier.apply() 读取并分发到对应的动画子系统
//! ```

use super::resolver::ResolvedEffect;
use vn_runtime::command::Position;

/// 统一动画/过渡请求
///
/// 将"对谁做"（`target`）和"做什么效果"（`effect`）组合为一个完整的请求。
/// 由 `CommandExecutor` 生产，由 `EffectApplier` 消费。
#[derive(Debug, Clone)]
pub struct EffectRequest {
    /// 动画目标（携带上下文数据）
    pub target: EffectTarget,
    /// 已解析的效果（携带效果类型、时长、缓动）
    pub effect: ResolvedEffect,
}

/// 动画/过渡目标
///
/// 每个变体携带该目标执行动画所需的**最少上下文数据**。
/// 效果相关参数（时长、缓动、遮罩路径等）由 `ResolvedEffect` 提供。
#[derive(Debug, Clone)]
pub enum EffectTarget {
    /// 角色淡入（新角色或重建角色）
    ///
    /// EffectApplier 将：注册到 AnimationSystem（如未注册）→ 启动 alpha 0→1 动画
    CharacterShow {
        /// 角色别名
        alias: String,
    },

    /// 角色淡出
    ///
    /// EffectApplier 将：获取已注册的 ObjectId → 启动 alpha 1→0 动画
    CharacterHide {
        /// 角色别名
        alias: String,
    },

    /// 角色位置移动
    ///
    /// EffectApplier 将：计算位置偏移 → 设置初始 position_x → 启动位移动画
    CharacterMove {
        /// 角色别名
        alias: String,
        /// 旧位置（用于计算偏移）
        old_position: Position,
        /// 新位置
        new_position: Position,
    },

    /// 背景过渡（dissolve，由 TransitionManager 驱动）
    ///
    /// EffectApplier 将：调用 renderer.start_background_transition_resolved()
    BackgroundTransition {
        /// 旧背景路径
        old_background: Option<String>,
    },

    /// 场景遮罩过渡（fade/fadewhite/rule，由 SceneTransitionManager 驱动）
    ///
    /// EffectApplier 将：根据 effect.kind 调用 renderer.start_scene_fade/fade_white/rule()
    /// 注意：mask_path 和 reversed 已编码在 EffectKind::Rule 中
    SceneTransition {
        /// 待切换的背景路径
        pending_background: String,
    },
}
