use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use vn_runtime::command::Command;
use vn_runtime::state::WaitingReason;

use crate::audio::AudioManager;
use crate::config::AppConfig;
use crate::render_state::{HostScreen, PlaybackMode};
use crate::resources::ResourceManager;
use crate::save_manager::SaveManager;

use super::*;

fn unique_temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("ring_host_dioxus_{name}_{suffix}"))
}

fn make_state_with_services(script_path: &str, script_content: &str) -> (AppStateInner, PathBuf) {
    let root = unique_temp_dir("state");
    let assets_dir = root.join("assets");
    let saves_dir = root.join("saves");
    std::fs::create_dir_all(assets_dir.join("scripts")).unwrap();
    std::fs::create_dir_all(&saves_dir).unwrap();
    std::fs::write(assets_dir.join(script_path), script_content).unwrap();

    let mut config = AppConfig::default();
    config.assets_root = assets_dir.clone();
    config.saves_dir = saves_dir.clone();
    config.start_script_path = script_path.to_string();

    let mut inner = AppStateInner::new();
    inner.persistent_store = PersistentStore::load(&saves_dir);
    inner.services = Some(Services {
        audio: AudioManager::new(),
        resources: ResourceManager::new(&assets_dir),
        saves: SaveManager::new(&saves_dir),
        config,
        manifest: crate::manifest::Manifest::with_defaults(),
        layout: crate::layout_config::UiLayoutConfig::default_for_tests(),
        screen_defs: crate::screen_defs::ScreenDefinitions::default_for_tests(),
    });
    (inner, root)
}

#[test]
fn frontend_owner_reclaim_invalidates_stale_token() {
    let mut inner = AppStateInner::new();
    let first = inner.frontend_connected(Some("first".to_string()));
    assert!(inner.assert_owner(&first.client_token).is_ok());

    let second = inner.frontend_connected(Some("second".to_string()));
    assert!(inner.assert_owner(&first.client_token).is_err());
    assert!(inner.assert_owner(&second.client_token).is_ok());
}

#[test]
fn overlay_screen_blocks_progression_and_resets_playback() {
    let mut inner = AppStateInner::new();
    inner.set_host_screen(HostScreen::InGame);
    inner.set_playback_mode(PlaybackMode::Auto);
    inner.waiting = WaitingFor::Time {
        remaining_ms: 1_000,
    };

    inner.set_host_screen(HostScreen::Save);
    inner.process_tick(1.0);

    assert_eq!(inner.playback_mode, PlaybackMode::Normal);
    assert_eq!(
        inner.waiting,
        WaitingFor::Time {
            remaining_ms: 1_000
        }
    );
}

#[test]
fn return_to_title_with_save_continue_writes_continue_file() {
    let script = "changeBG <img src=\"../backgrounds/entry.png\" />\n";
    let (mut inner, root) = make_state_with_services("scripts/scene.md", script);

    inner.init_game_from_resource("scripts/scene.md").unwrap();
    inner.return_to_title(true);

    assert!(inner.services().saves.has_continue());

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn restore_from_save_keeps_saved_render_snapshot_without_entry_tick() {
    let script = "changeBG <img src=\"../backgrounds/entry.png\" />\n";
    let (mut inner, root) = make_state_with_services("scripts/scene.md", script);

    let mut runtime_state = vn_runtime::state::RuntimeState::new("scene");
    runtime_state.position.set_path("scripts/scene.md");

    let save_data = vn_runtime::SaveData::new(1, runtime_state)
        .with_render(vn_runtime::RenderSnapshot {
            background: Some("backgrounds/saved.png".to_string()),
            characters: Vec::new(),
        })
        .with_history(vn_runtime::History::new());

    inner.restore_from_save(save_data).unwrap();

    assert_eq!(
        inner.render_state.current_background.as_deref(),
        Some("backgrounds/saved.png")
    );
    assert_eq!(inner.host_screen, HostScreen::InGame);

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn build_save_data_uses_snapshot_boundary_while_waiting_for_choice() {
    let script = r#"
："选择前。"
| 选择 |        |
| ---- | ------ |
| 选项A | label_a |
| 选项B | label_b |
**label_a**
："选了A。"
**label_b**
："选了B。"
"#;
    let (mut inner, root) = make_state_with_services("scripts/choice.md", script);

    inner.init_game_from_resource("scripts/choice.md").unwrap();
    inner.render_state.complete_typewriter();
    inner.process_click();
    inner.process_tick(0.0);

    assert_eq!(inner.waiting, WaitingFor::Choice);
    assert!(inner.render_state.choices.is_some());
    assert!(inner.snapshot_stack.last().is_some());

    let save_data = inner.build_save_data(1).unwrap();
    assert!(matches!(
        save_data.runtime_state.waiting,
        WaitingReason::WaitForClick
    ));

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn build_save_data_uses_snapshot_boundary_while_waiting_for_ui_result() {
    let script = r#"
："第二部分：测试地图语法糖 showMap。"
showMap "demo_world" as $destination

if $destination == "town"
  ："你通过 showMap 选择了小镇。"
else
  ："你通过 showMap 选择了其他地点。"
endif
"#;
    let (mut inner, root) = make_state_with_services("scripts/ui_save.md", script);

    inner.init_game_from_resource("scripts/ui_save.md").unwrap();
    inner.render_state.complete_typewriter();
    inner.process_click();
    inner.run_script_tick();

    assert_eq!(
        inner.waiting,
        WaitingFor::UIResult {
            key: "show_map".to_string()
        }
    );
    assert!(inner.render_state.active_ui_mode.is_some());
    assert!(inner.snapshot_stack.last().is_some());

    let save_data = inner.build_save_data(1).unwrap();
    assert!(matches!(
        save_data.runtime_state.waiting,
        WaitingReason::WaitForClick
    ));

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn restore_from_save_normalizes_legacy_ui_wait_to_click() {
    let script = "：\"恢复等待态。\"\n";
    let (mut inner, root) = make_state_with_services("scripts/restore.md", script);

    let mut runtime_state = vn_runtime::state::RuntimeState::new("restore");
    runtime_state.position.set_path("scripts/restore.md");
    runtime_state.wait(WaitingReason::ui_result("show_map", "destination"));

    let save_data = vn_runtime::SaveData::new(1, runtime_state)
        .with_render(vn_runtime::RenderSnapshot {
            background: Some("backgrounds/saved.png".to_string()),
            characters: Vec::new(),
        })
        .with_history(vn_runtime::History::new());

    inner.restore_from_save(save_data).unwrap();

    assert_eq!(inner.waiting, WaitingFor::Click);
    assert!(matches!(
        inner
            .runtime
            .as_ref()
            .expect("runtime should be restored")
            .waiting(),
        WaitingReason::WaitForClick
    ));
    assert!(inner.render_state.active_ui_mode.is_none());

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn cutscene_ducks_and_finish_restores_bgm_volume() {
    let (mut inner, root) = make_state_with_services("scripts/cutscene.md", "");

    inner
        .services_mut()
        .audio
        .play_bgm("audio/theme.ogg", true, None);
    inner.sync_audio(0.0);
    let base_volume = inner
        .render_state
        .audio
        .bgm
        .as_ref()
        .expect("bgm should exist before cutscene")
        .volume;

    inner.apply_runtime_tick_output(
        vec![Command::Cutscene {
            path: "video/opening.webm".to_string(),
        }],
        WaitingReason::signal("cutscene"),
    );
    inner.sync_audio(0.5);

    let ducked_volume = inner
        .render_state
        .audio
        .bgm
        .as_ref()
        .expect("bgm should still exist during cutscene")
        .volume;
    assert!(inner.render_state.cutscene.is_some());
    assert!(ducked_volume < base_volume);

    inner.finish_cutscene();
    inner.sync_audio(0.5);

    let restored_volume = inner
        .render_state
        .audio
        .bgm
        .as_ref()
        .expect("bgm should remain after cutscene")
        .volume;
    assert!(inner.render_state.cutscene.is_none());
    assert!((restored_volume - base_volume).abs() < f32::EPSILON);

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn handle_ui_result_applies_follow_up_dialogue_without_skipping() {
    let script = r#"
："第二部分：测试地图语法糖 showMap。"
showMap "demo_world" as $destination

if $destination == "town"
  ："你通过 showMap 选择了小镇。"
else
  ："你通过 showMap 选择了其他地点。"
endif

："第三部分：直接测试底层 requestUI。"
"#;
    let (mut inner, root) = make_state_with_services("scripts/ui.md", script);

    inner.init_game_from_resource("scripts/ui.md").unwrap();
    inner.render_state.complete_typewriter();
    inner.process_click();
    inner.run_script_tick();

    let active_mode = inner
        .render_state
        .active_ui_mode
        .as_ref()
        .expect("showMap should activate a UI mode");
    assert_eq!(active_mode.mode, "show_map");
    assert_eq!(
        inner.waiting,
        WaitingFor::UIResult {
            key: "show_map".to_string()
        }
    );

    inner
        .handle_ui_result(
            "show_map".to_string(),
            serde_json::Value::String("town".to_string()),
        )
        .unwrap();

    assert!(inner.render_state.active_ui_mode.is_none());
    assert_eq!(inner.waiting, WaitingFor::Click);
    assert_eq!(
        inner
            .render_state
            .dialogue
            .as_ref()
            .expect("follow-up dialogue should be rendered")
            .content,
        "你通过 showMap 选择了小镇。"
    );

    std::fs::remove_dir_all(root).ok();
}

#[test]
fn script_end_returns_to_title_screen() {
    let script = "：\"脚本结束测试。\"\n";
    let (mut inner, root) = make_state_with_services("scripts/end.md", script);

    inner.init_game_from_resource("scripts/end.md").unwrap();
    inner.render_state.complete_typewriter();
    inner.process_click();
    inner.process_tick(0.0);

    assert_eq!(inner.host_screen, HostScreen::Title);
    assert_eq!(inner.render_state.host_screen, HostScreen::Title);
    assert!(inner.runtime.is_none());
    assert!(inner.render_state.dialogue.is_none());

    std::fs::remove_dir_all(root).ok();
}
