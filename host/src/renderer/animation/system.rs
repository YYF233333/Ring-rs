//! # System 模块
//!
//! 通用动画系统管理器。
//!
//! 对象实现 `Animatable` trait，系统直接设置属性值：
//! ```rust,ignore
//! let obj_id = system.register(my_object);
//! system.animate_object::<MyObject>(obj_id, "alpha", 0.0, 1.0, 0.3)?;
//! // 值自动应用到对象，无需手动查询
//! ```

use std::any::TypeId;
use std::collections::HashMap;
use std::rc::Rc;

use super::traits::{AnimPropertyKey, Animatable, ObjectId};
use super::{Animation, AnimationEvent, AnimationId, AnimationState, EasingFunction};

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
/// 2. 直接设置对象属性（通过 Animatable trait）
/// 3. 不假设对象类型：对象自己决定如何使用这些值
pub struct AnimationSystem {
    /// 已注册的对象（ObjectId -> 对象）
    objects: HashMap<ObjectId, RegisteredObject>,
    /// 动画（AnimPropertyKey -> Animation）
    animations: HashMap<AnimPropertyKey, Animation>,
    /// 下一个动画 ID
    next_anim_id: u64,
    /// 下一个对象 ID
    next_object_id: u64,
    /// 待处理的事件队列
    events: Vec<AnimationEvent>,
}

impl Default for AnimationSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AnimationSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationSystem")
            .field("objects", &self.objects.len())
            .field("animations", &self.animations.len())
            .finish()
    }
}

impl AnimationSystem {
    /// 创建新的动画系统
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            animations: HashMap::new(),
            next_anim_id: 1,
            next_object_id: 1,
            events: Vec::new(),
        }
    }

    /// 生成下一个动画 ID
    fn next_animation_id(&mut self) -> AnimationId {
        let id = AnimationId::new(self.next_anim_id);
        self.next_anim_id += 1;
        id
    }

    /// 生成下一个对象 ID
    fn next_object_id(&mut self) -> ObjectId {
        let id = ObjectId::new(self.next_object_id);
        self.next_object_id += 1;
        id
    }

    // ========== 对象管理 ==========

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
        self.animations.retain(|key, _| key.object_id != object_id);
    }

    /// 检查对象是否已注册
    pub fn is_registered(&self, object_id: ObjectId) -> bool {
        self.objects.contains_key(&object_id)
    }

    /// 获取已注册对象数量
    pub fn registered_count(&self) -> usize {
        self.objects.len()
    }

    // ========== 动画控制 ==========

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
        self.animations.remove(&key);

        // 设置初始值
        registered.object.set_property(property_id, from);

        // 创建动画（使用内部键作为标识）
        let anim_id = self.next_animation_id();
        let animation = Animation::new_internal(anim_id, from, to, duration);

        self.animations.insert(key, animation);
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
        self.animations.remove(&key);

        // 设置初始值
        registered.object.set_property(property_id, from);

        // 创建动画
        let anim_id = self.next_animation_id();
        let animation = Animation::new_internal(anim_id, from, to, duration).with_easing(easing);

        self.animations.insert(key, animation);
        self.events.push(AnimationEvent::Started(anim_id));

        Ok(anim_id)
    }

    /// 更新所有动画
    ///
    /// # 返回
    /// 返回产生的事件列表
    pub fn update(&mut self, dt: f32) -> Vec<AnimationEvent> {
        let mut completed: Vec<AnimPropertyKey> = Vec::new();

        for (key, animation) in &mut self.animations {
            if animation.is_active() {
                animation.update(dt);
                // 直接更新对象属性
                if let Some(registered) = self.objects.get(&key.object_id) {
                    registered
                        .object
                        .set_property(key.property_id, animation.current_value());
                }
            }

            // 检查是否已结束
            if animation.is_finished() {
                completed.push(key.clone());
            }
        }

        // 发送完成事件并清理
        for key in completed {
            if let Some(animation) = self.animations.get(&key) {
                let event = if animation.state == AnimationState::Skipped {
                    AnimationEvent::Skipped(animation.id)
                } else {
                    AnimationEvent::Completed(animation.id)
                };
                self.events.push(event);
            }
            self.animations.remove(&key);
        }

        // 返回并清空事件队列
        std::mem::take(&mut self.events)
    }

    /// 跳过所有动画
    pub fn skip_all(&mut self) {
        for (key, animation) in &mut self.animations {
            if animation.is_active() {
                animation.skip();
                // 应用最终值
                if let Some(registered) = self.objects.get(&key.object_id) {
                    registered
                        .object
                        .set_property(key.property_id, animation.final_value());
                }
            }
        }
    }

    /// 跳过对象的所有动画
    pub fn skip_object_animations(&mut self, object_id: ObjectId) {
        for (key, animation) in self.animations.iter_mut() {
            if key.object_id == object_id && animation.is_active() {
                animation.skip();
                // 应用最终值
                if let Some(registered) = self.objects.get(&object_id) {
                    registered
                        .object
                        .set_property(key.property_id, animation.final_value());
                }
            }
        }
    }

    // ========== 查询方法 ==========

    /// 检查是否有活跃的动画
    pub fn has_active_animations(&self) -> bool {
        self.animations.values().any(|a| a.is_active())
    }

    /// 检查对象是否有活跃的动画
    pub fn has_object_animations(&self, object_id: ObjectId) -> bool {
        self.animations
            .iter()
            .any(|(key, anim)| key.object_id == object_id && anim.is_active())
    }

    /// 获取活跃动画数量
    pub fn active_count(&self) -> usize {
        self.animations.values().filter(|a| a.is_active()).count()
    }

    /// 获取动画的当前进度（0.0 - 1.0）
    pub fn get_progress(&self, id: AnimationId) -> Option<f32> {
        self.animations
            .values()
            .find(|a| a.id == id)
            .map(|a| a.progress)
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

    /// 清空所有动画和状态
    pub fn clear(&mut self) {
        self.animations.clear();
        self.events.clear();
        // 注意：不清除已注册的对象，它们仍然有效
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

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
    fn test_system_creation() {
        let system = AnimationSystem::new();
        assert_eq!(system.active_count(), 0);
        assert!(!system.has_active_animations());
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
        system
            .animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0)
            .unwrap();
        system
            .animate_object::<TestAnimatable>(id, "scale", 1.0, 2.0, 1.0)
            .unwrap();

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
        system
            .animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0)
            .unwrap();
        system.update(0.1);

        // 跳过动画
        system.skip_object_animations(id);

        // 值应该立即变为最终值
        assert_eq!(obj.alpha(), 1.0);
    }

    #[test]
    fn test_skip_all() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj.clone());

        // 启动动画
        system
            .animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0)
            .unwrap();
        system.update(0.1);

        // skip_all 应该跳过所有动画
        system.skip_all();

        assert_eq!(obj.alpha(), 1.0);
    }

    #[test]
    fn test_has_active_animations() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);

        // 启动动画
        system
            .animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0)
            .unwrap();

        assert!(system.has_active_animations());
        assert!(system.has_object_animations(id));
    }

    #[test]
    fn test_unregister_removes_animations() {
        let mut system = AnimationSystem::new();
        let obj = Rc::new(TestAnimatable::new());

        let id = system.register(obj);

        // 启动动画
        system
            .animate_object::<TestAnimatable>(id, "alpha", 0.0, 1.0, 1.0)
            .unwrap();
        assert!(system.has_object_animations(id));

        // 注销对象应该移除其动画
        system.unregister(id);
        assert!(!system.has_object_animations(id));
    }
}
