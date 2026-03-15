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
use crate::extensions::CapabilityId;
use std::collections::BTreeMap;
use vn_runtime::command::Position;

/// capability 参数值（用于诊断与扩展调试）。
#[derive(Debug, Clone, PartialEq)]
pub enum EffectParamValue {
    Number(f32),
    Bool(bool),
    String(String),
}

pub type EffectParams = BTreeMap<String, EffectParamValue>;

/// 统一动画/过渡请求
///
/// 将"对谁做"（`target`）和"做什么效果"（`effect`）组合为一个完整的请求。
/// 由 `CommandExecutor` 生产，由 `EffectApplier` 消费。
#[derive(Debug, Clone)]
pub struct EffectRequest {
    /// capability 路由 ID（扩展 API 稳定入口）
    pub capability_id: CapabilityId,
    /// 规范化参数快照（用于诊断/调试/后续第三方扩展）
    pub params: EffectParams,
    /// 动画目标（携带上下文数据）
    pub target: EffectTarget,
    /// 已解析的效果（携带效果类型、时长、缓动）
    pub effect: ResolvedEffect,
}

impl EffectRequest {
    /// 使用 target + effect 构造统一请求，并自动填充 capability 元数据。
    pub fn new(target: EffectTarget, effect: ResolvedEffect) -> Self {
        let capability_id = infer_capability_id(&target, &effect);
        let params = build_params(&effect);
        Self {
            capability_id,
            params,
            target,
            effect,
        }
    }
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

    /// 场景效果（镜头语言：shake/blur/dim 等）
    ///
    /// EffectApplier 将：根据 effect_name 分发到对应的场景效果处理器
    SceneEffect {
        /// 效果名称（如 "shakeSmall", "blurIn", "dimStep"）
        effect_name: String,
    },

    /// 标题字卡
    ///
    /// EffectApplier 将：设置渲染器的标题字卡状态并启动淡入淡出动画
    TitleCard {
        /// 显示文本
        text: String,
    },
}

fn infer_capability_id(target: &EffectTarget, effect: &ResolvedEffect) -> CapabilityId {
    let id = match (&effect.kind, target) {
        (super::registry::EffectKind::Dissolve, _) => "effect.dissolve".to_string(),
        (
            super::registry::EffectKind::Fade | super::registry::EffectKind::FadeWhite,
            EffectTarget::SceneTransition { .. },
        ) => "effect.fade".to_string(),
        (super::registry::EffectKind::Rule { .. }, EffectTarget::SceneTransition { .. }) => {
            "effect.rule_mask".to_string()
        }
        (super::registry::EffectKind::Move, EffectTarget::CharacterMove { .. }) => {
            "effect.move".to_string()
        }
        (_, EffectTarget::SceneEffect { effect_name }) => {
            let category = scene_effect_category(effect_name);
            format!("effect.scene.{}", category)
        }
        (_, EffectTarget::TitleCard { .. }) => "effect.scene.title_card".to_string(),
        _ => format!("effect.{}", effect.kind_name()),
    };
    CapabilityId::new(id)
}

/// 根据效果名推断场景效果类别
fn scene_effect_category(effect_name: &str) -> &str {
    let name_lower = effect_name.to_lowercase();
    if name_lower.contains("shake") || name_lower.contains("bounce") {
        "shake"
    } else if name_lower.contains("blur") || name_lower.contains("flashback") {
        "blur"
    } else if name_lower.contains("dim") {
        "dim"
    } else {
        "generic"
    }
}

fn build_params(effect: &ResolvedEffect) -> EffectParams {
    let mut params = BTreeMap::new();
    if let Some(duration) = effect.duration {
        params.insert("duration".to_string(), EffectParamValue::Number(duration));
    }
    if let super::registry::EffectKind::Rule {
        mask_path,
        reversed,
    } = &effect.kind
    {
        params.insert(
            "mask".to_string(),
            EffectParamValue::String(mask_path.clone()),
        );
        params.insert("reversed".to_string(), EffectParamValue::Bool(*reversed));
    }
    params
}

/// 创建 `EffectRequest` 并注入额外参数
///
/// 用于场景效果等需要在 capability 路由中携带自定义参数的场景。
impl EffectRequest {
    pub fn with_extra_params(
        target: EffectTarget,
        effect: ResolvedEffect,
        extra: impl IntoIterator<Item = (String, EffectParamValue)>,
    ) -> Self {
        let mut req = Self::new(target, effect);
        for (k, v) in extra {
            req.params.insert(k, v);
        }
        req
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::animation::EasingFunction;
    use crate::renderer::effects::EffectKind;

    #[test]
    fn infer_capability_for_rule_scene_transition() {
        let req = EffectRequest::new(
            EffectTarget::SceneTransition {
                pending_background: "bg.png".to_string(),
            },
            ResolvedEffect {
                kind: EffectKind::Rule {
                    mask_path: "masks/wipe.png".to_string(),
                    reversed: true,
                },
                duration: Some(0.8),
                easing: EasingFunction::Linear,
            },
        );
        assert_eq!(req.capability_id, "effect.rule_mask");
        assert!(matches!(
            req.params.get("reversed"),
            Some(EffectParamValue::Bool(true))
        ));
    }

    #[test]
    fn infer_capability_for_character_move() {
        let req = EffectRequest::new(
            EffectTarget::CharacterMove {
                alias: "a".to_string(),
                old_position: Position::Left,
                new_position: Position::Right,
            },
            ResolvedEffect {
                kind: EffectKind::Move,
                duration: Some(0.3),
                easing: EasingFunction::EaseInOut,
            },
        );
        assert_eq!(req.capability_id, "effect.move");
    }
}
