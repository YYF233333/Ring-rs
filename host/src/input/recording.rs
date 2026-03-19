//! 输入录制与回放子系统
//!
//! 提供输入事件的序列化模型（`InputEvent`）、环形录制缓冲区（`RecordingBuffer`）、
//! JSON Lines 格式导出（`RecordingExporter`）和回放读取（`InputReplayer`）。

use std::collections::VecDeque;
use std::io::{self, BufRead, BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

// ─── 键名序列化类型 ─────────────────────────────────────────────────

/// AI 可读的按键名（与 `winit::keyboard::KeyCode` 互转）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct KeyName(pub String);

impl KeyName {
    pub fn from_key_code(code: KeyCode) -> Self {
        Self(format!("{code:?}"))
    }

    pub fn to_key_code(&self) -> Option<KeyCode> {
        key_code_from_name(&self.0)
    }
}

/// 鼠标按键名
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButtonName {
    Left,
    Right,
    Middle,
}

// ─── InputEvent ─────────────────────────────────────────────────────

/// 语义中间层输入事件（录制/回放单元）
///
/// 不是 winit 原始事件，也不是 `RuntimeInput`。
/// 用于人类录制 → headless 回放 → AI 分析的完整管线。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InputEvent {
    KeyPress {
        key: KeyName,
    },
    KeyRelease {
        key: KeyName,
    },
    MouseMove {
        x: f32,
        y: f32,
    },
    MousePress {
        button: MouseButtonName,
        x: f32,
        y: f32,
    },
    MouseRelease {
        button: MouseButtonName,
        x: f32,
        y: f32,
    },
    MouseWheel {
        delta_x: f32,
        delta_y: f32,
    },
}

// ─── 录制文件格式 ───────────────────────────────────────────────────

/// 录制文件元数据（JSON Lines 首行）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMeta {
    pub version: u32,
    pub logical_width: u32,
    pub logical_height: u32,
    pub engine_version: String,
    pub recorded_at: String,
    pub duration_ms: u64,
    pub entry_script: String,
}

/// 录制文件行条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecordingEntry {
    Meta(RecordingMeta),
    Event { t_ms: u64, event: InputEvent },
}

// ─── RecordingBuffer ────────────────────────────────────────────────

/// 环形录制缓冲区（后台无感录制）
#[derive(Debug)]
pub struct RecordingBuffer {
    buffer: VecDeque<(u64, InputEvent)>,
    capacity: usize,
}

impl RecordingBuffer {
    /// 根据缓冲区大小（MB）创建录制缓冲区
    pub fn new(size_mb: u32) -> Self {
        let elem_size = std::mem::size_of::<(u64, InputEvent)>();
        let capacity = ((size_mb as usize) * 1024 * 1024)
            .checked_div(elem_size)
            .unwrap_or(0);
        Self {
            buffer: VecDeque::with_capacity(capacity.min(1024 * 1024)),
            capacity,
        }
    }

    /// 压入事件，超过容量时从头部弹出
    pub fn push(&mut self, t_ms: u64, event: InputEvent) {
        if self.capacity == 0 {
            return;
        }
        if self.buffer.len() >= self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back((t_ms, event));
    }

    /// 返回当前缓冲区引用（导出用）
    pub fn snapshot(&self) -> &VecDeque<(u64, InputEvent)> {
        &self.buffer
    }

    /// 缓冲区是否非空
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// 缓冲区当前条目数
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

// ─── RecordingExporter ──────────────────────────────────────────────

/// 录制文件导出错误
#[derive(Debug)]
pub enum RecordExportError {
    Io(io::Error),
    Serialize(serde_json::Error),
}

impl std::fmt::Display for RecordExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Serialize(e) => write!(f, "Serialize error: {e}"),
        }
    }
}

impl std::error::Error for RecordExportError {}

/// 录制导出器
pub struct RecordingExporter;

impl RecordingExporter {
    /// 将录制缓冲区导出为 JSON Lines 文件
    pub fn export(
        meta: &RecordingMeta,
        events: &VecDeque<(u64, InputEvent)>,
        path: &Path,
    ) -> Result<(), RecordExportError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(RecordExportError::Io)?;
        }

        let file = std::fs::File::create(path).map_err(RecordExportError::Io)?;
        let mut writer = BufWriter::new(file);

        let meta_line = serde_json::to_string(meta).map_err(RecordExportError::Serialize)?;
        writeln!(writer, "{meta_line}").map_err(RecordExportError::Io)?;

        for (t_ms, event) in events {
            let entry = serde_json::json!({ "t_ms": t_ms, "event": event });
            let line = serde_json::to_string(&entry).map_err(RecordExportError::Serialize)?;
            writeln!(writer, "{line}").map_err(RecordExportError::Io)?;
        }

        writer.flush().map_err(RecordExportError::Io)?;
        Ok(())
    }
}

// ─── InputReplayer ──────────────────────────────────────────────────

/// 回放加载错误
#[derive(Debug)]
pub enum ReplayLoadError {
    Io(io::Error),
    Parse(String),
    UnsupportedVersion(u32),
    Empty,
}

impl std::fmt::Display for ReplayLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::UnsupportedVersion(v) => write!(f, "Unsupported recording version: {v}"),
            Self::Empty => write!(f, "Recording file is empty"),
        }
    }
}

impl std::error::Error for ReplayLoadError {}

/// 输入回放器（读取 JSON Lines 录制文件，按时间调度事件）
pub struct InputReplayer {
    meta: RecordingMeta,
    events: Vec<(u64, InputEvent)>,
    cursor: usize,
}

impl InputReplayer {
    /// 从文件加载录制数据
    pub fn load(path: &Path) -> Result<Self, ReplayLoadError> {
        let file = std::fs::File::open(path).map_err(ReplayLoadError::Io)?;
        let reader = io::BufReader::new(file);
        let mut lines = reader.lines();

        let first_line = lines
            .next()
            .ok_or(ReplayLoadError::Empty)?
            .map_err(ReplayLoadError::Io)?;

        let meta: RecordingMeta = serde_json::from_str(&first_line)
            .map_err(|e| ReplayLoadError::Parse(format!("meta line: {e}")))?;

        if meta.version != 1 {
            return Err(ReplayLoadError::UnsupportedVersion(meta.version));
        }

        let mut events = Vec::new();
        for (line_num, line_result) in lines.enumerate() {
            let line = line_result.map_err(ReplayLoadError::Io)?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: serde_json::Value = serde_json::from_str(&line)
                .map_err(|e| ReplayLoadError::Parse(format!("line {}: {e}", line_num + 2)))?;

            let t_ms = entry["t_ms"].as_u64().ok_or_else(|| {
                ReplayLoadError::Parse(format!("line {}: missing t_ms", line_num + 2))
            })?;
            let event: InputEvent = serde_json::from_value(entry["event"].clone())
                .map_err(|e| ReplayLoadError::Parse(format!("line {}: {e}", line_num + 2)))?;

            events.push((t_ms, event));
        }

        Ok(Self {
            meta,
            events,
            cursor: 0,
        })
    }

    /// 返回所有 `t_ms <= elapsed_ms` 的事件并推进游标
    pub fn drain_until(&mut self, elapsed_ms: u64) -> Vec<InputEvent> {
        let mut result = Vec::new();
        while self.cursor < self.events.len() && self.events[self.cursor].0 <= elapsed_ms {
            result.push(self.events[self.cursor].1.clone());
            self.cursor += 1;
        }
        result
    }

    /// 所有事件是否已消费完
    pub fn is_exhausted(&self) -> bool {
        self.cursor >= self.events.len()
    }

    /// 访问元数据
    pub fn meta(&self) -> &RecordingMeta {
        &self.meta
    }
}

// ─── WindowEvent → InputEvent 转换 ─────────────────────────────────

/// 将 winit WindowEvent 转换为 InputEvent（用于录制）
pub fn convert_window_event(event: &WindowEvent, mouse_pos: (f32, f32)) -> Option<InputEvent> {
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            if let PhysicalKey::Code(key) = event.physical_key {
                match event.state {
                    ElementState::Pressed if !event.repeat => Some(InputEvent::KeyPress {
                        key: KeyName::from_key_code(key),
                    }),
                    ElementState::Released => Some(InputEvent::KeyRelease {
                        key: KeyName::from_key_code(key),
                    }),
                    _ => None,
                }
            } else {
                None
            }
        }
        WindowEvent::MouseInput {
            state,
            button: MouseButton::Left,
            ..
        } => match state {
            ElementState::Pressed => Some(InputEvent::MousePress {
                button: MouseButtonName::Left,
                x: mouse_pos.0,
                y: mouse_pos.1,
            }),
            ElementState::Released => Some(InputEvent::MouseRelease {
                button: MouseButtonName::Left,
                x: mouse_pos.0,
                y: mouse_pos.1,
            }),
        },
        WindowEvent::CursorMoved { position, .. } => Some(InputEvent::MouseMove {
            x: position.x as f32,
            y: position.y as f32,
        }),
        _ => None,
    }
}

// ─── KeyCode 反序列化辅助 ───────────────────────────────────────────

fn key_code_from_name(name: &str) -> Option<KeyCode> {
    match name {
        "Space" => Some(KeyCode::Space),
        "Enter" => Some(KeyCode::Enter),
        "Escape" => Some(KeyCode::Escape),
        "ArrowUp" => Some(KeyCode::ArrowUp),
        "ArrowDown" => Some(KeyCode::ArrowDown),
        "ArrowLeft" => Some(KeyCode::ArrowLeft),
        "ArrowRight" => Some(KeyCode::ArrowRight),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "KeyA" => Some(KeyCode::KeyA),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyS" => Some(KeyCode::KeyS),
        "F8" => Some(KeyCode::F8),
        _ => None,
    }
}

// ─── 测试 ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_buffer_push_and_snapshot() {
        let mut buf = RecordingBuffer::new(1);
        buf.push(
            0,
            InputEvent::KeyPress {
                key: KeyName("Space".into()),
            },
        );
        buf.push(
            100,
            InputEvent::KeyRelease {
                key: KeyName("Space".into()),
            },
        );
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.snapshot().len(), 2);
    }

    #[test]
    fn recording_buffer_overflow_evicts_oldest() {
        let elem_size = std::mem::size_of::<(u64, InputEvent)>();
        let mb_for_2 = ((elem_size * 2) as f64 / (1024.0 * 1024.0)).ceil() as u32;
        let mut buf = RecordingBuffer::new(mb_for_2);

        let cap = buf.capacity;
        for i in 0..cap + 5 {
            buf.push(
                i as u64,
                InputEvent::KeyPress {
                    key: KeyName("Space".into()),
                },
            );
        }
        assert_eq!(buf.len(), cap);
        assert_eq!(buf.snapshot().front().unwrap().0, 5);
    }

    #[test]
    fn recording_buffer_zero_size_is_noop() {
        let mut buf = RecordingBuffer::new(0);
        buf.push(
            0,
            InputEvent::KeyPress {
                key: KeyName("Space".into()),
            },
        );
        assert!(buf.is_empty());
    }

    #[test]
    fn input_event_roundtrip_serialization() {
        let events = vec![
            InputEvent::KeyPress {
                key: KeyName("Space".into()),
            },
            InputEvent::MousePress {
                button: MouseButtonName::Left,
                x: 100.0,
                y: 200.0,
            },
            InputEvent::MouseMove { x: 150.0, y: 250.0 },
        ];
        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let back: InputEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(*event, back);
        }
    }

    #[test]
    fn recording_entry_meta_serialization() {
        let meta = RecordingMeta {
            version: 1,
            logical_width: 1280,
            logical_height: 720,
            engine_version: "0.1.0".into(),
            recorded_at: "2026-01-01T00:00:00Z".into(),
            duration_ms: 5000,
            entry_script: "scripts/main.md".into(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        let back: RecordingMeta = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version, 1);
        assert_eq!(back.logical_width, 1280);
    }

    #[test]
    fn exporter_and_replayer_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.jsonl");

        let meta = RecordingMeta {
            version: 1,
            logical_width: 1280,
            logical_height: 720,
            engine_version: "0.1.0".into(),
            recorded_at: "2026-01-01T00:00:00Z".into(),
            duration_ms: 1000,
            entry_script: "scripts/main.md".into(),
        };

        let mut buf = VecDeque::new();
        buf.push_back((
            0,
            InputEvent::KeyPress {
                key: KeyName("Space".into()),
            },
        ));
        buf.push_back((
            500,
            InputEvent::MousePress {
                button: MouseButtonName::Left,
                x: 100.0,
                y: 200.0,
            },
        ));
        buf.push_back((
            1000,
            InputEvent::KeyRelease {
                key: KeyName("Space".into()),
            },
        ));

        RecordingExporter::export(&meta, &buf, &path).unwrap();

        let mut replayer = InputReplayer::load(&path).unwrap();
        assert_eq!(replayer.meta().version, 1);
        assert!(!replayer.is_exhausted());

        let batch1 = replayer.drain_until(100);
        assert_eq!(batch1.len(), 1);

        let batch2 = replayer.drain_until(500);
        assert_eq!(batch2.len(), 1);

        let batch3 = replayer.drain_until(2000);
        assert_eq!(batch3.len(), 1);
        assert!(replayer.is_exhausted());
    }
}
