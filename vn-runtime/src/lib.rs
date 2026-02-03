//! # VN Runtime
//!
//! Visual Novel Engine 的核心运行时库。
//!
//! ## 架构概述
//!
//! `vn-runtime` 是纯逻辑核心，不依赖任何 IO 或渲染引擎。
//! 它通过 **命令驱动模式** 与宿主层（Host）通信：
//!
//! ```text
//! Host                          Runtime
//!   │                              │
//!   │──── RuntimeInput ──────────►│
//!   │                              │ tick()
//!   │◄─── (Vec<Command>, WaitingReason) ──│
//!   │                              │
//! ```
//!
//! ## 核心类型
//!
//! - [`Command`]：Runtime 向 Host 发出的指令
//! - [`RuntimeInput`]：Host 向 Runtime 传递的输入
//! - [`WaitingReason`]：Runtime 的等待状态
//! - [`RuntimeState`]：可序列化的运行时状态
//!
//! ## 使用示例
//!
//! ```ignore
//! use vn_runtime::{VNRuntime, RuntimeInput};
//!
//! // 加载脚本并创建 Runtime
//! let script = Script::parse(script_text)?;
//! let mut runtime = VNRuntime::new(script);
//!
//! // 主循环
//! loop {
//!     let (commands, waiting) = runtime.tick(input);
//!     
//!     // Host 执行 commands
//!     for cmd in commands {
//!         host.execute(cmd);
//!     }
//!     
//!     // 根据 waiting 状态采集输入
//!     input = match waiting {
//!         WaitingReason::None => None,
//!         WaitingReason::WaitForClick => wait_for_click(),
//!         WaitingReason::WaitForChoice { .. } => wait_for_choice(),
//!         WaitingReason::WaitForTime(duration) => {
//!             sleep(duration);
//!             None
//!         }
//!         // ...
//!     };
//! }
//! ```
//!
//! ## 模块结构
//!
//! - [`command`]：Command 定义
//! - [`input`]：RuntimeInput 定义
//! - [`state`]：RuntimeState 和 WaitingReason 定义
//! - [`error`]：错误类型定义
//! - [`script`]：脚本解析（AST 和 Parser）
//! - [`runtime`]：执行引擎

pub mod command;
pub mod diagnostic;
pub mod error;
pub mod history;
pub mod input;
pub mod runtime;
pub mod save;
pub mod script;
pub mod state;

// 重导出核心类型
pub use command::{Choice, Command, Position, Transition, TransitionArg};
pub use diagnostic::{
    Diagnostic, DiagnosticLevel, DiagnosticResult, ResourceReference, ResourceType, analyze_script,
    extract_resource_references, get_defined_labels, get_jump_targets,
};
pub use error::{ParseError, RuntimeError, VnError, VnResult};
pub use history::{History, HistoryEvent};
pub use input::{RuntimeInput, SignalId};
pub use runtime::VNRuntime;
pub use save::{
    AudioState, CharacterSnapshot, RenderSnapshot, SaveData, SaveError, SaveMetadata, SaveVersion,
};
pub use script::{ChoiceOption, Parser, Script, ScriptNode};
pub use state::{RuntimeState, ScriptPosition, VarValue, WaitingReason};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_accessible() {
        // 验证所有公共类型都可以正常使用
        let _cmd = Command::ShowText {
            speaker: Some("Test".to_string()),
            content: "Hello".to_string(),
        };

        let _input = RuntimeInput::Click;

        let _waiting = WaitingReason::WaitForClick;

        let _state = RuntimeState::new("main");
    }
}
