//! # Host 层
//!
//! Visual Novel Engine 的宿主层实现。
//!
//! ## 渲染后端
//!
//! 使用 winit + wgpu + egui 作为渲染/窗口/UI 基础设施（RFC-007）。
//! [`backend::WgpuBackend`] 封装了 GPU 初始化与帧渲染循环。
//!
//! ## 架构说明
//!
//! Host 层负责：
//! - 窗口与渲染
//! - 资源加载
//! - 音频播放
//! - 输入采集
//! - 将 Runtime 的 Command 转换为实际效果
//!
//! Host 层不包含脚本逻辑，只负责执行 Runtime 发出的 Command。

pub mod app;
pub mod audio;
pub mod backend;
pub mod command_executor;
pub mod config;
pub mod extensions;
pub mod input;
pub mod manifest;
pub mod renderer;
pub mod rendering_types;
pub mod resources;
pub mod save_manager;
pub mod ui;
pub mod video;

pub use app::app_mode::{
    AppMode, InputCapture, NavigationStack, PlaybackMode, SaveLoadPage, SaveLoadTab, UserSettings,
};
pub use app::state::HostState;
pub use audio::AudioManager;
pub use command_executor::{AudioCommand, CommandExecutor, CommandOutput, ExecuteResult};
pub use config::{AppConfig, AssetSourceType, AudioConfig, DebugConfig, WindowConfig};
pub use extensions::{
    CapabilityDispatchResult, EngineContext, ExtensionDiagnostic, ExtensionRegistry,
};
pub use input::InputManager;
pub use manifest::Manifest;
pub use renderer::ObjectId;
pub use renderer::effects::{EffectRequest, EffectTarget};
pub use renderer::{AnimationSystem, RenderState, Renderer};
pub use rendering_types::{
    DrawCommand, NullTexture, NullTextureFactory, Texture, TextureContext, TextureFactory,
};
pub use resources::{
    CacheStats, LogicalPath, ResourceError, ResourceManager, ResourceSource, TextureCache,
};
pub use save_manager::SaveManager;

// 阶段 27：子系统容器类型
pub use app::{CoreSystems, GameSession, UiSystems};

// UI 布局配置 + 素材缓存
pub use ui::{ScaleContext, UiAssetCache, UiLayoutConfig};
