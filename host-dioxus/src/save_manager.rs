//! 存档管理系统

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::Serialize;
use tracing::info;
use vn_runtime::{SaveData, SaveError};

pub const MAX_SAVE_SLOTS: u32 = 99;
const CONTINUE_SAVE_NAME: &str = "continue.json";

pub struct SaveManager {
    saves_dir: PathBuf,
}

impl SaveManager {
    pub fn new(saves_dir: impl AsRef<Path>) -> Self {
        Self {
            saves_dir: saves_dir.as_ref().to_path_buf(),
        }
    }

    pub fn ensure_dir(&self) -> Result<(), SaveError> {
        if !self.saves_dir.exists() {
            fs::create_dir_all(&self.saves_dir)
                .map_err(|e| SaveError::IoError(format!("无法创建存档目录: {}", e)))?;
        }
        Ok(())
    }

    pub fn slot_path(&self, slot: u32) -> PathBuf {
        self.saves_dir.join(format!("slot_{:03}.json", slot))
    }

    pub fn save(&self, data: &SaveData) -> Result<(), SaveError> {
        self.ensure_dir()?;
        let path = self.slot_path(data.metadata.slot);
        let json = data.to_json()?;
        let mut file = File::create(&path)
            .map_err(|e| SaveError::IoError(format!("无法创建存档文件: {}", e)))?;
        file.write_all(json.as_bytes())
            .map_err(|e| SaveError::IoError(format!("无法写入存档文件: {}", e)))?;
        info!(path = %path.display(), "存档保存成功");
        Ok(())
    }

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
        info!(path = %path.display(), "存档读取成功");
        Ok(data)
    }

    pub fn delete(&self, slot: u32) -> Result<(), SaveError> {
        let path = self.slot_path(slot);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| SaveError::IoError(format!("无法删除存档文件: {}", e)))?;
            info!(path = %path.display(), "存档删除成功");
        }
        let thumb = self.thumbnail_path(slot);
        if thumb.exists() {
            let _ = fs::remove_file(&thumb);
        }
        Ok(())
    }

    pub fn thumbnail_path(&self, slot: u32) -> PathBuf {
        self.saves_dir.join(format!("thumb_{:03}.png", slot))
    }

    pub fn save_thumbnail_png(&self, slot: u32, png_bytes: &[u8]) -> Result<(), SaveError> {
        self.ensure_dir()?;
        let path = self.thumbnail_path(slot);
        let mut file = File::create(&path)
            .map_err(|e| SaveError::IoError(format!("创建缩略图文件失败: {e}")))?;
        file.write_all(png_bytes)
            .map_err(|e| SaveError::IoError(format!("写入缩略图失败: {e}")))?;
        info!(path = %path.display(), "缩略图保存成功");
        Ok(())
    }

    pub fn load_thumbnail_base64(&self, slot: u32) -> Option<String> {
        let path = self.thumbnail_path(slot);
        let bytes = fs::read(&path).ok()?;
        Some(base64::engine::general_purpose::STANDARD.encode(&bytes))
    }

    pub fn list_saves(&self) -> Vec<(u32, PathBuf)> {
        let mut saves = Vec::new();
        if !self.saves_dir.exists() {
            return saves;
        }
        if let Ok(entries) = fs::read_dir(&self.saves_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with("slot_")
                    && name.ends_with(".json")
                    && let Ok(slot) = name[5..8].parse::<u32>()
                {
                    saves.push((slot, path));
                }
            }
        }
        saves.sort_by_key(|(slot, _)| *slot);
        saves
    }

    pub fn get_save_info(&self, slot: u32) -> Option<SaveInfo> {
        if !self.slot_path(slot).exists() {
            return None;
        }
        let data = self.load(slot).ok()?;
        Some(SaveInfo {
            slot: Some(slot),
            timestamp: data.metadata.timestamp.clone(),
            chapter_title: data.metadata.chapter_title.clone(),
            script_id: data.runtime_state.position.script_id.clone(),
            play_time_secs: data.metadata.play_time_secs,
        })
    }

    fn continue_path(&self) -> PathBuf {
        self.saves_dir.join(CONTINUE_SAVE_NAME)
    }

    pub fn save_continue(&self, data: &SaveData) -> Result<(), SaveError> {
        self.ensure_dir()?;
        let path = self.continue_path();
        let json = data.to_json()?;
        let mut file = File::create(&path)
            .map_err(|e| SaveError::IoError(format!("无法创建 Continue 存档: {}", e)))?;
        file.write_all(json.as_bytes())
            .map_err(|e| SaveError::IoError(format!("无法写入 Continue 存档: {}", e)))?;
        info!(path = %path.display(), "Continue 存档保存成功");
        Ok(())
    }

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
        info!(path = %path.display(), "Continue 存档读取成功");
        Ok(data)
    }

    pub fn has_continue(&self) -> bool {
        self.continue_path().exists()
    }

    pub fn delete_continue(&self) -> Result<(), SaveError> {
        let path = self.continue_path();
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| SaveError::IoError(format!("无法删除 Continue 存档: {}", e)))?;
            info!(path = %path.display(), "Continue 存档删除成功");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SaveInfo {
    pub slot: Option<u32>,
    pub timestamp: String,
    pub chapter_title: Option<String>,
    pub script_id: String,
    pub play_time_secs: u64,
}
