//! # App 模块
//!
//! 应用状态与主循环逻辑。
//!
//! ## 子系统划分（阶段 27）
//!
//! `AppState` 按职责拆分为三个子系统容器：
//! - [`CoreSystems`]：渲染管线、媒体资源、效果系统
//! - [`UiSystems`]：导航、界面状态、UI 上下文
//! - [`GameSession`]：运行时游戏会话状态
//!
//! 其余配置/基础设施字段保留在 `AppState` 顶层。

mod bootstrap;
mod command_handlers;
mod draw;
mod init;
mod save;
mod script_loader;
mod update;

pub use bootstrap::*;
pub use command_handlers::*;
pub use draw::*;
pub use save::*;
pub use script_loader::*;
pub use update::*;

use crate::renderer::ObjectId;
use crate::renderer::{AnimationSystem, RenderState, Renderer};
use crate::resources::ResourceManager;
use crate::screens::{
    HistoryScreen, InGameMenuScreen, SaveLoadScreen, SettingsScreen, TitleScreen,
};
use crate::ui::{Theme, ToastManager, UiContext};
use crate::{
    AppConfig, AudioManager, CommandExecutor, HostState, InputManager, NavigationStack,
    PlaybackMode, UserSettings,
};
use std::collections::HashMap;
use vn_runtime::VNRuntime;
use vn_runtime::state::WaitingReason;

/// 用户设置文件路径
pub const USER_SETTINGS_PATH: &str = "user_settings.json";

// ─── 子系统容器 ──────────────────────────────────────────────────────────────────

/// 核心子系统：渲染管线 + 媒体资源 + 效果系统
///
/// 包含渲染、动画、音频、资源加载和命令执行的所有状态。
/// `command_handlers` 层的函数签名依赖此类型而非 `AppState`。
pub struct CoreSystems {
    /// 资源管理器（纹理缓存、文件读取）
    pub resource_manager: ResourceManager,
    /// 渲染器（背景/角色/过渡绘制）
    pub renderer: Renderer,
    /// 渲染状态（当前背景、可见角色、对话等）
    pub render_state: RenderState,
    /// 统一动画系统
    pub animation_system: AnimationSystem,
    /// 角色别名到动画系统 ObjectId 的映射
    pub character_object_ids: HashMap<String, ObjectId>,
    /// 命令执行器（将 Runtime Command 转换为渲染状态更新）
    pub command_executor: CommandExecutor,
    /// 音频管理器
    pub audio_manager: Option<AudioManager>,
}

/// UI 子系统：导航 + 界面状态 + UI 上下文
///
/// 包含所有界面（Title/Menu/SaveLoad/Settings/History）的状态
/// 和导航栈、Toast 管理器等 UI 基础设施。
pub struct UiSystems {
    /// 导航栈（管理界面切换和返回）
    pub navigation: NavigationStack,
    /// UI 上下文（主题、屏幕尺寸等）
    pub ui_context: UiContext,
    /// Toast 提示管理器
    pub toast_manager: ToastManager,
    /// 主标题界面
    pub title_screen: TitleScreen,
    /// 游戏内菜单
    pub ingame_menu: InGameMenuScreen,
    /// 存档/读档界面
    pub save_load_screen: SaveLoadScreen,
    /// 设置界面
    pub settings_screen: SettingsScreen,
    /// 历史界面
    pub history_screen: HistoryScreen,
}

/// 游戏会话状态：运行时脚本执行与推进控制
///
/// 包含 VNRuntime、等待状态、打字机计时器等与
/// 当前游戏会话直接相关的状态。
pub struct GameSession {
    /// VN Runtime（脚本模式）
    pub vn_runtime: Option<VNRuntime>,
    /// 当前等待原因
    pub waiting_reason: WaitingReason,
    /// 打字机计时器
    pub typewriter_timer: f32,
    /// 脚本是否执行完毕
    pub script_finished: bool,
    /// 资源清单（立绘配置等）
    pub manifest: crate::manifest::Manifest,
    /// 当前播放推进模式（Normal/Auto/Skip）
    pub playback_mode: PlaybackMode,
    /// Auto 模式的等待计时器（秒）
    pub auto_timer: f32,
}

// ─── AppState ────────────────────────────────────────────────────────────────────

/// 应用状态
///
/// 阶段 27 重构：字段按职责拆分为三个子系统容器（`core` / `ui` / `session`），
/// 配置与基础设施字段保留在顶层。
pub struct AppState {
    // ===== 子系统 =====
    /// 核心子系统（渲染/动画/资源/命令执行/音频）
    pub core: CoreSystems,
    /// UI 子系统（导航/界面状态/Toast）
    pub ui: UiSystems,
    /// 游戏会话状态（Runtime/等待/打字机/推进模式）
    pub session: GameSession,

    // ===== 配置与基础设施 =====
    /// 应用配置
    pub config: AppConfig,
    /// 宿主状态（运行标志、调试模式）
    pub host_state: HostState,
    /// 输入管理器
    pub input_manager: InputManager,
    /// 用户设置
    pub user_settings: UserSettings,
    /// 存档管理器
    pub save_manager: crate::save_manager::SaveManager,
    /// 当前存档槽位
    pub current_save_slot: u32,
    /// 可用脚本列表（路径；展示用 ID 可从路径提取）
    pub scripts: Vec<std::path::PathBuf>,
    /// 游戏开始时间（用于计算游戏时长）
    pub play_start_time: std::time::Instant,
    /// 资源加载是否完成
    pub loading_complete: bool,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let resource_manager = init::create_resource_manager(&config);
        let audio_manager = init::create_audio_manager(&config);
        let manifest = init::load_manifest(&config, &resource_manager);
        let save_manager = init::create_save_manager(&config);
        let scripts = init::scan_script_list(&config, &resource_manager);
        let (width, height) = init::window_size(&config);
        let user_settings = init::load_user_settings(USER_SETTINGS_PATH);

        // Dev Mode: 运行脚本检查
        init::run_script_check(&config, &scripts, &resource_manager);

        Self {
            core: CoreSystems {
                resource_manager,
                renderer: Renderer::new(width, height),
                render_state: RenderState::new(),
                animation_system: AnimationSystem::new(),
                character_object_ids: HashMap::new(),
                command_executor: CommandExecutor::new(),
                audio_manager,
            },
            ui: UiSystems {
                navigation: NavigationStack::new(),
                ui_context: UiContext::new(Theme::dark()),
                toast_manager: ToastManager::new(),
                title_screen: TitleScreen::new(),
                ingame_menu: InGameMenuScreen::new(),
                save_load_screen: SaveLoadScreen::new(),
                settings_screen: SettingsScreen::new(),
                history_screen: HistoryScreen::new(),
            },
            session: GameSession {
                vn_runtime: None,
                waiting_reason: WaitingReason::None,
                typewriter_timer: 0.0,
                script_finished: false,
                manifest,
                playback_mode: PlaybackMode::Normal,
                auto_timer: 0.0,
            },
            config,
            host_state: HostState::new(),
            input_manager: InputManager::new(),
            user_settings,
            save_manager,
            current_save_slot: 1,
            scripts,
            play_start_time: std::time::Instant::now(),
            loading_complete: false,
        }
    }
}
