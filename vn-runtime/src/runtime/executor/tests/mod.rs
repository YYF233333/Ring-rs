use super::*;
use crate::command::{Position, TextMode, Transition, TransitionArg};
use crate::script::{ChoiceOption, Expr};
use crate::state::VarValue;

mod high_value;
mod low_value;

fn test_ctx(script_root: &str) -> (Executor, RuntimeState, Script) {
    (
        Executor::new(),
        RuntimeState::new("test"),
        Script::new("test", vec![], script_root),
    )
}

fn test_env() -> (Executor, RuntimeState) {
    (Executor::new(), RuntimeState::new("test"))
}
