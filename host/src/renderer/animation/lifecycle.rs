//! 动画生命周期：跳过、注销、批量清理等。

use super::system::AnimationSystem;
use super::traits::ObjectId;

impl AnimationSystem {
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

    /// 注销对象
    ///
    /// 移除对象及其所有相关动画。
    pub fn unregister(&mut self, object_id: ObjectId) {
        self.objects.remove(&object_id);
        // 移除该对象的所有动画
        self.animations.retain(|key, _| key.object_id != object_id);
    }

    /// 清空所有动画和状态
    pub fn clear(&mut self) {
        self.animations.clear();
        self.events.clear();
        // 注意：不清除已注册的对象，它们仍然有效
    }
}
