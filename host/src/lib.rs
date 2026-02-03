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

pub mod resources;
pub mod renderer;
pub mod input;
pub mod command_executor;
pub mod state;
pub mod audio;
pub mod manifest;
pub mod save_manager;
pub mod config;
pub mod app_mode;
pub mod ui;
pub mod screens;

pub use state::HostState;
pub use resources::{ResourceManager, ResourceError, ResourceSource, FsSource, ZipSource, TextureCache, CacheStats};
pub use renderer::{Renderer, RenderState, DrawMode, TransitionManager, TransitionType, AnimationSystem};
// Trait-based 动画系统 API
pub use renderer::{Animatable, AnimPropertyKey, ObjectId, PropertyAccessor, SimplePropertyAccessor};
pub use renderer::{AnimatableCharacter, CharacterAnimData};
pub use renderer::{AnimatableBackgroundTransition, BackgroundTransitionData};
pub use input::InputManager;
pub use command_executor::{CommandExecutor, ExecuteResult, TransitionInfo, AudioCommand, CommandOutput, CharacterAnimationCommand};
pub use audio::AudioManager;
pub use manifest::Manifest;
pub use save_manager::SaveManager;
pub use config::{AppConfig, WindowConfig, DebugConfig, AudioConfig, AssetSourceType};
pub use app_mode::{AppMode, NavigationStack, InputCapture, SaveLoadTab, UserSettings};