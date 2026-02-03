//! # 界面模块
//!
//! 各个 UI 界面的实现。

pub mod history;
pub mod ingame_menu;
pub mod save_load;
pub mod settings;
pub mod title;

pub use history::HistoryScreen;
pub use ingame_menu::InGameMenuScreen;
pub use save_load::SaveLoadScreen;
pub use settings::SettingsScreen;
pub use title::TitleScreen;
