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
    /// 调试模式
    pub debug_mode: bool,
}

impl HostState {
    /// 创建新的 Host 状态
    pub fn new() -> Self {
        Self {
            running: true,
            debug_mode: false,
        }
    }

    /// 停止运行
    pub fn stop(&mut self) {
        self.running = false;
    }
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}
