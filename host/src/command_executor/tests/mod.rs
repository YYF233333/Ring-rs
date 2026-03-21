use super::*;
use crate::test_harness;
use vn_runtime::command::{Choice, Position, Transition};

struct TestCtx {
    executor: CommandExecutor,
    render_state: RenderState,
    resource_manager: ResourceManager,
}

impl TestCtx {
    fn new() -> Self {
        Self {
            executor: CommandExecutor::new(),
            render_state: RenderState::new(),
            resource_manager: test_harness::null_resource_manager(),
        }
    }

    fn execute(&mut self, cmd: &Command) -> ExecuteResult {
        self.executor
            .execute(cmd, &mut self.render_state, &self.resource_manager)
    }

    fn execute_batch(&mut self, commands: &[Command]) -> ExecuteResult {
        self.executor
            .execute_batch(commands, &mut self.render_state, &self.resource_manager)
    }
}

mod high_value;
mod low_value;
