//! # Toast 提示组件

/// Toast 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    /// 普通信息
    Info,
    /// 成功
    Success,
    /// 警告
    Warning,
    /// 错误
    Error,
}

/// 单个 Toast 消息
#[derive(Debug, Clone)]
pub struct Toast {
    /// 消息内容
    pub message: String,
    /// 类型
    pub toast_type: ToastType,
    /// 剩余显示时间
    pub remaining_time: f32,
    /// 淡出进度 (0.0 - 1.0)
    pub fade_progress: f32,
}

impl Toast {
    pub fn new(message: impl Into<String>, toast_type: ToastType, duration: f32) -> Self {
        Self {
            message: message.into(),
            toast_type,
            remaining_time: duration,
            fade_progress: 0.0,
        }
    }

    /// 更新状态，返回是否应该移除
    pub fn update(&mut self, dt: f32) -> bool {
        self.remaining_time -= dt;

        if self.remaining_time <= 0.3 {
            self.fade_progress = 1.0 - (self.remaining_time / 0.3).max(0.0);
        }

        self.remaining_time <= 0.0
    }
}

/// Toast 管理器
pub struct ToastManager {
    toasts: Vec<Toast>,
    default_duration: f32,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            default_duration: 2.5,
        }
    }

    /// 显示普通消息
    pub fn info(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Info);
    }

    /// 显示成功消息
    pub fn success(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Success);
    }

    /// 显示警告消息
    pub fn warning(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Warning);
    }

    /// 显示错误消息
    pub fn error(&mut self, message: impl Into<String>) {
        self.show(message, ToastType::Error);
    }

    /// 显示自定义类型消息
    pub fn show(&mut self, message: impl Into<String>, toast_type: ToastType) {
        self.toasts
            .push(Toast::new(message, toast_type, self.default_duration));
    }

    /// 更新所有 Toast
    pub fn update(&mut self, dt: f32) {
        self.toasts.retain_mut(|toast| !toast.update(dt));
    }

    /// 获取所有活跃 Toast（用于 egui 覆盖渲染）
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }

    /// 是否有活跃的 Toast
    pub fn has_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    /// 清空所有 Toast
    pub fn clear(&mut self) {
        self.toasts.clear();
    }
}
