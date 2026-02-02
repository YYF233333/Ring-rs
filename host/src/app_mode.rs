//! # 应用模式管理
//!
//! 管理应用的状态机、导航栈和输入捕获。

use serde::{Deserialize, Serialize};

/// 应用模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppMode {
    /// 主标题界面
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

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Title
    }
}

/// 存档/读档界面的当前标签页
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveLoadTab {
    Save,
    Load,
}

impl Default for SaveLoadTab {
    fn default() -> Self {
        SaveLoadTab::Load
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

/// 输入捕获状态
/// 
/// 控制不同模式下的输入行为，避免"双重消费"
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputCapture {
    /// 游戏输入（推进剧情、选择等）
    Game,
    /// 菜单输入（导航、选择菜单项）
    Menu,
    /// 无输入（过渡动画中等）
    Blocked,
}

impl Default for InputCapture {
    fn default() -> Self {
        InputCapture::Game
    }
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
        matches!(self, AppMode::InGame | AppMode::InGameMenu | AppMode::History)
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
    /// 是否开启自动播放
    pub auto_mode: bool,
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
            auto_mode: false,
        }
    }
}

impl UserSettings {
    /// 从文件加载设置，如果失败则使用默认值
    pub fn load(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_else(|e| {
                    eprintln!("⚠️ 解析用户设置失败，使用默认值: {}", e);
                    Self::default()
                })
            }
            Err(_) => {
                println!("📝 用户设置文件不存在，使用默认值");
                Self::default()
            }
        }
    }

    /// 保存设置到文件
    pub fn save(&self, path: &str) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化失败: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("写入失败: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_stack_basic() {
        let mut nav = NavigationStack::new();
        assert_eq!(nav.current(), AppMode::Title);
        assert!(!nav.can_go_back());

        nav.navigate_to(AppMode::Settings);
        assert_eq!(nav.current(), AppMode::Settings);
        assert!(nav.can_go_back());

        nav.go_back();
        assert_eq!(nav.current(), AppMode::Title);
        assert!(!nav.can_go_back());
    }

    #[test]
    fn test_navigation_stack_nested() {
        let mut nav = NavigationStack::new();
        
        // Title -> InGame (switch, no stack)
        nav.switch_to(AppMode::InGame);
        assert_eq!(nav.current(), AppMode::InGame);
        assert!(!nav.can_go_back());

        // InGame -> InGameMenu -> SaveLoad
        nav.navigate_to(AppMode::InGameMenu);
        nav.navigate_to(AppMode::SaveLoad);
        assert_eq!(nav.depth(), 2);

        // Back to InGameMenu
        nav.go_back();
        assert_eq!(nav.current(), AppMode::InGameMenu);

        // Back to InGame
        nav.go_back();
        assert_eq!(nav.current(), AppMode::InGame);
    }

    #[test]
    fn test_navigation_return_to_title() {
        let mut nav = NavigationStack::new();
        nav.switch_to(AppMode::InGame);
        nav.navigate_to(AppMode::InGameMenu);
        nav.navigate_to(AppMode::SaveLoad);

        nav.return_to_title();
        assert_eq!(nav.current(), AppMode::Title);
        assert!(!nav.can_go_back());
    }

    #[test]
    fn test_input_capture() {
        assert_eq!(AppMode::Title.default_input_capture(), InputCapture::Menu);
        assert_eq!(AppMode::InGame.default_input_capture(), InputCapture::Game);
        assert_eq!(AppMode::InGameMenu.default_input_capture(), InputCapture::Menu);
    }

    #[test]
    fn test_user_settings_default() {
        let settings = UserSettings::default();
        assert!((settings.bgm_volume - 0.8).abs() < 0.001);
        assert!(!settings.muted);
    }
}
