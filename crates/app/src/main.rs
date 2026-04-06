use std::path::PathBuf;

use all_bots::WizardState;
use custodex::state::AppState;
use dashboard::{DashboardConfig, new_dashboard};
use eframe::egui;

fn main() {
    tracing_subscriber::fmt::init();

    let (rt, dashboard, bot_launcher, custodex_dir) = setup();
    let app_state = AppState::new(rt, dashboard, bot_launcher, custodex_dir);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "custodex",
        options,
        Box::new(move |_cc| Ok(Box::new(CustodexApp::new(app_state)) as Box<dyn eframe::App>)),
    )
    .expect("eframe failed");
}

fn setup() -> (
    tokio::runtime::Runtime,
    dashboard::SharedDashboard,
    dashboard::gui::BotLauncher,
    PathBuf,
) {
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
        custodex_dir.clone(),
        Some(sandbox_exe),
    );

    (rt, dashboard, bot_launcher, custodex_dir)
}

struct CustodexApp {
    state: AppState,
    wizard: WizardState,
}

impl CustodexApp {
    fn new(state: AppState) -> Self {
        Self {
            state,
            wizard: WizardState::default(),
        }
    }
}

impl eframe::App for CustodexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        let status = self.state.get_service_status();
        let (input_tokens, output_tokens) = self.state.get_token_usage();

        // Status bar
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (text, color) = match &status {
                    dashboard::ServiceStatus::Stopped => ("Stopped", egui::Color32::GRAY),
                    dashboard::ServiceStatus::Starting => ("Starting...", egui::Color32::YELLOW),
                    dashboard::ServiceStatus::Running => ("Running", egui::Color32::GREEN),
                    dashboard::ServiceStatus::Error(e) => {
                        ui.label(egui::RichText::new(format!("Error: {e}")).color(egui::Color32::RED));
                        ("Error", egui::Color32::RED)
                    }
                };
                ui.label(egui::RichText::new(format!("Service: {text}")).color(color));
                ui.separator();
                match &status {
                    dashboard::ServiceStatus::Stopped | dashboard::ServiceStatus::Error(_) => {
                        if ui.button("Start").clicked() { self.state.start_bot(); }
                    }
                    _ => {
                        if ui.button("Stop").clicked() { self.state.stop_bot(); }
                    }
                }
                ui.separator();
                ui.label(format!("Instances: {}", self.state.saved_instances.len()));
                ui.separator();
                ui.label(format!("Tokens: {} in / {} out", input_tokens, output_tokens));
            });
        });

        // Left: instances
        egui::SidePanel::left("instances_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Instances");
                ui.separator();

                if self.state.log_filter_chat.is_some() {
                    if ui.button("Show All").clicked() {
                        self.state.set_log_filter_chat(None);
                    }
                    ui.separator();
                }

                if self.state.saved_instances.is_empty() {
                    ui.label("No instances.");
                } else {
                    for inst in &self.state.saved_instances {
                        let resp = ui.group(|ui| {
                            ui.label(egui::RichText::new(&inst.id).strong());
                            ui.label(inst.template.name());
                        });
                        if resp.response.clicked() {
                            // TODO: filter by instance
                        }
                    }
                }

                ui.add_space(8.0);
                if ui.button("+ New Instance").clicked() {
                    self.wizard.open();
                }
            });

        // Right: config
        egui::SidePanel::right("config_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Configuration");
                ui.separator();
                let d = self.state.dashboard.lock().unwrap();
                ui.label(format!("Token: {}", d.config.token_path));
                ui.label(format!("Sandbox: {}", d.config.sandbox_exe));
                ui.label(format!("Model: {}", if d.config.model.is_empty() { "(default)" } else { &d.config.model }));
            });

        // Center: event log
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Event Log");
                ui.separator();
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.state.log_filter_text);
                if ui.button("Clear").clicked() {
                    let mut d = self.state.dashboard.lock().unwrap();
                    d.log.clear();
                }
            });
            ui.separator();

            let entries = self.state.get_log_entries();
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for (chat_id, msg) in &entries {
                        let chat_str = chat_id.map(|id| format!("[{id}]")).unwrap_or_default();
                        ui.label(egui::RichText::new(format!("{chat_str:>14} {msg}")).monospace());
                    }
                });
        });

        // Wizard overlay
        if let Some(config) = self.wizard.render(ctx) {
            self.state.add_instance(config);
        }
    }
}
