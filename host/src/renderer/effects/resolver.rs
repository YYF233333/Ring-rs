//! # Effect Resolver
//!
//! 将 `vn_runtime::command::Transition` 解析为 `ResolvedEffect`。
//!
//! 这是 Transition → ResolvedEffect 的**唯一转换入口**。
//! 所有效果参数的提取、校验、默认值填充都在这里完成。

use super::registry::EffectKind;
use crate::renderer::animation::EasingFunction;
use vn_runtime::command::{Transition, TransitionArg};

/// 解析后的效果
///
/// 包含效果类型、持续时间、缓动函数等所有执行所需参数。
/// 由 [`resolve`] 从 `Transition` 解析得到。
///
/// ## 持续时间处理
///
/// `duration` 字段存储脚本中**显式指定**的持续时间。
/// 如果脚本未指定持续时间，该字段为 `None`。
/// 调用方通过 [`duration_or`](ResolvedEffect::duration_or) 提供上下文相关的默认值。
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedEffect {
    /// 效果类型
    pub kind: EffectKind,
    /// 显式指定的持续时间（秒），`None` 表示使用调用方默认值
    pub duration: Option<f32>,
    /// 缓动函数
    pub easing: EasingFunction,
}

impl ResolvedEffect {
    /// 创建无效果实例
    pub fn none() -> Self {
        Self {
            kind: EffectKind::None,
            duration: Some(0.0),
            easing: EasingFunction::Linear,
        }
    }

    /// 获取持续时间，未指定时使用提供的默认值
    pub fn duration_or(&self, default: f32) -> f32 {
        self.duration.unwrap_or(default)
    }

    /// 是否是 alpha 类效果（dissolve / fade 在立绘上下文中）
    ///
    /// 用于判断是否应产生 alpha 动画。
    /// 注意：`Fade` 在 changeScene 中是黑屏遮罩，但在立绘上下文中视为 alpha 效果。
    pub fn is_alpha_effect(&self) -> bool {
        matches!(self.kind, EffectKind::Dissolve | EffectKind::Fade)
    }

    /// 是否是位置移动效果
    pub fn is_move_effect(&self) -> bool {
        matches!(self.kind, EffectKind::Move)
    }

    /// 是否是场景遮罩效果（fade/fadewhite/rule）
    pub fn is_scene_mask_effect(&self) -> bool {
        matches!(
            self.kind,
            EffectKind::Fade | EffectKind::FadeWhite | EffectKind::Rule { .. }
        )
    }
}

/// 将 `Transition` 解析为 `ResolvedEffect`
///
/// 这是效果解析的**唯一入口**。所有需要从 `Transition` 提取效果信息的地方
/// 都应调用此函数，而非手动解析 `Transition.name` / `Transition.get_duration()`。
///
/// ## 效果名称映射（大小写不敏感）
///
/// | 脚本名称 | EffectKind | 说明 |
/// |----------|------------|------|
/// | `dissolve` | `Dissolve` | Alpha 交叉淡化 |
/// | `fade` | `Fade` | 黑屏遮罩 / 立绘上下文等价 dissolve |
/// | `fadewhite` | `FadeWhite` | 白屏遮罩 |
/// | `rule` | `Rule { mask_path, reversed }` | 图片遮罩 |
/// | `move` / `slide` | `Move` | 位置移动 |
/// | `none` | `None` | 无效果 |
/// | 其他 | `Dissolve`（降级） | 未知名称降级为 dissolve |
pub fn resolve(transition: &Transition) -> ResolvedEffect {
    let name_lower = transition.name.to_lowercase();
    let explicit_duration = transition.get_duration().map(|d| d as f32);

    match name_lower.as_str() {
        "dissolve" => ResolvedEffect {
            kind: EffectKind::Dissolve,
            duration: explicit_duration,
            easing: EasingFunction::EaseInOut,
        },

        "fade" => ResolvedEffect {
            kind: EffectKind::Fade,
            duration: explicit_duration,
            easing: EasingFunction::EaseInOut,
        },

        "fadewhite" => ResolvedEffect {
            kind: EffectKind::FadeWhite,
            duration: explicit_duration,
            easing: EasingFunction::EaseInOut,
        },

        "rule" => {
            let mask_path = transition
                .get_named("mask")
                .and_then(|arg| {
                    if let TransitionArg::String(s) = arg {
                        Some(s.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            let reversed = transition.get_reversed().unwrap_or(false);

            ResolvedEffect {
                kind: EffectKind::Rule {
                    mask_path,
                    reversed,
                },
                duration: explicit_duration,
                easing: EasingFunction::Linear,
            }
        }

        "move" | "slide" => ResolvedEffect {
            kind: EffectKind::Move,
            duration: explicit_duration,
            easing: EasingFunction::EaseInOut,
        },

        "none" => ResolvedEffect::none(),

        _ => {
            tracing::warn!(
                name = %transition.name,
                "未知效果名，降级为 dissolve"
            );
            ResolvedEffect {
                kind: EffectKind::Dissolve,
                duration: explicit_duration,
                easing: EasingFunction::EaseInOut,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::registry::defaults;
    use super::*;
    use vn_runtime::command::{Transition, TransitionArg};

    // ========== 基本解析测试 ==========

    #[test]
    fn test_resolve_dissolve_no_args() {
        let t = Transition::simple("dissolve");
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::Dissolve);
        assert_eq!(effect.duration, None);
        assert_eq!(
            effect.duration_or(defaults::DISSOLVE_DURATION),
            defaults::DISSOLVE_DURATION
        );
    }

    #[test]
    fn test_resolve_dissolve_with_duration() {
        let t = Transition::with_args("dissolve", vec![TransitionArg::Number(1.5)]);
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::Dissolve);
        assert_eq!(effect.duration, Some(1.5));
        assert_eq!(effect.duration_or(0.3), 1.5);
    }

    #[test]
    fn test_resolve_dissolve_case_insensitive() {
        let t = Transition::simple("Dissolve");
        let effect = resolve(&t);
        assert_eq!(effect.kind, EffectKind::Dissolve);

        let t2 = Transition::simple("DISSOLVE");
        let effect2 = resolve(&t2);
        assert_eq!(effect2.kind, EffectKind::Dissolve);
    }

    #[test]
    fn test_resolve_fade() {
        let t = Transition::simple("fade");
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::Fade);
        assert_eq!(effect.duration, None);
        // fade 在场景上下文中默认 0.5
        assert_eq!(
            effect.duration_or(defaults::FADE_DURATION),
            defaults::FADE_DURATION
        );
        // fade 在立绘上下文中使用 dissolve 默认值 0.3
        assert_eq!(
            effect.duration_or(defaults::CHARACTER_ALPHA_DURATION),
            defaults::CHARACTER_ALPHA_DURATION
        );
    }

    #[test]
    fn test_resolve_fade_with_duration() {
        let t = Transition::with_args("fade", vec![TransitionArg::Number(2.0)]);
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::Fade);
        assert_eq!(effect.duration, Some(2.0));
    }

    #[test]
    fn test_resolve_fadewhite() {
        let t = Transition::simple("fadewhite");
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::FadeWhite);
        assert_eq!(effect.duration, None);
        assert_eq!(
            effect.duration_or(defaults::FADE_WHITE_DURATION),
            defaults::FADE_WHITE_DURATION
        );
    }

    #[test]
    fn test_resolve_rule_with_params() {
        let t = Transition::with_named_args(
            "rule",
            vec![
                (Some("duration".to_string()), TransitionArg::Number(0.8)),
                (
                    Some("mask".to_string()),
                    TransitionArg::String("masks/wipe.png".to_string()),
                ),
                (Some("reversed".to_string()), TransitionArg::Bool(true)),
            ],
        );
        let effect = resolve(&t);

        match &effect.kind {
            EffectKind::Rule {
                mask_path,
                reversed,
            } => {
                assert_eq!(mask_path, "masks/wipe.png");
                assert!(*reversed);
            }
            other => panic!("Expected Rule, got {:?}", other),
        }
        assert_eq!(effect.duration, Some(0.8));
    }

    #[test]
    fn test_resolve_rule_defaults() {
        let t = Transition::simple("rule");
        let effect = resolve(&t);

        match &effect.kind {
            EffectKind::Rule {
                mask_path,
                reversed,
            } => {
                assert_eq!(mask_path, "");
                assert!(!reversed);
            }
            other => panic!("Expected Rule, got {:?}", other),
        }
        assert_eq!(effect.duration, None);
        assert_eq!(
            effect.duration_or(defaults::RULE_DURATION),
            defaults::RULE_DURATION
        );
    }

    #[test]
    fn test_resolve_move() {
        let t = Transition::simple("move");
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::Move);
        assert_eq!(effect.duration, None);
        assert_eq!(
            effect.duration_or(defaults::MOVE_DURATION),
            defaults::MOVE_DURATION
        );
    }

    #[test]
    fn test_resolve_slide_is_move() {
        let t = Transition::with_args("slide", vec![TransitionArg::Number(0.5)]);
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::Move);
        assert_eq!(effect.duration, Some(0.5));
    }

    #[test]
    fn test_resolve_none() {
        let t = Transition::simple("none");
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::None);
        assert_eq!(effect.duration, Some(0.0));
    }

    #[test]
    fn test_resolve_unknown_falls_back_to_dissolve() {
        let t = Transition::simple("unknown_effect");
        let effect = resolve(&t);

        assert_eq!(effect.kind, EffectKind::Dissolve);
    }

    // ========== 语义辅助方法测试 ==========

    #[test]
    fn test_is_alpha_effect() {
        assert!(resolve(&Transition::simple("dissolve")).is_alpha_effect());
        assert!(resolve(&Transition::simple("fade")).is_alpha_effect());
        assert!(!resolve(&Transition::simple("fadewhite")).is_alpha_effect());
        assert!(!resolve(&Transition::simple("move")).is_alpha_effect());
        assert!(!resolve(&Transition::simple("rule")).is_alpha_effect());
        assert!(!resolve(&Transition::simple("none")).is_alpha_effect());
    }

    #[test]
    fn test_is_move_effect() {
        assert!(resolve(&Transition::simple("move")).is_move_effect());
        assert!(resolve(&Transition::simple("slide")).is_move_effect());
        assert!(!resolve(&Transition::simple("dissolve")).is_move_effect());
    }

    #[test]
    fn test_is_scene_mask_effect() {
        assert!(resolve(&Transition::simple("fade")).is_scene_mask_effect());
        assert!(resolve(&Transition::simple("fadewhite")).is_scene_mask_effect());
        assert!(resolve(&Transition::simple("rule")).is_scene_mask_effect());
        assert!(!resolve(&Transition::simple("dissolve")).is_scene_mask_effect());
        assert!(!resolve(&Transition::simple("move")).is_scene_mask_effect());
    }

    // ========== 一致性测试 ==========

    #[test]
    fn test_dissolve_consistency_across_contexts() {
        // 同一个 dissolve 在任何上下文中解析结果应一致
        let transitions = vec![
            Transition::simple("dissolve"),
            Transition::with_args("dissolve", vec![TransitionArg::Number(0.5)]),
            Transition::with_named_args(
                "dissolve",
                vec![(Some("duration".to_string()), TransitionArg::Number(0.5))],
            ),
        ];

        for t in &transitions {
            let effect = resolve(t);
            assert_eq!(effect.kind, EffectKind::Dissolve);
        }
    }

    #[test]
    fn test_duration_or_respects_explicit_over_default() {
        let t = Transition::with_args("dissolve", vec![TransitionArg::Number(2.0)]);
        let effect = resolve(&t);

        // 即使传入不同的默认值，显式指定的 duration 优先
        assert_eq!(effect.duration_or(0.3), 2.0);
        assert_eq!(effect.duration_or(0.5), 2.0);
        assert_eq!(effect.duration_or(999.0), 2.0);
    }

    #[test]
    fn test_duration_or_uses_default_when_not_specified() {
        let t = Transition::simple("dissolve");
        let effect = resolve(&t);

        // 未指定 duration 时，使用调用方的默认值
        assert_eq!(effect.duration_or(0.3), 0.3);
        assert_eq!(effect.duration_or(0.5), 0.5);
        assert_eq!(effect.duration_or(1.0), 1.0);
    }
}
