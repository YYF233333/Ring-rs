//! 存档管理系统
//!
//! 负责存档的读写、slot 管理和 continue 存档。
//!
//! 文件布局：
//! ```text
//! saves/
//! ├── continue.json
//! ├── slot_001.json
//! ├── slot_002.json
//! └── ...
//! ```

use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde::Serialize;
use tracing::info;
use vn_runtime::{SaveData, SaveError};

/// 最大存档槽位数
#[allow(dead_code)]
pub const MAX_SAVE_SLOTS: u32 = 99;

const CONTINUE_SAVE_NAME: &str = "continue.json";

/// 存档管理器
pub struct SaveManager {
    saves_dir: PathBuf,
}

impl SaveManager {
    /// 创建存档管理器
    pub fn new(saves_dir: impl AsRef<Path>) -> Self {
        Self {
            saves_dir: saves_dir.as_ref().to_path_buf(),
        }
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

        info!(path = %path.display(), "存档保存成功");
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
        info!(path = %path.display(), "存档读取成功");
        Ok(data)
    }

    /// 删除存档
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

    /// 缩略图文件路径
    pub fn thumbnail_path(&self, slot: u32) -> PathBuf {
        self.saves_dir.join(format!("thumb_{:03}.png", slot))
    }

    /// 保存已编码的 PNG 字节为缩略图文件
    pub fn save_thumbnail_png(&self, slot: u32, png_bytes: &[u8]) -> Result<(), String> {
        self.ensure_dir().map_err(|e| e.to_string())?;
        let path = self.thumbnail_path(slot);
        let mut file = File::create(&path).map_err(|e| format!("创建缩略图文件失败: {e}"))?;
        file.write_all(png_bytes)
            .map_err(|e| format!("写入缩略图失败: {e}"))?;
        info!(path = %path.display(), "缩略图保存成功");
        Ok(())
    }

    /// 加载缩略图并返回 base64 编码的 PNG
    pub fn load_thumbnail_base64(&self, slot: u32) -> Option<String> {
        let path = self.thumbnail_path(slot);
        let bytes = fs::read(&path).ok()?;
        Some(base64::engine::general_purpose::STANDARD.encode(&bytes))
    }

    /// 检查存档是否存在
    #[allow(dead_code)]
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

    /// 获取存档信息（不加载完整数据）
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

    // ── Continue 存档 ────────────────────────────────────────────────────────

    fn continue_path(&self) -> PathBuf {
        self.saves_dir.join(CONTINUE_SAVE_NAME)
    }

    /// 保存 Continue 存档
    #[allow(dead_code)]
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
        info!(path = %path.display(), "Continue 存档读取成功");
        Ok(data)
    }

    /// 检查 Continue 存档是否存在
    pub fn has_continue(&self) -> bool {
        self.continue_path().exists()
    }
}

/// 存档信息（用于前端列表展示）
#[derive(Debug, Clone, Serialize)]
pub struct SaveInfo {
    pub slot: Option<u32>,
    pub timestamp: String,
    pub chapter_title: Option<String>,
    pub script_id: String,
    pub play_time_secs: u64,
}

impl SaveInfo {
    /// 格式化时间戳为可读格式
    #[allow(dead_code)]
    pub fn formatted_timestamp(&self) -> String {
        if let Ok(secs) = self.timestamp.parse::<u64>()
            && let Some(dt) = DateTime::<Utc>::from_timestamp(secs as i64, 0)
        {
            return dt.format("%Y-%m-%d %H:%M").to_string();
        }
        self.timestamp.clone()
    }
}
