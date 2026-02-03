//! # 命令执行集成测试
//!
//! 测试 Runtime → CommandExecutor → RenderState 的执行链路。
//! 这些测试不依赖真实的渲染/音频设备。

use host::command_executor::{CommandExecutor, ExecuteResult};
use host::renderer::RenderState;
use host::resources::ResourceManager;
use vn_runtime::command::{Choice, Command, Position, Transition, TransitionArg};

/// 创建测试用的 ResourceManager（不需要真实文件）
fn test_resource_manager() -> ResourceManager {
    ResourceManager::new("assets", 64)
}

/// 测试基本的对话流程
#[test]
fn test_dialogue_flow() {
    let mut executor = CommandExecutor::new();
    let mut state = RenderState::new();
    let rm = test_resource_manager();

    // 1. 设置背景
    let cmd = Command::ShowBackground {
        path: "backgrounds/room.png".to_string(),
        transition: None,
    };
    let result = executor.execute(&cmd, &mut state, &rm);
    assert_eq!(result, ExecuteResult::Ok);
    assert_eq!(
        state.current_background,
        Some("backgrounds/room.png".to_string())
    );

    // 2. 显示角色
    let cmd = Command::ShowCharacter {
        path: "characters/alice.png".to_string(),
        alias: "alice".to_string(),
        position: Position::Center,
        transition: None,
    };
    let result = executor.execute(&cmd, &mut state, &rm);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(state.visible_characters.contains_key("alice"));

    // 3. 显示对话
    let cmd = Command::ShowText {
        speaker: Some("Alice".to_string()),
        content: "你好！".to_string(),
    };
    let result = executor.execute(&cmd, &mut state, &rm);
    assert_eq!(result, ExecuteResult::WaitForClick);
    assert!(state.dialogue.is_some());
    let dialogue = state.dialogue.as_ref().unwrap();
    assert_eq!(dialogue.speaker, Some("Alice".to_string()));

    // 4. 隐藏角色
    let cmd = Command::HideCharacter {
        alias: "alice".to_string(),
        transition: None,
    };
    let result = executor.execute(&cmd, &mut state, &rm);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(!state.visible_characters.contains_key("alice"));
}

/// 测试选择分支流程
#[test]
fn test_choice_flow() {
    let mut executor = CommandExecutor::new();
    let mut state = RenderState::new();
    let rm = test_resource_manager();

    // 显示对话（引入选择）
    let cmd = Command::ShowText {
        speaker: Some("旁白".to_string()),
        content: "你会怎么选择？".to_string(),
    };
    executor.execute(&cmd, &mut state, &rm);
    assert!(state.dialogue.is_some());

    // 显示选择
    let cmd = Command::PresentChoices {
        style: None,
        choices: vec![
            Choice {
                text: "选项 A".to_string(),
                target_label: "route_a".to_string(),
            },
            Choice {
                text: "选项 B".to_string(),
                target_label: "route_b".to_string(),
            },
            Choice {
                text: "选项 C".to_string(),
                target_label: "route_c".to_string(),
            },
        ],
    };
    let result = executor.execute(&cmd, &mut state, &rm);
    assert_eq!(result, ExecuteResult::WaitForChoice { choice_count: 3 });

    // 验证选择状态
    let choices = state.choices.as_ref().unwrap();
    assert_eq!(choices.choices.len(), 3);
    assert_eq!(choices.choices[0].text, "选项 A");
    assert_eq!(choices.choices[2].target_label, "route_c");

    // 对话框应该被清空（选择界面会清除对话）
    assert!(state.dialogue.is_none());
}

/// 测试章节标记
#[test]
fn test_chapter_mark() {
    let mut executor = CommandExecutor::new();
    let mut state = RenderState::new();
    let rm = test_resource_manager();

    let cmd = Command::ChapterMark {
        title: "序章：开始".to_string(),
        level: 1,
    };
    let result = executor.execute(&cmd, &mut state, &rm);
    assert_eq!(result, ExecuteResult::WaitForClick);

    let chapter = state.chapter_mark.as_ref().unwrap();
    assert_eq!(chapter.title, "序章：开始");
    assert_eq!(chapter.level, 1);
}

/// 测试音频命令输出
#[test]
fn test_audio_commands() {
    let mut executor = CommandExecutor::new();
    let mut state = RenderState::new();
    let rm = test_resource_manager();

    // 播放 BGM
    let cmd = Command::PlayBgm {
        path: "bgm/main_theme.mp3".to_string(),
        looping: true,
    };
    executor.execute(&cmd, &mut state, &rm);
    assert!(executor.last_output.audio_command.is_some());

    // 播放音效
    let cmd = Command::PlaySfx {
        path: "sfx/click.wav".to_string(),
    };
    executor.execute(&cmd, &mut state, &rm);
    assert!(executor.last_output.audio_command.is_some());

    // 停止 BGM
    let cmd = Command::StopBgm {
        fade_out: Some(2.0),
    };
    executor.execute(&cmd, &mut state, &rm);
    assert!(executor.last_output.audio_command.is_some());
}

/// 测试带过渡效果的背景切换
#[test]
fn test_background_with_transition() {
    let mut executor = CommandExecutor::new();
    let mut state = RenderState::new();
    let rm = test_resource_manager();

    // 设置初始背景
    state.set_background("old_bg.png".to_string());

    // 带过渡效果切换背景
    let transition = Transition::with_args(
        "dissolve",
        vec![TransitionArg::Number(0.5)], // 0.5 秒
    );
    let cmd = Command::ShowBackground {
        path: "new_bg.png".to_string(),
        transition: Some(transition),
    };
    let result = executor.execute(&cmd, &mut state, &rm);
    assert_eq!(result, ExecuteResult::Ok);

    // 检查过渡信息
    assert!(
        executor
            .last_output
            .transition_info
            .has_background_transition
    );
    assert_eq!(
        executor.last_output.transition_info.old_background,
        Some("old_bg.png".to_string())
    );
}

/// 测试多角色场景
#[test]
fn test_multiple_characters() {
    let mut executor = CommandExecutor::new();
    let mut state = RenderState::new();
    let rm = test_resource_manager();

    // 显示多个角色
    for (alias, pos) in [
        ("alice", Position::Left),
        ("bob", Position::Center),
        ("carol", Position::Right),
    ] {
        let cmd = Command::ShowCharacter {
            path: format!("characters/{}.png", alias),
            alias: alias.to_string(),
            position: pos,
            transition: None,
        };
        executor.execute(&cmd, &mut state, &rm);
    }

    assert_eq!(state.visible_characters.len(), 3);
    assert!(state.visible_characters.contains_key("alice"));
    assert!(state.visible_characters.contains_key("bob"));
    assert!(state.visible_characters.contains_key("carol"));

    // 验证位置
    assert_eq!(
        state.visible_characters.get("alice").unwrap().position,
        Position::Left
    );
    assert_eq!(
        state.visible_characters.get("carol").unwrap().position,
        Position::Right
    );
}

/// 测试 changeScene 的 UI 隐藏行为
#[test]
fn test_change_scene_hides_ui() {
    let mut executor = CommandExecutor::new();
    let mut state = RenderState::new();
    let rm = test_resource_manager();

    // 设置初始状态
    state.set_background("old_scene.png".to_string());
    state.show_character(
        "alice".to_string(),
        "alice.png".to_string(),
        Position::Center,
    );
    assert!(state.ui_visible);
    assert!(!state.visible_characters.is_empty());

    // 执行 changeScene (fade)
    let transition = Transition::simple("fade");
    let cmd = Command::ChangeScene {
        path: "new_scene.png".to_string(),
        transition: Some(transition),
    };
    executor.execute(&cmd, &mut state, &rm);

    // changeScene 应该：
    // 1. 隐藏 UI
    // 2. 清除所有角色
    // 3. 产生场景切换命令
    assert!(!state.ui_visible);
    assert!(state.visible_characters.is_empty());
    assert!(executor.last_output.scene_transition.is_some());
}
