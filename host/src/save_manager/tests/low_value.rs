use super::*;

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
