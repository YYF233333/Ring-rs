use super::*;

#[test]
fn test_save_and_load() {
    let (manager, _guard) = temp_manager_with_dir();

    let state = RuntimeState::new("test_script");
    let save_data = SaveData::new(1, state).with_chapter("测试章节");

    manager.save(&save_data).unwrap();
    assert!(manager.exists(1));

    let loaded = manager.load(1).unwrap();
    assert_eq!(loaded.metadata.slot, 1);
    assert_eq!(loaded.metadata.chapter_title, Some("测试章节".to_string()));
}

#[test]
fn test_slot_not_found() {
    let (manager, _guard) = temp_manager_with_dir();

    let result = manager.load(99);
    assert!(matches!(result, Err(SaveError::NotFound(_))));
}

#[test]
fn test_list_saves() {
    let (manager, _guard) = temp_manager_with_dir();

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
}

#[test]
fn test_list_saves_empty_dir() {
    let (manager, _guard) = temp_manager();
    assert!(manager.list_saves().is_empty());
}

#[test]
fn test_delete_save() {
    let (manager, _guard) = temp_manager();

    manager.save(&make_save(2, "script_a")).unwrap();
    assert!(manager.exists(2));

    manager.delete(2).unwrap();
    assert!(!manager.exists(2));
}

#[test]
fn test_delete_nonexistent_is_ok() {
    let (manager, _guard) = temp_manager_with_dir();

    assert!(manager.delete(50).is_ok());
}

#[test]
fn test_get_save_info() {
    let (manager, _guard) = temp_manager();

    let data = make_save(7, "chapter_one").with_chapter("第一章");
    manager.save(&data).unwrap();

    let info = manager.get_save_info(7).unwrap();
    assert_eq!(info.slot, Some(7));
    assert_eq!(info.chapter_title, Some("第一章".to_string()));
    assert_eq!(info.script_id, "chapter_one");
}

#[test]
fn test_get_save_info_missing_returns_none() {
    let (manager, _guard) = temp_manager_with_dir();

    assert!(manager.get_save_info(42).is_none());
}

#[test]
fn test_next_available_slot_fresh() {
    let (manager, _guard) = temp_manager();
    assert_eq!(manager.next_available_slot(), Some(1));
}

#[test]
fn test_next_available_slot_skips_used() {
    let (manager, _guard) = temp_manager();

    manager.save(&make_save(1, "s")).unwrap();
    manager.save(&make_save(2, "s")).unwrap();

    assert_eq!(manager.next_available_slot(), Some(3));
}

#[test]
fn test_load_corrupted_json_returns_error() {
    let (manager, _guard) = temp_manager_with_dir();

    fs::write(manager.slot_path(10), b"not valid json").unwrap();

    let result = manager.load(10);
    assert!(result.is_err());
}

#[test]
fn test_continue_save_and_load() {
    let (manager, _guard) = temp_manager();

    assert!(!manager.has_continue());

    manager.save_continue(&make_save(0, "main_story")).unwrap();
    assert!(manager.has_continue());

    let loaded = manager.load_continue().unwrap();
    assert_eq!(loaded.runtime_state.position.script_id, "main_story");
}

#[test]
fn test_continue_load_missing_returns_error() {
    let (manager, _guard) = temp_manager_with_dir();

    let result = manager.load_continue();
    assert!(matches!(result, Err(SaveError::NotFound(_))));
}

#[test]
fn test_continue_delete() {
    let (manager, _guard) = temp_manager();

    manager.save_continue(&make_save(0, "ep1")).unwrap();
    assert!(manager.has_continue());

    manager.delete_continue().unwrap();
    assert!(!manager.has_continue());
    assert!(manager.delete_continue().is_ok());
}

#[test]
fn test_get_continue_info() {
    let (manager, _guard) = temp_manager();

    manager
        .save_continue(&make_save(0, "prologue").with_chapter("序章"))
        .unwrap();

    let info = manager.get_continue_info().unwrap();
    assert!(info.slot.is_none());
    assert_eq!(info.chapter_title, Some("序章".to_string()));
    assert_eq!(info.script_id, "prologue");
}

#[test]
fn test_get_continue_info_missing_returns_none() {
    let (manager, _guard) = temp_manager_with_dir();

    assert!(manager.get_continue_info().is_none());
}

#[test]
fn test_load_thumbnail_bytes_missing_returns_none() {
    let (manager, _guard) = temp_manager_with_dir();

    assert!(manager.load_thumbnail_bytes(1).is_none());
}

#[test]
fn test_delete_save_also_removes_thumbnail() {
    let (manager, _guard) = temp_manager();

    manager.save(&make_save(3, "s")).unwrap();
    let thumb_path = manager.thumbnail_path(3);
    fs::write(&thumb_path, b"fake png").unwrap();
    assert!(thumb_path.exists());

    manager.delete(3).unwrap();
    assert!(!manager.exists(3));
    assert!(!thumb_path.exists());
}

#[test]
fn test_save_thumbnail_and_load_thumbnail_bytes_round_trip() {
    let (manager, _guard) = temp_manager_with_dir();
    let rgba: Vec<u8> = vec![
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
    ];

    manager.save_thumbnail(7, &rgba, 2, 2, 2, 2).unwrap();

    let bytes = manager
        .load_thumbnail_bytes(7)
        .expect("应能读回刚保存的缩略图");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn test_save_thumbnail_rejects_invalid_rgba_buffer() {
    let (manager, _guard) = temp_manager_with_dir();
    let rgba = vec![255, 0, 0, 255, 0, 255, 0];

    let err = manager
        .save_thumbnail(8, &rgba, 2, 2, 2, 2)
        .expect_err("无效 RGBA 缓冲区应报错");
    assert!(err.contains("无法从 RGBA 数据创建图像"));
    assert!(manager.load_thumbnail_bytes(8).is_none());
}

// ============ 路径格式 / SaveInfo 时间戳（自 low_value 迁入）===========

#[test]
fn test_slot_path_format() {
    let manager = SaveManager::new(PathBuf::from("/saves"));
    assert_eq!(
        path_file_name(manager.slot_path(1).as_path()),
        "slot_001.json"
    );
    assert_eq!(
        path_file_name(manager.slot_path(99).as_path()),
        "slot_099.json"
    );
}

#[test]
fn test_thumbnail_path_format() {
    let manager = SaveManager::new(PathBuf::from("/saves"));
    assert_eq!(
        path_file_name(manager.thumbnail_path(5).as_path()),
        "thumb_005.png"
    );
}

#[test]
fn test_save_info_formatted_timestamp_numeric() {
    let info = SaveInfo {
        slot: Some(1),
        timestamp: "1710511800".to_string(),
        chapter_title: None,
        script_id: "s".to_string(),
        play_time_secs: 0,
    };
    assert!(info.formatted_timestamp().contains("2024"));
}

#[test]
fn test_save_info_formatted_timestamp_fallback() {
    let info = SaveInfo {
        slot: None,
        timestamp: "not-a-number".to_string(),
        chapter_title: None,
        script_id: "s".to_string(),
        play_time_secs: 0,
    };
    assert_eq!(info.formatted_timestamp(), "not-a-number");
}
