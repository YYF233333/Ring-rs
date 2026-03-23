//! # UI 模式插件系统
//!
//! 提供 `UiModeHandler` trait 和 `UiModeRegistry`，
//! 使新增 UI 模式只需：实现 trait → 注册 → 脚本调用。
//!
//! 每个 handler 对应 `requestUI` 命令的一个 `mode` 值，
//! 拥有独立的 activate / render / deactivate 生命周期。

pub mod map_handler;

use std::collections::HashMap;

use crate::resources::ResourceManager;
use crate::ui::layout::ScaleContext;
use vn_runtime::state::VarValue;

/// UI 模式处理器
///
/// 实现此 trait 以注册自定义 UI 模式。
/// 每个模式对应 `Command::RequestUI.mode` 的一个值。
pub trait UiModeHandler: std::fmt::Debug + Send {
    /// 模式标识符（与 `Command::RequestUI.mode` 匹配）
    fn mode_id(&self) -> &str;

    /// 收到 `Command::RequestUI` 时激活此模式
    ///
    /// `key` 用于回传 `RuntimeInput::UIResult` 时的匹配。
    /// `params` 是脚本传入的参数。
    /// `resources` 用于加载模式所需的资源。
    fn activate(
        &mut self,
        key: String,
        params: &HashMap<String, VarValue>,
        resources: &ResourceManager,
    ) -> Result<(), UiModeError>;

    /// 每帧渲染
    ///
    /// 在 egui context 内调用。返回 `Active` 表示继续渲染，
    /// 返回 `Completed(value)` 表示用户完成交互，携带结果值。
    fn render(&mut self, ctx: &egui::Context, scale: &ScaleContext) -> UiModeStatus;

    /// 模式结束或被取消后清理内部状态和资源
    fn deactivate(&mut self);
}

/// UI 模式每帧渲染返回的状态
#[derive(Debug)]
pub enum UiModeStatus {
    /// 模式仍然活跃，继续渲染
    Active,
    /// 用户完成交互，携带结果值
    Completed(VarValue),
    /// 用户取消（Esc 等），无结果
    Cancelled,
}

/// UI 模式错误
#[derive(Debug, thiserror::Error)]
pub enum UiModeError {
    #[error("unknown UI mode: {0}")]
    UnknownMode(String),
    #[error("another UI mode is already active")]
    AlreadyActive,
    #[error("resource load failed: {0}")]
    ResourceLoadFailed(String),
    #[error("invalid parameters: {0}")]
    InvalidParams(String),
}

/// 从 registry 中取出的活跃 UI 模式（临时持有，用于渲染）
pub struct ActiveUiMode {
    pub mode_id: String,
    pub key: String,
    pub handler: Box<dyn UiModeHandler>,
}

/// UI 模式注册表与运行时调度
pub struct UiModeRegistry {
    handlers: HashMap<String, Box<dyn UiModeHandler>>,
    active: Option<ActiveUiMode>,
}

impl Default for UiModeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl UiModeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            active: None,
        }
    }

    /// 注册一个 UI 模式 handler
    pub fn register(&mut self, handler: Box<dyn UiModeHandler>) {
        let id = handler.mode_id().to_string();
        self.handlers.insert(id, handler);
    }

    /// 激活指定模式
    pub fn activate(
        &mut self,
        mode: &str,
        key: String,
        params: &HashMap<String, VarValue>,
        resources: &ResourceManager,
    ) -> Result<(), UiModeError> {
        if self.active.is_some() {
            return Err(UiModeError::AlreadyActive);
        }
        let mut handler = self
            .handlers
            .remove(mode)
            .ok_or_else(|| UiModeError::UnknownMode(mode.to_string()))?;

        if let Err(e) = handler.activate(key.clone(), params, resources) {
            self.handlers.insert(mode.to_string(), handler);
            return Err(e);
        }

        self.active = Some(ActiveUiMode {
            mode_id: mode.to_string(),
            key,
            handler,
        });
        Ok(())
    }

    /// 是否有活跃模式
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// 将活跃 handler 取出（用于渲染期间获取 &mut 访问）
    ///
    /// 渲染完成后必须调用 `restore_active` 或 `complete_active` 归还。
    pub fn take_active(&mut self) -> Option<ActiveUiMode> {
        self.active.take()
    }

    /// 渲染后归还活跃 handler（模式仍在继续）
    pub fn restore_active(&mut self, active: ActiveUiMode) {
        self.active = Some(active);
    }

    /// 模式完成或取消，deactivate 并归还 handler 到注册表
    pub fn complete_active(&mut self, mut active: ActiveUiMode) {
        active.handler.deactivate();
        self.handlers.insert(active.mode_id, active.handler);
    }

    /// 强制取消当前活跃模式
    pub fn cancel_current(&mut self) {
        if let Some(mut active) = self.active.take() {
            active.handler.deactivate();
            self.handlers.insert(active.mode_id, active.handler);
        }
    }
}
