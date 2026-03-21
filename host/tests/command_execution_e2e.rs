//! 端到端集成测试：脚本 → Parser → VNRuntime → CommandExecutor → RenderState 验证。
//!
//! 验证 vn-runtime 的 Command 输出能被 host 的 CommandExecutor 正确消费，
//! 确保两个 crate 间的契约不发生静默不兼容。

use std::sync::Arc;

use host::{CommandExecutor, NullTextureFactory, RenderState, ResourceManager, TextureContext};
use vn_runtime::{Parser, RuntimeInput, VNRuntime};

fn setup() -> (VNRuntime, CommandExecutor, RenderState, ResourceManager) {
    let script_text = "\
changeBG <img src=\"bg/school.jpg\" />

羽艾：\"你好。\"

show <img src=\"char/a.png\" /> as alice at center with dissolve

路汐：\"早上好。\"

hide alice with fade

clearCharacters

stopBGM";

    let mut parser = Parser::new();
    let script = parser.parse("test", script_text).unwrap();
    let runtime = VNRuntime::new(script);
    let executor = CommandExecutor::new();
    let render_state = RenderState::new();
    let mut resource_manager = ResourceManager::new("assets", 256);
    resource_manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    (runtime, executor, render_state, resource_manager)
}

#[test]
fn all_commands_execute_without_panic() {
    let (mut runtime, mut executor, mut render_state, resource_manager) = setup();

    let mut tick_count = 0;
    loop {
        let input = if tick_count == 0 {
            None
        } else {
            Some(RuntimeInput::Click)
        };

        let (commands, _waiting) = runtime.tick(input).unwrap();

        for cmd in &commands {
            let _result = executor.execute(cmd, &mut render_state, &resource_manager);
        }

        tick_count += 1;

        if runtime.is_finished() || tick_count > 20 {
            break;
        }
    }

    assert!(tick_count > 1, "should have ticked more than once");
}

#[test]
fn show_background_updates_render_state() {
    let mut parser = Parser::new();
    let script = parser
        .parse("test", r#"changeBG <img src="bg/sky.jpg" />"#)
        .unwrap();
    let mut runtime = VNRuntime::new(script);
    let mut executor = CommandExecutor::new();
    let mut render_state = RenderState::new();
    let mut resource_manager = ResourceManager::new("assets", 256);
    resource_manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let (commands, _) = runtime.tick(None).unwrap();
    for cmd in &commands {
        executor.execute(cmd, &mut render_state, &resource_manager);
    }

    assert_eq!(
        render_state.current_background.as_deref(),
        Some("bg/sky.jpg"),
        "render state should reflect the new background"
    );
}

#[test]
fn show_text_updates_dialogue_state() {
    let mut parser = Parser::new();
    let script = parser.parse("test", r#"羽艾："测试对话""#).unwrap();
    let mut runtime = VNRuntime::new(script);
    let mut executor = CommandExecutor::new();
    let mut render_state = RenderState::new();
    let resource_manager = ResourceManager::new("assets", 256);

    let (commands, _) = runtime.tick(None).unwrap();
    for cmd in &commands {
        executor.execute(cmd, &mut render_state, &resource_manager);
    }

    let dialogue = render_state.dialogue.as_ref();
    assert!(dialogue.is_some(), "dialogue state should be set");
    let d = dialogue.unwrap();
    assert_eq!(d.speaker.as_deref(), Some("羽艾"));
    assert_eq!(d.content, "测试对话");
}

#[test]
fn executor_handles_unknown_bg_gracefully() {
    let mut parser = Parser::new();
    let script = parser
        .parse(
            "test",
            r#"changeBG <img src="nonexistent.png" /> with dissolve"#,
        )
        .unwrap();
    let mut runtime = VNRuntime::new(script);
    let mut executor = CommandExecutor::new();
    let mut render_state = RenderState::new();
    let resource_manager = ResourceManager::new("assets", 256);

    let (commands, _) = runtime.tick(None).unwrap();
    for cmd in &commands {
        let _result = executor.execute(cmd, &mut render_state, &resource_manager);
    }

    assert_eq!(
        render_state.current_background.as_deref(),
        Some("nonexistent.png"),
        "background path should be set even if texture load fails"
    );
}
