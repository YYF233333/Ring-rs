//! # Traits 模块
//!
//! 基于 Trait 的动画系统核心接口定义。
//!
//! ## 核心概念
//!
//! - `ObjectId`: 由 AnimationSystem 分配的唯一对象标识符
//! - `PropertyAccessor`: 属性访问器接口（getter/setter）
//! - `Animatable`: 可动画对象接口

use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;

/// 对象唯一标识符
///
/// 由 `AnimationSystem` 在对象注册时分配，保证全局唯一。
/// 使用内部计数器生成，不会重复。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId(pub(crate) u64);

impl ObjectId {
    /// 创建新的对象 ID（仅供 AnimationSystem 内部使用）
    pub(crate) fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取内部 ID 值
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ObjectId({})", self.0)
    }
}

/// 属性键（基于 Trait 的版本）
///
/// 使用 `TypeId + ObjectId + property_id` 组合作为唯一键：
/// - `type_id`: 对象类型（编译期确定，用于类型安全检查）
/// - `object_id`: 系统分配的唯一对象标识符
/// - `property_id`: 属性名称（编译期字符串字面量）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnimPropertyKey {
    /// 对象类型 ID（用于类型安全检查）
    pub type_id: TypeId,
    /// 对象实例 ID
    pub object_id: ObjectId,
    /// 属性名称
    pub property_id: &'static str,
}

impl AnimPropertyKey {
    /// 创建属性键
    pub fn new<T: 'static>(object_id: ObjectId, property_id: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            object_id,
            property_id,
        }
    }

    /// 创建属性键（不指定类型）
    pub fn untyped(object_id: ObjectId, property_id: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<()>(),
            object_id,
            property_id,
        }
    }
}

impl std::fmt::Display for AnimPropertyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.object_id, self.property_id)
    }
}

/// 属性访问器接口
///
/// 提供对单个 f32 属性的 getter/setter 访问。
/// 对象为每个可动画属性创建独立的访问器实例。
///
/// ## 设计说明
///
/// 使用 `Rc<RefCell<T>>` 模式实现内部可变性：
/// - 访问器持有 `RefCell` 的不可变引用
/// - 可以同时动画多个属性，无需担心借用冲突
pub trait PropertyAccessor {
    /// 获取当前值
    fn get(&self) -> f32;

    /// 设置新值
    fn set(&mut self, value: f32);
}

/// 可动画对象接口
///
/// 对象通过实现此 trait 声明自己有哪些属性可以被动画。
/// 由 `AnimationSystem` 统一分配唯一标识符，对象无需管理。
///
/// ## 实现示例
///
/// ```rust,ignore
/// struct Character {
///     inner: Rc<RefCell<CharacterData>>,
/// }
///
/// struct CharacterData {
///     alpha: f32,
///     position_x: f32,
///     position_y: f32,
/// }
///
/// impl Animatable for Character {
///     fn get_property(&self, property_id: &str) -> Option<f32> {
///         let data = self.inner.borrow();
///         match property_id {
///             "alpha" => Some(data.alpha),
///             "position_x" => Some(data.position_x),
///             "position_y" => Some(data.position_y),
///             _ => None,
///         }
///     }
///
///     fn set_property(&self, property_id: &str, value: f32) -> bool {
///         let mut data = self.inner.borrow_mut();
///         match property_id {
///             "alpha" => { data.alpha = value; true }
///             "position_x" => { data.position_x = value; true }
///             "position_y" => { data.position_y = value; true }
///             _ => false,
///         }
///     }
///
///     fn property_list(&self) -> &'static [&'static str] {
///         &["alpha", "position_x", "position_y"]
///     }
/// }
/// ```
pub trait Animatable: 'static {
    /// 获取属性的当前值
    ///
    /// # 参数
    /// - `property_id`: 属性名称
    ///
    /// # 返回
    /// - `Some(value)`: 属性存在，返回当前值
    /// - `None`: 属性不存在
    fn get_property(&self, property_id: &str) -> Option<f32>;

    /// 设置属性的新值
    ///
    /// # 参数
    /// - `property_id`: 属性名称
    /// - `value`: 新值
    ///
    /// # 返回
    /// - `true`: 设置成功
    /// - `false`: 属性不存在或设置失败
    fn set_property(&self, property_id: &str, value: f32) -> bool;

    /// 获取所有可动画属性的列表
    ///
    /// 用于调试和验证。
    fn property_list(&self) -> &'static [&'static str];
}

/// 简单的 f32 属性访问器实现
///
/// 使用 `Rc<RefCell<f32>>` 包装单个 f32 值。
#[derive(Debug, Clone)]
pub struct SimplePropertyAccessor {
    value: Rc<RefCell<f32>>,
}

impl SimplePropertyAccessor {
    /// 创建新的属性访问器
    pub fn new(initial_value: f32) -> Self {
        Self {
            value: Rc::new(RefCell::new(initial_value)),
        }
    }

    /// 获取值的引用（用于共享）
    pub fn value_ref(&self) -> Rc<RefCell<f32>> {
        self.value.clone()
    }
}

impl PropertyAccessor for SimplePropertyAccessor {
    fn get(&self) -> f32 {
        *self.value.borrow()
    }

    fn set(&mut self, value: f32) {
        *self.value.borrow_mut() = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id() {
        let id1 = ObjectId::new(1);
        let id2 = ObjectId::new(2);
        let id1_copy = ObjectId::new(1);

        assert_eq!(id1, id1_copy);
        assert_ne!(id1, id2);
        assert_eq!(id1.value(), 1);
    }

    #[test]
    fn test_anim_property_key() {
        struct TestObject;

        let id = ObjectId::new(1);
        let key1 = AnimPropertyKey::new::<TestObject>(id, "alpha");
        let key2 = AnimPropertyKey::new::<TestObject>(id, "alpha");
        let key3 = AnimPropertyKey::new::<TestObject>(id, "position");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_simple_property_accessor() {
        let mut accessor = SimplePropertyAccessor::new(0.5);

        assert_eq!(accessor.get(), 0.5);

        accessor.set(0.8);
        assert_eq!(accessor.get(), 0.8);

        // 测试共享引用
        let shared = accessor.value_ref();
        accessor.set(1.0);
        assert_eq!(*shared.borrow(), 1.0);
    }

    // 测试 Animatable trait 的实现
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
    fn test_animatable_trait() {
        let obj = TestAnimatable::new();

        assert_eq!(obj.get_property("alpha"), Some(1.0));
        assert_eq!(obj.get_property("scale"), Some(1.0));
        assert_eq!(obj.get_property("unknown"), None);

        assert!(obj.set_property("alpha", 0.5));
        assert_eq!(obj.get_property("alpha"), Some(0.5));

        assert!(!obj.set_property("unknown", 0.0));

        assert_eq!(obj.property_list(), &["alpha", "scale"]);
    }
}
