use super::*;
use crate::command::Position;
use crate::history::HistoryEvent;

#[test]
fn test_save_version_compatibility() {
    let current = SaveVersion::current();
    assert!(current.is_compatible());

    let old_minor = SaveVersion { major: 1, minor: 0 };
    assert!(old_minor.is_compatible());

    let incompatible = SaveVersion { major: 2, minor: 0 };
    assert!(!incompatible.is_compatible());
}

#[test]
fn test_save_version_current_and_to_string() {
    let v = SaveVersion::current();
    assert_eq!(
        v.to_string(),
        format!("{}.{}", SAVE_VERSION_MAJOR, SAVE_VERSION_MINOR)
    );
}

#[test]
fn test_save_metadata_builders() {
    let md = SaveMetadata::new(7)
        .with_chapter("第二章")
        .with_play_time(123);
    assert_eq!(md.slot, 7);
    assert_eq!(md.chapter_title, Some("第二章".to_string()));
    assert_eq!(md.play_time_secs, 123);
    // timestamp 只要求存在且非空（当前实现是 unix seconds）
    assert!(!md.timestamp.is_empty());
    assert!(md.timestamp.chars().all(|c| c.is_ascii_digit()));
    assert!(md.timestamp.parse::<u64>().is_ok());
}

#[test]
fn test_save_data_serialization() {
    let mut state = RuntimeState::new("test_script");
    state.current_background = Some("bg.png".to_string());
    state.visible_characters.insert(
        "char1".to_string(),
        ("char1.png".to_string(), Position::Center),
    );

    let save_data = SaveData::new(1, state)
        .with_chapter("第一章")
        .with_audio(AudioState {
            current_bgm: Some("bgm.mp3".to_string()),
            bgm_looping: true,
        });

    // 序列化
    let json = save_data.to_json().unwrap();
    assert!(json.contains("test_script"));
    assert!(json.contains("第一章"));

    // 反序列化
    let loaded = SaveData::from_json(&json).unwrap();
    assert_eq!(loaded.metadata.slot, 1);
    assert_eq!(loaded.metadata.chapter_title, Some("第一章".to_string()));
    assert_eq!(loaded.runtime_state.position.script_id, "test_script");
}

#[test]
fn test_save_data_with_render_and_history() {
    let state = RuntimeState::new("test_script");

    let render = RenderSnapshot {
        background: Some("bg.png".to_string()),
        characters: vec![CharacterSnapshot {
            alias: "char1".to_string(),
            texture_path: "char1.png".to_string(),
            position: "Center".to_string(),
        }],
    };

    let mut history = History::new();
    history.push(HistoryEvent::dialogue(
        Some("北风".to_string()),
        "你好".to_string(),
    ));

    let save_data = SaveData::new(1, state)
        .with_render(render.clone())
        .with_history(history.clone());

    assert_eq!(save_data.render.background, render.background);
    assert_eq!(save_data.render.characters.len(), 1);
    assert_eq!(save_data.render.characters[0].alias, "char1");

    assert_eq!(save_data.history.len(), history.len());
}

#[test]
fn test_incompatible_version_error() {
    let json = r#"{
        "version": { "major": 99, "minor": 0 },
        "metadata": { "slot": 1, "timestamp": "0", "chapter_title": null, "play_time_secs": 0 },
        "runtime_state": {
            "position": { "script_id": "test", "node_index": 0 },
            "variables": {},
            "waiting": "None",
            "visible_characters": {},
            "current_background": null
        },
        "audio": { "current_bgm": null, "bgm_looping": false },
        "render": { "background": null, "characters": [] },
        "history": { "events": [], "max_events": 1000 }
    }"#;

    let result = SaveData::from_json(json);
    assert!(matches!(result, Err(SaveError::IncompatibleVersion { .. })));
}

#[test]
fn test_mode_data_backward_compatibility() {
    // 旧存档无 mode_data 字段 → 反序列化应成功，mode_data 为空
    let json = r#"{
        "version": { "major": 1, "minor": 0 },
        "metadata": { "slot": 1, "timestamp": "0", "chapter_title": null, "play_time_secs": 0 },
        "runtime_state": {
            "position": { "script_id": "test", "node_index": 0 },
            "variables": {},
            "waiting": "None",
            "visible_characters": {},
            "current_background": null
        },
        "audio": { "current_bgm": null, "bgm_looping": false },
        "render": { "background": null, "characters": [] },
        "history": { "events": [], "max_events": 1000 }
    }"#;

    let data = SaveData::from_json(json).unwrap();
    assert!(data.mode_data.is_empty());
}

#[test]
fn test_mode_data_round_trip() {
    let state = RuntimeState::new("test");
    let mut mode_data = BTreeMap::new();
    mode_data.insert(
        "card_battle".to_string(),
        serde_json::json!({"deck": ["fire", "ice"], "score": 42}),
    );

    let save = SaveData::new(1, state).with_mode_data(mode_data);
    let json = save.to_json().unwrap();
    assert!(json.contains("card_battle"));

    let loaded = SaveData::from_json(&json).unwrap();
    assert_eq!(loaded.mode_data.len(), 1);
    assert_eq!(loaded.mode_data["card_battle"]["score"], 42);
}

#[test]
fn test_mode_data_empty_not_serialized() {
    let state = RuntimeState::new("test");
    let save = SaveData::new(1, state);
    let json = save.to_json().unwrap();
    // mode_data 为空时不应出现在 JSON 中
    assert!(!json.contains("mode_data"));
}

#[test]
fn test_save_error_display() {
    let e = SaveError::IoError("disk full".to_string());
    assert_eq!(e.to_string(), "文件操作失败: disk full");

    let e = SaveError::NotFound("slot_001.json".to_string());
    assert_eq!(e.to_string(), "存档不存在: slot_001.json");

    let e = SaveError::SerializationFailed("bad".to_string());
    assert_eq!(e.to_string(), "序列化失败: bad");

    let e = SaveError::DeserializationFailed("bad".to_string());
    assert_eq!(e.to_string(), "反序列化失败: bad");

    let e = SaveError::IncompatibleVersion {
        save_version: "2.0".to_string(),
        current_version: "1.0".to_string(),
    };
    assert!(e.to_string().contains("存档版本不兼容"));
    assert!(e.to_string().contains("2.0"));
    assert!(e.to_string().contains("1.0"));
}
