//! 系统模块

mod setup;
mod input;
mod runtime;
mod commands;
mod ui;

pub use setup::setup_system;
pub use input::input_system;
pub use runtime::tick_runtime_system;
pub use commands::execute_commands_system;
pub use ui::{update_dialogue_system, update_characters_system};

