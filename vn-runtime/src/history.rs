//! # History 模块
//!
//! 历史记录数据模型，用于支持历史回看功能。
//!
//! ## 设计原则
//!
//! - 记录游戏中的关键事件（对话、选择、章节标记等）
//! - 所有数据可序列化，与存档系统对齐
//! - 不记录临时状态（如过渡动画）

use serde::{Deserialize, Serialize};

/// 历史事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryEvent {
    /// 对话事件
    Dialogue {
        /// 说话者（None 表示旁白）
        speaker: Option<String>,
        /// 对话内容
        content: String,
        /// 时间戳（Unix 秒）
        timestamp: u64,
    },

    /// 章节标记
    ChapterMark {
        /// 章节标题
        title: String,
        /// 时间戳
        timestamp: u64,
    },

    /// 选择事件
    ChoiceMade {
        /// 选项列表（所有可选项）
        options: Vec<String>,
        /// 选择的索引
        selected_index: usize,
        /// 时间戳
        timestamp: u64,
    },

    /// 跳转事件
    Jump {
        /// 跳转目标标签
        label: String,
        /// 时间戳
        timestamp: u64,
    },

    /// 背景切换（可选记录，用于回看时恢复视觉上下文）
    BackgroundChange {
        /// 新背景路径
        path: String,
        /// 时间戳
        timestamp: u64,
    },

    /// BGM 切换（可选记录）
    BgmChange {
        /// 新 BGM 路径（None 表示停止）
        path: Option<String>,
        /// 时间戳
        timestamp: u64,
    },
}

impl HistoryEvent {
    /// 获取事件时间戳
    pub fn timestamp(&self) -> u64 {
        match self {
            HistoryEvent::Dialogue { timestamp, .. } => *timestamp,
            HistoryEvent::ChapterMark { timestamp, .. } => *timestamp,
            HistoryEvent::ChoiceMade { timestamp, .. } => *timestamp,
            HistoryEvent::Jump { timestamp, .. } => *timestamp,
            HistoryEvent::BackgroundChange { timestamp, .. } => *timestamp,
            HistoryEvent::BgmChange { timestamp, .. } => *timestamp,
        }
    }

    /// 创建对话事件
    ///
    /// `now` 为 Unix 秒时间戳，由 Host 提供。
    pub fn dialogue(speaker: Option<String>, content: String, now: u64) -> Self {
        HistoryEvent::Dialogue {
            speaker,
            content,
            timestamp: now,
        }
    }

    /// 创建章节标记事件
    pub fn chapter_mark(title: String, now: u64) -> Self {
        HistoryEvent::ChapterMark {
            title,
            timestamp: now,
        }
    }

    /// 创建选择事件
    pub fn choice_made(options: Vec<String>, selected_index: usize, now: u64) -> Self {
        HistoryEvent::ChoiceMade {
            options,
            selected_index,
            timestamp: now,
        }
    }

    /// 创建跳转事件
    pub fn jump(label: String, now: u64) -> Self {
        HistoryEvent::Jump {
            label,
            timestamp: now,
        }
    }

    /// 创建背景切换事件
    pub fn background_change(path: String, now: u64) -> Self {
        HistoryEvent::BackgroundChange {
            path,
            timestamp: now,
        }
    }

    /// 创建 BGM 切换事件
    pub fn bgm_change(path: Option<String>, now: u64) -> Self {
        HistoryEvent::BgmChange {
            path,
            timestamp: now,
        }
    }
}

/// 历史记录容器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct History {
    /// 事件列表（按时间顺序）
    events: Vec<HistoryEvent>,
    /// 最大记录数（防止内存无限增长）
    max_events: usize,
}

impl History {
    /// 创建新的历史记录
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            max_events: 1000, // 默认最多记录 1000 条
        }
    }

    /// 设置最大记录数
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    /// 添加事件
    pub fn push(&mut self, event: HistoryEvent) {
        self.events.push(event);

        // 如果超过最大数量，移除最早的事件
        while self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    /// 追加文本到最近一条 Dialogue 事件（用于 extend 命令）
    ///
    /// 如果最近一条事件不是 Dialogue，则创建新的旁白 Dialogue。
    /// `now` 仅在需要创建新事件时使用。
    pub fn append_to_last_dialogue(&mut self, text: &str, now: u64) {
        if let Some(HistoryEvent::Dialogue { content, .. }) = self.events.last_mut() {
            content.push_str(text);
        } else {
            self.push(HistoryEvent::dialogue(None, text.to_string(), now));
        }
    }

    /// 获取所有事件
    pub fn events(&self) -> &[HistoryEvent] {
        &self.events
    }

    /// 获取对话事件数量
    pub fn dialogue_count(&self) -> usize {
        self.events
            .iter()
            .filter(|e| matches!(e, HistoryEvent::Dialogue { .. }))
            .count()
    }

    /// 获取最近的 N 条对话
    pub fn recent_dialogues(&self, count: usize) -> Vec<&HistoryEvent> {
        self.events
            .iter()
            .filter(|e| matches!(e, HistoryEvent::Dialogue { .. }))
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// 清空历史
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// 获取事件总数
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_basic() {
        let mut history = History::new();
        assert!(history.is_empty());

        history.push(HistoryEvent::dialogue(
            Some("角色A".to_string()),
            "你好".to_string(),
            0,
        ));
        history.push(HistoryEvent::dialogue(None, "旁白文本".to_string(), 0));
        history.push(HistoryEvent::chapter_mark("第一章".to_string(), 0));

        assert_eq!(history.len(), 3);
        assert_eq!(history.dialogue_count(), 2);
    }

    #[test]
    fn test_history_max_events() {
        let mut history = History::new().with_max_events(5);

        for i in 0..10 {
            history.push(HistoryEvent::dialogue(None, format!("对话 {}", i), 0));
        }

        assert_eq!(history.len(), 5);
        // 应该保留最后 5 条
        fn dialogue_content(event: &HistoryEvent) -> Option<&str> {
            match event {
                HistoryEvent::Dialogue { content, .. } => Some(content),
                _ => None,
            }
        }

        assert_eq!(dialogue_content(&history.events()[0]), Some("对话 5"));
        // 覆盖非对话分支
        assert_eq!(
            dialogue_content(&HistoryEvent::chapter_mark("章节".to_string(), 0)),
            None
        );
    }

    #[test]
    fn test_recent_dialogues() {
        let mut history = History::new();

        history.push(HistoryEvent::dialogue(None, "对话1".to_string(), 0));
        history.push(HistoryEvent::chapter_mark("章节".to_string(), 0));
        history.push(HistoryEvent::dialogue(None, "对话2".to_string(), 0));
        history.push(HistoryEvent::dialogue(None, "对话3".to_string(), 0));

        let recent = history.recent_dialogues(2);
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_history_serialization() {
        let mut history = History::new();
        history.push(HistoryEvent::dialogue(
            Some("A".to_string()),
            "内容".to_string(),
            0,
        ));
        history.push(HistoryEvent::choice_made(
            vec!["选项1".to_string(), "选项2".to_string()],
            0,
            0,
        ));

        let json = serde_json::to_string(&history).unwrap();
        let loaded: History = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_history_event_timestamp_covers_all_variants() {
        // 目标：覆盖 HistoryEvent::timestamp() 的所有 match 分支
        let _ = HistoryEvent::dialogue(Some("A".to_string()), "内容".to_string(), 100).timestamp();
        let _ = HistoryEvent::chapter_mark("第一章".to_string(), 101).timestamp();
        let _ =
            HistoryEvent::choice_made(vec!["A".to_string(), "B".to_string()], 1, 102).timestamp();
        let _ = HistoryEvent::jump("label".to_string(), 103).timestamp();
        let _ = HistoryEvent::background_change("bg.png".to_string(), 104).timestamp();
        let _ = HistoryEvent::bgm_change(Some("bgm.mp3".to_string()), 105).timestamp();
    }

    #[test]
    fn test_history_event_timestamp_returns_stored_value() {
        let cases = [
            HistoryEvent::Dialogue {
                speaker: Some("A".to_string()),
                content: "内容".to_string(),
                timestamp: 11,
            },
            HistoryEvent::ChapterMark {
                title: "第一章".to_string(),
                timestamp: 12,
            },
            HistoryEvent::ChoiceMade {
                options: vec!["A".to_string(), "B".to_string()],
                selected_index: 1,
                timestamp: 13,
            },
            HistoryEvent::Jump {
                label: "end".to_string(),
                timestamp: 14,
            },
            HistoryEvent::BackgroundChange {
                path: "bg.png".to_string(),
                timestamp: 15,
            },
            HistoryEvent::BgmChange {
                path: Some("bgm.mp3".to_string()),
                timestamp: 16,
            },
        ];

        let expected = [11, 12, 13, 14, 15, 16];
        for (event, timestamp) in cases.iter().zip(expected) {
            assert_eq!(event.timestamp(), timestamp);
        }
    }

    #[test]
    fn test_constructors_pass_through_timestamp() {
        assert_eq!(
            HistoryEvent::dialogue(None, "内容".to_string(), 42).timestamp(),
            42
        );
        assert_eq!(
            HistoryEvent::chapter_mark("第一章".to_string(), 43).timestamp(),
            43
        );
    }

    #[test]
    fn test_history_clear() {
        let mut history = History::new();
        history.push(HistoryEvent::dialogue(None, "对话1".to_string(), 0));
        assert!(!history.is_empty());

        history.clear();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }
}
