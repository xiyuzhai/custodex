use std::path::PathBuf;

use dashboard::{DashboardConfig, new_dashboard};

fn main() {
    tracing_subscriber::fmt::init();

    // Set up home directories
    let home_root = PathBuf::from(".local/home");
    let custodex_dir = home_root.join(".custodex");
    let work_dir = home_root.join(code_agent::TEMPLATE_NAME);

    std::fs::create_dir_all(&custodex_dir).expect("failed to create .custodex dir");
    std::fs::create_dir_all(&work_dir).expect("failed to create working dir");

    let token_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".local/telegram_bot_token".to_string());
    let token = std::fs::read_to_string(&token_path)
        .unwrap_or_else(|e| panic!("failed to read token from {token_path}: {e}"))
        .trim()
        .to_string();

    let sandbox_exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../codex/codex-rs/target/release/codex-linux-sandbox");

    let dashboard = new_dashboard(DashboardConfig {
        token_path: token_path.clone(),
        sandbox_exe: sandbox_exe.display().to_string(),
        model: String::new(),
    });

    let bot_launcher = code_agent::make_launcher(
        token,
        dashboard.clone(),
        work_dir,
        custodex_dir,
        Some(sandbox_exe),
    );

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "custodex",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(dashboard::gui::ControlPanel::new(
                rt,
                dashboard,
                bot_launcher,
            )) as Box<dyn eframe::App>)
        }),
    )
    .expect("eframe failed");
}
