mod adapter;
mod bot;
mod gui;
mod monitor;
mod session;

use monitor::{MonitorConfig, new_shared_monitor};

fn main() {
    tracing_subscriber::fmt::init();

    let token_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".local/telegram_bot_token".to_string());
    let token = std::fs::read_to_string(&token_path)
        .unwrap_or_else(|e| panic!("failed to read token from {token_path}: {e}"))
        .trim()
        .to_string();

    let sandbox_exe = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../codex/codex-rs/target/release/codex-linux-sandbox");

    let monitor = new_shared_monitor(MonitorConfig {
        token_path: token_path.clone(),
        sandbox_exe: sandbox_exe.display().to_string(),
        model: String::new(), // will be filled from config later
    });

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "codex-telegram",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(gui::ControlPanel::new(rt, token, monitor)) as Box<dyn eframe::App>)
        }),
    )
    .expect("eframe failed");
}
