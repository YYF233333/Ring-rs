use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use vn_runtime::state::VarValue;

use crate::audio::AudioManager;
use crate::config::AppConfig;
use crate::error::{HostError, HostResult};
use crate::render_state::RenderState;
use crate::resources::ResourceManager;
use crate::save_manager::SaveManager;

/// 用户可调设置（前端 ↔ 后端同步）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub bgm_volume: f32,
    pub sfx_volume: f32,
    pub text_speed: f32,
    pub auto_delay: f32,
    pub fullscreen: bool,
    #[serde(default)]
    pub muted: bool,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            bgm_volume: 80.0,
            sfx_volume: 100.0,
            text_speed: 40.0,
            auto_delay: 2.0,
            fullscreen: false,
            muted: false,
        }
    }
}

/// 对话历史条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub speaker: Option<String>,
    pub text: String,
}

/// Dioxus Desktop 托管的全局应用状态
#[derive(Clone)]
pub struct AppState {
    pub inner: std::sync::Arc<std::sync::Mutex<super::AppStateInner>>,
}

/// 当前会话的前端 owner。
#[derive(Debug, Clone)]
pub struct SessionOwner {
    pub token: String,
    pub label: String,
}

/// 前端连接后返回的会话信息。
#[derive(Debug, Clone, Serialize)]
pub struct FrontendSession {
    pub client_token: String,
    pub render_state: RenderState,
}

// ── 持久化存储 ──────────────────────────────────────────────────────────────

pub const PERSISTENT_FILE: &str = "persistent.json";

/// 持久化变量存储（跨会话保留的 `$persistent.key` 变量）
pub struct PersistentStore {
    pub saves_dir: PathBuf,
    pub variables: HashMap<String, VarValue>,
}

impl PersistentStore {
    /// 创建空 store
    pub fn empty() -> Self {
        Self {
            saves_dir: PathBuf::new(),
            variables: HashMap::new(),
        }
    }

    /// 从存档目录加载；文件不存在或解析失败时返回空 store
    pub fn load(saves_dir: impl AsRef<Path>) -> Self {
        let saves_dir = saves_dir.as_ref().to_path_buf();
        let path = saves_dir.join(PERSISTENT_FILE);

        let variables = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_else(|| {
                    warn!(path = %path.display(), "持久化变量加载失败，使用空 store");
                    HashMap::new()
                })
        } else {
            HashMap::new()
        };

        Self {
            saves_dir,
            variables,
        }
    }

    /// 写入磁盘
    pub fn save(&self) -> HostResult<()> {
        if !self.saves_dir.exists() {
            fs::create_dir_all(&self.saves_dir)?;
        }
        let path = self.saves_dir.join(PERSISTENT_FILE);
        let content = serde_json::to_string_pretty(&self.variables)
            .map_err(|e| HostError::Internal(format!("持久化变量序列化失败: {e}")))?;
        fs::write(&path, content)?;
        info!(path = %path.display(), count = self.variables.len(), "持久化变量保存成功");
        Ok(())
    }

    /// 将 runtime persistent_variables 合并入 store（runtime 值覆盖）
    pub fn merge_from(&mut self, vars: &HashMap<String, VarValue>) {
        for (k, v) in vars {
            self.variables.insert(k.clone(), v.clone());
        }
    }
}

// ── 快照栈 ──────────────────────────────────────────────────────────────────

/// 状态快照（用于 Backspace 回退）
pub struct Snapshot {
    pub render_state: RenderState,
    pub runtime_state: vn_runtime::state::RuntimeState,
    pub runtime_history: vn_runtime::History,
    pub current_bgm: Option<String>,
}

/// 快照栈
pub struct SnapshotStack {
    snapshots: Vec<Snapshot>,
    max_size: usize,
}

impl SnapshotStack {
    pub fn new(max_size: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_size,
        }
    }

    pub fn push(&mut self, snapshot: Snapshot) {
        if self.snapshots.len() >= self.max_size {
            self.snapshots.remove(0);
        }
        self.snapshots.push(snapshot);
    }

    pub fn pop(&mut self) -> Option<Snapshot> {
        self.snapshots.pop()
    }

    pub fn last(&self) -> Option<&Snapshot> {
        self.snapshots.last()
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
    }
}

// ── 应用状态 ────────────────────────────────────────────────────────────────

/// setup() 中一次性初始化的子系统集合。
/// 初始化后不可能为 None——通过 `services()` 访问器断言此不变量。
pub struct Services {
    pub audio: AudioManager,
    pub resources: ResourceManager,
    pub saves: SaveManager,
    pub config: AppConfig,
    pub manifest: crate::manifest::Manifest,
    /// UI 布局配置（从 layout.json 加载）
    pub layout: crate::layout_config::UiLayoutConfig,
    /// 界面行为定义（从 screens.json 加载）
    pub screen_defs: crate::screen_defs::ScreenDefinitions,
}

/// Shake 动画的运行时状态
pub struct ShakeAnimation {
    pub amplitude_x: f32,
    pub amplitude_y: f32,
    pub duration: f32,
    pub elapsed: f32,
}

/// Host 侧 Signal 等待的具体种类。
///
/// 与 `vn_runtime::command::SIGNAL_*` 常量一一对应，
/// 编译期保证穷举，消除字符串拼写错误风险。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalKind {
    SceneTransition,
    TitleCard,
    SceneEffect,
    Cutscene,
}

/// Host 侧的等待状态
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum WaitingFor {
    Nothing,
    Click,
    Choice,
    Time {
        remaining_ms: u64,
    },
    Cutscene,
    Signal(SignalKind),
    /// 等待 UI 交互结果（`requestUI` / `callGame` / `showMap`），对应 Runtime 的 WaitForUIResult
    UIResult {
        key: String,
    },
}
