//! # 场景效果与字卡命令执行

use crate::renderer::RenderState;
use crate::renderer::animation::EasingFunction;
use crate::renderer::effects::{
    EffectKind, EffectParamValue, EffectRequest, EffectTarget, ResolvedEffect,
};
use vn_runtime::command::TransitionArg;

use super::CommandExecutor;
use super::types::ExecuteResult;

impl CommandExecutor {
    /// 执行 SceneEffect 命令
    ///
    /// 将场景效果参数解析为 `EffectRequest`，交由 `EffectApplier` 分发。
    pub(super) fn execute_scene_effect(
        &mut self,
        name: &str,
        args: &[(Option<String>, TransitionArg)],
        _render_state: &mut RenderState,
    ) -> ExecuteResult {
        let duration = extract_duration(args);

        let effect = ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: name.to_string(),
            },
            duration,
            easing: EasingFunction::EaseInOut,
        };

        let extra_params = args.iter().filter_map(|(key, val)| {
            let k = key.as_deref()?;
            if k == "duration" {
                return None;
            }
            let v = match val {
                TransitionArg::Number(n) => EffectParamValue::Number(*n as f32),
                TransitionArg::String(s) => EffectParamValue::String(s.clone()),
                TransitionArg::Bool(b) => EffectParamValue::Bool(*b),
            };
            Some((k.to_string(), v))
        });

        self.last_output
            .effect_requests
            .push(EffectRequest::with_extra_params(
                EffectTarget::SceneEffect {
                    effect_name: name.to_string(),
                },
                effect,
                extra_params,
            ));

        ExecuteResult::Ok
    }

    /// 执行 TitleCard 命令
    ///
    /// 设置渲染状态的标题字卡并产出效果请求。
    pub(super) fn execute_title_card(
        &mut self,
        text: &str,
        duration: f64,
        render_state: &mut RenderState,
    ) -> ExecuteResult {
        render_state.title_card = Some(crate::renderer::TitleCardState {
            text: text.to_string(),
            duration: duration as f32,
            elapsed: 0.0,
        });

        let effect = ResolvedEffect {
            kind: EffectKind::SceneEffect {
                name: "titleCard".to_string(),
            },
            duration: Some(duration as f32),
            easing: EasingFunction::EaseInOut,
        };

        self.last_output.effect_requests.push(EffectRequest::new(
            EffectTarget::TitleCard {
                text: text.to_string(),
            },
            effect,
        ));

        ExecuteResult::Ok
    }
}

fn extract_duration(args: &[(Option<String>, TransitionArg)]) -> Option<f32> {
    args.iter()
        .find(|(k, _)| k.as_deref() == Some("duration"))
        .and_then(|(_, v)| match v {
            TransitionArg::Number(n) => Some(*n as f32),
            _ => None,
        })
}
