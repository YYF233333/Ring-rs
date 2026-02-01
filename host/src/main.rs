//! # Host 主程序
//!
//! Visual Novel Engine 的宿主层入口。

use macroquad::prelude::*;
use host::HostState;
use host::resources::ResourceManager;
use host::renderer::{Renderer, RenderState};
use host::renderer::render_state::ChoiceItem;
use host::{InputManager, CommandExecutor, ExecuteResult, AudioCommand, AudioManager};
use vn_runtime::command::{Command, Choice, Position};
use vn_runtime::state::WaitingReason;
use vn_runtime::input::RuntimeInput;
use vn_runtime::{VNRuntime, Parser};
use std::collections::HashMap;

/// 窗口配置
const WINDOW_WIDTH: f32 = 1280.0;
const WINDOW_HEIGHT: f32 = 720.0;
const WINDOW_TITLE: &str = "Visual Novel Engine";

/// 打字机效果速度（每秒字符数）
const TYPEWRITER_SPEED: f32 = 30.0;

/// 演示模式状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DemoState {
    ShowBackground,
    ShowCharacter,
    ShowDialogue,
    ShowChoices,
    ShowChapter,
    Complete,
}

/// 运行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    /// 演示模式（原有的硬编码演示）
    Demo,
    /// 命令模式（CommandExecutor 演示）
    Command,
    /// 脚本模式（真正的 VNRuntime 集成）
    Script,
}

/// 应用状态
struct AppState {
    host_state: HostState,
    resource_manager: ResourceManager,
    renderer: Renderer,
    render_state: RenderState,
    input_manager: InputManager,
    command_executor: CommandExecutor,
    audio_manager: Option<AudioManager>,
    textures: HashMap<String, Texture2D>,
    demo_state: DemoState,
    waiting_reason: WaitingReason,
    typewriter_timer: f32,
    loading_complete: bool,
    /// 命令队列（用于演示 CommandExecutor）
    command_queue: Vec<Command>,
    /// 当前命令索引
    command_index: usize,
    /// 当前运行模式
    run_mode: RunMode,
    /// VN Runtime（脚本模式）
    vn_runtime: Option<VNRuntime>,
    /// 脚本是否执行完毕
    script_finished: bool,
    /// 当前脚本索引
    script_index: usize,
    /// 资源清单（立绘配置等）
    manifest: host::manifest::Manifest,
}

impl AppState {
    fn new() -> Self {
        // 初始化音频管理器
        let audio_manager = match AudioManager::new("F:/Code/Ring-rs/assets") {
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
        let manifest = match host::manifest::Manifest::load("F:/Code/Ring-rs/assets/manifest.json") {
            Ok(m) => {
                println!("✅ 资源清单加载成功");
                m
            }
            Err(e) => {
                eprintln!("⚠️ 资源清单加载失败，使用默认配置: {}", e);
                host::manifest::Manifest::with_defaults()
            }
        };

        Self {
            host_state: HostState::new(),
            resource_manager: ResourceManager::new("F:/Code/Ring-rs/assets"),
            renderer: Renderer::new(1920.0, 1080.0),
            render_state: RenderState::new(),
            input_manager: InputManager::new(),
            command_executor: CommandExecutor::new(),
            audio_manager,
            textures: HashMap::new(),
            demo_state: DemoState::ShowBackground,
            waiting_reason: WaitingReason::None,
            typewriter_timer: 0.0,
            loading_complete: false,
            command_queue: Vec::new(),
            command_index: 0,
            run_mode: RunMode::Demo,
            vn_runtime: None,
            script_finished: false,
            script_index: 0,
            manifest,
        }
    }
}

/// 主函数
#[macroquad::main(window_conf)]
async fn main() {
    // 初始化应用状态
    let mut app_state = AppState::new();

    // 加载资源
    load_resources(&mut app_state).await;

    // 主循环
    while app_state.host_state.running {
        // 更新逻辑
        update(&mut app_state);

        // 渲染
        draw(&app_state);

        // 等待下一帧
        next_frame().await;
    }
}

/// 加载所有资源
async fn load_resources(app_state: &mut AppState) {
    println!("📦 开始加载资源...");

    // 加载中文字体（使用黑体）
    let font_path = "F:/Code/Ring-rs/assets/fonts/simhei.ttf";
    if let Err(e) = app_state.renderer.init(font_path).await {
        eprintln!("⚠️ 字体加载失败，使用默认字体: {}", e);
    }

    // 加载背景（PNG 和 JPG）
    let bg_paths = [
        "backgrounds/black.png",
        "backgrounds/white.png",
        "backgrounds/BG12_pl_n_19201440.jpg",
        "backgrounds/BG12_pl_cy_19201440.jpg",
        "backgrounds/cg1.jpg",
    ];
    for path in &bg_paths {
        // 获取规范化后的完整路径作为缓存键
        let full_path = app_state.resource_manager.resolve_path(path);
        match app_state.resource_manager.load_texture(path).await {
            Ok(texture) => {
                app_state.textures.insert(full_path, texture);
            }
            Err(e) => {
                eprintln!("❌ 加载背景失败: {} - {}", path, e);
            }
        }
    }

    // 加载角色立绘
    let char_paths = [
        "characters/北风-日常服.png",
        "characters/北风-日常服2.png",
    ];
    for path in &char_paths {
        // 获取规范化后的完整路径作为缓存键
        let full_path = app_state.resource_manager.resolve_path(path);
        match app_state.resource_manager.load_texture(path).await {
            Ok(texture) => {
                app_state.textures.insert(full_path, texture);
            }
            Err(e) => {
                eprintln!("❌ 加载角色失败: {} - {}", path, e);
            }
        }
    }

    app_state.loading_complete = true;
    println!("📦 资源加载完成！共 {} 个纹理", app_state.textures.len());

    // 尝试加载脚本
    load_script(app_state);

    // 初始化演示场景
    init_demo_scene(app_state);
}

/// 可用的脚本列表
const SCRIPTS: &[(&str, &str)] = &[
    ("demo", "F:/Code/Ring-rs/assets/scripts/demo.md"),
    ("test_comprehensive", "F:/Code/Ring-rs/assets/scripts/test_comprehensive.md"),
];

/// 加载脚本文件
fn load_script(app_state: &mut AppState) {
    let (script_id, script_path) = SCRIPTS[app_state.script_index % SCRIPTS.len()];
    
    println!("📜 加载脚本 [{}/{}]: {} ({})", 
        app_state.script_index + 1, SCRIPTS.len(), script_id, script_path);
    
    // 提取脚本所在目录作为 base_path（用于解析相对路径）
    let base_path = std::path::Path::new(script_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    
    println!("📁 脚本目录: {}", base_path);
    
    match std::fs::read_to_string(script_path) {
        Ok(script_text) => {
            let mut parser = Parser::new();
            match parser.parse_with_base_path(script_id, &script_text, &base_path) {
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
            eprintln!("❌ 脚本文件加载失败: {} - {}", script_path, e);
        }
    }
}

/// 初始化演示场景
fn init_demo_scene(app_state: &mut AppState) {
    // 设置背景
    app_state.render_state.set_background("backgrounds/black.png".to_string());
    // 设置初始等待状态
    app_state.waiting_reason = WaitingReason::WaitForClick;

    // 初始化命令队列（用于演示 CommandExecutor）
    app_state.command_queue = vec![
        // 播放 BGM 测试
        Command::PlayBgm {
            path: "bgm/Signal.mp3".to_string(),
            looping: true,
        },
        Command::ShowBackground {
            path: "backgrounds/black.png".to_string(),
            transition: None,
        },
        Command::ShowCharacter {
            path: "characters/北风-日常服.png".to_string(),
            alias: "beifeng".to_string(),
            position: Position::Center,
            transition: None,
        },
        Command::ShowText {
            speaker: Some("北风".to_string()),
            content: "你好，这是通过 CommandExecutor 执行的对话！\n按 F2 切换到命令模式，BGM 正在播放中 🎵".to_string(),
        },
        Command::PresentChoices {
            style: None,
            choices: vec![
                Choice { text: "了解更多".to_string(), target_label: "more".to_string() },
                Choice { text: "继续前进".to_string(), target_label: "continue".to_string() },
            ],
        },
        Command::ChapterMark {
            title: "第一章 命令系统".to_string(),
            level: 1,
        },
        Command::ShowBackground {
            path: "backgrounds/white.png".to_string(),
            transition: None,
        },
        Command::ShowCharacter {
            path: "characters/北风-日常服2.png".to_string(),
            alias: "beifeng2".to_string(),
            position: Position::Right,
            transition: None,
        },
        Command::ShowText {
            speaker: Some("北风".to_string()),
            content: "按空格键停止 BGM（淡出 2 秒）".to_string(),
        },
        // 停止 BGM 测试（带淡出）
        Command::StopBgm {
            fade_out: Some(2.0),
        },
        Command::ShowText {
            speaker: Some("北风".to_string()),
            content: "BGM 已停止！命令模式演示完成。按空格键重新开始。".to_string(),
        },
    ];
}

/// 窗口配置
fn window_conf() -> Conf {
    Conf {
        window_title: WINDOW_TITLE.to_string(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        window_resizable: false,
        fullscreen: false,
        ..Default::default()
    }
}

/// 更新逻辑
fn update(app_state: &mut AppState) {
    let dt = get_frame_time();

    // 检查窗口关闭
    if is_key_pressed(KeyCode::Escape) {
        app_state.host_state.stop();
    }

    // 切换调试模式
    if is_key_pressed(KeyCode::F1) {
        app_state.host_state.debug_mode = !app_state.host_state.debug_mode;
    }

    // 切换模式 (F2: 命令模式, F3: 脚本模式)
    if is_key_pressed(KeyCode::F2) {
        match app_state.run_mode {
            RunMode::Command => {
                // 从命令模式切换回演示模式
                app_state.run_mode = RunMode::Demo;
                app_state.demo_state = DemoState::ShowBackground;
                app_state.render_state = RenderState::new();
                app_state.render_state.set_background("backgrounds/black.png".to_string());
                app_state.waiting_reason = WaitingReason::WaitForClick;
                println!("🎮 切换到演示模式");
            }
            _ => {
                // 进入命令模式
                app_state.run_mode = RunMode::Command;
                app_state.command_index = 0;
                app_state.render_state = RenderState::new();
                execute_next_command(app_state);
                println!("🎮 切换到命令模式");
            }
        }
    }
    
    if is_key_pressed(KeyCode::F3) {
        if app_state.vn_runtime.is_some() {
            match app_state.run_mode {
                RunMode::Script => {
                    // 从脚本模式切换回演示模式
                    app_state.run_mode = RunMode::Demo;
                    app_state.demo_state = DemoState::ShowBackground;
                    app_state.render_state = RenderState::new();
                    app_state.render_state.set_background("backgrounds/black.png".to_string());
                    app_state.waiting_reason = WaitingReason::WaitForClick;
                    app_state.script_finished = false;
                    println!("🎮 切换到演示模式");
                }
                _ => {
                    // 进入脚本模式
                    app_state.run_mode = RunMode::Script;
                    app_state.render_state = RenderState::new();
                    app_state.script_finished = false;
                    // 重新加载脚本以重置状态
                    load_script(app_state);
                    // 执行第一次 tick
                    run_script_tick(app_state, None);
                    println!("🎮 切换到脚本模式");
                }
            }
        } else {
            println!("⚠️ 脚本未加载，无法切换到脚本模式");
        }
    }

    // F4: 切换脚本
    if is_key_pressed(KeyCode::F4) {
        app_state.script_index = (app_state.script_index + 1) % SCRIPTS.len();
        load_script(app_state);
        // 如果在脚本模式，重新开始
        if app_state.run_mode == RunMode::Script {
            app_state.render_state = RenderState::new();
            app_state.script_finished = false;
            run_script_tick(app_state, None);
        }
    }

    // 更新过渡效果
    app_state.command_executor.update_transition(dt);
    app_state.renderer.update_transition(dt);

    // 更新音频状态（淡入淡出等）
    if let Some(ref mut audio_manager) = app_state.audio_manager {
        audio_manager.update(dt);
    }

    // 音量控制快捷键
    if is_key_pressed(KeyCode::M) {
        if let Some(ref mut audio_manager) = app_state.audio_manager {
            audio_manager.toggle_mute();
            let muted = if audio_manager.is_muted() { "静音" } else { "取消静音" };
            println!("🔊 {}", muted);
        }
    }

    // 使用 InputManager 处理输入
    if let Some(input) = app_state.input_manager.update(&app_state.waiting_reason) {
        match app_state.run_mode {
            RunMode::Demo => handle_runtime_input(app_state, input),
            RunMode::Command => handle_command_mode_input(app_state, input),
            RunMode::Script => handle_script_mode_input(app_state, input),
        }
    }

    // 同步选择索引到 RenderState，并更新选择框矩形
    if let Some(ref mut choices) = app_state.render_state.choices {
        // 更新选择框矩形（用于鼠标悬停检测）
        let choice_rects = app_state.renderer.get_choice_rects(choices.choices.len());
        app_state.input_manager.set_choice_rects(choice_rects);
        
        // 同步选择索引和悬停状态
        choices.selected_index = app_state.input_manager.selected_index;
        choices.hovered_index = app_state.input_manager.hovered_index;
    }

    // 按数字键直接切换演示状态（调试用）
    if is_key_pressed(KeyCode::Key1) {
        app_state.demo_state = DemoState::ShowBackground;
        app_state.waiting_reason = WaitingReason::WaitForClick;
        app_state.render_state = RenderState::new();
        app_state.render_state.set_background("backgrounds/black.png".to_string());
    }
    if is_key_pressed(KeyCode::Key2) {
        app_state.demo_state = DemoState::ShowCharacter;
        app_state.waiting_reason = WaitingReason::WaitForClick;
        app_state.render_state.set_background("backgrounds/black.png".to_string());
        app_state.render_state.show_character(
            "beifeng".to_string(),
            "characters/北风-日常服.png".to_string(),
            Position::Center,
        );
    }
    if is_key_pressed(KeyCode::Key3) {
        app_state.demo_state = DemoState::ShowDialogue;
        app_state.waiting_reason = WaitingReason::WaitForClick;
        app_state.render_state.set_background("backgrounds/black.png".to_string());
        app_state.render_state.show_character(
            "beifeng".to_string(),
            "characters/北风-日常服.png".to_string(),
            Position::Center,
        );
        app_state.render_state.start_typewriter(
            Some("北风".to_string()),
            "你好，欢迎来到 Visual Novel Engine 的演示！这是一个使用 Rust 和 macroquad 构建的视觉小说引擎。".to_string(),
        );
        app_state.typewriter_timer = 0.0;
    }
    if is_key_pressed(KeyCode::Key4) {
        app_state.demo_state = DemoState::ShowChoices;
        app_state.waiting_reason = WaitingReason::WaitForChoice { choice_count: 3 };
        app_state.input_manager.reset_choice(3);
        app_state.render_state.set_choices(vec![
            ChoiceItem { text: "选项一：前往森林探险".to_string(), target_label: "forest".to_string() },
            ChoiceItem { text: "选项二：返回村庄休息".to_string(), target_label: "village".to_string() },
            ChoiceItem { text: "选项三：继续向前走".to_string(), target_label: "forward".to_string() },
        ], None);
    }
    if is_key_pressed(KeyCode::Key5) {
        app_state.demo_state = DemoState::ShowChapter;
        app_state.waiting_reason = WaitingReason::WaitForClick;
        app_state.render_state = RenderState::new();
        app_state.render_state.set_chapter_mark("第一章 相遇".to_string(), 1);
    }

    // 更新打字机效果
    if let Some(ref dialogue) = app_state.render_state.dialogue {
        if !dialogue.is_complete {
            app_state.typewriter_timer += dt * TYPEWRITER_SPEED;
            while app_state.typewriter_timer >= 1.0 {
                app_state.typewriter_timer -= 1.0;
                if app_state.render_state.advance_typewriter() {
                    break;
                }
            }
        }
    }
}

/// 处理来自 InputManager 的 RuntimeInput
fn handle_runtime_input(app_state: &mut AppState, input: RuntimeInput) {
    match input {
        RuntimeInput::Click => {
            handle_click(app_state);
        }
        RuntimeInput::ChoiceSelected { index } => {
            handle_choice_selected(app_state, index);
        }
        RuntimeInput::Signal { id } => {
            println!("收到信号: {}", id);
            // 信号处理暂不实现
        }
    }
}

/// 处理点击输入
fn handle_click(app_state: &mut AppState) {
    // 如果对话正在打字，先完成打字
    if !app_state.render_state.is_dialogue_complete() {
        app_state.render_state.complete_typewriter();
        return;
    }

    // 根据当前状态切换到下一个状态
    match app_state.demo_state {
        DemoState::ShowBackground => {
            app_state.demo_state = DemoState::ShowCharacter;
            app_state.waiting_reason = WaitingReason::WaitForClick;
            app_state.render_state.show_character(
                "beifeng".to_string(),
                "characters/北风-日常服.png".to_string(),
                Position::Center,
            );
        }
        DemoState::ShowCharacter => {
            app_state.demo_state = DemoState::ShowDialogue;
            app_state.waiting_reason = WaitingReason::WaitForClick;
            app_state.render_state.start_typewriter(
                Some("北风".to_string()),
                "你好，欢迎来到 Visual Novel Engine 的演示！\n这是一个使用 Rust 和 macroquad 构建的视觉小说引擎。".to_string(),
            );
            app_state.typewriter_timer = 0.0;
        }
        DemoState::ShowDialogue => {
            // 进入选择界面
            app_state.demo_state = DemoState::ShowChoices;
            app_state.waiting_reason = WaitingReason::WaitForChoice { choice_count: 3 };
            app_state.input_manager.reset_choice(3);
            app_state.render_state.clear_dialogue();
            app_state.render_state.set_choices(vec![
                ChoiceItem { text: "选项一：前往森林探险".to_string(), target_label: "forest".to_string() },
                ChoiceItem { text: "选项二：返回村庄休息".to_string(), target_label: "village".to_string() },
                ChoiceItem { text: "选项三：继续向前走".to_string(), target_label: "forward".to_string() },
            ], None);
        }
        DemoState::ShowChoices => {
            // 选择界面不响应普通点击，只响应 ChoiceSelected
        }
        DemoState::ShowChapter => {
            app_state.demo_state = DemoState::Complete;
            app_state.waiting_reason = WaitingReason::WaitForClick;
            app_state.render_state.clear_chapter_mark();
            app_state.render_state.set_background("backgrounds/white.png".to_string());
            app_state.render_state.show_character(
                "beifeng2".to_string(),
                "characters/北风-日常服2.png".to_string(),
                Position::Right,
            );
            app_state.render_state.set_dialogue(
                Some("北风".to_string()),
                "演示完成！按空格键或点击屏幕重新开始。".to_string(),
            );
        }
        DemoState::Complete => {
            // 重新开始演示
            app_state.demo_state = DemoState::ShowBackground;
            app_state.waiting_reason = WaitingReason::WaitForClick;
            app_state.render_state = RenderState::new();
            app_state.render_state.set_background("backgrounds/black.png".to_string());
        }
    }
}

/// 处理选择输入
fn handle_choice_selected(app_state: &mut AppState, index: usize) {
    if app_state.demo_state != DemoState::ShowChoices {
        return;
    }

    // 获取选择的选项
    let choice_text = app_state.render_state.choices
        .as_ref()
        .and_then(|c| c.choices.get(index))
        .map(|item| item.text.clone())
        .unwrap_or_else(|| format!("选项 {}", index + 1));

    println!("✅ 用户选择了: {} (索引: {})", choice_text, index);

    // 清除选择界面，显示章节标题
    app_state.demo_state = DemoState::ShowChapter;
    app_state.waiting_reason = WaitingReason::WaitForClick;
    app_state.render_state.clear_choices();
    app_state.render_state.hide_all_characters();
    app_state.render_state.set_chapter_mark("第一章 相遇".to_string(), 1);
}

/// 处理命令模式下的输入
fn handle_command_mode_input(app_state: &mut AppState, input: RuntimeInput) {
    match input {
        RuntimeInput::Click => {
            // 如果对话正在打字，先完成打字
            if !app_state.render_state.is_dialogue_complete() {
                app_state.render_state.complete_typewriter();
                return;
            }

            // 执行下一条命令
            execute_next_command(app_state);
        }
        RuntimeInput::ChoiceSelected { index } => {
            // 获取选择的选项
            let choice_text = app_state.render_state.choices
                .as_ref()
                .and_then(|c| c.choices.get(index))
                .map(|item| item.text.clone())
                .unwrap_or_else(|| format!("选项 {}", index + 1));

            println!("✅ [命令模式] 用户选择了: {} (索引: {})", choice_text, index);

            // 清除选择界面，执行下一条命令
            app_state.render_state.clear_choices();
            execute_next_command(app_state);
        }
        RuntimeInput::Signal { id } => {
            println!("收到信号: {}", id);
        }
    }
}

/// 执行下一条命令
fn execute_next_command(app_state: &mut AppState) {
    if app_state.command_index >= app_state.command_queue.len() {
        // 命令执行完毕，重新开始
        app_state.command_index = 0;
        app_state.render_state = RenderState::new();
        println!("🔄 命令执行完毕，重新开始");
    }

    // 获取当前命令
    let command = app_state.command_queue[app_state.command_index].clone();
    app_state.command_index += 1;

    println!("▶️ 执行命令 {}: {:?}", app_state.command_index, command);

    // 执行命令
    let result = app_state.command_executor.execute(
        &command,
        &mut app_state.render_state,
        &app_state.resource_manager,
    );

    // 应用过渡效果
    apply_transition_effect(app_state);
    
    // 处理音频命令
    handle_audio_command(app_state);

    // 根据执行结果设置等待状态
    match result {
        ExecuteResult::Ok => {
            // 继续执行下一条命令
            execute_next_command(app_state);
        }
        ExecuteResult::WaitForClick => {
            app_state.waiting_reason = WaitingReason::WaitForClick;
            app_state.typewriter_timer = 0.0;
        }
        ExecuteResult::WaitForChoice { choice_count } => {
            app_state.waiting_reason = WaitingReason::WaitForChoice { choice_count };
            app_state.input_manager.reset_choice(choice_count);
        }
        ExecuteResult::WaitForTime(ms) => {
            app_state.waiting_reason = WaitingReason::WaitForTime(
                std::time::Duration::from_millis(ms)
            );
        }
        ExecuteResult::Loading => {
            // 资源加载中，等待
            app_state.waiting_reason = WaitingReason::None;
        }
        ExecuteResult::Error(e) => {
            eprintln!("❌ 命令执行失败: {}", e);
            // 跳过错误，继续执行
            execute_next_command(app_state);
        }
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
// 脚本模式处理
//=============================================================================

/// 处理脚本模式下的输入
fn handle_script_mode_input(app_state: &mut AppState, input: RuntimeInput) {
    // 如果对话正在打字，先完成打字
    if !app_state.render_state.is_dialogue_complete() {
        app_state.render_state.complete_typewriter();
        return;
    }

    // 如果脚本已执行完毕，重新加载
    if app_state.script_finished {
        println!("🔄 脚本执行完毕，重新开始");
        load_script(app_state);
        app_state.render_state = RenderState::new();
        app_state.script_finished = false;
        run_script_tick(app_state, None);
        return;
    }

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
            if is_finished {
                app_state.script_finished = true;
                println!("📜 脚本执行完毕！按空格键重新开始");
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
fn draw(app_state: &AppState) {
    // 使用渲染器渲染
    app_state.renderer.render(&app_state.render_state, &app_state.textures, &app_state.resource_manager, &app_state.manifest);

    // 显示调试信息
    if app_state.host_state.debug_mode {
        draw_debug_info(app_state);
    }

    // 显示操作提示
    draw_help_text(app_state);
}

/// 绘制调试信息
fn draw_debug_info(app_state: &AppState) {
    let fps = get_fps();
    let texture_count = app_state.textures.len();
    let char_count = app_state.render_state.visible_characters.len();
    let has_bg = app_state.render_state.current_background.is_some();
    let has_dialogue = app_state.render_state.dialogue.is_some();

    // 绘制半透明背景
    draw_rectangle(5.0, 5.0, 280.0, 140.0, Color::new(0.0, 0.0, 0.0, 0.7));
    
    // 调试信息使用自定义字体
    let lines = [
        format!("FPS: {}", fps),
        format!("纹理数量: {}", texture_count),
        format!("角色数量: {}", char_count),
        format!("背景: {}", has_bg),
        format!("对话: {}", has_dialogue),
        format!("状态: {:?}", app_state.demo_state),
    ];
    
    for (i, line) in lines.iter().enumerate() {
        app_state.renderer.text_renderer.draw_ui_text(
            line,
            10.0,
            25.0 + i as f32 * 20.0,
            16.0,
            GREEN,
        );
    }
}

/// 绘制帮助文本
fn draw_help_text(app_state: &AppState) {
    let mode_text = match app_state.run_mode {
        RunMode::Demo => "[演示模式]",
        RunMode::Command => "[命令模式]",
        RunMode::Script => "[脚本模式]",
    };
    
    let help_text = match app_state.run_mode {
        RunMode::Script => {
            if app_state.script_finished {
                "空格键:重新开始"
            } else {
                match &app_state.waiting_reason {
                    WaitingReason::WaitForChoice { .. } => "↑↓选择 回车确认",
                    _ => "空格键:下一步",
                }
            }
        }
        RunMode::Command => {
            match &app_state.waiting_reason {
                WaitingReason::WaitForChoice { .. } => "↑↓选择 回车确认",
                _ => "空格键:下一步",
            }
        }
        RunMode::Demo => {
            match app_state.demo_state {
                DemoState::ShowBackground => "空格键:显示角色",
                DemoState::ShowCharacter => "空格键:显示对话",
                DemoState::ShowDialogue => "空格键:显示选项",
                DemoState::ShowChoices => "↑↓选择 回车确认",
                DemoState::ShowChapter => "空格键:继续",
                DemoState::Complete => "空格键:重新开始",
            }
        }
    };

    let screen_h = screen_height();
    
    // 底部提示（使用自定义字体）
    let script_name = SCRIPTS[app_state.script_index % SCRIPTS.len()].0;
    app_state.renderer.text_renderer.draw_ui_text(
        &format!("{} {} | ESC退出 | F1调试 | F2命令 | F3脚本 | F4切换脚本({})", mode_text, help_text, script_name),
        10.0,
        screen_h - 10.0,
        18.0,
        Color::new(1.0, 1.0, 1.0, 0.7),
    );
}
