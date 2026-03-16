//! # PersistentStore 模块
//!
//! 管理跨会话持久化变量的读取与写入。
//!
//! ## 文件布局
//!
//! ```text
//! saves/
//! ├── persistent.json   # 持久化变量存储（fullRestart 时写入，启动时读取）
//! └── ...
//! ```
//!
//! ## 设计说明
//!
//! - key 以 bare key 形式存储（不含 `persistent.` 前缀）
//! - `merge_from` 将 runtime 中的 `persistent_variables` 合并到 store（runtime 值优先）
//! - 文件不存在时 `load` 返回空 store（不报错）

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use tracing::{error, info, warn};
use vn_runtime::state::VarValue;

/// 持久化变量文件名
const PERSISTENT_FILE: &str = "persistent.json";

/// 持久化变量存储
///
/// 保存跨会话持久化的脚本变量（通过 `$persistent.key` 访问）。
/// key 为 bare key，不含 `persistent.` 前缀。
pub struct PersistentStore {
    /// 存档目录
    saves_dir: PathBuf,
    /// 持久化变量（bare key → value）
    pub variables: HashMap<String, VarValue>,
}

impl PersistentStore {
    /// 创建空的持久化变量存储（无磁盘关联，用于测试或默认初始化）
    pub fn empty() -> Self {
        Self {
            saves_dir: PathBuf::new(),
            variables: HashMap::new(),
        }
    }

    /// 从存档目录加载持久化变量
    ///
    /// 若文件不存在，返回空 store；若 JSON 解析失败，打印警告并返回空 store。
    pub fn load(saves_dir: impl AsRef<Path>) -> Self {
        let saves_dir = saves_dir.as_ref().to_path_buf();
        let path = saves_dir.join(PERSISTENT_FILE);

        let variables = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str::<HashMap<String, VarValue>>(&content) {
                    Ok(vars) => {
                        info!(path = %path.display(), count = vars.len(), "持久化变量加载成功");
                        vars
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "持久化变量 JSON 解析失败，使用空 store");
                        HashMap::new()
                    }
                },
                Err(e) => {
                    warn!(path = %path.display(), error = %e, "持久化变量文件读取失败，使用空 store");
                    HashMap::new()
                }
            }
        } else {
            info!(path = %path.display(), "持久化变量文件不存在，使用空 store");
            HashMap::new()
        };

        Self {
            saves_dir,
            variables,
        }
    }

    /// 将持久化变量写入磁盘
    ///
    /// 若目录不存在，尝试创建。写入失败时打印错误。
    pub fn save(&self) -> Result<(), String> {
        if !self.saves_dir.exists() {
            fs::create_dir_all(&self.saves_dir).map_err(|e| format!("无法创建存档目录: {}", e))?;
        }

        let path = self.saves_dir.join(PERSISTENT_FILE);
        let content = serde_json::to_string_pretty(&self.variables)
            .map_err(|e| format!("持久化变量序列化失败: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("持久化变量写入失败: {}", e))?;

        info!(path = %path.display(), count = self.variables.len(), "持久化变量保存成功");
        Ok(())
    }

    /// 将 runtime `persistent_variables` 合并入 store（runtime 值覆盖 store 中已有值）
    pub fn merge_from(&mut self, vars: &HashMap<String, VarValue>) {
        for (k, v) in vars {
            self.variables.insert(k.clone(), v.clone());
        }
    }

    /// 尝试保存，失败时记录错误（便于调用方忽略错误）
    pub fn save_or_log(&self) {
        if let Err(e) = self.save() {
            error!(error = %e, "持久化变量保存失败");
        }
    }

    /// 检查指定赛季是否已通关
    ///
    /// 等价于 `variables["complete_<season>"] == true`。
    pub fn is_season_complete(&self, season: &str) -> bool {
        let key = format!("complete_{season}");
        self.variables
            .get(&key)
            .is_some_and(|v| matches!(v, VarValue::Bool(true)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn empty_store_has_no_variables() {
        let store = PersistentStore::empty();
        assert!(store.variables.is_empty());
    }

    #[test]
    fn merge_from_inserts_new_keys() {
        let mut store = PersistentStore::empty();
        let mut vars = HashMap::new();
        vars.insert("key1".to_string(), VarValue::Int(42));
        store.merge_from(&vars);
        assert_eq!(store.variables.get("key1"), Some(&VarValue::Int(42)));
    }

    #[test]
    fn merge_from_overwrites_existing() {
        let mut store = PersistentStore::empty();
        store.variables.insert("key1".to_string(), VarValue::Int(1));
        let mut vars = HashMap::new();
        vars.insert("key1".to_string(), VarValue::Int(99));
        store.merge_from(&vars);
        assert_eq!(store.variables.get("key1"), Some(&VarValue::Int(99)));
    }

    #[test]
    fn merge_from_preserves_keys_not_in_source() {
        let mut store = PersistentStore::empty();
        store
            .variables
            .insert("keep".to_string(), VarValue::Bool(true));
        let vars: HashMap<String, VarValue> = HashMap::new();
        store.merge_from(&vars);
        assert!(store.variables.contains_key("keep"));
    }

    #[test]
    fn is_season_complete_true() {
        let mut store = PersistentStore::empty();
        store
            .variables
            .insert("complete_s1".to_string(), VarValue::Bool(true));
        assert!(store.is_season_complete("s1"));
    }

    #[test]
    fn is_season_complete_false_when_bool_false() {
        let mut store = PersistentStore::empty();
        store
            .variables
            .insert("complete_s1".to_string(), VarValue::Bool(false));
        assert!(!store.is_season_complete("s1"));
    }

    #[test]
    fn is_season_complete_false_when_missing() {
        let store = PersistentStore::empty();
        assert!(!store.is_season_complete("s1"));
    }

    #[test]
    fn is_season_complete_false_when_wrong_type() {
        let mut store = PersistentStore::empty();
        store
            .variables
            .insert("complete_s1".to_string(), VarValue::Int(1));
        assert!(!store.is_season_complete("s1"));
    }

    #[test]
    fn load_returns_empty_when_file_not_found() {
        let tmp = TempDir::new().unwrap();
        let store = PersistentStore::load(tmp.path());
        assert!(store.variables.is_empty());
    }

    #[test]
    fn load_valid_json_populates_variables() {
        let tmp = TempDir::new().unwrap();
        let json = r#"{"complete_s1":{"Bool":true},"score":{"Int":100}}"#;
        std::fs::write(tmp.path().join("persistent.json"), json).unwrap();
        let store = PersistentStore::load(tmp.path());
        assert_eq!(
            store.variables.get("complete_s1"),
            Some(&VarValue::Bool(true))
        );
        assert_eq!(store.variables.get("score"), Some(&VarValue::Int(100)));
    }

    #[test]
    fn load_invalid_json_returns_empty() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("persistent.json"), "not valid json").unwrap();
        let store = PersistentStore::load(tmp.path());
        assert!(store.variables.is_empty());
    }

    #[test]
    fn save_and_reload_round_trip() {
        let tmp = TempDir::new().unwrap();
        let mut store = PersistentStore::load(tmp.path());
        store
            .variables
            .insert("complete_s2".to_string(), VarValue::Bool(true));
        store
            .variables
            .insert("high_score".to_string(), VarValue::Int(9999));
        store.save().expect("save should succeed");

        let loaded = PersistentStore::load(tmp.path());
        assert_eq!(
            loaded.variables.get("complete_s2"),
            Some(&VarValue::Bool(true))
        );
        assert_eq!(
            loaded.variables.get("high_score"),
            Some(&VarValue::Int(9999))
        );
    }

    #[test]
    fn save_or_log_does_not_panic() {
        let tmp = TempDir::new().unwrap();
        let mut store = PersistentStore::load(tmp.path());
        store
            .variables
            .insert("flag".to_string(), VarValue::Bool(false));
        store.save_or_log(); // should complete without panic
        assert!(tmp.path().join("persistent.json").exists());
    }
}
