//! # Host 层
//!
//! Visual Novel Engine 的宿主层实现，使用 macroquad 作为渲染和 IO 引擎。
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

pub mod app_mode;
pub mod audio;
pub mod command_executor;
pub mod config;
pub mod input;
pub mod manifest;
pub mod renderer;
pub mod resources;
pub mod save_manager;
pub mod screens;
pub mod state;
pub mod ui;

pub use renderer::{
    AnimationSystem, DrawMode, RenderState, Renderer, TransitionManager, TransitionType,
};
pub use resources::{
    CacheStats, FsSource, ResourceError, ResourceManager, ResourceSource, TextureCache, ZipSource,
};
pub use state::HostState;
// Trait-based 动画系统 API
pub use app_mode::{AppMode, InputCapture, NavigationStack, SaveLoadTab, UserSettings};
pub use audio::AudioManager;
pub use command_executor::{
    AudioCommand, CharacterAnimationCommand, CommandExecutor, CommandOutput, ExecuteResult,
    TransitionInfo,
};
pub use config::{AppConfig, AssetSourceType, AudioConfig, DebugConfig, WindowConfig};
pub use input::InputManager;
pub use manifest::Manifest;
pub use renderer::{
    AnimPropertyKey, Animatable, ObjectId, PropertyAccessor, SimplePropertyAccessor,
};
pub use renderer::{AnimatableBackgroundTransition, BackgroundTransitionData};
pub use renderer::{AnimatableCharacter, CharacterAnimData};
pub use save_manager::SaveManager;
