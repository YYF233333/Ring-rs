//! # Effect Registry
//!
//! 效果类型定义与默认参数。
//! 这是所有效果名称、默认值的**唯一来源**。

/// 效果类型
///
/// 标识一个过渡/动画效果的类型及其关联参数。
/// 由 [`resolve`](super::resolve) 从 `vn_runtime::command::Transition` 解析得到。
///
/// ## 语义说明
///
/// - `Dissolve`：alpha 交叉淡化（背景/立绘通用）
/// - `Fade`：黑屏遮罩过渡（仅 changeScene）；在立绘上下文中当作 Dissolve 处理
/// - `FadeWhite`：白屏遮罩过渡（仅 changeScene）
/// - `Rule`：图片遮罩过渡（changeScene/showBackground）
/// - `Move`：位置移动动画（仅立绘）
/// - `None`：无效果（瞬间切换）
#[derive(Debug, Clone, PartialEq)]
pub enum EffectKind {
    /// 无效果（瞬间切换）
    None,
    /// Alpha 交叉淡化（溶解）
    ///
    /// 适用于：背景过渡、立绘显示/隐藏
    Dissolve,
    /// 黑屏遮罩过渡
    ///
    /// 适用于：场景切换（changeScene）；在立绘上下文中等价于 Dissolve
    Fade,
    /// 白屏遮罩过渡
    ///
    /// 适用于：场景切换（changeScene）
    FadeWhite,
    /// 图片遮罩过渡（Rule）
    ///
    /// 适用于：场景切换（changeScene）
    Rule {
        /// 遮罩图片路径（原始路径，未经 ResourceManager 规范化）
        mask_path: String,
        /// 是否反向
        reversed: bool,
    },
    /// 位置移动动画
    ///
    /// 适用于：立绘位置变更（`show alias at pos with move`）
    Move,
}

/// 各效果的默认持续时间（秒）
///
/// 这些常量是效果参数的**唯一来源**，任何需要默认持续时间的地方
/// 都应使用这些常量，而非硬编码数字。
pub mod defaults {
    /// Dissolve（交叉淡化）默认时长
    pub const DISSOLVE_DURATION: f32 = 0.3;
    /// Fade（黑屏遮罩）默认时长
    pub const FADE_DURATION: f32 = 0.5;
    /// FadeWhite（白屏遮罩）默认时长
    pub const FADE_WHITE_DURATION: f32 = 0.5;
    /// Rule（图片遮罩）默认时长
    pub const RULE_DURATION: f32 = 0.5;
    /// Move（位置移动）默认时长
    pub const MOVE_DURATION: f32 = 0.3;
    /// 立绘 alpha 动画（dissolve/fade 在立绘上下文）默认时长
    ///
    /// 注意：`fade` 在立绘上下文中被视为 dissolve，使用此默认值
    pub const CHARACTER_ALPHA_DURATION: f32 = DISSOLVE_DURATION;
    /// 背景过渡（dissolve）默认时长
    pub const BACKGROUND_DISSOLVE_DURATION: f32 = DISSOLVE_DURATION;
}
