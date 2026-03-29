use crate::render_state::{HostScreen, PlaybackMode};

pub fn parse_host_screen(screen: &str) -> Result<HostScreen, String> {
    match screen {
        "title" => Ok(HostScreen::Title),
        "ingame" => Ok(HostScreen::InGame),
        "ingame_menu" => Ok(HostScreen::InGameMenu),
        "save" => Ok(HostScreen::Save),
        "load" => Ok(HostScreen::Load),
        "settings" => Ok(HostScreen::Settings),
        "history" => Ok(HostScreen::History),
        other => Err(format!("未知 host_screen: {other}")),
    }
}

pub fn parse_playback_mode(mode: &str) -> Result<PlaybackMode, String> {
    match mode {
        "normal" => Ok(PlaybackMode::Normal),
        "auto" => Ok(PlaybackMode::Auto),
        "skip" => Ok(PlaybackMode::Skip),
        other => Err(format!("未知 playback_mode: {other}")),
    }
}
