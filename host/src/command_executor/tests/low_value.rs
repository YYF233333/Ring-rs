use super::*;

#[test]
fn test_executor_creation() {
    let executor = CommandExecutor::new();
    assert!(executor.last_output.effect_requests.is_empty());
}

#[test]
fn test_show_background_no_transition_no_effect() {
    let mut ctx = TestCtx::new();
    let cmd = Command::ShowBackground {
        path: "bg.png".to_string(),
        transition: None,
    };

    ctx.execute(&cmd);
    assert!(ctx.executor.last_output.effect_requests.is_empty());
    assert_eq!(
        ctx.render_state.current_background,
        Some("bg.png".to_string())
    );
}

#[test]
fn test_full_restart_is_noop() {
    let mut ctx = TestCtx::new();
    let result = ctx.execute(&Command::FullRestart);
    assert_eq!(result, ExecuteResult::Ok);
    assert!(ctx.executor.last_output.effect_requests.is_empty());
    assert!(ctx.executor.last_output.audio_command.is_none());
}
