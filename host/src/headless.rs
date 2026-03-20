//! Headless 测试模式
//!
//! 无窗口、无 GPU 运行环境，用于 AI 自动调试管线的快进回放。
//! 通过 `--headless --replay-input=<path>` 启动。

use std::path::PathBuf;
use std::time::Instant;

use tracing::info;

use crate::AppConfig;
use crate::AppMode;
use crate::app::{self, AppInit, AppState};
use crate::input::recording::InputReplayer;
use crate::rendering_types::{NullTextureFactory, TextureContext};

/// Headless 模式错误
#[derive(Debug)]
pub enum HeadlessError {
    ReplayLoad(crate::input::recording::ReplayLoadError),
    EventStream(std::io::Error),
    Validation(String),
}

impl std::fmt::Display for HeadlessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReplayLoad(e) => write!(f, "Replay load error: {e}"),
            Self::EventStream(e) => write!(f, "Event stream error: {e}"),
            Self::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl std::error::Error for HeadlessError {}

impl From<crate::input::recording::ReplayLoadError> for HeadlessError {
    fn from(e: crate::input::recording::ReplayLoadError) -> Self {
        Self::ReplayLoad(e)
    }
}

/// Headless CLI 参数（从 main.rs 的 Cli 传入的子集）
pub struct HeadlessCli {
    pub replay_input: PathBuf,
    pub event_stream: Option<PathBuf>,
    pub exit_on: String,
    pub max_frames: Option<u64>,
    pub timeout_sec: Option<u64>,
}

/// 运行 headless 模式
pub fn run(config: AppConfig, cli: &crate::headless::HeadlessCli) -> Result<(), HeadlessError> {
    let es_path = cli
        .event_stream
        .clone()
        .unwrap_or_else(|| PathBuf::from("events.jsonl"));

    let mut app_state = AppState::new(
        config.clone(),
        AppInit {
            headless: true,
            event_stream_path: Some(es_path),
        },
    );

    app_state
        .core
        .resource_manager
        .set_texture_context(TextureContext::new(std::sync::Arc::new(NullTextureFactory)));

    app_state
        .core
        .renderer
        .set_screen_size(config.window.width as f32, config.window.height as f32);

    let mut replayer = InputReplayer::load(&cli.replay_input)?;

    if replayer.meta().logical_width != config.window.width
        || replayer.meta().logical_height != config.window.height
    {
        tracing::warn!(
            recording_size = format!(
                "{}x{}",
                replayer.meta().logical_width,
                replayer.meta().logical_height
            ),
            config_size = format!("{}x{}", config.window.width, config.window.height),
            "录制分辨率与配置不一致"
        );
    }

    app::load_resources(&mut app_state);
    let start_path = app_state.config.start_script_path.clone();
    if app::load_script_from_logical_path(&mut app_state, &start_path) {
        info!(path = %start_path, "Start script loaded (headless)");
        app::run_script_tick(&mut app_state, None);
    }

    app_state.ui.navigation.switch_to(AppMode::InGame);

    headless_loop(&mut app_state, &mut replayer, cli)?;

    app_state.event_stream.flush();
    info!("Headless 运行完成");
    Ok(())
}

fn headless_loop(
    app_state: &mut AppState,
    replayer: &mut InputReplayer,
    cli: &HeadlessCli,
) -> Result<(), HeadlessError> {
    let fixed_dt: f32 = 1.0 / 60.0;
    let dt_ms: u64 = 16;
    let mut elapsed_ms: u64 = 0;
    let mut frame_count: u64 = 0;
    let wall_start = Instant::now();

    loop {
        app_state.event_stream.set_logical_time_ms(elapsed_ms);
        let events = replayer.drain_until(elapsed_ms);
        app_state.input_manager.inject_replay_events(&events);

        app_state.input_manager.begin_frame(fixed_dt);
        app::update(app_state, fixed_dt);
        app_state.input_manager.end_frame();

        elapsed_ms += dt_ms;
        frame_count += 1;

        if should_exit(app_state, replayer, cli, frame_count, &wall_start) {
            break;
        }
    }
    Ok(())
}

fn should_exit(
    app_state: &AppState,
    replayer: &InputReplayer,
    cli: &HeadlessCli,
    frame_count: u64,
    wall_start: &Instant,
) -> bool {
    if let Some(max) = cli.max_frames
        && frame_count >= max
    {
        info!(frames = frame_count, "已达最大帧数限制");
        return true;
    }

    if let Some(timeout) = cli.timeout_sec
        && wall_start.elapsed().as_secs() >= timeout
    {
        info!(timeout_sec = timeout, "已达超时限制");
        return true;
    }

    match cli.exit_on.as_str() {
        "replay-end" => {
            if replayer.is_exhausted() {
                info!("回放数据已耗尽");
                return true;
            }
        }
        "script-finished" => {
            if app_state.session.script_finished {
                info!("脚本执行完毕");
                return true;
            }
        }
        _ => {}
    }

    false
}
