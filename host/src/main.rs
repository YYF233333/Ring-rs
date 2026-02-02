//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。

use macroquad::prelude::*;
use host::HostState;
use host::resources::ResourceManager;
use host::renderer::{Renderer, RenderState, SceneMaskType};
use host::{InputManager, CommandExecutor, ExecuteResult, AudioCommand, AudioManager, AppConfig, AssetSourceType};
use host::{AppMode, NavigationStack, SaveLoadTab, UserSettings};
use host::ZipSource;
use std::sync::Arc;
use host::ui::{UiContext, Theme, ToastManager};
use host::screens::{TitleScreen, InGameMenuScreen, SaveLoadScreen, SettingsScreen, HistoryScreen};
use host::screens::title::TitleAction;
use host::screens::ingame_menu::InGameMenuAction;
use host::screens::save_load::SaveLoadAction;
use host::screens::settings::SettingsAction;
use host::screens::history::HistoryAction;
use vn_runtime::state::WaitingReason;
use vn_runtime::input::RuntimeInput;
use vn_runtime::{VNRuntime, Parser};
use std::path::PathBuf;

/// 配置文件路径
const CONFIG_PATH: &str = "config.json";
/// 用户设置文件路径
const USER_SETTINGS_PATH: &str = "user_settings.json";

/// 应用状态
struct AppState {
    /// 应用配置
    config: AppConfig,
    host_state: HostState,
    resource_manager: ResourceManager,
    renderer: Renderer,
    render_state: RenderState,
    input_manager: InputManager,
    command_executor: CommandExecutor,
    audio_manager: Option<AudioManager>,
    waiting_reason: WaitingReason,
    typewriter_timer: f32,
    loading_complete: bool,
    /// VN Runtime（脚本模式）
    vn_runtime: Option<VNRuntime>,
    /// 脚本是否执行完毕
    script_finished: bool,
    /// 当前脚本索引
    script_index: usize,
    /// 资源清单（立绘配置等）
    manifest: host::manifest::Manifest,
    /// 存档管理器
    save_manager: host::save_manager::SaveManager,
    /// 当前存档槽位
    current_save_slot: u32,
    /// 可用脚本列表 (id, path)
    scripts: Vec<(String, PathBuf)>,
    /// 游戏开始时间（用于计算游戏时长）
    play_start_time: std::time::Instant,
    
    // ===== 阶段16新增：UI 系统 =====
    /// 导航栈（管理界面切换和返回）
    navigation: NavigationStack,
    /// UI 上下文
    ui_context: UiContext,
    /// 用户设置
    user_settings: UserSettings,
    /// Toast 提示管理器
    toast_manager: ToastManager,
    
    // ===== 各界面状态 =====
    /// 主标题界面
    title_screen: TitleScreen,
    /// 游戏内菜单
    ingame_menu: InGameMenuScreen,
    /// 存档/读档界面
    save_load_screen: SaveLoadScreen,
    /// 设置界面
    settings_screen: SettingsScreen,
    /// 历史界面
    history_screen: HistoryScreen,
}

impl AppState {
    fn new(config: AppConfig) -> Self {
        let assets_root = config.assets_root.to_string_lossy().to_string();
        let saves_dir = config.saves_dir.to_string_lossy().to_string();
        
        // 根据配置选择资源来源
        let resource_manager = match config.asset_source {
            AssetSourceType::Fs => {
                println!("📂 资源来源: 文件系统 ({})", assets_root);
                ResourceManager::new(&assets_root, config.resources.texture_cache_size_mb)
            }
            AssetSourceType::Zip => {
                let zip_path = config.zip_path.as_ref()
                    .expect("Zip 模式必须配置 zip_path");
                println!("📦 资源来源: ZIP 文件 ({})", zip_path);
                ResourceManager::with_source(
                    &assets_root,
                    Arc::new(ZipSource::new(zip_path)),
                    config.resources.texture_cache_size_mb,
                )
            }
        };

        // 初始化音频管理器
        let audio_manager = match AudioManager::new(&assets_root) {
            Ok(am) => {
                println!("✅ 音频系统初始化成功");
                Some(am)
            }
            Err(e) => {
                eprintln!("⚠️ 音频系统初始化失败: {}", e);
                None
            }
        };

        // 加载资源清单（立绘配置）
        let manifest_path = config.manifest_full_path();
        let manifest = match host::manifest::Manifest::load(&manifest_path.to_string_lossy()) {
            Ok(m) => {
                println!("✅ 资源清单加载成功: {:?}", manifest_path);
                m
            }
            Err(e) => {
                eprintln!("⚠️ 资源清单加载失败，使用默认配置: {}", e);
                host::manifest::Manifest::with_defaults()
            }
        };

        // 初始化存档管理器
        let save_manager = host::save_manager::SaveManager::new(&saves_dir);
        println!("✅ 存档管理器初始化成功: {}", saves_dir);

        // 扫描脚本目录
        let scripts = scan_scripts(&config.assets_root);
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
            script_index: 0,
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
        }
    }
}

/// 扫描脚本目录，返回 (script_id, script_path) 列表
fn scan_scripts(assets_root: &PathBuf) -> Vec<(String, PathBuf)> {
    let scripts_dir = assets_root.join("scripts");
    let mut scripts = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&scripts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(stem) = path.file_stem() {
                    let script_id = stem.to_string_lossy().to_string();
                    scripts.push((script_id, path));
                }
            }
        }
    }

    // 按文件名排序，确保顺序稳定
    scripts.sort_by(|a, b| a.0.cmp(&b.0));
    scripts
}

/// 主函数
#[macroquad::main(window_conf)]
async fn main() {
    // 加载配置文件
    let config = AppConfig::load(CONFIG_PATH);
    println!("✅ 配置加载完成: {:?}", CONFIG_PATH);
    println!("   assets_root: {:?}", config.assets_root);
    println!("   saves_dir: {:?}", config.saves_dir);
    println!("   start_script_path: {:?}", config.start_script_path);

    // **验证配置（必须配置 start_script_path）**
    if let Err(e) = config.validate() {
        panic!("❌ 配置验证失败: {}", e);
    }

    // 初始化应用状态
    let mut app_state = AppState::new(config);

    // 加载资源
    load_resources(&mut app_state).await;

    // 主循环
    while app_state.host_state.running {
        // 更新逻辑
        update(&mut app_state);

        // 确保渲染所需资源已加载（按需加载）
        ensure_render_resources(&mut app_state).await;

        // 渲染
        draw(&mut app_state);

        // 等待下一帧
        next_frame().await;
    }
    
    // 退出前保存 Continue 存档
    save_continue(&mut app_state);
}

/// 加载所有资源
async fn load_resources(app_state: &mut AppState) {
    println!("📦 开始加载资源...");

    // 加载字体（使用配置中的字体路径）
    let font_path = app_state.config.assets_root.join(&app_state.config.default_font);
    println!("✅ 加载字体: {:?}", font_path);
    if let Err(e) = app_state.renderer.init(&font_path.to_string_lossy()).await {
        eprintln!("⚠️ 字体加载失败，回退到 macroquad 默认字体（仅支持 ASCII）: {}", e);
    }

    // 预加载必需的 UI 纹理（用于过渡效果）
    // 其他资源改为按需加载（由 TextureCache 管理）
    let essential_textures = [
        "backgrounds/black.png",
        "backgrounds/white.png",
    ];
    for path in &essential_textures {
        match app_state.resource_manager.load_texture(path).await {
            Ok(_) => println!("✅ 预加载: {}", path),
            Err(e) => eprintln!("⚠️ 预加载失败: {} - {}", path, e),
        }
    }

    app_state.loading_complete = true;
    let stats = app_state.resource_manager.texture_cache_stats();
    println!("📦 资源加载完成！{}", stats.format());

    // 预加载脚本（但不开始游戏）
    load_script(app_state);
}

/// 从命令列表中收集需要预取的资源路径
fn collect_prefetch_paths(commands: &[vn_runtime::Command]) -> Vec<String> {
    use vn_runtime::command::TransitionArg;
    use vn_runtime::Command;

    let mut paths = Vec::new();

    for command in commands {
        match command {
            Command::ShowBackground { path, .. } => {
                paths.push(path.clone());
            }
            Command::ChangeScene { path, transition } => {
                paths.push(path.clone());
                // Rule 过渡还需要遮罩纹理
                if let Some(trans) = transition {
                    if let Some(TransitionArg::String(mask)) = trans.get_named("mask") {
                        paths.push(mask.clone());
                    }
                }
            }
            Command::ShowCharacter { path, .. } => {
                paths.push(path.clone());
            }
            _ => {}
        }
    }

    paths
}

/// 确保渲染所需资源已加载（按需加载）
///
/// 检查 RenderState 中引用的资源，如果尚未缓存则加载。
async fn ensure_render_resources(app_state: &mut AppState) {
    // 收集需要加载的资源路径
    let mut paths_to_load: Vec<String> = Vec::new();

    // 检查当前背景
    if let Some(ref bg_path) = app_state.render_state.current_background {
        if !app_state.resource_manager.has_texture(bg_path) {
            paths_to_load.push(bg_path.clone());
        }
    }

    // 检查可见角色
    for character in app_state.render_state.visible_characters.values() {
        if !app_state.resource_manager.has_texture(&character.texture_path) {
            paths_to_load.push(character.texture_path.clone());
        }
    }

    // 检查场景遮罩（Rule 效果需要遮罩纹理）
    if let Some(ref mask) = app_state.render_state.scene_mask {
        if let SceneMaskType::Rule { mask_path, .. } = &mask.mask_type {
            if !app_state.resource_manager.has_texture(mask_path) {
                paths_to_load.push(mask_path.clone());
            }
        }
    }

    // 加载缺失的资源
    for path in paths_to_load {
        match app_state.resource_manager.load_texture(&path).await {
            Ok(_) => println!("📦 按需加载: {}", path),
            Err(e) => eprintln!("❌ 加载失败: {} - {}", path, e),
        }
    }
}

/// 可用的脚本列表
/// 加载脚本文件
/// 从指定路径加载脚本
fn load_script_from_path(app_state: &mut AppState, script_path: &PathBuf) -> bool {
    // 提取脚本 ID（文件名，不含扩展名）
    let script_id = script_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    println!("📜 加载脚本: {} ({:?})", script_id, script_path);
    
    // 提取脚本所在目录作为 base_path（用于解析相对路径）
    let base_path = script_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    println!("📁 脚本目录: {}", base_path);
    
    match std::fs::read_to_string(script_path) {
        Ok(script_text) => {
            let mut parser = Parser::new();
            match parser.parse_with_base_path(&script_id, &script_text, &base_path) {
                Ok(script) => {
                    println!("✅ 脚本解析成功！节点数: {}", script.len());
                    
                    // 打印警告
                    for warning in parser.warnings() {
                        println!("⚠️ 解析警告: {}", warning);
                    }
                    
                    // 创建 VNRuntime
                    app_state.vn_runtime = Some(VNRuntime::new(script));
                    true
                }
                Err(e) => {
                    eprintln!("❌ 脚本解析失败: {}", e);
                    false
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 无法读取脚本文件: {}", e);
            false
        }
    }
}

/// 根据脚本 ID 加载脚本（用于存档恢复）
fn load_script_by_id(app_state: &mut AppState, script_id: &str) -> bool {
    // 在 scripts 列表中查找
    if let Some((_, path)) = app_state.scripts.iter().find(|(id, _)| id == script_id) {
        let path = path.clone();
        return load_script_from_path(app_state, &path);
    }
    
    // 尝试在 assets/scripts 目录下查找
    let script_path = app_state.config.assets_root
        .join("scripts")
        .join(format!("{}.md", script_id));
    
    if script_path.exists() {
        return load_script_from_path(app_state, &script_path);
    }
    
    eprintln!("❌ 找不到脚本: {}", script_id);
    false
}

/// 旧版 load_script（保留兼容性，使用 script_index）
fn load_script(app_state: &mut AppState) {
    if app_state.scripts.is_empty() {
        eprintln!("❌ 没有找到脚本文件");
        return;
    }

    let script_count = app_state.scripts.len();
    let (script_id, script_path) = &app_state.scripts[app_state.script_index % script_count];
    
    println!("📜 加载脚本 [{}/{}]: {} ({:?})", 
        app_state.script_index + 1, script_count, script_id, script_path);
    
    // 提取脚本所在目录作为 base_path（用于解析相对路径）
    let base_path = script_path
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    println!("📁 脚本目录: {}", base_path);
    
    match std::fs::read_to_string(script_path) {
        Ok(script_text) => {
            let mut parser = Parser::new();
            match parser.parse_with_base_path(&script_id, &script_text, &base_path) {
                Ok(script) => {
                    println!("✅ 脚本解析成功！节点数: {}", script.len());
                    
                    // 打印警告
                    for warning in parser.warnings() {
                        println!("⚠️ 解析警告: {}", warning);
                    }
                    
                    // 创建 VNRuntime
                    app_state.vn_runtime = Some(VNRuntime::new(script));
                    println!("✅ VNRuntime 创建成功！按 F3 切换到脚本模式，F4 切换脚本");
                }
                Err(e) => {
                    eprintln!("❌ 脚本解析失败: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ 脚本文件加载失败: {:?} - {}", script_path, e);
        }
    }
}

/// 窗口配置
fn window_conf() -> Conf {
    // 在窗口创建前读取配置（此函数在 main 之前被 macroquad 调用）
    let config = AppConfig::load(CONFIG_PATH);
    
    Conf {
        window_title: config.window.title,
        window_width: config.window.width as i32,
        window_height: config.window.height as i32,
        window_resizable: false,
        fullscreen: config.window.fullscreen,
        ..Default::default()
    }
}

/// 更新场景遮罩状态
///
/// 三阶段流程：
/// 1. phase 0: 遮罩淡入（UI 隐藏）
/// 2. phase 1: 遮罩淡出（UI 仍隐藏）
/// 3. phase 2: UI 淡入（0.2s dissolve）
fn update_scene_mask(render_state: &mut host::renderer::RenderState, dt: f32) {
    let mut pending_background: Option<String> = None;
    let mut should_show_ui = false;
    let mut completed = false;

    if let Some(ref mut mask) = render_state.scene_mask {
        completed = mask.update(dt);

        // 在遮罩中点时切换背景
        // Fade/FadeWhite: phase 1 开始时（遮罩全覆盖后）
        // Rule: phase 2 开始时（黑屏停顿结束后，即将从黑屏溶解到新背景）
        if mask.is_at_midpoint() {
            pending_background = mask.pending_background.take();
        }

        // 当进入 UI 淡入阶段时，恢复 UI 可见性
        // Fade/FadeWhite: phase 2
        // Rule: phase 3
        if mask.is_ui_fading_in() && !render_state.ui_visible {
            should_show_ui = true;
        }
    }

    if let Some(path) = pending_background {
        render_state.set_background(path);
    }

    if should_show_ui {
        render_state.ui_visible = true;
    }

    if completed {
        // 遮罩完成，清除状态
        if let Some(ref mut mask) = render_state.scene_mask {
            if let Some(path) = mask.pending_background.take() {
                render_state.set_background(path);
            }
        }
        render_state.scene_mask = None;
        render_state.ui_visible = true;
    }
}

/// 更新逻辑
fn update(app_state: &mut AppState) {
    let dt = get_frame_time();

    // 更新 UI 上下文
    app_state.ui_context.update();

    // 更新 Toast
    app_state.toast_manager.update(dt);

    // 切换调试模式（全局可用）
    if is_key_pressed(KeyCode::F1) {
        app_state.host_state.debug_mode = !app_state.host_state.debug_mode;
        println!("🔧 调试模式: {}", if app_state.host_state.debug_mode { "开启" } else { "关闭" });
    }

    // 根据当前模式处理更新
    let current_mode = app_state.navigation.current();
    match current_mode {
        AppMode::Title => update_title(app_state),
        AppMode::InGame => update_ingame(app_state, dt),
        AppMode::InGameMenu => update_ingame_menu(app_state),
        AppMode::SaveLoad => update_save_load(app_state),
        AppMode::Settings => update_settings(app_state),
        AppMode::History => update_history(app_state),
    }

    // 游戏进行时的通用更新（过渡效果、音频等）
    if current_mode.is_in_game() {
        // 更新过渡效果
        app_state.command_executor.update_transition(dt);
        app_state.renderer.update_transition(dt);

        // 更新场景遮罩状态
        update_scene_mask(&mut app_state.render_state, dt);

        // 更新角色过渡效果
        app_state.render_state.update_character_transitions(dt);
    }

    // 更新音频状态（所有模式都需要）
    if let Some(ref mut audio_manager) = app_state.audio_manager {
        audio_manager.update(dt);
    }
}

/// 更新主标题界面
fn update_title(app_state: &mut AppState) {
    // 初始化界面
    if app_state.title_screen.needs_init() {
        app_state.title_screen.init(
            &app_state.save_manager,
            &app_state.ui_context.theme,
            app_state.ui_context.screen_width,
            app_state.ui_context.screen_height,
        );
    }

    // 处理用户操作
    match app_state.title_screen.update(&app_state.ui_context) {
        TitleAction::StartGame => {
            // 开始新游戏时删除旧的 Continue 存档
            let _ = app_state.save_manager.delete_continue();
            start_new_game(app_state);
        }
        TitleAction::Continue => {
            // 读取专用 Continue 存档
            if app_state.title_screen.has_continue() {
                load_continue(app_state);
            }
        }
        TitleAction::LoadGame => {
            app_state.save_load_screen = SaveLoadScreen::new().with_tab(SaveLoadTab::Load);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        TitleAction::Settings => {
            app_state.settings_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::Settings);
        }
        TitleAction::Exit => {
            app_state.host_state.stop();
        }
        TitleAction::None => {}
    }
}

/// 更新游戏进行中
fn update_ingame(app_state: &mut AppState, dt: f32) {
    // ESC 打开系统菜单
    if is_key_pressed(KeyCode::Escape) {
        app_state.ingame_menu.mark_needs_init();
        app_state.navigation.navigate_to(AppMode::InGameMenu);
        return;
    }

    // 开发者快捷键（后续考虑 feature gate）
    #[cfg(debug_assertions)]
    {
        if is_key_pressed(KeyCode::F5) {
            quick_save(app_state);
        }
        if is_key_pressed(KeyCode::F9) {
            quick_load(app_state);
        }
    }

    // 使用 InputManager 处理游戏输入（传入 dt 用于长按快进）
    if let Some(input) = app_state.input_manager.update(&app_state.waiting_reason, dt) {
        handle_script_mode_input(app_state, input);
    }

    // 同步选择索引到 RenderState
    if let Some(ref mut choices) = app_state.render_state.choices {
        let choice_rects = app_state.renderer.get_choice_rects(choices.choices.len());
        app_state.input_manager.set_choice_rects(choice_rects);
        choices.selected_index = app_state.input_manager.selected_index;
        choices.hovered_index = app_state.input_manager.hovered_index;
    }

    // 更新打字机效果
    if let Some(ref dialogue) = app_state.render_state.dialogue {
        if !dialogue.is_complete {
            app_state.typewriter_timer += dt * app_state.user_settings.text_speed;
            while app_state.typewriter_timer >= 1.0 {
                app_state.typewriter_timer -= 1.0;
                if app_state.render_state.advance_typewriter() {
                    break;
                }
            }
        }
    }
}

/// 更新游戏内菜单
fn update_ingame_menu(app_state: &mut AppState) {
    if app_state.ingame_menu.needs_init() {
        app_state.ingame_menu.init(&app_state.ui_context);
    }

    match app_state.ingame_menu.update(&app_state.ui_context) {
        InGameMenuAction::Resume => {
            app_state.navigation.go_back();
        }
        InGameMenuAction::Save => {
            app_state.save_load_screen = SaveLoadScreen::new().with_tab(SaveLoadTab::Save);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        InGameMenuAction::Load => {
            app_state.save_load_screen = SaveLoadScreen::new().with_tab(SaveLoadTab::Load);
            app_state.save_load_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::SaveLoad);
        }
        InGameMenuAction::Settings => {
            app_state.settings_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::Settings);
        }
        InGameMenuAction::History => {
            app_state.history_screen.mark_needs_init();
            app_state.navigation.navigate_to(AppMode::History);
        }
        InGameMenuAction::ReturnToTitle => {
            // 用户主动返回，保存 Continue 存档
            return_to_title_from_game(app_state, true);
        }
        InGameMenuAction::Exit => {
            app_state.host_state.stop();
        }
        InGameMenuAction::None => {}
    }
}

/// 更新存档/读档界面
fn update_save_load(app_state: &mut AppState) {
    if app_state.save_load_screen.needs_init() {
        app_state.save_load_screen.init(&app_state.ui_context, &app_state.save_manager);
    }
    if app_state.save_load_screen.needs_refresh() {
        app_state.save_load_screen.refresh_saves(&app_state.save_manager);
    }

    match app_state.save_load_screen.update(&app_state.ui_context) {
        SaveLoadAction::Back => {
            app_state.navigation.go_back();
        }
        SaveLoadAction::Save(slot) => {
            app_state.current_save_slot = slot;
            quick_save(app_state);
            app_state.toast_manager.success(format!("已保存到槽位 {}", slot));
            app_state.save_load_screen.refresh_saves(&app_state.save_manager);
        }
        SaveLoadAction::Load(slot) => {
            load_game(app_state, slot);
            app_state.toast_manager.success(format!("已读取槽位 {}", slot));
        }
        SaveLoadAction::Delete(slot) => {
            if app_state.save_manager.delete(slot).is_ok() {
                app_state.toast_manager.info(format!("已删除槽位 {}", slot));
                app_state.save_load_screen.refresh_saves(&app_state.save_manager);
            } else {
                app_state.toast_manager.error("删除失败");
            }
        }
        SaveLoadAction::None => {}
    }
}

/// 更新设置界面
fn update_settings(app_state: &mut AppState) {
    if app_state.settings_screen.needs_init() {
        app_state.settings_screen.init(&app_state.ui_context, &app_state.user_settings);
    }

    match app_state.settings_screen.update(&app_state.ui_context) {
        SettingsAction::Back => {
            app_state.navigation.go_back();
        }
        SettingsAction::Apply => {
            // 应用设置
            app_state.user_settings = app_state.settings_screen.settings().clone();
            
            // 应用音量
            if let Some(ref mut audio) = app_state.audio_manager {
                audio.set_bgm_volume(app_state.user_settings.bgm_volume);
                audio.set_sfx_volume(app_state.user_settings.sfx_volume);
                audio.set_muted(app_state.user_settings.muted);
            }

            // 保存设置
            if let Err(e) = app_state.user_settings.save(USER_SETTINGS_PATH) {
                eprintln!("⚠️ 保存用户设置失败: {}", e);
                app_state.toast_manager.error("设置保存失败");
            } else {
                app_state.toast_manager.success("设置已保存");
            }

            app_state.navigation.go_back();
        }
        SettingsAction::None => {}
    }
}

/// 更新历史界面
fn update_history(app_state: &mut AppState) {
    if app_state.history_screen.needs_init() {
        if let Some(ref runtime) = app_state.vn_runtime {
            app_state.history_screen.init(&app_state.ui_context, runtime.history());
        }
    }

    match app_state.history_screen.update(&app_state.ui_context) {
        HistoryAction::Back => {
            app_state.navigation.go_back();
        }
        HistoryAction::None => {}
    }
}

/// 开始新游戏（使用 config.start_script_path）
fn start_new_game(app_state: &mut AppState) {
    // 使用配置的入口脚本
    let script_path = app_state.config.start_script_full_path();
    
    if load_script_from_path(app_state, &script_path) {
        app_state.render_state = RenderState::new();
        app_state.script_finished = false;
        app_state.play_start_time = std::time::Instant::now();
        
        // 执行第一次 tick
        run_script_tick(app_state, None);
        
        // 切换到游戏模式
        app_state.navigation.switch_to(AppMode::InGame);
        println!("🎮 开始新游戏: {:?}", script_path);
    } else {
        app_state.toast_manager.error("无法加载入口脚本");
    }
}

/// 读取存档（槽位）
fn load_game(app_state: &mut AppState, slot: u32) {
    app_state.current_save_slot = slot;
    if quick_load(app_state) {
        // 成功读档后切换到游戏模式
        app_state.navigation.switch_to(AppMode::InGame);
    }
}

/// 读取 Continue 存档
fn load_continue(app_state: &mut AppState) {
    // 读取 Continue 存档
    let save_data = match app_state.save_manager.load_continue() {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ Continue 读取失败: {}", e);
            app_state.toast_manager.error("Continue 存档读取失败");
            return;
        }
    };

    // 恢复游戏状态
    if restore_from_save_data(app_state, save_data) {
        // 成功读档后切换到游戏模式
        app_state.navigation.switch_to(AppMode::InGame);
        println!("🎮 继续游戏");
    }
}


//=============================================================================
// 过渡效果处理
//=============================================================================

/// 应用过渡效果
fn apply_transition_effect(app_state: &mut AppState) {
    let transition_info = &app_state.command_executor.last_output.transition_info;
    
    if transition_info.has_background_transition {
        app_state.renderer.start_background_transition(
            transition_info.old_background.clone(),
            transition_info.transition.as_ref(),
        );
    }
}

/// 处理音频命令
fn handle_audio_command(app_state: &mut AppState) {
    let audio_cmd = app_state.command_executor.last_output.audio_command.clone();
    
    if let Some(ref mut audio_manager) = app_state.audio_manager {
        if let Some(cmd) = audio_cmd {
            match cmd {
                AudioCommand::PlayBgm { path, looping, fade_in: _ } => {
                    // BGM 切换自带交叉淡化效果（规范要求）
                    // 如果当前有 BGM 在播放，使用交叉淡化；否则直接播放（带淡入）
                    const CROSSFADE_DURATION: f32 = 1.0; // 交叉淡化时长
                    if audio_manager.is_bgm_playing() {
                        audio_manager.crossfade_bgm(&path, looping, CROSSFADE_DURATION);
                    } else {
                        audio_manager.play_bgm(&path, looping, Some(CROSSFADE_DURATION));
                    }
                }
                AudioCommand::StopBgm { fade_out } => {
                    audio_manager.stop_bgm(fade_out);
                }
                AudioCommand::PlaySfx { path } => {
                    audio_manager.play_sfx(&path);
                }
            }
        }
    }
}

//=============================================================================
// 存档系统
//=============================================================================

/// 构建当前游戏状态的存档数据
fn build_save_data(app_state: &AppState, slot: u32) -> Option<vn_runtime::SaveData> {
    let runtime = app_state.vn_runtime.as_ref()?;

    // 构建存档数据
    let runtime_state = runtime.state().clone();
    let mut save_data = vn_runtime::SaveData::new(slot, runtime_state);

    // 设置章节标题（如果有）
    if let Some(ref chapter) = app_state.render_state.chapter_mark {
        save_data = save_data.with_chapter(&chapter.title);
    }

    // 设置游戏时长
    let play_time = app_state.play_start_time.elapsed().as_secs();
    save_data.metadata.play_time_secs = play_time;

    // 设置音频状态
    if let Some(ref audio) = app_state.audio_manager {
        save_data = save_data.with_audio(vn_runtime::AudioState {
            current_bgm: audio.current_bgm_path().map(|s| s.to_string()),
            bgm_looping: true, // 假设 BGM 总是循环
        });
    }

    // 设置渲染快照
    let render_snapshot = vn_runtime::RenderSnapshot {
        background: app_state.render_state.current_background.clone(),
        characters: app_state.render_state.visible_characters
            .iter()
            .map(|(alias, sprite)| vn_runtime::CharacterSnapshot {
                alias: alias.clone(),
                texture_path: sprite.texture_path.clone(),
                position: format!("{:?}", sprite.position),
            })
            .collect(),
    };
    save_data = save_data.with_render(render_snapshot);

    // 设置历史记录
    save_data = save_data.with_history(runtime.history().clone());

    Some(save_data)
}

/// 快速保存（到槽位）
fn quick_save(app_state: &mut AppState) {
    // 只在游戏模式下可以保存
    if !app_state.navigation.current().is_in_game() {
        println!("⚠️ 只能在游戏中保存");
        return;
    }

    let slot = app_state.current_save_slot;
    
    let Some(save_data) = build_save_data(app_state, slot) else {
        println!("⚠️ 没有可保存的游戏状态");
        return;
    };

    // 保存
    match app_state.save_manager.save(&save_data) {
        Ok(()) => println!("💾 快速保存成功 (槽位 {})", slot),
        Err(e) => eprintln!("❌ 保存失败: {}", e),
    }
}

/// 保存 Continue 存档（用于"继续"功能）
fn save_continue(app_state: &mut AppState) {
    // 只在有游戏状态时保存
    if app_state.vn_runtime.is_none() {
        return;
    }

    // 使用槽位 0 作为 Continue 存档的元数据标记
    let Some(save_data) = build_save_data(app_state, 0) else {
        return;
    };

    // 保存 Continue 存档
    match app_state.save_manager.save_continue(&save_data) {
        Ok(()) => println!("💾 Continue 存档保存成功"),
        Err(e) => eprintln!("⚠️ Continue 存档保存失败: {}", e),
    }
}

/// 从游戏状态返回主界面
/// 用于脚本执行完毕或用户主动返回时清理状态并跳转到 Title
/// 
/// # 参数
/// - `should_save_continue`: 是否保存 Continue 存档。脚本执行完毕时应该为 `false`，用户主动返回时为 `true`
fn return_to_title_from_game(app_state: &mut AppState, should_save_continue: bool) {
    // 只在用户主动返回时保存 Continue 存档
    // 脚本执行完毕时不保存，避免下次 Continue 直接跳到末尾
    if should_save_continue {
        save_continue(app_state);
    }
    
    // 停止音乐
    if let Some(ref mut audio) = app_state.audio_manager {
        audio.stop_bgm(Some(0.5));
    }
    
    // 清理游戏状态
    app_state.vn_runtime = None;
    app_state.render_state = RenderState::new();
    app_state.script_finished = false;
    
    // 返回标题
    app_state.navigation.return_to_title();
    app_state.title_screen.mark_needs_init();
}

/// 从存档数据恢复游戏状态
fn restore_from_save_data(app_state: &mut AppState, save_data: vn_runtime::SaveData) -> bool {
    // 加载对应的脚本
    let script_id = &save_data.runtime_state.position.script_id;
    
    if !load_script_by_id(app_state, script_id) {
        eprintln!("❌ 找不到脚本: {}", script_id);
        return false;
    }

    // 恢复 Runtime 状态和历史记录
    if let Some(ref mut runtime) = app_state.vn_runtime {
        runtime.restore_state(save_data.runtime_state);
        runtime.restore_history(save_data.history);
    }

    // 恢复渲染状态
    app_state.render_state = RenderState::new();
    app_state.render_state.current_background = save_data.render.background;
    for char_snap in save_data.render.characters {
        // 尝试解析 position（简化处理，默认 Center）
        let position = vn_runtime::Position::Center;
        app_state.render_state.show_character(
            char_snap.alias,
            char_snap.texture_path,
            position,
        );
    }

    // 恢复音频状态
    if let Some(ref mut audio) = app_state.audio_manager {
        if let Some(ref bgm_path) = save_data.audio.current_bgm {
            audio.play_bgm(bgm_path, save_data.audio.bgm_looping, Some(0.5));
        }
    }

    // 设置游戏状态
    app_state.script_finished = false;
    app_state.waiting_reason = WaitingReason::WaitForClick;
    app_state.play_start_time = std::time::Instant::now(); // 重置开始时间

    true
}

/// 快速读取（从槽位）
fn quick_load(app_state: &mut AppState) -> bool {
    let slot = app_state.current_save_slot;

    // 读取存档
    let save_data = match app_state.save_manager.load(slot) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ 读取失败: {}", e);
            return false;
        }
    };

    if restore_from_save_data(app_state, save_data) {
        println!("💾 快速读取成功 (槽位 {})", slot);
        true
    } else {
        false
    }
}

//=============================================================================
// 脚本模式处理
//=============================================================================

/// 处理脚本模式下的输入
fn handle_script_mode_input(app_state: &mut AppState, input: RuntimeInput) {
    // 如果转场正在进行（changeBG），允许输入用于跳过转场
    if app_state.renderer.transition.is_active() {
        // 跳过转场效果
        app_state.renderer.transition.skip();
        return;
    }

    // 如果场景遮罩正在进行（changeScene），允许输入用于跳过转场
    if let Some(ref mut mask) = app_state.render_state.scene_mask {
        if !mask.is_mask_complete() {
            // 跳过当前阶段的转场动画
            mask.skip_current_phase();
            return;
        }
    }

    // 如果对话正在打字，先完成打字
    if !app_state.render_state.is_dialogue_complete() {
        app_state.render_state.complete_typewriter();
        return;
    }

    // 脚本执行完毕后已自动返回主界面，这里不再处理

    // 将输入传递给 VNRuntime
    run_script_tick(app_state, Some(input));
}

/// 执行一次 VNRuntime tick
fn run_script_tick(app_state: &mut AppState, input: Option<RuntimeInput>) {
    // 如果是选择输入，先清除选择界面
    if let Some(RuntimeInput::ChoiceSelected { index }) = &input {
        println!("📜 用户选择了选项 {}", index + 1);
        app_state.render_state.clear_choices();
    }

    // 先执行 tick 并收集结果
    let tick_result = {
        let runtime = match app_state.vn_runtime.as_mut() {
            Some(r) => r,
            None => {
                eprintln!("❌ VNRuntime 未初始化");
                return;
            }
        };
        runtime.tick(input)
    };

    // 处理 tick 结果
    match tick_result {
        Ok((commands, waiting)) => {
            println!("📜 tick 返回 {} 条命令, 等待状态: {:?}", commands.len(), waiting);

            // 收集命令中的资源路径（用于预取统计）
            let prefetch_paths = collect_prefetch_paths(&commands);
            if !prefetch_paths.is_empty() {
                println!("  📦 预取资源: {:?}", prefetch_paths);
            }

            // 执行所有命令
            for command in &commands {
                println!("  ▶️ {:?}", command);
                let result = app_state.command_executor.execute(
                    command,
                    &mut app_state.render_state,
                    &app_state.resource_manager,
                );

                // 应用过渡效果
                apply_transition_effect(app_state);

                // 处理音频命令
                handle_audio_command(app_state);

                // 检查执行结果
                if let ExecuteResult::Error(e) = result {
                    eprintln!("  ❌ 命令执行失败: {}", e);
                }
            }

            // 更新等待状态
            app_state.waiting_reason = waiting.clone();

            // 如果是选择等待，重置选择索引
            if let WaitingReason::WaitForChoice { choice_count } = &waiting {
                app_state.input_manager.reset_choice(*choice_count);
            }

            // 检查脚本是否执行完毕
            let is_finished = app_state.vn_runtime.as_ref()
                .map(|r| r.is_finished())
                .unwrap_or(false);
            if is_finished && !app_state.script_finished {
                app_state.script_finished = true;
                println!("📜 脚本执行完毕，自动返回主界面");
                // 自动返回主界面，不保存 Continue 存档（避免下次 Continue 直接跳到末尾）
                return_to_title_from_game(app_state, false);
            }

            // 重置打字机计时器
            app_state.typewriter_timer = 0.0;
        }
        Err(e) => {
            eprintln!("❌ Runtime tick 错误: {:?}", e);
        }
    }
}

/// 渲染函数
fn draw(app_state: &mut AppState) {
    let current_mode = app_state.navigation.current();

    // 根据当前模式绘制
    match current_mode {
        AppMode::Title => {
            app_state.title_screen.draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::InGame => {
            // 渲染游戏画面
            app_state.renderer.render(&app_state.render_state, &app_state.resource_manager, &app_state.manifest);
        }
        AppMode::InGameMenu => {
            // 先渲染游戏画面，再渲染菜单覆盖层
            app_state.renderer.render(&app_state.render_state, &app_state.resource_manager, &app_state.manifest);
            app_state.ingame_menu.draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::SaveLoad => {
            // 如果是从游戏内打开，先渲染游戏画面
            if app_state.vn_runtime.is_some() {
                app_state.renderer.render(&app_state.render_state, &app_state.resource_manager, &app_state.manifest);
            }
            app_state.save_load_screen.draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::Settings => {
            app_state.settings_screen.draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
        AppMode::History => {
            // 先渲染游戏画面，再渲染历史覆盖层
            app_state.renderer.render(&app_state.render_state, &app_state.resource_manager, &app_state.manifest);
            app_state.history_screen.draw(&app_state.ui_context, &app_state.renderer.text_renderer);
        }
    }

    // 绘制 Toast 提示（所有模式都可显示）
    app_state.toast_manager.draw(&app_state.ui_context, &app_state.renderer.text_renderer);

    // 显示调试信息
    if app_state.host_state.debug_mode {
        draw_debug_info(app_state);
    }
}

/// 绘制调试信息
fn draw_debug_info(app_state: &AppState) {
    let fps = get_fps();
    let char_count = app_state.render_state.visible_characters.len();
    let has_bg = app_state.render_state.current_background.is_some();
    let has_dialogue = app_state.render_state.dialogue.is_some();
    let current_mode = app_state.navigation.current();

    // 获取缓存统计
    let cache_stats = app_state.resource_manager.texture_cache_stats();

    // 绘制半透明背景（加高以容纳更多信息）
    // 注意：使用较高的 alpha 值确保可见性
    draw_rectangle(5.0, 5.0, 320.0, 240.0, Color::new(0.0, 0.0, 0.0, 0.85));

    // 基础信息
    let mut lines: Vec<(String, Color)> = vec![
        (format!("FPS: {}", fps), GREEN),
        (format!("模式: {:?}", current_mode), GREEN),
        (format!("角色: {} | 背景: {} | 对话: {}", char_count, has_bg, has_dialogue), GREEN),
    ];

    // 缓存统计
    lines.push(("--- 纹理缓存 ---".to_string(), YELLOW));
    lines.push((
        format!(
            "条目: {} | 占用: {:.1}MB / {:.1}MB",
            cache_stats.entries,
            cache_stats.used_bytes as f64 / 1024.0 / 1024.0,
            cache_stats.budget_bytes as f64 / 1024.0 / 1024.0
        ),
        WHITE,
    ));
    lines.push((
        format!(
            "命中率: {:.1}% ({}/{})",
            cache_stats.hit_rate * 100.0,
            cache_stats.hits,
            cache_stats.hits + cache_stats.misses
        ),
        if cache_stats.hit_rate > 0.8_f64 { GREEN } else if cache_stats.hit_rate > 0.5_f64 { YELLOW } else { RED },
    ));
    lines.push((
        format!("驱逐次数: {}", cache_stats.evictions),
        if cache_stats.evictions == 0 { GREEN } else { YELLOW },
    ));

    // 资源来源
    let source_info = match app_state.config.asset_source {
        AssetSourceType::Fs => "文件系统".to_string(),
        AssetSourceType::Zip => format!("ZIP: {}", app_state.config.zip_path.as_deref().unwrap_or("?")),
    };
    lines.push((format!("来源: {}", source_info), Color::new(0.7, 0.7, 0.7, 1.0))); // 灰色

    // 绘制所有行
    for (i, (line, color)) in lines.iter().enumerate() {
        let y = 25.0 + i as f32 * 22.0;
        // 使用文本渲染器绘制（支持中文）
        app_state.renderer.text_renderer.draw_ui_text(
            line,
            10.0,
            y,
            16.0,
            *color,
        );
    }
}
