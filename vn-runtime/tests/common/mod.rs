//! 共享集成测试基础设施
//!
//! 提供 `ScriptTestHarness`，封装"脚本文本 → Runtime → tick → 断言 Commands"的常见模式，
//! 消除各 integration test 中的重复 boilerplate。

#![allow(dead_code)]

use vn_runtime::{Command, Parser, RuntimeInput, SaveData, Script, VNRuntime, WaitingReason};

// ── ScriptTestHarness ────────────────────────────────────────────────

/// 集成测试用 harness：从脚本文本构建 Runtime 并提供便捷的 tick/断言操作。
pub struct ScriptTestHarness {
    runtime: VNRuntime,
    script_text: String,
}

impl ScriptTestHarness {
    /// 从脚本文本构建 harness。解析失败会 panic（测试场景下合理）。
    pub fn new(script_text: &str) -> Self {
        let script = Self::parse_script(script_text);
        Self {
            runtime: VNRuntime::new(script),
            script_text: script_text.to_owned(),
        }
    }

    /// 首次 tick（无用户输入）。
    pub fn tick(&mut self) -> TickResult {
        let (commands, waiting) = self.runtime.tick(None).expect("tick failed");
        TickResult { commands, waiting }
    }

    /// 模拟用户点击后 tick。
    pub fn click(&mut self) -> TickResult {
        let (commands, waiting) = self
            .runtime
            .tick(Some(RuntimeInput::Click))
            .expect("click-tick failed");
        TickResult { commands, waiting }
    }

    /// 模拟用户选择分支后 tick。
    pub fn choose(&mut self, index: usize) -> TickResult {
        let (commands, waiting) = self
            .runtime
            .tick(Some(RuntimeInput::ChoiceSelected { index }))
            .expect("choose-tick failed");
        TickResult { commands, waiting }
    }

    /// 保存当前状态，从 JSON 恢复为新 harness。验证 round-trip 可行性。
    pub fn save_and_restore(&self) -> Self {
        let state = self.runtime.state().clone();
        let history = self.runtime.history().clone();
        let save_data = SaveData::new(1, state);
        let json = save_data.to_json().expect("save to JSON failed");
        let loaded = SaveData::from_json(&json).expect("load from JSON failed");
        let script = Self::parse_script(&self.script_text);
        Self {
            runtime: VNRuntime::restore(script, loaded.runtime_state, history),
            script_text: self.script_text.clone(),
        }
    }

    pub fn runtime(&self) -> &VNRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut VNRuntime {
        &mut self.runtime
    }

    fn parse_script(text: &str) -> Script {
        let mut parser = Parser::new();
        parser.parse("test", text).expect("script parse failed")
    }
}

// ── TickResult ───────────────────────────────────────────────────────

/// 单次 tick 的结果，附带便捷查询和断言方法。
pub struct TickResult {
    pub commands: Vec<Command>,
    pub waiting: WaitingReason,
}

impl TickResult {
    /// 是否包含指定内容的 ShowText（不限说话人）。
    pub fn has_text(&self, content: &str) -> bool {
        self.commands
            .iter()
            .any(|c| matches!(c, Command::ShowText { content: c, .. } if c == content))
    }

    /// 是否包含指定说话人 + 内容的 ShowText。
    pub fn has_text_from(&self, speaker: &str, content: &str) -> bool {
        self.commands.iter().any(|c| {
            matches!(c, Command::ShowText { speaker: Some(s), content: ct, .. }
                if s == speaker && ct == content)
        })
    }

    /// 是否包含无说话人（旁白）的 ShowText。
    pub fn has_narration(&self, content: &str) -> bool {
        self.commands.iter().any(|c| {
            matches!(c, Command::ShowText { speaker: None, content: ct, .. }
                if ct == content)
        })
    }

    /// 是否包含指定路径的 ShowBackground。
    pub fn has_background(&self, path: &str) -> bool {
        self.commands
            .iter()
            .any(|c| matches!(c, Command::ShowBackground { path: p, .. } if p == path))
    }

    /// 是否包含指定 alias 的 ShowCharacter。
    pub fn has_character(&self, alias: &str) -> bool {
        self.commands
            .iter()
            .any(|c| matches!(c, Command::ShowCharacter { alias: a, .. } if a == alias))
    }

    /// 断言等待用户点击。
    #[track_caller]
    pub fn assert_waiting_click(&self) {
        assert_eq!(
            self.waiting,
            WaitingReason::WaitForClick,
            "expected WaitForClick, got: {:?}",
            self.waiting
        );
    }

    /// 断言不处于等待状态。
    #[track_caller]
    pub fn assert_not_waiting(&self) {
        assert_eq!(
            self.waiting,
            WaitingReason::None,
            "expected no waiting, got: {:?}",
            self.waiting
        );
    }

    /// 断言等待用户选择，并返回选项数量。
    #[track_caller]
    pub fn assert_waiting_choice(&self) -> usize {
        match self.waiting {
            WaitingReason::WaitForChoice { choice_count } => choice_count,
            ref other => panic!("expected WaitForChoice, got: {other:?}"),
        }
    }

    /// 断言等待指定时长（允许 10ms 误差）。
    #[track_caller]
    pub fn assert_waiting_time(&self, expected_secs: f64) {
        match self.waiting {
            WaitingReason::WaitForTime(d) => {
                let actual = d.as_secs_f64();
                assert!(
                    (actual - expected_secs).abs() < 0.01,
                    "expected WaitForTime({expected_secs}s), got WaitForTime({actual}s)"
                );
            }
            ref other => panic!("expected WaitForTime, got: {other:?}"),
        }
    }
}
