#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--headless-harness") {
        if let Err(error) = host_tauri_lib::run_headless_harness_from_env() {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return;
    }
    host_tauri_lib::run();
}
