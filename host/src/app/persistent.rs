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
