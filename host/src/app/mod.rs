//! # App 模块
//!
//! 应用状态与主循环逻辑。

mod command_handlers;
mod draw;
mod save;
mod script_loader;
mod update;

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
    AppConfig, AssetSourceType, AudioManager, CommandExecutor, HostState, InputManager,
    NavigationStack, UserSettings, ZipSource,
};
use std::collections::HashMap;
use std::sync::Arc;
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
        let assets_root = config.assets_root.to_string_lossy().to_string();
        let saves_dir = config.saves_dir.to_string_lossy().to_string();

        // 根据配置选择资源来源
        let resource_manager = match config.asset_source {
            AssetSourceType::Fs => {
                println!("📂 资源来源: 文件系统 ({})", assets_root);
                ResourceManager::new(&assets_root, config.resources.texture_cache_size_mb)
            }
            AssetSourceType::Zip => {
                let zip_path = config.zip_path.as_ref().expect("Zip 模式必须配置 zip_path");
                println!("📦 资源来源: ZIP 文件 ({})", zip_path);
                ResourceManager::with_source(
                    &assets_root,
                    Arc::new(ZipSource::new(zip_path)),
                    config.resources.texture_cache_size_mb,
                )
            }
        };

        // 初始化音频管理器（根据资源来源选择模式）
        let audio_manager = match config.asset_source {
            AssetSourceType::Fs => match AudioManager::new(&assets_root) {
                Ok(am) => {
                    println!("✅ 音频系统初始化成功");
                    Some(am)
                }
                Err(e) => {
                    eprintln!("⚠️ 音频系统初始化失败: {}", e);
                    None
                }
            },
            AssetSourceType::Zip => match AudioManager::new_zip_mode(&assets_root) {
                Ok(am) => {
                    println!("✅ 音频系统初始化成功 (ZIP 模式)");
                    Some(am)
                }
                Err(e) => {
                    eprintln!("⚠️ 音频系统初始化失败: {}", e);
                    None
                }
            },
        };

        // 加载资源清单（立绘配置）
        let manifest = match config.asset_source {
            AssetSourceType::Fs => {
                let manifest_path = config.manifest_full_path();
                match crate::manifest::Manifest::load(&manifest_path.to_string_lossy()) {
                    Ok(m) => {
                        println!("✅ 资源清单加载成功: {:?}", manifest_path);
                        m
                    }
                    Err(e) => {
                        eprintln!("⚠️ 资源清单加载失败，使用默认配置: {}", e);
                        crate::manifest::Manifest::with_defaults()
                    }
                }
            }
            AssetSourceType::Zip => {
                // ZIP 模式：通过 ResourceManager 读取
                let manifest_path = &config.manifest_path;
                match resource_manager.read_text(manifest_path) {
                    Ok(content) => {
                        match crate::manifest::Manifest::load_from_bytes(content.as_bytes()) {
                            Ok(m) => {
                                println!("✅ 资源清单加载成功: {}", manifest_path);
                                m
                            }
                            Err(e) => {
                                eprintln!("⚠️ 资源清单解析失败，使用默认配置: {}", e);
                                crate::manifest::Manifest::with_defaults()
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ 资源清单加载失败，使用默认配置: {}", e);
                        crate::manifest::Manifest::with_defaults()
                    }
                }
            }
        };

        // 初始化存档管理器
        let save_manager = crate::save_manager::SaveManager::new(&saves_dir);
        println!("✅ 存档管理器初始化成功: {}", saves_dir);

        // 扫描脚本目录
        let scripts = match config.asset_source {
            AssetSourceType::Fs => scan_scripts(&config.assets_root),
            AssetSourceType::Zip => scan_scripts_from_zip(&resource_manager),
        };
        println!("📜 发现 {} 个脚本文件", scripts.len());

        // 从配置获取窗口尺寸
        let (width, height) = (config.window.width as f32, config.window.height as f32);

        // 加载用户设置
        let user_settings = UserSettings::load(USER_SETTINGS_PATH);
        println!("✅ 用户设置加载完成");

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
