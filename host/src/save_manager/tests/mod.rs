use super::*;
use std::env;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use vn_runtime::RuntimeState;

static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

fn path_file_name(p: &Path) -> &str {
    p.file_name().and_then(|n| n.to_str()).unwrap()
}

fn unique_temp_dir() -> PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let thread_id = std::thread::current().id();
    env::temp_dir().join(format!("ring_rs_test_saves_{}_{:?}", id, thread_id))
}

fn make_save(slot: u32, script_id: &str) -> SaveData {
    SaveData::new(slot, RuntimeState::new(script_id))
}

/// 测试用临时目录，drop 时自动清理。
struct TempSaveDir(PathBuf);

impl Drop for TempSaveDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn temp_manager() -> (SaveManager, TempSaveDir) {
    let dir = unique_temp_dir();
    let guard = TempSaveDir(dir.clone());
    let manager = SaveManager::new(&dir);
    (manager, guard)
}

fn temp_manager_with_dir() -> (SaveManager, TempSaveDir) {
    let (manager, guard) = temp_manager();
    manager.ensure_dir().unwrap();
    (manager, guard)
}
mod high_value;
mod low_value;
