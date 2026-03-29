use std::fs;
use std::path::PathBuf;

use crate::initialize_inner;
use crate::state::AppStateInner;

fn parse_env_f32(key: &str, default: f32) -> Result<f32, String> {
    match std::env::var(key) {
        Ok(value) => value
            .parse::<f32>()
            .map_err(|e| format!("{key} 解析失败: {e}")),
        Err(_) => Ok(default),
    }
}

fn parse_env_usize(key: &str, default: usize) -> Result<usize, String> {
    match std::env::var(key) {
        Ok(value) => value
            .parse::<usize>()
            .map_err(|e| format!("{key} 解析失败: {e}")),
        Err(_) => Ok(default),
    }
}

fn parse_env_bool(key: &str, default: bool) -> bool {
    match std::env::var(key) {
        Ok(value) => matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"),
        Err(_) => default,
    }
}

fn default_output_path() -> PathBuf {
    crate::find_project_root()
        .join("artifacts")
        .join("host-tauri")
        .join("harness-cli-bundle.json")
}

pub fn run_from_env() -> Result<(), Box<dyn std::error::Error>> {
    let dt = parse_env_f32("RING_HARNESS_DT", 1.0 / 60.0)?;
    let max_steps = parse_env_usize("RING_HARNESS_MAX_STEPS", 600)?;
    let stop_on_wait = parse_env_bool("RING_HARNESS_STOP_ON_WAIT", true);
    let stop_on_script_finished = parse_env_bool("RING_HARNESS_STOP_ON_SCRIPT_FINISHED", true);
    let output_path = std::env::var_os("RING_HARNESS_OUTPUT")
        .map(PathBuf::from)
        .unwrap_or_else(default_output_path);

    let mut inner = AppStateInner::new();
    initialize_inner(&mut inner)?;

    let session = inner.frontend_connected(Some("headless-cli".to_string()));
    let config = inner.services().config.clone();
    let script_path = std::env::var("RING_HARNESS_SCRIPT").unwrap_or(config.start_script_path);
    let start_label = std::env::var("RING_HARNESS_LABEL").ok();

    match start_label.as_deref().filter(|label| !label.is_empty()) {
        Some(label) => inner.init_game_from_resource_at_label(&script_path, label)?,
        None => inner.init_game_from_resource(&script_path)?,
    }

    let bundle = inner.debug_run_until(dt, max_steps, stop_on_wait, stop_on_script_finished);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&bundle)?)?;

    println!(
        "Headless harness complete: owner={}, stop_reason={}, steps={}, output={}",
        session.client_token,
        bundle.metadata.stop_reason,
        bundle.metadata.steps_run,
        output_path.display()
    );

    Ok(())
}
