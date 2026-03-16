//! # 应用模式管理
//!
//! 管理应用的状态机、导航栈和输入捕获。

use serde::{Deserialize, Serialize};
use tracing::warn;

/// 应用模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppMode {
    /// 主标题界面
    #[default]
    Title,
    /// 游戏进行中
    InGame,
    /// 游戏内系统菜单（暂停）
    InGameMenu,
    /// 存档/读档界面
    SaveLoad,
    /// 设置界面
    Settings,
    /// 历史回看界面
    History,
}

/// 存档/读档界面的当前标签页
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SaveLoadTab {
    Save,
    #[default]
    Load,
}

/// 存读档页面的分页标识
///
/// 每页 6 个槽位。槽位 ID 映射：
/// - `Manual(1..=9)` → slot 1-54
/// - `Quick` → slot 55-60
/// - `Auto` → slot 61-66
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveLoadPage {
    /// 手动存档页 (1-9)
    Manual(u8),
    /// 快速存档页
    Quick,
    /// 自动存档页
    Auto,
}

impl Default for SaveLoadPage {
    fn default() -> Self {
        Self::Manual(1)
    }
}

impl SaveLoadPage {
    pub const SLOTS_PER_PAGE: u32 = 6;

    /// 该页对应的起始 slot ID（1-indexed）
    pub fn first_slot(self) -> u32 {
        match self {
            Self::Manual(n) => (n as u32 - 1) * Self::SLOTS_PER_PAGE + 1,
            Self::Quick => 55,
            Self::Auto => 61,
        }
    }

    /// 该页对应的 slot ID 范围（含两端）
    pub fn slot_range(self) -> std::ops::RangeInclusive<u32> {
        let start = self.first_slot();
        start..=(start + Self::SLOTS_PER_PAGE - 1)
    }

    /// 页面显示标签
    pub fn label(self) -> &'static str {
        match self {
            Self::Manual(n) => match n {
                1 => "1",
                2 => "2",
                3 => "3",
                4 => "4",
                5 => "5",
                6 => "6",
                7 => "7",
                8 => "8",
                9 => "9",
                _ => "?",
            },
            Self::Quick => "Q",
            Self::Auto => "A",
        }
    }

    /// 所有分页，按显示顺序排列
    pub fn all_pages() -> &'static [SaveLoadPage] {
        use SaveLoadPage::*;
        &[
            Auto,
            Quick,
            Manual(1),
            Manual(2),
            Manual(3),
            Manual(4),
            Manual(5),
            Manual(6),
            Manual(7),
            Manual(8),
            Manual(9),
        ]
    }

    /// 前一页
    pub fn prev(self) -> Option<Self> {
        let pages = Self::all_pages();
        let idx = pages.iter().position(|p| *p == self)?;
        if idx > 0 { Some(pages[idx - 1]) } else { None }
    }

    /// 后一页
    pub fn next(self) -> Option<Self> {
        let pages = Self::all_pages();
        let idx = pages.iter().position(|p| *p == self)?;
        pages.get(idx + 1).copied()
    }
}

/// 导航栈管理器
///
/// 用于管理界面的返回逻辑，例如：
/// - 从 InGameMenu 打开 SaveLoad，返回时回到 InGameMenu
/// - 从 Title 打开 SaveLoad，返回时回到 Title
#[derive(Debug, Clone)]
pub struct NavigationStack {
    stack: Vec<AppMode>,
    current: AppMode,
}

impl Default for NavigationStack {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationStack {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            current: AppMode::Title,
        }
    }

    /// 获取当前模式
    pub fn current(&self) -> AppMode {
        self.current
    }

    /// 导航到新模式（将当前模式压入栈）
    pub fn navigate_to(&mut self, mode: AppMode) {
        // 不重复压入相同模式
        if self.current != mode {
            self.stack.push(self.current);
            self.current = mode;
        }
    }

    /// 直接切换模式（不压栈，用于如 Title -> InGame 这种不需要返回的切换）
    pub fn switch_to(&mut self, mode: AppMode) {
        self.stack.clear();
        self.current = mode;
    }

    /// 替换当前模式（不压栈也不弹栈），用于同级页面间切换
    pub fn replace_current(&mut self, mode: AppMode) {
        self.current = mode;
    }

    /// 返回上一个模式
    pub fn go_back(&mut self) -> Option<AppMode> {
        if let Some(prev) = self.stack.pop() {
            self.current = prev;
            Some(prev)
        } else {
            None
        }
    }

    /// 返回到标题界面（清空栈）
    pub fn return_to_title(&mut self) {
        self.stack.clear();
        self.current = AppMode::Title;
    }

    /// 检查是否可以返回
    pub fn can_go_back(&self) -> bool {
        !self.stack.is_empty()
    }

    /// 获取栈深度
    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

/// 播放推进模式
///
/// 控制游戏中剧情的推进方式：
/// - `Normal`：等待用户点击/按键推进（默认）
/// - `Auto`：对话完成后等待 `auto_delay` 秒自动推进
/// - `Skip`：立即完成所有演出并推进（Ctrl 按住时激活）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackMode {
    /// 正常模式：等待用户点击推进
    #[default]
    Normal,
    /// 自动模式：对话完成后等待 auto_delay 秒自动推进
    Auto,
    /// 跳过模式：立即完成所有演出并推进（Ctrl 按住时激活）
    Skip,
}

/// 输入捕获状态
///
/// 控制不同模式下的输入行为，避免"双重消费"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputCapture {
    /// 游戏输入（推进剧情、选择等）
    #[default]
    Game,
    /// 菜单输入（导航、选择菜单项）
    Menu,
    /// 无输入（过渡动画中等）
    Blocked,
}

impl AppMode {
    /// 获取该模式对应的默认输入捕获状态
    pub fn default_input_capture(&self) -> InputCapture {
        match self {
            AppMode::Title => InputCapture::Menu,
            AppMode::InGame => InputCapture::Game,
            AppMode::InGameMenu => InputCapture::Menu,
            AppMode::SaveLoad => InputCapture::Menu,
            AppMode::Settings => InputCapture::Menu,
            AppMode::History => InputCapture::Menu,
        }
    }

    /// 是否是游戏进行中（需要显示游戏内容）
    pub fn is_in_game(&self) -> bool {
        matches!(
            self,
            AppMode::InGame | AppMode::InGameMenu | AppMode::History
        )
    }

    /// 是否是覆盖层界面（在游戏画面上方显示）
    pub fn is_overlay(&self) -> bool {
        matches!(self, AppMode::InGameMenu | AppMode::History)
    }

    /// 是否是全屏界面（完全覆盖游戏画面）
    pub fn is_fullscreen_ui(&self) -> bool {
        matches!(self, AppMode::Title | AppMode::SaveLoad | AppMode::Settings)
    }
}

/// 玩家设置（与 config.json 分离，保存玩家偏好）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    /// BGM 音量 (0.0 - 1.0)
    pub bgm_volume: f32,
    /// SFX 音量 (0.0 - 1.0)
    pub sfx_volume: f32,
    /// 是否静音
    pub muted: bool,
    /// 是否全屏
    pub fullscreen: bool,
    /// 文字速度（每秒字符数）
    pub text_speed: f32,
    /// 自动播放延迟（秒）
    pub auto_delay: f32,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            bgm_volume: 0.8,
            sfx_volume: 1.0,
            muted: false,
            fullscreen: false,
            text_speed: 30.0,
            auto_delay: 2.0,
        }
    }
}

impl UserSettings {
    /// 从文件加载设置，如果失败则使用默认值
    pub fn load(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                warn!(error = %e, "解析用户设置失败，使用默认值");
                Self::default()
            }),
            Err(_) => {
                warn!("用户设置文件不存在，使用默认值");
                Self::default()
            }
        }
    }

    /// 保存设置到文件
    pub fn save(&self, path: &str) -> Result<(), String> {
        let content =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(path, content).map_err(|e| format!("写入失败: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
