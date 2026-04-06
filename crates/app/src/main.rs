use std::path::PathBuf;

use all_bots::WizardState;
use dashboard::{DashboardConfig, gui::DashboardPanels, new_dashboard};
use eframe::egui;

fn main() {
    tracing_subscriber::fmt::init();

    // Set up home directories
    let home_root = PathBuf::from(".local/home");
    let custodex_dir = home_root.join(".custodex");
    std::fs::create_dir_all(&custodex_dir).expect("failed to create .custodex dir");

    let token_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".local/telegram_bot_token".to_string());
    let token = std::fs::read_to_string(&token_path)
        .unwrap_or_else(|e| panic!("failed to read token from {token_path}: {e}"))
        .trim()
        .to_string();

    let sandbox_exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../codex/codex-rs/target/release/codex-linux-sandbox");

    let work_dir = home_root.join(code_agent::TEMPLATE_NAME);
    std::fs::create_dir_all(&work_dir).expect("failed to create working dir");

    let dashboard = new_dashboard(DashboardConfig {
        token_path: token_path.clone(),
        sandbox_exe: sandbox_exe.display().to_string(),
        model: String::new(),
    });

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    let bot_launcher = code_agent::make_launcher(
        token,
        dashboard.clone(),
        work_dir,
        custodex_dir,
        Some(sandbox_exe),
    );

    let panels = DashboardPanels::new(rt, dashboard, bot_launcher);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "custodex",
        options,
        Box::new(move |_cc| Ok(Box::new(CustodexApp::new(panels)) as Box<dyn eframe::App>)),
    )
    .expect("eframe failed");
}

struct CustodexApp {
    panels: DashboardPanels,
    wizard: WizardState,
}

impl CustodexApp {
    fn new(panels: DashboardPanels) -> Self {
        Self {
            panels,
            wizard: WizardState::default(),
        }
    }
}

impl eframe::App for CustodexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        // Status bar
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            self.panels.render_status_bar(ui);
        });

        // Left: instances
        egui::SidePanel::left("instances_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                let new_clicked = self.panels.render_instances_panel(ui);
                if new_clicked {
                    self.wizard.open();
                }
            });

        // Right: config
        egui::SidePanel::right("config_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                self.panels.render_config_panel(ui);
            });

        // Center: event log
        egui::CentralPanel::default().show(ctx, |ui| {
            self.panels.render_event_log(ui);
        });

        // Wizard overlay
        if let Some(_config) = self.wizard.render(ctx) {
            // TODO: use config to launch a new instance
            let mut d = self.panels.dashboard.lock().unwrap();
            d.push_log(None, "New instance created from wizard.".to_string());
        }
    }
}
