/// 场景过渡类型
#[derive(Debug, Clone)]
pub enum SceneTransitionType {
    /// 黑屏淡入淡出
    Fade,
    /// 白屏淡入淡出
    FadeWhite,
    /// 图片遮罩（Rule-based dissolve）
    Rule {
        /// 遮罩图片路径
        mask_path: String,
        /// 是否反向
        reversed: bool,
    },
}

/// 场景过渡阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneTransitionPhase {
    /// 空闲状态
    Idle,
    /// 阶段 1：遮罩淡入 / 旧背景溶解到黑屏
    FadeIn,
    /// 阶段 2：黑屏停顿（仅 Rule 效果）
    Blackout,
    /// 阶段 3：遮罩淡出 / 黑屏溶解到新背景
    FadeOut,
    /// 阶段 4：UI 淡入
    UIFadeIn,
    /// 完成
    Completed,
}
