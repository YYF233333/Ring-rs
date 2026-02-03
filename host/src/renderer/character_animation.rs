//! # Character Animation 模块
//!
//! 可动画的角色状态实现，使用 `Animatable` trait。
//!
//! ## 设计说明
//!
//! `AnimatableCharacter` 使用 `Rc<RefCell<T>>` 实现内部可变性，
//! 允许同时动画多个属性而不违反 Rust 的借用规则。

use std::cell::RefCell;
use std::rc::Rc;

use super::animation::Animatable;

/// 可动画角色的内部数据
#[derive(Debug, Clone)]
pub struct CharacterAnimData {
    /// 角色别名（标识符）
    pub alias: String,
    /// 透明度 (0.0 - 1.0)
    pub alpha: f32,
    /// 位置 X（相对于默认位置的偏移）
    pub position_x: f32,
    /// 位置 Y（相对于默认位置的偏移）
    pub position_y: f32,
    /// 缩放 X
    pub scale_x: f32,
    /// 缩放 Y
    pub scale_y: f32,
    /// 旋转角度（弧度）
    pub rotation: f32,
}

impl Default for CharacterAnimData {
    fn default() -> Self {
        Self {
            alias: String::new(),
            alpha: 1.0,
            position_x: 0.0,
            position_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
        }
    }
}

impl CharacterAnimData {
    /// 创建新的角色动画数据
    pub fn new(alias: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            ..Default::default()
        }
    }

    /// 创建透明的角色（用于淡入动画）
    pub fn transparent(alias: impl Into<String>) -> Self {
        Self {
            alias: alias.into(),
            alpha: 0.0,
            ..Default::default()
        }
    }
}

/// 可动画角色
///
/// 使用 `Rc<RefCell<CharacterAnimData>>` 包装内部数据，
/// 允许同时动画多个属性。
///
/// ## 支持的属性
///
/// - `"alpha"`: 透明度 (0.0 - 1.0)
/// - `"position_x"`: X 位置偏移
/// - `"position_y"`: Y 位置偏移
/// - `"scale_x"`: X 缩放
/// - `"scale_y"`: Y 缩放
/// - `"rotation"`: 旋转角度（弧度）
///
/// ## 使用示例
///
/// ```rust,ignore
/// // 创建角色
/// let character = AnimatableCharacter::new("alice");
///
/// // 注册到动画系统
/// let obj_id = animation_system.register(Rc::new(character));
///
/// // 启动淡入动画
/// animation_system.animate_object::<AnimatableCharacter>(
///     obj_id, "alpha", 0.0, 1.0, 0.3
/// )?;
///
/// // 读取当前值
/// let alpha = character.get("alpha");
/// ```
#[derive(Debug, Clone)]
pub struct AnimatableCharacter {
    /// 内部数据（使用 RefCell 实现内部可变性）
    data: Rc<RefCell<CharacterAnimData>>,
}

impl AnimatableCharacter {
    /// 支持的属性列表
    pub const PROPERTIES: &'static [&'static str] = &[
        "alpha",
        "position_x",
        "position_y",
        "scale_x",
        "scale_y",
        "rotation",
    ];

    /// 创建新的可动画角色
    pub fn new(alias: impl Into<String>) -> Self {
        Self {
            data: Rc::new(RefCell::new(CharacterAnimData::new(alias))),
        }
    }

    /// 创建透明的角色（用于淡入动画）
    pub fn transparent(alias: impl Into<String>) -> Self {
        Self {
            data: Rc::new(RefCell::new(CharacterAnimData::transparent(alias))),
        }
    }

    /// 从现有数据创建
    pub fn from_data(data: CharacterAnimData) -> Self {
        Self {
            data: Rc::new(RefCell::new(data)),
        }
    }

    /// 获取角色别名
    pub fn alias(&self) -> String {
        self.data.borrow().alias.clone()
    }

    /// 获取属性值
    pub fn get(&self, property_id: &str) -> Option<f32> {
        let data = self.data.borrow();
        match property_id {
            "alpha" => Some(data.alpha),
            "position_x" => Some(data.position_x),
            "position_y" => Some(data.position_y),
            "scale_x" => Some(data.scale_x),
            "scale_y" => Some(data.scale_y),
            "rotation" => Some(data.rotation),
            _ => None,
        }
    }

    /// 设置属性值
    pub fn set(&self, property_id: &str, value: f32) -> bool {
        let mut data = self.data.borrow_mut();
        match property_id {
            "alpha" => {
                data.alpha = value.clamp(0.0, 1.0);
                true
            }
            "position_x" => {
                data.position_x = value;
                true
            }
            "position_y" => {
                data.position_y = value;
                true
            }
            "scale_x" => {
                data.scale_x = value;
                true
            }
            "scale_y" => {
                data.scale_y = value;
                true
            }
            "rotation" => {
                data.rotation = value;
                true
            }
            _ => false,
        }
    }

    /// 获取透明度
    pub fn alpha(&self) -> f32 {
        self.data.borrow().alpha
    }

    /// 设置透明度
    pub fn set_alpha(&self, alpha: f32) {
        self.data.borrow_mut().alpha = alpha.clamp(0.0, 1.0);
    }

    /// 获取位置
    pub fn position(&self) -> (f32, f32) {
        let data = self.data.borrow();
        (data.position_x, data.position_y)
    }

    /// 设置位置
    pub fn set_position(&self, x: f32, y: f32) {
        let mut data = self.data.borrow_mut();
        data.position_x = x;
        data.position_y = y;
    }

    /// 获取缩放
    pub fn scale(&self) -> (f32, f32) {
        let data = self.data.borrow();
        (data.scale_x, data.scale_y)
    }

    /// 设置缩放
    pub fn set_scale(&self, x: f32, y: f32) {
        let mut data = self.data.borrow_mut();
        data.scale_x = x;
        data.scale_y = y;
    }

    /// 获取旋转
    pub fn rotation(&self) -> f32 {
        self.data.borrow().rotation
    }

    /// 设置旋转
    pub fn set_rotation(&self, rotation: f32) {
        self.data.borrow_mut().rotation = rotation;
    }

    /// 获取数据引用（用于共享）
    pub fn data_ref(&self) -> Rc<RefCell<CharacterAnimData>> {
        self.data.clone()
    }

    /// 获取完整数据副本
    pub fn snapshot(&self) -> CharacterAnimData {
        self.data.borrow().clone()
    }
}

impl Animatable for AnimatableCharacter {
    fn get_property(&self, property_id: &str) -> Option<f32> {
        self.get(property_id)
    }

    fn set_property(&self, property_id: &str, value: f32) -> bool {
        self.set(property_id, value)
    }

    fn property_list(&self) -> &'static [&'static str] {
        Self::PROPERTIES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animatable_character_creation() {
        let char = AnimatableCharacter::new("alice");
        assert_eq!(char.alias(), "alice");
        assert_eq!(char.alpha(), 1.0);
        assert_eq!(char.position(), (0.0, 0.0));
        assert_eq!(char.scale(), (1.0, 1.0));
        assert_eq!(char.rotation(), 0.0);
    }

    #[test]
    fn test_animatable_character_transparent() {
        let char = AnimatableCharacter::transparent("bob");
        assert_eq!(char.alpha(), 0.0);
    }

    #[test]
    fn test_property_access() {
        let char = AnimatableCharacter::new("test");

        // 测试所有属性
        for prop in AnimatableCharacter::PROPERTIES {
            assert!(char.get(prop).is_some(), "Property {} should exist", prop);
        }

        assert!(char.get("unknown").is_none());
    }

    #[test]
    fn test_property_modification() {
        let char = AnimatableCharacter::new("test");

        assert!(char.set("alpha", 0.5));
        assert_eq!(char.alpha(), 0.5);

        assert!(char.set("position_x", 100.0));
        assert_eq!(char.position().0, 100.0);

        // Alpha 应该被 clamp 到 0-1 范围
        assert!(char.set("alpha", 2.0));
        assert_eq!(char.alpha(), 1.0);

        assert!(char.set("alpha", -1.0));
        assert_eq!(char.alpha(), 0.0);

        // 未知属性返回 false
        assert!(!char.set("unknown", 0.0));
    }

    #[test]
    fn test_animatable_trait() {
        let char = AnimatableCharacter::new("test");

        // 通过 trait 方法访问
        assert_eq!(char.get_property("alpha"), Some(1.0));
        assert!(char.set_property("alpha", 0.3));
        assert_eq!(char.get_property("alpha"), Some(0.3));

        assert_eq!(char.property_list(), AnimatableCharacter::PROPERTIES);
    }

    #[test]
    fn test_data_sharing() {
        let char = AnimatableCharacter::new("test");
        let data_ref = char.data_ref();

        // 修改原始角色
        char.set_alpha(0.5);

        // 共享引用应该看到相同的值
        assert_eq!(data_ref.borrow().alpha, 0.5);

        // 通过共享引用修改
        data_ref.borrow_mut().alpha = 0.8;

        // 原始角色应该看到修改
        assert_eq!(char.alpha(), 0.8);
    }

    #[test]
    fn test_clone() {
        let char1 = AnimatableCharacter::new("test");
        char1.set_alpha(0.5);

        let char2 = char1.clone();

        // 克隆后两者共享相同的内部数据
        assert_eq!(char2.alpha(), 0.5);

        char1.set_alpha(0.3);
        assert_eq!(char2.alpha(), 0.3);
    }

    #[test]
    fn test_snapshot() {
        let char = AnimatableCharacter::new("test");
        char.set_alpha(0.5);
        char.set_position(10.0, 20.0);

        let snapshot = char.snapshot();

        // 修改原始角色
        char.set_alpha(0.8);

        // 快照不受影响
        assert_eq!(snapshot.alpha, 0.5);
        assert_eq!(snapshot.position_x, 10.0);
        assert_eq!(snapshot.position_y, 20.0);
    }
}
