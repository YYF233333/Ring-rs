//! # Parser 测试

use super::*;
#[allow(unused_imports)]
use crate::command::{Position, TextMode, TransitionArg};
use crate::script::ast::ScriptNode;

mod high_value;
mod low_value;
mod snapshots;

fn parse_ok(input: &str) -> crate::script::ast::Script {
    let mut parser = Parser::new();
    parser.parse("test", input).unwrap()
}

fn parse_single_node(input: &str) -> ScriptNode {
    let script = parse_ok(input);
    assert_eq!(script.nodes.len(), 1, "Expected exactly one node");
    script.nodes.into_iter().next().unwrap()
}

fn parse_err(input: &str) -> crate::error::ParseError {
    let mut parser = Parser::new();
    parser.parse("test", input).unwrap_err()
}
