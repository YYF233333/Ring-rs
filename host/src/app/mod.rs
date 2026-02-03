//! # App 模块
//!
//! 应用状态与主循环逻辑。

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
    UserSettings,
};
use std::collections::HashMap;
use vn_runtime::VNRuntime;
use vn_runtime::state::WaitingReason;

/// 用户设置文件路径
pub const USER_SETTINGS_PATH: &str = "user_settings.json";

/// 应用状态
pub struct AppState {
    /// 应用配置
    pub config: AppConfig,
    pub host_state: HostState,
    pub resource_manager: ResourceManager,
    pub renderer: Renderer,
    pub render_state: RenderState,
    pub input_manager: InputManager,
    pub command_executor: CommandExecutor,
    pub audio_manager: Option<AudioManager>,
    pub waiting_reason: WaitingReason,
    pub typewriter_timer: f32,
    pub loading_complete: bool,
    /// VN Runtime（脚本模式）
    pub vn_runtime: Option<VNRuntime>,
    /// 脚本是否执行完毕
    pub script_finished: bool,
    /// 资源清单（立绘配置等）
    pub manifest: crate::manifest::Manifest,
    /// 存档管理器
    pub save_manager: crate::save_manager::SaveManager,
    /// 当前存档槽位
    pub current_save_slot: u32,
    /// 可用脚本列表 (id, path)
    pub scripts: Vec<(String, std::path::PathBuf)>,
    /// 游戏开始时间（用于计算游戏时长）
    pub play_start_time: std::time::Instant,

    // ===== 阶段16新增：UI 系统 =====
    /// 导航栈（管理界面切换和返回）
    pub navigation: NavigationStack,
    /// UI 上下文
    pub ui_context: UiContext,
    /// 用户设置
    pub user_settings: UserSettings,
    /// Toast 提示管理器
    pub toast_manager: ToastManager,

    // ===== 各界面状态 =====
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

    // ===== 阶段19新增：动画系统 =====
    /// 统一动画系统
    pub animation_system: AnimationSystem,
    /// 角色别名到动画系统 ObjectId 的映射
    pub character_object_ids: HashMap<String, ObjectId>,
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

        Self {
            config,
            host_state: HostState::new(),
            resource_manager,
            renderer: Renderer::new(width, height),
            render_state: RenderState::new(),
            input_manager: InputManager::new(),
            command_executor: CommandExecutor::new(),
            audio_manager,
            waiting_reason: WaitingReason::None,
            typewriter_timer: 0.0,
            loading_complete: false,
            vn_runtime: None,
            script_finished: false,
            manifest,
            save_manager,
            current_save_slot: 1,
            scripts,
            play_start_time: std::time::Instant::now(),

            // UI 系统
            navigation: NavigationStack::new(),
            ui_context: UiContext::new(Theme::dark()),
            user_settings,
            toast_manager: ToastManager::new(),

            // 界面状态
            title_screen: TitleScreen::new(),
            ingame_menu: InGameMenuScreen::new(),
            save_load_screen: SaveLoadScreen::new(),
            settings_screen: SettingsScreen::new(),
            history_screen: HistoryScreen::new(),

            // 动画系统
            animation_system: AnimationSystem::new(),
            character_object_ids: HashMap::new(),
        }
    }
}
