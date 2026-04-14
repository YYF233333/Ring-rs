//! # Save 模块
//!
//! 存档/读档系统的数据模型。
//!
//! ## 设计原则
//!
//! - 所有存档数据必须可序列化（JSON）
//! - 必须有版本号，支持向后兼容检测
//! - 存档应包含足够信息恢复游戏状态

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::history::History;
use crate::state::RuntimeState;

/// 存档格式版本
///
/// 版本号含义：
/// - MAJOR: 不兼容的格式变更
/// - MINOR: 向后兼容的新字段
pub const SAVE_VERSION_MAJOR: u32 = 1;
pub const SAVE_VERSION_MINOR: u32 = 0;

/// 存档版本信息
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveVersion {
    pub major: u32,
    pub minor: u32,
}

impl std::fmt::Display for SaveVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl SaveVersion {
    /// 当前版本
    pub fn current() -> Self {
        Self {
            major: SAVE_VERSION_MAJOR,
            minor: SAVE_VERSION_MINOR,
        }
    }

    /// 检查是否兼容
    ///
    /// 兼容规则：
    /// - major 必须相同
    /// - minor 可以不同（向后兼容）
    pub fn is_compatible(&self) -> bool {
        self.major == SAVE_VERSION_MAJOR
    }
}

/// 存档元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// 存档槽位号（1-based）
    pub slot: u32,
    /// 保存时间（ISO 8601 格式）
    pub timestamp: String,
    /// 章节标题（用于 UI 显示）
    pub chapter_title: Option<String>,
    /// 游戏时长（秒）
    pub play_time_secs: u64,
}

impl SaveMetadata {
    /// 创建新的元数据
    ///
    /// `now_secs` 为 Unix 秒时间戳，由 Host 提供。
    pub fn new(slot: u32, now_secs: u64) -> Self {
        Self {
            slot,
            timestamp: format!("{now_secs}"),
            chapter_title: None,
            play_time_secs: 0,
        }
    }

    /// 设置章节标题
    pub fn with_chapter(mut self, title: impl Into<String>) -> Self {
        self.chapter_title = Some(title.into());
        self
    }

    /// 设置游戏时长
    pub fn with_play_time(mut self, secs: u64) -> Self {
        self.play_time_secs = secs;
        self
    }
}

/// 音频状态（用于恢复）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AudioState {
    /// 当前 BGM 路径（None 表示无 BGM）
    pub current_bgm: Option<String>,
    /// BGM 是否循环
    pub bgm_looping: bool,
}

/// 渲染状态快照（用于恢复）
///
/// 只保存必要的恢复信息，不保存临时状态（如过渡动画进度）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderSnapshot {
    /// 当前背景路径
    pub background: Option<String>,
    /// 可见角色列表 (alias -> (path, position_name))
    pub characters: Vec<CharacterSnapshot>,
}

/// 角色快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSnapshot {
    pub alias: String,
    pub texture_path: String,
    pub position: String,
}

/// 存档数据
///
/// 包含恢复游戏状态所需的所有信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// 存档格式版本
    pub version: SaveVersion,
    /// 存档元数据
    pub metadata: SaveMetadata,
    /// Runtime 状态
    pub runtime_state: RuntimeState,
    /// 音频状态
    pub audio: AudioState,
    /// 渲染快照
    pub render: RenderSnapshot,
    /// 历史记录
    pub history: History,
    /// 模态扩展数据（各 mode 可存入自己的状态）
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mode_data: BTreeMap<String, serde_json::Value>,
}

impl SaveData {
    /// 创建新的存档数据
    ///
    /// `now_secs` 为 Unix 秒时间戳，由 Host 提供。
    pub fn new(slot: u32, runtime_state: RuntimeState, now_secs: u64) -> Self {
        Self {
            version: SaveVersion::current(),
            metadata: SaveMetadata::new(slot, now_secs),
            runtime_state,
            audio: AudioState::default(),
            render: RenderSnapshot::default(),
            history: History::new(),
            mode_data: BTreeMap::new(),
        }
    }

    /// 设置音频状态
    pub fn with_audio(mut self, audio: AudioState) -> Self {
        self.audio = audio;
        self
    }

    /// 设置渲染快照
    pub fn with_render(mut self, render: RenderSnapshot) -> Self {
        self.render = render;
        self
    }

    /// 设置章节标题
    pub fn with_chapter(mut self, title: impl Into<String>) -> Self {
        self.metadata.chapter_title = Some(title.into());
        self
    }

    /// 设置历史记录
    pub fn with_history(mut self, history: History) -> Self {
        self.history = history;
        self
    }

    /// 设置模态扩展数据
    pub fn with_mode_data(mut self, mode_data: BTreeMap<String, serde_json::Value>) -> Self {
        self.mode_data = mode_data;
        self
    }

    /// 序列化为 JSON 字符串
    pub fn to_json(&self) -> Result<String, SaveError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| SaveError::SerializationFailed(e.to_string()))
    }

    /// 从 JSON 字符串反序列化
    pub fn from_json(json: &str) -> Result<Self, SaveError> {
        let data: SaveData = serde_json::from_str(json)
            .map_err(|e| SaveError::DeserializationFailed(e.to_string()))?;

        // 检查版本兼容性
        if !data.version.is_compatible() {
            return Err(SaveError::IncompatibleVersion {
                save_version: data.version.to_string(),
                current_version: SaveVersion::current().to_string(),
            });
        }

        Ok(data)
    }
}

/// 存档错误
#[derive(Debug, Clone, PartialEq)]
pub enum SaveError {
    /// 序列化失败
    SerializationFailed(String),
    /// 反序列化失败
    DeserializationFailed(String),
    /// 版本不兼容
    IncompatibleVersion {
        save_version: String,
        current_version: String,
    },
    /// 文件操作失败
    IoError(String),
    /// 存档不存在
    NotFound(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::SerializationFailed(e) => write!(f, "序列化失败: {}", e),
            SaveError::DeserializationFailed(e) => write!(f, "反序列化失败: {}", e),
            SaveError::IncompatibleVersion {
                save_version,
                current_version,
            } => {
                write!(
                    f,
                    "存档版本不兼容: 存档版本 {} vs 当前版本 {}",
                    save_version, current_version
                )
            }
            SaveError::IoError(e) => write!(f, "文件操作失败: {}", e),
            SaveError::NotFound(path) => write!(f, "存档不存在: {}", path),
        }
    }
}

impl std::error::Error for SaveError {}

#[cfg(test)]
mod tests;
