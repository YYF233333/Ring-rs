//! # SaveManager 模块
//!
//! 存档文件管理，负责存档的读写和 slot 管理。
//!
//! ## 文件布局
//!
//! ```text
//! saves/
//! ├── continue.json     # 专用"继续"存档（退出/返回标题时自动维护）
//! ├── slot_001.json
//! ├── slot_002.json
//! └── ...
//! ```

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use vn_runtime::{SaveData, SaveError};

/// 最大存档槽位数
pub const MAX_SAVE_SLOTS: u32 = 99;

/// Continue 存档文件名
const CONTINUE_SAVE_NAME: &str = "continue.json";

/// 存档管理器
pub struct SaveManager {
    /// 存档目录
    saves_dir: PathBuf,
}

impl SaveManager {
    /// 创建存档管理器
    ///
    /// # 参数
    ///
    /// - `saves_dir`: 存档目录路径
    pub fn new(saves_dir: impl AsRef<Path>) -> Self {
        let saves_dir = saves_dir.as_ref().to_path_buf();
        Self { saves_dir }
    }

    /// 确保存档目录存在
    pub fn ensure_dir(&self) -> Result<(), SaveError> {
        if !self.saves_dir.exists() {
            fs::create_dir_all(&self.saves_dir)
                .map_err(|e| SaveError::IoError(format!("无法创建存档目录: {}", e)))?;
        }
        Ok(())
    }

    /// 获取存档文件路径
    pub fn slot_path(&self, slot: u32) -> PathBuf {
        self.saves_dir.join(format!("slot_{:03}.json", slot))
    }

    /// 保存存档
    pub fn save(&self, data: &SaveData) -> Result<(), SaveError> {
        self.ensure_dir()?;

        let path = self.slot_path(data.metadata.slot);
        let json = data.to_json()?;

        let mut file = File::create(&path)
            .map_err(|e| SaveError::IoError(format!("无法创建存档文件: {}", e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| SaveError::IoError(format!("无法写入存档文件: {}", e)))?;

        println!("💾 存档保存成功: {:?}", path);
        Ok(())
    }

    /// 读取存档
    pub fn load(&self, slot: u32) -> Result<SaveData, SaveError> {
        let path = self.slot_path(slot);

        if !path.exists() {
            return Err(SaveError::NotFound(path.to_string_lossy().to_string()));
        }

        let mut file = File::open(&path)
            .map_err(|e| SaveError::IoError(format!("无法打开存档文件: {}", e)))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|e| SaveError::IoError(format!("无法读取存档文件: {}", e)))?;

        let data = SaveData::from_json(&json)?;

        println!("💾 存档读取成功: {:?}", path);
        Ok(data)
    }

    /// 删除存档
    pub fn delete(&self, slot: u32) -> Result<(), SaveError> {
        let path = self.slot_path(slot);

        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| SaveError::IoError(format!("无法删除存档文件: {}", e)))?;
            println!("💾 存档删除成功: {:?}", path);
        }

        Ok(())
    }

    /// 检查存档是否存在
    pub fn exists(&self, slot: u32) -> bool {
        self.slot_path(slot).exists()
    }

    /// 列出所有存档
    pub fn list_saves(&self) -> Vec<(u32, PathBuf)> {
        let mut saves = Vec::new();

        if !self.saves_dir.exists() {
            return saves;
        }

        if let Ok(entries) = fs::read_dir(&self.saves_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // 解析 slot_XXX.json
                    if name.starts_with("slot_") && name.ends_with(".json") {
                        if let Ok(slot) = name[5..8].parse::<u32>() {
                            saves.push((slot, path));
                        }
                    }
                }
            }
        }

        saves.sort_by_key(|(slot, _)| *slot);
        saves
    }

    /// 获取下一个可用的存档槽位
    pub fn next_available_slot(&self) -> Option<u32> {
        for slot in 1..=MAX_SAVE_SLOTS {
            if !self.exists(slot) {
                return Some(slot);
            }
        }
        None
    }

    /// 获取存档信息（不加载完整数据）
    pub fn get_save_info(&self, slot: u32) -> Option<SaveInfo> {
        let path = self.slot_path(slot);

        if !path.exists() {
            return None;
        }

        // 尝试读取并解析元数据
        if let Ok(data) = self.load(slot) {
            Some(SaveInfo {
                slot: Some(slot),
                timestamp: data.metadata.timestamp.clone(),
                chapter_title: data.metadata.chapter_title.clone(),
                script_id: data.runtime_state.position.script_id.clone(),
                play_time_secs: data.metadata.play_time_secs,
            })
        } else {
            None
        }
    }

    // =========================================================================
    // Continue 存档（专用"继续"存档）
    // =========================================================================

    /// Continue 存档路径
    fn continue_path(&self) -> PathBuf {
        self.saves_dir.join(CONTINUE_SAVE_NAME)
    }

    /// 保存 Continue 存档
    ///
    /// 在返回标题 / 退出游戏时调用，记录当前游戏位置。
    pub fn save_continue(&self, data: &SaveData) -> Result<(), SaveError> {
        self.ensure_dir()?;

        let path = self.continue_path();
        let json = data.to_json()?;

        let mut file = File::create(&path)
            .map_err(|e| SaveError::IoError(format!("无法创建 Continue 存档: {}", e)))?;

        file.write_all(json.as_bytes())
            .map_err(|e| SaveError::IoError(format!("无法写入 Continue 存档: {}", e)))?;

        println!("💾 Continue 存档保存成功: {:?}", path);
        Ok(())
    }

    /// 读取 Continue 存档
    pub fn load_continue(&self) -> Result<SaveData, SaveError> {
        let path = self.continue_path();

        if !path.exists() {
            return Err(SaveError::NotFound("Continue 存档不存在".to_string()));
        }

        let mut file = File::open(&path)
            .map_err(|e| SaveError::IoError(format!("无法打开 Continue 存档: {}", e)))?;

        let mut json = String::new();
        file.read_to_string(&mut json)
            .map_err(|e| SaveError::IoError(format!("无法读取 Continue 存档: {}", e)))?;

        let data = SaveData::from_json(&json)?;

        println!("💾 Continue 存档读取成功");
        Ok(data)
    }

    /// 检查 Continue 存档是否存在
    pub fn has_continue(&self) -> bool {
        self.continue_path().exists()
    }

    /// 删除 Continue 存档
    pub fn delete_continue(&self) -> Result<(), SaveError> {
        let path = self.continue_path();

        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| SaveError::IoError(format!("无法删除 Continue 存档: {}", e)))?;
            println!("💾 Continue 存档已删除");
        }

        Ok(())
    }

    /// 获取 Continue 存档信息
    pub fn get_continue_info(&self) -> Option<SaveInfo> {
        if let Ok(data) = self.load_continue() {
            Some(SaveInfo {
                slot: None, // Continue 没有槽位号
                timestamp: data.metadata.timestamp.clone(),
                chapter_title: data.metadata.chapter_title.clone(),
                script_id: data.runtime_state.position.script_id.clone(),
                play_time_secs: data.metadata.play_time_secs,
            })
        } else {
            None
        }
    }
}

/// 存档信息（用于 UI 显示）
#[derive(Debug, Clone)]
pub struct SaveInfo {
    /// 槽位号（Continue 存档为 None）
    pub slot: Option<u32>,
    /// 保存时间（ISO 8601 格式）
    pub timestamp: String,
    /// 章节标题
    pub chapter_title: Option<String>,
    /// 脚本 ID
    pub script_id: String,
    /// 游戏时长（秒）
    pub play_time_secs: u64,
}

impl SaveInfo {
    /// 格式化时间戳为可读格式
    pub fn formatted_timestamp(&self) -> String {
        // 尝试解析 Unix 时间戳
        if let Ok(secs) = self.timestamp.parse::<u64>() {
            format_unix_timestamp(secs)
        } else {
            // 已经是格式化的字符串
            self.timestamp.clone()
        }
    }

    /// 格式化游玩时间
    pub fn formatted_play_time(&self) -> String {
        format_play_time(self.play_time_secs)
    }
}

/// 格式化 Unix 时间戳为可读格式
fn format_unix_timestamp(secs: u64) -> String {
    use std::time::{Duration, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(secs);

    // 简单格式化（不依赖 chrono）
    if let Ok(since_epoch) = datetime.duration_since(UNIX_EPOCH) {
        let total_secs = since_epoch.as_secs();
        // 计算年月日时分（简化版，不考虑时区和闰年精确性）
        let days = total_secs / 86400;
        let time_of_day = total_secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;

        // 粗略计算年份（从 1970 开始）
        let years = 1970 + (days / 365);
        let remaining_days = days % 365;
        let month = remaining_days / 30 + 1;
        let day = remaining_days % 30 + 1;

        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}",
            years,
            month.min(12),
            day.min(31),
            hours,
            minutes
        )
    } else {
        secs.to_string()
    }
}

/// 格式化游玩时间
fn format_play_time(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::atomic::{AtomicU32, Ordering};
    use vn_runtime::RuntimeState;

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_temp_dir() -> PathBuf {
        let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let thread_id = std::thread::current().id();
        env::temp_dir().join(format!("ring_rs_test_saves_{}_{:?}", id, thread_id))
    }

    #[test]
    fn test_save_and_load() {
        let dir = unique_temp_dir();
        let manager = SaveManager::new(&dir);
        manager.ensure_dir().unwrap();

        let state = RuntimeState::new("test_script");
        let save_data = SaveData::new(1, state).with_chapter("测试章节");

        // 保存
        manager.save(&save_data).unwrap();
        assert!(manager.exists(1));

        // 读取
        let loaded = manager.load(1).unwrap();
        assert_eq!(loaded.metadata.slot, 1);
        assert_eq!(loaded.metadata.chapter_title, Some("测试章节".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_slot_not_found() {
        let dir = unique_temp_dir();
        let manager = SaveManager::new(&dir);
        manager.ensure_dir().unwrap();

        let result = manager.load(99);
        assert!(matches!(result, Err(SaveError::NotFound(_))));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_list_saves() {
        let dir = unique_temp_dir();
        let manager = SaveManager::new(&dir);
        manager.ensure_dir().unwrap();

        // 创建几个存档
        for slot in [1, 3, 5] {
            let state = RuntimeState::new("test");
            let data = SaveData::new(slot, state);
            manager.save(&data).unwrap();
        }

        let saves = manager.list_saves();
        assert_eq!(saves.len(), 3);
        assert_eq!(saves[0].0, 1);
        assert_eq!(saves[1].0, 3);
        assert_eq!(saves[2].0, 5);

        let _ = fs::remove_dir_all(&dir);
    }
}
