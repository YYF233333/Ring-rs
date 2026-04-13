use std::fs;
use std::path::PathBuf;

use crate::init::initialize_inner;
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
    crate::init::find_project_root()
        .join("artifacts")
        .join("host-dioxus")
        .join("harness-cli-bundle.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Keys chosen to be unique and clearly test-only; unlikely to be set in any real env.
    const KEY_F32_ABSENT: &str = "RING_TEST_HC_F32_ABSENT_7A2B";
    const KEY_F32_VALID: &str = "RING_TEST_HC_F32_VALID_7A2B";
    const KEY_F32_INVALID: &str = "RING_TEST_HC_F32_INVALID_7A2B";
    const KEY_USIZE_ABSENT: &str = "RING_TEST_HC_USIZE_ABSENT_7A2B";
    const KEY_USIZE_VALID: &str = "RING_TEST_HC_USIZE_VALID_7A2B";
    const KEY_BOOL_ABSENT: &str = "RING_TEST_HC_BOOL_ABSENT_7A2B";
    const KEY_BOOL_TRUTHY: &str = "RING_TEST_HC_BOOL_TRUTHY_7A2B";
    const KEY_BOOL_FALSY: &str = "RING_TEST_HC_BOOL_FALSY_7A2B";

    #[test]
    fn parse_env_f32_returns_default_when_absent() {
        let result = parse_env_f32(KEY_F32_ABSENT, 42.0);
        assert_eq!(result.unwrap(), 42.0);
    }

    #[test]
    fn parse_env_f32_parses_valid_value() {
        unsafe {
            std::env::set_var(KEY_F32_VALID, "3.14");
        }
        let result = parse_env_f32(KEY_F32_VALID, 0.0);
        unsafe {
            std::env::remove_var(KEY_F32_VALID);
        }
        assert!((result.unwrap() - 3.14_f32).abs() < 1e-5);
    }

    #[test]
    fn parse_env_f32_errors_on_invalid_value() {
        unsafe {
            std::env::set_var(KEY_F32_INVALID, "not_a_number");
        }
        let result = parse_env_f32(KEY_F32_INVALID, 0.0);
        unsafe {
            std::env::remove_var(KEY_F32_INVALID);
        }
        assert!(result.is_err());
    }

    #[test]
    fn parse_env_usize_returns_default_when_absent() {
        let result = parse_env_usize(KEY_USIZE_ABSENT, 100);
        assert_eq!(result.unwrap(), 100);
    }

    #[test]
    fn parse_env_usize_parses_valid_value() {
        unsafe {
            std::env::set_var(KEY_USIZE_VALID, "256");
        }
        let result = parse_env_usize(KEY_USIZE_VALID, 0);
        unsafe {
            std::env::remove_var(KEY_USIZE_VALID);
        }
        assert_eq!(result.unwrap(), 256);
    }

    #[test]
    fn parse_env_bool_returns_default_when_absent() {
        assert!(!parse_env_bool(KEY_BOOL_ABSENT, false));
        assert!(parse_env_bool(KEY_BOOL_ABSENT, true));
    }

    #[test]
    fn parse_env_bool_recognizes_truthy_values() {
        for truthy in &["1", "true", "TRUE", "yes", "YES"] {
            unsafe {
                std::env::set_var(KEY_BOOL_TRUTHY, truthy);
            }
            let result = parse_env_bool(KEY_BOOL_TRUTHY, false);
            unsafe {
                std::env::remove_var(KEY_BOOL_TRUTHY);
            }
            assert!(result, "expected truthy for '{truthy}'");
        }
    }

    #[test]
    fn parse_env_bool_treats_other_values_as_false() {
        for falsy in &["0", "false", "FALSE", "no", "NO", "off"] {
            unsafe {
                std::env::set_var(KEY_BOOL_FALSY, falsy);
            }
            let result = parse_env_bool(KEY_BOOL_FALSY, true);
            unsafe {
                std::env::remove_var(KEY_BOOL_FALSY);
            }
            assert!(!result, "expected falsy for '{falsy}'");
        }
    }
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

    tracing::info!(
        owner = %session.client_token,
        stop_reason = %bundle.metadata.stop_reason,
        steps = bundle.metadata.steps_run,
        output = %output_path.display(),
        "Headless harness complete"
    );

    Ok(())
}
