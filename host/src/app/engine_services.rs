//! EngineServices trait 在 CoreSystems 上的实现。
//!
//! 将 extensions 模块定义的 trait 桥接到 app 内部的具体子系统，
//! 消除 extensions → app 的反向依赖。

use std::rc::Rc;

use crate::extensions::EngineServices;
use crate::renderer::animation::EasingFunction;
use crate::renderer::animation::ObjectId;
use crate::renderer::character_animation::AnimatableCharacter;
use crate::renderer::effects::ResolvedEffect;

use super::CoreSystems;

impl EngineServices for CoreSystems {
    fn get_character_object_id(&self, alias: &str) -> Option<ObjectId> {
        self.character_object_ids.get(alias).copied()
    }

    fn get_character_anim(&self, alias: &str) -> Option<AnimatableCharacter> {
        self.render_state.get_character_anim(alias).cloned()
    }

    fn ensure_character_registered(
        &mut self,
        alias: &str,
        character: &AnimatableCharacter,
    ) -> ObjectId {
        if let Some(&id) = self.character_object_ids.get(alias) {
            id
        } else {
            let id = self.animation_system.register(Rc::new(character.clone()));
            self.character_object_ids.insert(alias.to_string(), id);
            id
        }
    }

    fn animate_character(
        &mut self,
        id: ObjectId,
        property: &'static str,
        from: f32,
        to: f32,
        duration: f32,
    ) -> Result<(), String> {
        self.animation_system
            .animate_object::<AnimatableCharacter>(id, property, from, to, duration)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn animate_character_with_easing(
        &mut self,
        id: ObjectId,
        property: &'static str,
        from: f32,
        to: f32,
        duration: f32,
        easing: EasingFunction,
    ) -> Result<(), String> {
        self.animation_system
            .animate_object_with_easing::<AnimatableCharacter>(
                id, property, from, to, duration, easing,
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn start_background_transition(&mut self, old_bg: Option<String>, effect: &ResolvedEffect) {
        self.renderer
            .start_background_transition_resolved(old_bg, effect);
    }

    fn start_scene_fade(&mut self, duration: f32, pending_bg: String) {
        self.renderer.start_scene_fade(duration, pending_bg);
    }

    fn start_scene_fade_white(&mut self, duration: f32, pending_bg: String) {
        self.renderer.start_scene_fade_white(duration, pending_bg);
    }

    fn start_scene_rule(
        &mut self,
        duration: f32,
        pending_bg: String,
        mask: String,
        reversed: bool,
    ) {
        self.renderer
            .start_scene_rule(duration, pending_bg, mask, reversed);
    }

    fn start_shake(&mut self, amplitude_x: f32, amplitude_y: f32, duration: f32) {
        self.renderer
            .start_shake(amplitude_x, amplitude_y, duration);
    }

    fn start_blur_transition(&mut self, from: f32, to: f32, duration: f32) {
        self.renderer.start_blur_transition(from, to, duration);
    }

    fn screen_size(&self) -> (f32, f32) {
        (self.renderer.screen_width(), self.renderer.screen_height())
    }

    fn scene_blur_amount_mut(&mut self) -> &mut f32 {
        &mut self.render_state.scene_effect.blur_amount
    }

    fn scene_dim_level_mut(&mut self) -> &mut f32 {
        &mut self.render_state.scene_effect.dim_level
    }
}
