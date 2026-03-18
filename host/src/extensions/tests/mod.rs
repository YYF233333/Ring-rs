mod high_value;
mod low_value;

use super::*;

use crate::manifest::Manifest;
use crate::renderer::animation::{EasingFunction, ObjectId};
use crate::renderer::character_animation::AnimatableCharacter;
use crate::renderer::effects::{
    EffectKind, EffectParamValue, EffectRequest, EffectTarget, ResolvedEffect,
};

// ============ 测试辅助 ============

#[derive(Debug)]
struct DummyExtension {
    manifest: ExtensionManifest,
}

impl EffectExtension for DummyExtension {
    fn manifest(&self) -> &ExtensionManifest {
        &self.manifest
    }

    fn on_request(
        &self,
        _request: &crate::renderer::effects::EffectRequest,
        _ctx: &mut EngineContext<'_>,
    ) -> Result<(), ExtensionError> {
        Ok(())
    }
}

#[derive(Debug)]
struct FailingExtension {
    manifest: ExtensionManifest,
}

impl EffectExtension for FailingExtension {
    fn manifest(&self) -> &ExtensionManifest {
        &self.manifest
    }

    fn on_request(
        &self,
        request: &crate::renderer::effects::EffectRequest,
        _ctx: &mut EngineContext<'_>,
    ) -> Result<(), ExtensionError> {
        Err(ExtensionError::Runtime {
            capability_id: request.capability_id.clone(),
            message: "deliberate failure".to_string(),
        })
    }
}

fn make_manifest() -> ExtensionManifest {
    ExtensionManifest {
        name: "test.ext".to_string(),
        version: "0.1.0".to_string(),
        engine_api_version: "1.0.0".to_string(),
        capabilities: vec!["effect.test".to_string()],
        dependencies: Vec::new(),
    }
}

fn dummy_resolved_effect() -> ResolvedEffect {
    ResolvedEffect {
        kind: EffectKind::None,
        duration: Some(0.5),
        easing: EasingFunction::Linear,
    }
}

/// Mock EngineServices 用于单元测试
#[derive(Default)]
struct MockEngineServices {
    scene_blur: f32,
    scene_dim: f32,
    start_shake_calls: Vec<(f32, f32, f32)>,
    start_blur_calls: Vec<(f32, f32, f32)>,
    start_scene_fade_calls: Vec<(f32, String)>,
    start_scene_fade_white_calls: Vec<(f32, String)>,
    start_background_transition_called: bool,
    start_scene_rule_calls: Vec<(f32, String, String, bool)>,
    screen_size: (f32, f32),
    animate_character_with_easing_calls: Vec<(&'static str, f32, f32, f32)>,
    /// 如果非 None，则 get_character_anim 返回该角色
    character_to_return: Option<AnimatableCharacter>,
}

impl EngineServices for MockEngineServices {
    fn get_character_object_id(&self, _alias: &str) -> Option<ObjectId> {
        None
    }

    fn get_character_anim(&self, _alias: &str) -> Option<AnimatableCharacter> {
        self.character_to_return.clone()
    }

    fn ensure_character_registered(
        &mut self,
        _alias: &str,
        _character: &AnimatableCharacter,
    ) -> ObjectId {
        ObjectId::new(42)
    }

    fn animate_character(
        &mut self,
        _id: ObjectId,
        _property: &'static str,
        _from: f32,
        _to: f32,
        _duration: f32,
    ) -> Result<(), String> {
        Ok(())
    }

    fn animate_character_with_easing(
        &mut self,
        _id: ObjectId,
        property: &'static str,
        from: f32,
        to: f32,
        duration: f32,
        _easing: EasingFunction,
    ) -> Result<(), String> {
        self.animate_character_with_easing_calls
            .push((property, from, to, duration));
        Ok(())
    }

    fn start_background_transition(&mut self, _old_bg: Option<String>, _effect: &ResolvedEffect) {
        self.start_background_transition_called = true;
    }

    fn start_scene_fade(&mut self, duration: f32, pending_bg: String) {
        self.start_scene_fade_calls.push((duration, pending_bg));
    }

    fn start_scene_fade_white(&mut self, duration: f32, pending_bg: String) {
        self.start_scene_fade_white_calls
            .push((duration, pending_bg));
    }

    fn start_scene_rule(
        &mut self,
        duration: f32,
        pending_bg: String,
        mask: String,
        reversed: bool,
    ) {
        self.start_scene_rule_calls
            .push((duration, pending_bg, mask, reversed));
    }

    fn start_shake(&mut self, amplitude_x: f32, amplitude_y: f32, duration: f32) {
        self.start_shake_calls
            .push((amplitude_x, amplitude_y, duration));
    }

    fn start_blur_transition(&mut self, from: f32, to: f32, duration: f32) {
        self.start_blur_calls.push((from, to, duration));
    }

    fn screen_size(&self) -> (f32, f32) {
        self.screen_size
    }

    fn scene_blur_amount_mut(&mut self) -> &mut f32 {
        &mut self.scene_blur
    }

    fn scene_dim_level_mut(&mut self) -> &mut f32 {
        &mut self.scene_dim
    }
}

fn make_engine_context<'a>(
    svc: &'a mut MockEngineServices,
    manifest: &'a Manifest,
) -> EngineContext<'a> {
    EngineContext::new(svc, manifest)
}

// ============ 内建效果测试 helper（供 high_value 使用） ============

fn builtin_registry() -> ExtensionRegistry {
    build_builtin_registry(ENGINE_API_VERSION).expect("builtin registry")
}

/// 使用默认 Manifest 与 MockEngineServices 对内建 registry 分发请求，返回 (结果, svc)。
fn dispatch_builtin(request: &EffectRequest) -> (CapabilityDispatchResult, MockEngineServices) {
    let registry = builtin_registry();
    let manifest = Manifest::with_defaults();
    let mut svc = MockEngineServices::default();
    let mut ctx = make_engine_context(&mut svc, &manifest);
    let result = registry.dispatch(request, &mut ctx);
    drop(ctx);
    (result, svc)
}
