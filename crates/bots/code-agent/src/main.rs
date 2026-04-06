mod bot;

use dashboard::{DashboardConfig, new_dashboard};

fn main() {
    tracing_subscriber::fmt::init();

    let token_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".local/telegram_bot_token".to_string());
    let token = std::fs::read_to_string(&token_path)
        .unwrap_or_else(|e| panic!("failed to read token from {token_path}: {e}"))
        .trim()
        .to_string();

    let dashboard = new_dashboard(DashboardConfig {
        token_path: token_path.clone(),
        sandbox_exe: String::new(),
        model: String::new(),
    });

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let bot_launcher: dashboard::gui::BotLauncher =
        Box::new(|token, dash| Box::pin(bot::run_bot(token, dash)));

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "codex-telegram: code-agent",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(dashboard::gui::ControlPanel::new(
                rt,
                token,
                dashboard,
                bot_launcher,
            )) as Box<dyn eframe::App>)
        }),
    )
    .expect("eframe failed");
}
