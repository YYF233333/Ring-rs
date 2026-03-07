//! # 主题加载器

use std::fs;
use std::path::Path;

use macroquad::prelude::Color;
use serde::Deserialize;
use tracing::warn;

use super::Theme;

#[derive(Debug, Deserialize, Default)]
pub struct UiThemeOverride {
    pub palette: Option<PaletteOverride>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PaletteOverride {
    pub bg_primary: Option<[f32; 4]>,
    pub bg_secondary: Option<[f32; 4]>,
    pub bg_panel: Option<[f32; 4]>,
    pub text_primary: Option<[f32; 4]>,
    pub text_secondary: Option<[f32; 4]>,
    pub accent: Option<[f32; 4]>,
}

fn color_from_rgba(rgba: [f32; 4]) -> Color {
    Color::new(rgba[0], rgba[1], rgba[2], rgba[3])
}

/// 从 json 覆盖主题；失败时记录诊断并返回默认主题
pub fn load_theme_with_override(default_theme: Theme, path: &Path) -> Theme {
    if !path.exists() {
        warn!(path = ?path, "UI 主题覆盖文件不存在，使用默认主题");
        return default_theme;
    }

    let content = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            warn!(path = ?path, error = %e, "读取 UI 主题覆盖文件失败，使用默认主题");
            return default_theme;
        }
    };

    let override_cfg: UiThemeOverride = match serde_json::from_str(&content) {
        Ok(cfg) => cfg,
        Err(e) => {
            warn!(path = ?path, error = %e, "解析 UI 主题覆盖文件失败，使用默认主题");
            return default_theme;
        }
    };

    let mut theme = default_theme;
    if let Some(p) = override_cfg.palette {
        if let Some(c) = p.bg_primary {
            theme.tokens.palette.bg_primary = color_from_rgba(c);
        }
        if let Some(c) = p.bg_secondary {
            theme.tokens.palette.bg_secondary = color_from_rgba(c);
        }
        if let Some(c) = p.bg_panel {
            theme.tokens.palette.bg_panel = color_from_rgba(c);
        }
        if let Some(c) = p.text_primary {
            theme.tokens.palette.text_primary = color_from_rgba(c);
        }
        if let Some(c) = p.text_secondary {
            theme.tokens.palette.text_secondary = color_from_rgba(c);
        }
        if let Some(c) = p.accent {
            theme.tokens.palette.accent = color_from_rgba(c);
        }
    }

    theme.sync_legacy_fields();
    theme
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_file(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}.json"))
    }

    #[test]
    fn missing_override_file_should_fallback_to_default() {
        let default_theme = Theme::dark();
        let missing = unique_temp_file("ring-ui-theme-missing");
        let loaded = load_theme_with_override(default_theme.clone(), &missing);
        assert!((loaded.bg_primary.r - default_theme.bg_primary.r).abs() < 0.0001);
    }

    #[test]
    fn override_should_patch_palette_fields() {
        let default_theme = Theme::dark();
        let path = unique_temp_file("ring-ui-theme-ok");
        std::fs::write(
            &path,
            r#"{"palette":{"accent":[1.0,0.0,0.0,1.0],"text_secondary":[0.1,0.2,0.3,1.0]}}"#,
        )
        .expect("write temp theme override");

        let loaded = load_theme_with_override(default_theme, &path);
        assert!((loaded.accent.r - 1.0).abs() < 0.0001);
        assert!((loaded.accent.g - 0.0).abs() < 0.0001);
        assert!((loaded.text_secondary.g - 0.2).abs() < 0.0001);

        let _ = std::fs::remove_file(path);
    }
}
