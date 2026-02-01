//! # SaveManager 模块
//!
//! 存档文件管理，负责存档的读写和 slot 管理。
//!
//! ## 文件布局
//!
//! ```text
//! saves/
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
                slot,
                timestamp: data.metadata.timestamp.clone(),
                chapter_title: data.metadata.chapter_title.clone(),
                script_id: data.runtime_state.position.script_id.clone(),
            })
        } else {
            None
        }
    }
}

/// 存档信息（用于 UI 显示）
#[derive(Debug, Clone)]
pub struct SaveInfo {
    pub slot: u32,
    pub timestamp: String,
    pub chapter_title: Option<String>,
    pub script_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use vn_runtime::RuntimeState;
    use std::env;
    use std::sync::atomic::{AtomicU32, Ordering};

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
        let save_data = SaveData::new(1, state)
            .with_chapter("测试章节");

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
