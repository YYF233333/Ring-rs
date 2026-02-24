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
