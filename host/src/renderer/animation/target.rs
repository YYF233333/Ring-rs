//! # Target 模块
//!
//! 动画目标抽象，标识可以被动画的对象。

/// 动画目标
///
/// 标识一个可以被动画系统控制的对象。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimationTarget {
    /// 背景
    Background,

    /// 角色立绘
    Character {
        /// 角色别名
        alias: String,
    },

    /// 场景遮罩（用于场景切换）
    SceneMask,

    /// UI 元素
    UIElement {
        /// UI 元素 ID
        id: String,
    },

    /// 自定义目标（用于扩展）
    Custom {
        /// 自定义标识
        id: String,
    },
}

impl AnimationTarget {
    /// 创建背景目标
    pub fn background() -> Self {
        Self::Background
    }

    /// 创建角色目标
    pub fn character(alias: impl Into<String>) -> Self {
        Self::Character {
            alias: alias.into(),
        }
    }

    /// 创建场景遮罩目标
    pub fn scene_mask() -> Self {
        Self::SceneMask
    }

    /// 创建 UI 元素目标
    pub fn ui_element(id: impl Into<String>) -> Self {
        Self::UIElement { id: id.into() }
    }

    /// 创建自定义目标
    pub fn custom(id: impl Into<String>) -> Self {
        Self::Custom { id: id.into() }
    }

    /// 获取目标的描述字符串（用于调试）
    pub fn description(&self) -> String {
        match self {
            Self::Background => "Background".to_string(),
            Self::Character { alias } => format!("Character({})", alias),
            Self::SceneMask => "SceneMask".to_string(),
            Self::UIElement { id } => format!("UIElement({})", id),
            Self::Custom { id } => format!("Custom({})", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_equality() {
        let t1 = AnimationTarget::character("alice");
        let t2 = AnimationTarget::character("alice");
        let t3 = AnimationTarget::character("bob");

        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
    }

    #[test]
    fn test_target_hash() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        map.insert(AnimationTarget::background(), 1);
        map.insert(AnimationTarget::character("alice"), 2);

        assert_eq!(map.get(&AnimationTarget::background()), Some(&1));
        assert_eq!(map.get(&AnimationTarget::character("alice")), Some(&2));
    }
}
