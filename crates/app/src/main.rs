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

    let sandbox_exe = codex_sandbox::sandbox_exe();

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
    selected_instance: Option<usize>,
}

impl CustodexApp {
    fn new(state: AppState) -> Self {
        Self {
            state,
            wizard: WizardState::default(),
            selected_instance: None,
        }
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        let status = self.state.get_service_status();
        let (input_tokens, output_tokens) = self.state.get_token_usage();

        ui.horizontal(|ui| {
            let (dot, text, color) = match &status {
                dashboard::ServiceStatus::Stopped => ("○", "Stopped", egui::Color32::GRAY),
                dashboard::ServiceStatus::Starting => ("◐", "Starting", egui::Color32::YELLOW),
                dashboard::ServiceStatus::Running => ("●", "Running", egui::Color32::from_rgb(166, 227, 161)),
                dashboard::ServiceStatus::Error(_) => ("●", "Error", egui::Color32::from_rgb(243, 139, 168)),
            };
            ui.label(egui::RichText::new(dot).color(color));
            ui.label(egui::RichText::new(text).color(color));

            ui.separator();

            match &status {
                dashboard::ServiceStatus::Stopped | dashboard::ServiceStatus::Error(_) => {
                    if ui.button("Start").clicked() {
                        self.state.start_bot();
                    }
                }
                _ => {
                    if ui.button("Stop").clicked() {
                        self.state.stop_bot();
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{input_tokens} in / {output_tokens} out"))
                        .color(egui::Color32::from_rgb(166, 173, 200)),
                );
            });
        });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        if ui
            .add(egui::Button::new("+ New Instance").min_size(egui::vec2(ui.available_width(), 0.0)))
            .clicked()
        {
            self.wizard.open();
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut clicked_idx = None;
            for (idx, inst) in self.state.saved_instances.iter().enumerate() {
                let is_selected = self.selected_instance == Some(idx);

                let resp = ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let dot_color = if is_selected {
                            egui::Color32::from_rgb(137, 180, 250)
                        } else {
                            egui::Color32::GRAY
                        };
                        ui.label(egui::RichText::new("●").color(dot_color));
                        ui.label(egui::RichText::new(inst.template.name()).strong());
                    });
                    ui.label(
                        egui::RichText::new(&inst.id)
                            .color(egui::Color32::from_rgb(166, 173, 200))
                            .small(),
                    );
                });

                if resp.response.interact(egui::Sense::click()).clicked() {
                    clicked_idx = Some(idx);
                }
            }

            if let Some(idx) = clicked_idx {
                if self.selected_instance == Some(idx) {
                    self.selected_instance = None;
                } else {
                    self.selected_instance = Some(idx);
                    self.state.log_filter_text.clear();
                }
            }
        });
    }

    fn render_detail_view(&mut self, ui: &mut egui::Ui) {
        let Some(idx) = self.selected_instance else {
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new("Select an instance to view details")
                        .color(egui::Color32::from_rgb(166, 173, 200)),
                );
            });
            return;
        };

        let Some(inst) = self.state.saved_instances.get(idx) else {
            self.selected_instance = None;
            return;
        };

        // Instance header
        ui.heading(&inst.id);
        ui.horizontal(|ui| {
            ui.label("Template:");
            ui.label(egui::RichText::new(inst.template.name()).strong());
        });
        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.label(inst.template.description());
        });

        // Show template-specific config
        match &inst.config {
            all_bots::TemplateConfig::CodeAgent(cfg) => {
                ui.horizontal(|ui| {
                    ui.label("Work dir:");
                    ui.label(&cfg.work_dir);
                });
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.label(if cfg.model.is_empty() { "(default)" } else { &cfg.model });
                });
                ui.horizontal(|ui| {
                    ui.label("Auto-approve:");
                    ui.label(if cfg.auto_approve { "yes" } else { "no" });
                });
            }
            all_bots::TemplateConfig::SimpleChat(cfg) => {
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.label(if cfg.model.is_empty() { "(default)" } else { &cfg.model });
                });
            }
            all_bots::TemplateConfig::AutoApprove(cfg) => {
                ui.horizontal(|ui| {
                    ui.label("Work dir:");
                    ui.label(&cfg.work_dir);
                });
                ui.horizontal(|ui| {
                    ui.label("Model:");
                    ui.label(if cfg.model.is_empty() { "(default)" } else { &cfg.model });
                });
                ui.horizontal(|ui| {
                    ui.label("Auto-approve:");
                    ui.label(
                        egui::RichText::new("ALL commands")
                            .color(egui::Color32::from_rgb(250, 179, 135)),
                    );
                });
            }
        }

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Event log for this instance
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Event Log").strong());
            ui.separator();
            ui.label("Filter:");
            ui.add(
                egui::TextEdit::singleline(&mut self.state.log_filter_text).desired_width(120.0),
            );
            if ui.button("Clear").clicked() {
                let mut d = self.state.dashboard.lock().unwrap();
                d.log.clear();
            }
        });
        ui.add_space(4.0);

        let entries = self.state.get_log_entries();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for (chat_id, msg) in &entries {
                    let chat_str = chat_id.map(|id| format!("[{id}]")).unwrap_or_default();
                    ui.label(
                        egui::RichText::new(format!("{chat_str:>14} {msg}")).monospace(),
                    );
                }
            });
    }
}

impl eframe::App for CustodexApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        // Status bar
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            self.render_status_bar(ui);
        });

        // Left sidebar: instances
        egui::SidePanel::left("sidebar")
            .default_width(180.0)
            .min_width(140.0)
            .show(ctx, |ui| {
                self.render_sidebar(ui);
            });

        // Right: detail view of selected instance
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_detail_view(ui);
        });

        // Wizard overlay
        if let Some(config) = self.wizard.render(ctx) {
            self.state.add_instance(config);
        }
    }
}
