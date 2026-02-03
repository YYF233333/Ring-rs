//! # System 模块
//!
//! 通用动画系统管理器。
//!
//! 支持两种使用模式：
//!
//! ## 模式 1：值缓存模式（旧 API）
//!
//! 对象通过 PropertyKey 查询当前值，自己决定如何应用：
//! ```rust,ignore
//! let id = system.animate(PropertyKey::character_alpha("alice"), 0.0, 1.0, 0.3);
//! let alpha = system.get_value(&PropertyKey::character_alpha("alice"));
//! ```
//!
//! ## 模式 2：Trait-based 模式（新 API）
//!
//! 对象实现 `Animatable` trait，系统直接设置属性值：
//! ```rust,ignore
//! let obj_id = system.register(my_object);
//! system.animate_object::<MyObject>(obj_id, "alpha", 0.0, 1.0, 0.3);
//! // 值自动应用到对象，无需手动查询
//! ```

use std::any::TypeId;
use std::collections::HashMap;
use std::rc::Rc;

use super::traits::{Animatable, AnimPropertyKey, ObjectId};
use super::{Animation, AnimationEvent, AnimationId, AnimationState, EasingFunction, PropertyKey};

/// 已注册的可动画对象
struct RegisteredObject {
    /// 对象的 trait object
    object: Rc<dyn Animatable>,
    /// 对象的类型 ID
    type_id: TypeId,
}

/// 动画系统
///
/// 管理所有动画实例，提供统一的更新和查询接口。
///
/// ## 设计理念
///
/// 动画系统只负责：
/// 1. 管理时间轴：知道某个属性从 A 到 B 需要在 duration 内变化
/// 2. 维护当前值：通过 PropertyKey 查询当前值（值缓存模式）
/// 3. 或直接设置对象属性（Trait-based 模式）
/// 4. 不假设对象类型：对象自己决定如何使用这些值
pub struct AnimationSystem {
    /// 活跃的动画（正在播放或等待中）
    animations: HashMap<AnimationId, Animation>,
    /// 当前值缓存（PropertyKey -> 当前值）- 用于值缓存模式
    values: HashMap<PropertyKey, f32>,
    /// 下一个动画 ID
    next_id: u64,
    /// 待处理的事件队列
    events: Vec<AnimationEvent>,

    // ===== Trait-based 模式新增字段 =====
    /// 已注册的对象（ObjectId -> 对象）
    objects: HashMap<ObjectId, RegisteredObject>,
    /// 下一个对象 ID
    next_object_id: u64,
    /// Trait-based 动画（AnimPropertyKey -> Animation）
    /// 与 animations 分开存储，避免类型混淆
    object_animations: HashMap<AnimPropertyKey, Animation>,
}

impl Default for AnimationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AnimationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationSystem")
            .field("animations", &self.animations.len())
            .field("values", &self.values.len())
            .field("objects", &self.objects.len())
            .field("object_animations", &self.object_animations.len())
            .finish()
    }
}

impl AnimationSystem {
    /// 创建新的动画系统
    pub fn new() -> Self {
        Self {
            animations: HashMap::new(),
            values: HashMap::new(),
            next_id: 1,
            events: Vec::new(),
            objects: HashMap::new(),
            next_object_id: 1,
            object_animations: HashMap::new(),
        }
    }

    /// 生成下一个动画 ID
    fn next_animation_id(&mut self) -> AnimationId {
        let id = AnimationId::new(self.next_id);
        self.next_id += 1;
        id
    }

    /// 启动动画
    ///
    /// 如果该属性已有动画在进行，会先取消旧动画。
    pub fn start(&mut self, animation: Animation) -> AnimationId {
        let id = animation.id;
        let key = animation.key.clone();

        // 取消同一属性的现有动画
        self.cancel_key_animations(&key);

        // 设置属性的初始值
        self.values.insert(key, animation.initial_value());

        // 添加新动画
        self.animations.insert(id, animation);
        self.events.push(AnimationEvent::Started(id));

        id
    }

    /// 创建并启动新动画
    pub fn animate(
        &mut self,
        key: PropertyKey,
        from: f32,
        to: f32,
        duration: f32,
    ) -> AnimationId {
        let id = self.next_animation_id();
        let animation = Animation::new(id, key, from, to, duration);
        self.start(animation)
    }

    /// 创建并启动新动画（带缓动函数）
    pub fn animate_with_easing(
        &mut self,
        key: PropertyKey,
        from: f32,
        to: f32,
        duration: f32,
        easing: EasingFunction,
    ) -> AnimationId {
        let id = self.next_animation_id();
        let animation = Animation::new(id, key, from, to, duration).with_easing(easing);
        self.start(animation)
    }

    /// 快捷方法：启动淡入动画（0 → 1）
    pub fn fade_in(&mut self, key: PropertyKey, duration: f32) -> AnimationId {
        self.animate(key, 0.0, 1.0, duration)
    }

    /// 快捷方法：启动淡出动画（1 → 0）
    pub fn fade_out(&mut self, key: PropertyKey, duration: f32) -> AnimationId {
        self.animate(key, 1.0, 0.0, duration)
    }

    /// 快捷方法：启动透明度动画
    pub fn animate_alpha(
        &mut self,
        key: PropertyKey,
        from: f32,
        to: f32,
        duration: f32,
    ) -> AnimationId {
        self.animate(key, from, to, duration)
    }

    /// 更新所有动画
    ///
    /// # 返回
    /// 返回产生的事件列表
    pub fn update(&mut self, dt: f32) -> Vec<AnimationEvent> {
        let mut completed = Vec::new();

        // 更新值缓存模式的动画
        for (id, animation) in &mut self.animations {
            if animation.is_active() {
                animation.update(dt);
                // 更新当前值
                self.values
                    .insert(animation.key.clone(), animation.current_value());
            }

            // 检查是否已结束
            if animation.is_finished() {
                completed.push(*id);
            }
        }

        // 发送完成事件并清理（值缓存模式）
        for id in completed {
            if let Some(animation) = self.animations.get(&id) {
                let event = if animation.state == AnimationState::Skipped {
                    AnimationEvent::Skipped(id)
                } else {
                    AnimationEvent::Completed(id)
                };
                self.events.push(event);
            }
            self.animations.remove(&id);
        }

        // 更新 trait-based 模式的动画
        let mut object_completed: Vec<AnimPropertyKey> = Vec::new();

        for (key, animation) in &mut self.object_animations {
            if animation.is_active() {
                animation.update(dt);
                // 直接更新对象属性
                if let Some(registered) = self.objects.get(&key.object_id) {
                    registered.object.set_property(key.property_id, animation.current_value());
                }
            }

            // 检查是否已结束
            if animation.is_finished() {
                object_completed.push(key.clone());
            }
        }

        // 发送完成事件并清理（trait-based 模式）
        for key in object_completed {
            if let Some(animation) = self.object_animations.get(&key) {
                let event = if animation.state == AnimationState::Skipped {
                    AnimationEvent::Skipped(animation.id)
                } else {
                    AnimationEvent::Completed(animation.id)
                };
                self.events.push(event);
            }
            self.object_animations.remove(&key);
        }

        // 返回并清空事件队列
        std::mem::take(&mut self.events)
    }

    /// 跳过指定动画
    pub fn skip(&mut self, id: AnimationId) {
        if let Some(animation) = self.animations.get_mut(&id) {
            animation.skip();
            // 应用最终值
            self.values
                .insert(animation.key.clone(), animation.final_value());
        }
    }

    /// 跳过指定属性的所有动画
    pub fn skip_key(&mut self, key: &PropertyKey) {
        let ids: Vec<_> = self
            .animations
            .iter()
            .filter(|(_, anim)| &anim.key == key)
            .map(|(id, _)| *id)
            .collect();

        for id in ids {
            self.skip(id);
        }
    }

    /// 跳过所有动画（包括值缓存模式和 trait-based 模式）
    pub fn skip_all(&mut self) {
        // 跳过值缓存模式的动画
        let ids: Vec<_> = self.animations.keys().copied().collect();
        for id in ids {
            self.skip(id);
        }

        // 跳过 trait-based 模式的动画
        for (key, animation) in &mut self.object_animations {
            if animation.is_active() {
                animation.skip();
                // 应用最终值
                if let Some(registered) = self.objects.get(&key.object_id) {
                    registered.object.set_property(key.property_id, animation.final_value());
                }
            }
        }
    }

    /// 取消指定属性的所有动画（不应用最终状态）
    pub fn cancel_key_animations(&mut self, key: &PropertyKey) {
        self.animations.retain(|_, anim| &anim.key != key);
    }

    /// 取消指定动画
    pub fn cancel(&mut self, id: AnimationId) {
        self.animations.remove(&id);
    }

    /// 获取属性的当前值
    ///
    /// 如果属性不存在，返回 None
    pub fn get_value(&self, key: &PropertyKey) -> Option<f32> {
        self.values.get(key).copied()
    }

    /// 获取属性的当前值，不存在时返回默认值
    pub fn get_value_or(&self, key: &PropertyKey, default: f32) -> f32 {
        self.values.get(key).copied().unwrap_or(default)
    }

    /// 直接设置属性值（不经过动画）
    pub fn set_value(&mut self, key: PropertyKey, value: f32) {
        self.values.insert(key, value);
    }

    /// 移除属性值
    pub fn remove_value(&mut self, key: &PropertyKey) {
        self.values.remove(key);
    }

    /// 移除属性（清除值和相关动画）
    pub fn remove_key(&mut self, key: &PropertyKey) {
        self.values.remove(key);
        self.animations.retain(|_, anim| &anim.key != key);
    }

    /// 检查是否有活跃的动画（包括值缓存模式和 trait-based 模式）
    pub fn has_active_animations(&self) -> bool {
        self.animations.values().any(|a| a.is_active())
            || self.object_animations.values().any(|a| a.is_active())
    }

    /// 检查指定属性是否有活跃的动画（值缓存模式）
    pub fn has_active_animation(&self, key: &PropertyKey) -> bool {
        self.animations
            .values()
            .any(|a| &a.key == key && a.is_active())
    }

    /// 获取活跃动画数量（包括值缓存模式和 trait-based 模式）
    pub fn active_count(&self) -> usize {
        self.animations.values().filter(|a| a.is_active()).count()
            + self.object_animations.values().filter(|a| a.is_active()).count()
    }

    /// 获取指定动画的状态
    pub fn get_animation(&self, id: AnimationId) -> Option<&Animation> {
        self.animations.get(&id)
    }

    /// 检查动画是否完成
    pub fn is_completed(&self, id: AnimationId) -> bool {
        self.animations
            .get(&id)
            .map(|a| a.is_finished())
            .unwrap_or(true) // 不存在的动画视为已完成
    }

    /// 获取动画的当前进度（0.0 - 1.0）
    pub fn get_progress(&self, id: AnimationId) -> Option<f32> {
        self.animations.get(&id).map(|a| a.progress)
    }

    /// 获取所有当前值（用于调试）
    pub fn get_all_values(&self) -> &HashMap<PropertyKey, f32> {
        &self.values
    }

    /// 获取所有活跃动画（用于调试）
    pub fn get_active_animations(&self) -> Vec<&Animation> {
        self.animations.values().filter(|a| a.is_active()).collect()
    }

    /// 清空所有动画和状态
    pub fn clear(&mut self) {
        self.animations.clear();
        self.values.clear();
        self.events.clear();
        self.object_animations.clear();
        // 注意：不清除已注册的对象，它们仍然有效
    }

    // ========== Trait-based 模式 API ==========

    /// 生成下一个对象 ID
    fn next_object_id(&mut self) -> ObjectId {
        let id = ObjectId::new(self.next_object_id);
        self.next_object_id += 1;
        id
    }

    /// 注册可动画对象
    ///
    /// 系统分配唯一的 `ObjectId` 并返回，后续通过此 ID 引用对象。
    /// 即使同一类型的对象（如同名角色）多次注册，也会获得不同的 ID。
    ///
    /// # 参数
    /// - `object`: 实现了 `Animatable` trait 的对象
    ///
    /// # 返回
    /// 系统分配的唯一 `ObjectId`
    ///
    /// # 示例
    /// ```rust,ignore
    /// let character = Rc::new(Character::new("alice"));
    /// let id = animation_system.register(character);
    /// ```
    pub fn register<T: Animatable>(&mut self, object: Rc<T>) -> ObjectId {
        let id = self.next_object_id();
        let registered = RegisteredObject {
            object: object as Rc<dyn Animatable>,
            type_id: TypeId::of::<T>(),
        };
        self.objects.insert(id, registered);
        id
    }

    /// 注销对象
    ///
    /// 移除对象及其所有相关动画。
    pub fn unregister(&mut self, object_id: ObjectId) {
        self.objects.remove(&object_id);
        // 移除该对象的所有动画
        self.object_animations
            .retain(|key, _| key.object_id != object_id);
    }

    /// 检查对象是否已注册
    pub fn is_registered(&self, object_id: ObjectId) -> bool {
        self.objects.contains_key(&object_id)
    }

    /// 获取已注册对象数量
    pub fn registered_count(&self) -> usize {
        self.objects.len()
    }

    /// 启动对象属性动画
    ///
    /// # 类型参数
    /// - `T`: 对象类型（用于类型安全检查）
    ///
    /// # 参数
    /// - `object_id`: 对象 ID
    /// - `property_id`: 属性名称
    /// - `from`: 起始值
    /// - `to`: 目标值
    /// - `duration`: 动画时长（秒）
    ///
    /// # 返回
    /// - `Ok(AnimationId)`: 动画启动成功
    /// - `Err(String)`: 错误信息
    ///
    /// # 示例
    /// ```rust,ignore
    /// animation_system.animate_object::<Character>(
    ///     obj_id, "alpha", 0.0, 1.0, 0.3
    /// )?;
    /// ```
    pub fn animate_object<T: 'static>(
        &mut self,
        object_id: ObjectId,
        property_id: &'static str,
        from: f32,
        to: f32,
        duration: f32,
    ) -> Result<AnimationId, String> {
        // 验证对象存在且类型匹配
        let registered = self
            .objects
            .get(&object_id)
            .ok_or_else(|| format!("Object {} not registered", object_id))?;

        if registered.type_id != TypeId::of::<T>() {
            return Err(format!(
                "Type mismatch: expected {:?}, got {:?}",
                TypeId::of::<T>(),
                registered.type_id
            ));
        }

        // 验证属性存在
        if registered.object.get_property(property_id).is_none() {
            return Err(format!(
                "Property '{}' not found on object {}",
                property_id, object_id
            ));
        }

        // 创建属性键
        let key = AnimPropertyKey::new::<T>(object_id, property_id);

        // 取消同一属性的现有动画
        self.object_animations.remove(&key);

        // 设置初始值
        registered.object.set_property(property_id, from);

        // 创建动画
        let anim_id = self.next_animation_id();
        let animation = Animation::new(
            anim_id,
            PropertyKey::custom(format!("{}:{}", object_id, property_id)),
            from,
            to,
            duration,
        );

        self.object_animations.insert(key, animation);
        self.events.push(AnimationEvent::Started(anim_id));

        Ok(anim_id)
    }

    /// 启动对象属性动画（带缓动函数）
    pub fn animate_object_with_easing<T: 'static>(
        &mut self,
        object_id: ObjectId,
        property_id: &'static str,
        from: f32,
        to: f32,
        duration: f32,
        easing: EasingFunction,
    ) -> Result<AnimationId, String> {
        // 验证对象存在且类型匹配
        let registered = self
            .objects
            .get(&object_id)
            .ok_or_else(|| format!("Object {} not registered", object_id))?;

        if registered.type_id != TypeId::of::<T>() {
            return Err(format!(
                "Type mismatch: expected {:?}, got {:?}",
                TypeId::of::<T>(),
                registered.type_id
            ));
        }

        // 验证属性存在
        if registered.object.get_property(property_id).is_none() {
            return Err(format!(
                "Property '{}' not found on object {}",
                property_id, object_id
            ));
        }

        // 创建属性键
        let key = AnimPropertyKey::new::<T>(object_id, property_id);

        // 取消同一属性的现有动画
        self.object_animations.remove(&key);

        // 设置初始值
        registered.object.set_property(property_id, from);

        // 创建动画
        let anim_id = self.next_animation_id();
        let animation = Animation::new(
            anim_id,
            PropertyKey::custom(format!("{}:{}", object_id, property_id)),
            from,
            to,
            duration,
        )
        .with_easing(easing);

        self.object_animations.insert(key, animation);
        self.events.push(AnimationEvent::Started(anim_id));

        Ok(anim_id)
    }

    /// 获取对象属性的当前值
    pub fn get_object_property<T: 'static>(
        &self,
        object_id: ObjectId,
        property_id: &str,
    ) -> Option<f32> {
        let registered = self.objects.get(&object_id)?;

        if registered.type_id != TypeId::of::<T>() {
            return None;
        }

        registered.object.get_property(property_id)
    }

    /// 直接设置对象属性值（不经过动画）
    pub fn set_object_property<T: 'static>(
        &mut self,
        object_id: ObjectId,
        property_id: &str,
        value: f32,
    ) -> bool {
        if let Some(registered) = self.objects.get(&object_id) {
            if registered.type_id == TypeId::of::<T>() {
                return registered.object.set_property(property_id, value);
            }
        }
        false
    }

    /// 检查对象是否有活跃的动画
    pub fn has_object_animations(&self, object_id: ObjectId) -> bool {
        self.object_animations
            .iter()
            .any(|(key, anim)| key.object_id == object_id && anim.is_active())
    }

    /// 跳过对象的所有动画
    pub fn skip_object_animations(&mut self, object_id: ObjectId) {
        for (key, animation) in self.object_animations.iter_mut() {
            if key.object_id == object_id && animation.is_active() {
                animation.skip();
                // 应用最终值
                if let Some(registered) = self.objects.get(&object_id) {
                    registered.object.set_property(key.property_id, animation.final_value());
                }
            }
        }
    }

    // ========== 便捷方法：角色动画 ==========

    /// 角色淡入
    pub fn character_fade_in(&mut self, alias: &str, duration: f32) -> AnimationId {
        self.fade_in(PropertyKey::character_alpha(alias), duration)
    }

    /// 角色淡出
    pub fn character_fade_out(&mut self, alias: &str, duration: f32) -> AnimationId {
        self.fade_out(PropertyKey::character_alpha(alias), duration)
    }

    /// 获取角色透明度
    pub fn get_character_alpha(&self, alias: &str) -> f32 {
        self.get_value_or(&PropertyKey::character_alpha(alias), 1.0)
    }

    /// 设置角色透明度（直接设置，不动画）
    pub fn set_character_alpha(&mut self, alias: &str, alpha: f32) {
        self.set_value(PropertyKey::character_alpha(alias), alpha);
    }

    /// 移除角色相关的所有值和动画
    pub fn remove_character(&mut self, alias: &str) {
        // 移除所有与该角色相关的键
        let keys_to_remove: Vec<_> = self
            .values
            .keys()
            .filter(|k| k.character_alias() == Some(alias))
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.remove_key(&key);
        }
    }

    // ========== 便捷方法：背景动画 ==========

    /// 背景淡入
    pub fn background_fade_in(&mut self, duration: f32) -> AnimationId {
        self.fade_in(PropertyKey::background_alpha(), duration)
    }

    /// 背景淡出
    pub fn background_fade_out(&mut self, duration: f32) -> AnimationId {
        self.fade_out(PropertyKey::background_alpha(), duration)
    }

    /// 获取背景透明度
    pub fn get_background_alpha(&self) -> f32 {
        self.get_value_or(&PropertyKey::background_alpha(), 1.0)
    }

    /// 获取旧背景透明度
    pub fn get_old_background_alpha(&self) -> f32 {
        self.get_value_or(&PropertyKey::old_background_alpha(), 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_system_creation() {
        let system = AnimationSystem::new();
        assert_eq!(system.active_count(), 0);
        assert!(!system.has_active_animations());
    }

    #[test]
    fn test_animate() {
        let mut system = AnimationSystem::new();
        let key = PropertyKey::character_alpha("alice");

        let _id = system.fade_in(key.clone(), 1.0);
        assert_eq!(system.active_count(), 1);
        assert!(system.has_active_animation(&key));

        // 初始透明度应该是 0
        let alpha = system.get_value(&key);
        assert_eq!(alpha, Some(0.0));
    }

    #[test]
    fn test_update() {
        let mut system = AnimationSystem::new();
        let key = PropertyKey::character_alpha("alice");

        system.fade_in(key.clone(), 1.0);

        // 更新半程
        system.update(0.1); // 进入 Playing
        system.update(0.4);
        let alpha = system.get_value(&key).unwrap();
        assert!(alpha > 0.0);
        assert!(alpha < 1.0);

        // 更新完成
        let events = system.update(0.6);
        assert!(events
            .iter()
            .any(|e| matches!(e, AnimationEvent::Completed(_))));

        let alpha = system.get_value(&key).unwrap();
        assert_eq!(alpha, 1.0);
    }

    #[test]
    fn test_skip() {
        let mut system = AnimationSystem::new();
        let key = PropertyKey::character_alpha("alice");

        let id = system.fade_in(key.clone(), 1.0);
        system.update(0.1); // 进入 Playing

        system.skip(id);
        let events = system.update(0.0);

        assert!(events
            .iter()
            .any(|e| matches!(e, AnimationEvent::Skipped(_))));

        let alpha = system.get_value(&key).unwrap();
        assert_eq!(alpha, 1.0);
    }

    #[test]
    fn test_skip_all() {
        let mut system = AnimationSystem::new();

        system.character_fade_in("alice", 1.0);
        system.character_fade_in("bob", 1.0);
        system.update(0.1); // 进入 Playing

        system.skip_all();
        system.update(0.0);

        assert_eq!(system.active_count(), 0);
    }

    #[test]
    fn test_replace_animation() {
        let mut system = AnimationSystem::new();
        let key = PropertyKey::character_alpha("alice");

        // 启动淡入
        system.fade_in(key.clone(), 1.0);
        system.update(0.5);

        // 启动新的淡出（会取消淡入）
        system.fade_out(key.clone(), 1.0);

        // 应该只有一个动画
        assert_eq!(system.active_count(), 1);
    }

    #[test]
    fn test_remove_character() {
        let mut system = AnimationSystem::new();

        system.character_fade_in("alice", 1.0);
        system.set_value(PropertyKey::character_position_x("alice"), 100.0);
        system.update(0.5);

        system.remove_character("alice");

        assert_eq!(system.active_count(), 0);
        assert!(system.get_value(&PropertyKey::character_alpha("alice")).is_none());
        assert!(system
            .get_value(&PropertyKey::character_position_x("alice"))
            .is_none());
    }

    #[test]
    fn test_get_value_or() {
        let system = AnimationSystem::new();
        let key = PropertyKey::character_alpha("alice");

        // 不存在的值返回默认值
        assert_eq!(system.get_value_or(&key, 0.5), 0.5);
    }

    // ========== Trait-based API 测试 ==========

    /// 测试用的可动画对象
    struct TestAnimatable {
        alpha: Rc<RefCell<f32>>,
        scale: Rc<RefCell<f32>>,
    }

    impl TestAnimatable {
        fn new() -> Self {
            Self {
                alpha: Rc::new(RefCell::new(1.0)),
                scale: Rc::new(RefCell::new(1.0)),
            }
        }

        fn alpha(&self) -> f32 {
            *self.alpha.borrow()
        }

        fn scale(&self) -> f32 {
            *self.scale.borrow()
        }
    }

    impl Animatable for TestAnimatable {
        fn get_property(&self, property_id: &str) -> Option<f32> {
            match property_id {
                "alpha" => Some(*self.alpha.borrow()),
                "scale" => Some(*self.scale.borrow()),
                _ => None,
            }
        }

        fn set_property(&self, property_id: &str, value: f32) -> bool {
            match property_id {
                "alpha" => {
                    *self.alpha.borrow_mut() = value;
                    true
                }
                "scale" => {
                    *self.scale.borrow_mut() = value;
                    true
                }
                _ => false,
            }
        }

        fn property_list(&self) -> &'static [&'static str] {
            &["alpha", "scale"]
        }
    }

    #[test]
    fn test_register_object() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);
        assert!(system.is_registered(id));
        assert_eq!(system.registered_count(), 1);
    }

    #[test]
    fn test_register_multiple_objects() {
        let mut system = AnimationSystem::new();
        let obj1 = Rc::new(TestAnimatable::new());
        let obj2 = Rc::new(TestAnimatable::new());

        let id1 = system.register(obj1);
        let id2 = system.register(obj2);

        // 两个对象应该有不同的 ID
        assert_ne!(id1, id2);
        assert_eq!(system.registered_count(), 2);
    }

    #[test]
    fn test_unregister_object() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);
        assert!(system.is_registered(id));

        system.unregister(id);
        assert!(!system.is_registered(id));
        assert_eq!(system.registered_count(), 0);
    }

    #[test]
    fn test_animate_object() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj.clone());

        // 启动动画
        let result = system.animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0);
        assert!(result.is_ok());

        // 初始值应该被设置
        assert_eq!(obj.alpha(), 0.0);

        // 更新动画
        system.update(0.1); // 进入 Playing
        system.update(0.4);

        // 值应该在 0 和 1 之间
        let alpha = obj.alpha();
        assert!(alpha > 0.0);
        assert!(alpha < 1.0);

        // 完成动画
        system.update(0.6);
        assert_eq!(obj.alpha(), 1.0);
    }

    #[test]
    fn test_animate_object_type_mismatch() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);

        // 使用错误的类型应该失败
        struct OtherType;
        let result = system.animate_object::<OtherType>(id, "alpha", 0.0, 1.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_animate_object_invalid_property() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);

        // 使用不存在的属性应该失败
        let result = system.animate_object::<TestAnimatable>(id, "unknown", 0.0, 1.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_animate_object_invalid_id() {
        let mut system = AnimationSystem::new();

        // 使用无效的对象 ID 应该失败
        let invalid_id = ObjectId::new(999);
        let result = system.animate_object::<TestAnimatable>(invalid_id, "alpha", 0.0, 1.0, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_animate_multiple_properties() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj.clone());

        // 同时动画多个属性
        system.animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0).unwrap();
        system.animate_object::<TestAnimatable>(id, "scale", 1.0, 2.0, 1.0).unwrap();

        // 更新动画
        system.update(0.1);
        system.update(0.4);

        // 两个属性都应该在变化
        let alpha = obj.alpha();
        let scale = obj.scale();
        assert!(alpha > 0.0 && alpha < 1.0);
        assert!(scale > 1.0 && scale < 2.0);

        // 完成动画
        system.update(0.6);
        assert_eq!(obj.alpha(), 1.0);
        assert_eq!(obj.scale(), 2.0);
    }

    #[test]
    fn test_skip_object_animations() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj.clone());

        // 启动动画
        system.animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0).unwrap();
        system.update(0.1);

        // 跳过动画
        system.skip_object_animations(id);

        // 值应该立即变为最终值
        assert_eq!(obj.alpha(), 1.0);
    }

    #[test]
    fn test_skip_all_includes_object_animations() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj.clone());

        // 启动两种类型的动画
        system.character_fade_in("alice", 1.0);
        system.animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0).unwrap();
        system.update(0.1);

        // skip_all 应该跳过所有类型的动画
        system.skip_all();

        assert_eq!(obj.alpha(), 1.0);
        assert_eq!(system.get_character_alpha("alice"), 1.0);
    }

    #[test]
    fn test_has_active_animations_includes_objects() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);

        // 启动对象动画
        system.animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0).unwrap();

        assert!(system.has_active_animations());
        assert!(system.has_object_animations(id));
    }

    #[test]
    fn test_unregister_removes_animations() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);

        // 启动动画
        system.animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0).unwrap();
        assert!(system.has_object_animations(id));

        // 注销对象应该移除其动画
        system.unregister(id);
        assert!(!system.has_object_animations(id));
    }
}
