//! EngineServices trait — 扩展访问核心系统的受控接口。
//!
//! 定义在 extensions 模块中，由 app::CoreSystems 实现，
//! 使 extensions 不再直接依赖 app 模块。

use crate::renderer::animation::EasingFunction;
use crate::renderer::animation::ObjectId;
use crate::renderer::character_animation::AnimatableCharacter;
use crate::renderer::effects::ResolvedEffect;

/// 扩展可访问的引擎核心能力。
///
/// 方法基于 builtin_effects.rs 的实际访问模式提取，将泛型动画 API
/// 固化为角色专用方法，保持 trait object-safe。
pub trait EngineServices {
    // -- 角色注册与查询 --

    /// 查询角色别名对应的动画系统 ObjectId。
    fn get_character_object_id(&self, alias: &str) -> Option<ObjectId>;

    /// 查询角色别名对应的动画状态（克隆）。
    fn get_character_anim(&self, alias: &str) -> Option<AnimatableCharacter>;

    /// 确保角色已注册到动画系统，返回 ObjectId。
    /// 如果已注册则直接返回，否则注册并记录映射。
    fn ensure_character_registered(
        &mut self,
        alias: &str,
        character: &AnimatableCharacter,
    ) -> ObjectId;

    // -- 角色动画 --

    /// 启动角色属性动画（线性缓动）。
    fn animate_character(
        &mut self,
        id: ObjectId,
        property: &'static str,
        from: f32,
        to: f32,
        duration: f32,
    ) -> Result<(), String>;

    /// 启动角色属性动画（指定缓动函数）。
    fn animate_character_with_easing(
        &mut self,
        id: ObjectId,
        property: &'static str,
        from: f32,
        to: f32,
        duration: f32,
        easing: EasingFunction,
    ) -> Result<(), String>;

    // -- 背景/场景过渡 --

    /// 启动背景 dissolve 过渡。
    fn start_background_transition(&mut self, old_bg: Option<String>, effect: &ResolvedEffect);

    /// 启动场景 fade（黑屏过渡）。
    fn start_scene_fade(&mut self, duration: f32, pending_bg: String);

    /// 启动场景 fade white（白屏过渡）。
    fn start_scene_fade_white(&mut self, duration: f32, pending_bg: String);

    /// 启动场景 rule mask 过渡。
    fn start_scene_rule(&mut self, duration: f32, pending_bg: String, mask: String, reversed: bool);

    /// 启动震动效果。
    fn start_shake(&mut self, amplitude_x: f32, amplitude_y: f32, duration: f32);

    /// 启动模糊过渡。
    fn start_blur_transition(&mut self, from: f32, to: f32, duration: f32);

    // -- 场景状态 --

    /// 获取屏幕尺寸 (width, height)。
    fn screen_size(&self) -> (f32, f32);

    /// 获取场景模糊值的可变引用。
    fn scene_blur_amount_mut(&mut self) -> &mut f32;

    /// 获取场景暗化值的可变引用。
    fn scene_dim_level_mut(&mut self) -> &mut f32;
}
