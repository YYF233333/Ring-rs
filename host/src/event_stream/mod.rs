//! 结构化事件流调试基础设施
//!
//! 在引擎关键边界点产出 JSON Lines 格式的 `EngineEvent`，
//! 供 AI 分析定位 root cause。
//!
//! GUI 模式下通过 `--event-stream=<path>` 启用；
//! headless 模式下默认启用（自动生成路径或由 CLI 指定）。

use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

// ─── EngineEvent ────────────────────────────────────────────────────

/// 引擎结构化事件（对应事件流的一行）
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum EngineEvent {
    ScriptTick {
        node_index: usize,
        commands_count: usize,
        waiting_reason: String,
    },
    CommandProduced {
        variant: String,
        summary: serde_json::Value,
    },
    CommandExecuted {
        variant: String,
        result: String,
    },
    StateChanged {
        field: String,
        from: String,
        to: String,
    },
    InputReceived {
        variant: String,
    },
    TransitionUpdate {
        transition_type: String,
        phase: String,
        progress: f32,
    },
    AudioEvent {
        action: String,
        path: Option<String>,
        volume: Option<f32>,
    },
}

// ─── TimeSource ─────────────────────────────────────────────────────

/// 事件流时间基准
pub enum TimeSource {
    /// 使用 `Instant::elapsed()` 计算挂钟时间（GUI 模式）
    Wall(Instant),
    /// 由调用方传入逻辑累积时间（headless 模式）
    Logical,
}

// ─── TestEventCollector（测试专用） ─────────────────────────────────

/// 内存事件收集器，用于测试中断言事件序列。
#[cfg(test)]
pub(crate) struct TestEventCollector {
    events: Vec<(u64, EngineEvent)>,
}

#[cfg(test)]
impl TestEventCollector {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn events(&self) -> &[(u64, EngineEvent)] {
        &self.events
    }

    pub fn drain(&mut self) -> Vec<(u64, EngineEvent)> {
        std::mem::take(&mut self.events)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    fn push(&mut self, ts_ms: u64, event: EngineEvent) {
        self.events.push((ts_ms, event));
    }
}

// ─── EventStream ────────────────────────────────────────────────────

/// 结构化事件流写入器
///
/// `writer` 为 `None` 时所有 `emit` 调用为 no-op（零开销禁用）。
/// Logical 模式下由调用方每帧设置当前逻辑时间（见 `set_logical_time_ms`）。
pub struct EventStream {
    writer: Option<BufWriter<std::fs::File>>,
    time_source: TimeSource,
    /// Logical 模式下使用的当前逻辑时间（ms），每帧由 headless 设置
    logical_ms: u64,
    /// 测试专用：内存事件收集器
    #[cfg(test)]
    collector: Option<TestEventCollector>,
}

impl EventStream {
    /// 创建已启用的事件流（Wall clock 模式）
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        Ok(Self {
            writer: Some(BufWriter::new(file)),
            time_source: TimeSource::Wall(Instant::now()),
            logical_ms: 0,
            #[cfg(test)]
            collector: None,
        })
    }

    /// 创建已启用的事件流（Logical 时间模式，headless 用）
    pub fn new_logical(path: &Path) -> Result<Self, std::io::Error> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        Ok(Self {
            writer: Some(BufWriter::new(file)),
            time_source: TimeSource::Logical,
            logical_ms: 0,
            #[cfg(test)]
            collector: None,
        })
    }

    /// 创建禁用的事件流（所有操作为 no-op）
    pub fn disabled() -> Self {
        Self {
            writer: None,
            time_source: TimeSource::Logical,
            logical_ms: 0,
            #[cfg(test)]
            collector: None,
        }
    }

    /// 创建内存模式的事件流（测试专用），事件收集到内存中供断言。
    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        Self {
            writer: None,
            time_source: TimeSource::Logical,
            logical_ms: 0,
            collector: Some(TestEventCollector::new()),
        }
    }

    /// 获取内存收集器的引用（测试专用）。
    #[cfg(test)]
    pub(crate) fn collector(&self) -> Option<&TestEventCollector> {
        self.collector.as_ref()
    }

    /// 获取内存收集器的可变引用（测试专用）。
    #[cfg(test)]
    pub(crate) fn collector_mut(&mut self) -> Option<&mut TestEventCollector> {
        self.collector.as_mut()
    }

    /// 设置当前逻辑时间（仅 Logical 模式有效）。Headless 应在每帧开始时调用。
    pub fn set_logical_time_ms(&mut self, ms: u64) {
        self.logical_ms = ms;
    }

    /// 事件流是否启用
    pub fn is_enabled(&self) -> bool {
        self.writer.is_some()
    }

    /// 发出事件（Wall clock 模式：自动计算 ts_ms）
    pub fn emit(&mut self, event: EngineEvent) {
        let ts_ms = match &self.time_source {
            TimeSource::Wall(start) => start.elapsed().as_millis() as u64,
            TimeSource::Logical => self.logical_ms,
        };

        #[cfg(test)]
        if let Some(ref mut collector) = self.collector {
            collector.push(ts_ms, event.clone());
        }

        let Some(ref mut writer) = self.writer else {
            return;
        };
        Self::write_event(writer, ts_ms, &event);
    }

    /// 脚本 tick 完成
    pub fn on_script_tick(
        &mut self,
        node_index: usize,
        commands_count: usize,
        waiting_reason: &str,
    ) {
        self.emit(EngineEvent::ScriptTick {
            node_index,
            commands_count,
            waiting_reason: waiting_reason.to_string(),
        });
    }

    /// Runtime 产出 Command
    pub fn on_command_produced(&mut self, variant: &str, summary: &str) {
        self.emit(EngineEvent::CommandProduced {
            variant: variant.to_string(),
            summary: serde_json::json!(summary),
        });
    }

    /// CommandExecutor 执行 Command 完成
    pub fn on_command_executed(&mut self, variant: &str, result: &str) {
        self.emit(EngineEvent::CommandExecuted {
            variant: variant.to_string(),
            result: result.to_string(),
        });
    }

    /// 关键状态变更
    pub fn on_state_changed(&mut self, field: &str, from: &str, to: &str) {
        self.emit(EngineEvent::StateChanged {
            field: field.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        });
    }

    /// 用户输入到达脚本模式处理入口
    pub fn on_input_received(&mut self, variant: &str) {
        self.emit(EngineEvent::InputReceived {
            variant: variant.to_string(),
        });
    }

    /// 过渡更新
    pub fn on_transition_update(&mut self, transition_type: &str, phase: &str, progress: f32) {
        self.emit(EngineEvent::TransitionUpdate {
            transition_type: transition_type.to_string(),
            phase: phase.to_string(),
            progress,
        });
    }

    /// 音频动作
    pub fn on_audio_event(&mut self, action: &str, path: Option<&str>, volume: Option<f32>) {
        self.emit(EngineEvent::AudioEvent {
            action: action.to_string(),
            path: path.map(|p| p.to_string()),
            volume,
        });
    }

    /// 发出事件（指定逻辑时间戳，headless 模式用）
    pub fn emit_with_ts(&mut self, ts_ms: u64, event: EngineEvent) {
        #[cfg(test)]
        if let Some(ref mut collector) = self.collector {
            collector.push(ts_ms, event.clone());
        }

        let Some(ref mut writer) = self.writer else {
            return;
        };
        Self::write_event(writer, ts_ms, &event);
    }

    /// 刷新缓冲区
    pub fn flush(&mut self) {
        if let Some(ref mut writer) = self.writer {
            let _ = writer.flush();
        }
    }

    fn write_event(writer: &mut BufWriter<std::fs::File>, ts_ms: u64, event: &EngineEvent) {
        #[derive(Serialize)]
        struct Line<'a> {
            ts_ms: u64,
            #[serde(flatten)]
            event: &'a EngineEvent,
        }
        let line = Line { ts_ms, event };
        if let Ok(json) = serde_json::to_string(&line) {
            let _ = writeln!(writer, "{json}");
        }
    }
}

// ─── 辅助函数 ───────────────────────────────────────────────────────

/// 从 Command 提取变体名
pub fn command_variant_name(cmd: &vn_runtime::Command) -> String {
    format!("{cmd:?}")
        .split_once(['(', '{', ' '])
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| format!("{cmd:?}"))
}

/// 从 Command 生成概要信息
pub fn command_summary(cmd: &vn_runtime::Command) -> serde_json::Value {
    serde_json::json!(format!("{cmd:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_event_serialization() {
        let event = EngineEvent::ScriptTick {
            node_index: 5,
            commands_count: 3,
            waiting_reason: "WaitForClick".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("ScriptTick"));
        assert!(json.contains("\"node_index\":5"));
    }

    #[test]
    fn event_stream_disabled_is_noop() {
        let mut es = EventStream::disabled();
        assert!(!es.is_enabled());
        es.emit(EngineEvent::ScriptTick {
            node_index: 0,
            commands_count: 0,
            waiting_reason: "None".into(),
        });
    }

    #[test]
    fn event_stream_writes_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_events.jsonl");

        {
            let mut es = EventStream::new(&path).unwrap();
            es.emit(EngineEvent::ScriptTick {
                node_index: 1,
                commands_count: 2,
                waiting_reason: "WaitForClick".into(),
            });
            es.emit(EngineEvent::AudioEvent {
                action: "play_bgm".into(),
                path: Some("bgm/main.mp3".into()),
                volume: Some(0.8),
            });
            es.flush();
        }

        let content = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("ScriptTick"));
        assert!(lines[1].contains("AudioEvent"));
    }

    #[test]
    fn command_variant_name_extracts_name() {
        use vn_runtime::Command;
        let cmd = Command::ShowBackground {
            path: "bg/test.png".into(),
            transition: None,
        };
        let name = command_variant_name(&cmd);
        assert!(name.starts_with("ShowBackground"));
    }

    #[test]
    fn in_memory_collects_events() {
        let mut es = EventStream::in_memory();
        assert!(!es.is_enabled());

        es.on_script_tick(0, 1, "WaitForClick");
        es.on_command_executed("ShowDialogue", "ok");

        let collector = es.collector().unwrap();
        assert_eq!(collector.len(), 2);

        let events = &collector.events();
        assert_eq!(
            events[0].1,
            EngineEvent::ScriptTick {
                node_index: 0,
                commands_count: 1,
                waiting_reason: "WaitForClick".into(),
            }
        );
        assert_eq!(
            events[1].1,
            EngineEvent::CommandExecuted {
                variant: "ShowDialogue".into(),
                result: "ok".into(),
            }
        );
    }

    #[test]
    fn in_memory_drain_clears() {
        let mut es = EventStream::in_memory();
        es.on_input_received("Click");
        es.on_input_received("Click");

        let drained = es.collector_mut().unwrap().drain();
        assert_eq!(drained.len(), 2);
        assert_eq!(es.collector().unwrap().len(), 0);
    }
}
