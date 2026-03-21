//! # State 模块
//!
//! 定义 Host 层的核心状态结构。

/// Host 层状态
///
/// 管理 Host 层的所有运行时状态。
#[derive(Debug)]
pub struct HostState {
    /// 是否正在运行
    pub running: bool,
    /// 是否以 headless 模式运行（无窗口/无 GPU）
    pub headless: bool,
    /// 窗口缩放因子（物理像素 / 逻辑像素），录制导出时使用
    pub scale_factor: f64,
    /// headless 模式下由 EguiAction::Exit 设置
    pub exit_requested: bool,
}

#[allow(clippy::new_without_default)]
impl HostState {
    /// 创建新的 Host 状态
    pub fn new(headless: bool) -> Self {
        Self {
            running: true,
            headless,
            scale_factor: 1.0,
            exit_requested: false,
        }
    }

    /// 停止运行
    pub fn stop(&mut self) {
        self.running = false;
    }
}
