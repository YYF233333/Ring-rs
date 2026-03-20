//! 只读查询：注册状态、活跃动画、进度与属性值。

use super::AnimationId;
use super::system::AnimationSystem;
use super::traits::ObjectId;

impl AnimationSystem {
    /// 检查对象是否已注册
    pub fn is_registered(&self, object_id: ObjectId) -> bool {
        self.objects.contains_key(&object_id)
    }

    /// 获取已注册对象数量
    pub fn registered_count(&self) -> usize {
        self.objects.len()
    }

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
        use std::any::TypeId;

        let registered = self.objects.get(&object_id)?;

        if registered.type_id != TypeId::of::<T>() {
            return None;
        }

        registered.object.get_property(property_id)
    }
}
