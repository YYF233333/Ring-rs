//! Visual Novel Engine - Host (Bevy 前端)
//!
//! 负责渲染、音频、输入处理，驱动 vn-runtime 执行脚本。

use bevy::prelude::*;
use std::fs;

mod plugin;
mod components;
mod systems;
mod resources;

use plugin::VNPlugin;
use resources::VNState;
use vn_runtime::{Parser, VNRuntime};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Visual Novel Engine".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VNPlugin)
        .add_systems(Startup, load_script_system.after(systems::setup_system))
        .run();
}

/// 加载脚本系统
fn load_script_system(mut vn_state: ResMut<VNState>) {
    // 尝试加载脚本
    let script_path = "host/assets/scripts/demo.md";
    
    match fs::read_to_string(script_path) {
        Ok(content) => {
            info!("Loading script from: {}", script_path);
            
            let mut parser = Parser::new();
            match parser.parse("demo", &content) {
                Ok(script) => {
                    info!("Script loaded successfully: {} nodes", script.nodes.len());
                    let runtime = VNRuntime::new(script);
                    vn_state.runtime = Some(runtime);
                }
                Err(e) => {
                    error!("Failed to parse script: {:?}", e);
                }
            }
        }
        Err(e) => {
            warn!("Could not load script file '{}': {}", script_path, e);
            info!("Running without script. Create 'assets/scripts/demo.md' to test.");
        }
    }
}
