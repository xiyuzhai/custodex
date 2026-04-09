use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use all_bots::WizardState;
use custodex::state::AppState;
use dashboard::{DashboardConfig, new_dashboard};
use eframe::egui;
use tracing_subscriber::prelude::*;
use tracing_subscriber::filter::LevelFilter;

fn init_logging(custodex_dir: &std::path::Path) {
    let system_log_path = custodex_dir.join("system.log");
    let _ = std::fs::create_dir_all(custodex_dir);
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(system_log_path)
        .expect("failed to open system log");

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .with_filter(LevelFilter::WARN);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(move || file.try_clone().expect("failed to clone system log handle"));

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(file_layer)
        .init();
}

fn main() {
    let (rt, dashboard, custodex_dir, token, sandbox_exe) = setup();
    init_logging(&custodex_dir);
    let app_state = AppState::new(rt, dashboard, custodex_dir, token, sandbox_exe);

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
    PathBuf,
    String,
    Option<PathBuf>,
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

    let dashboard = new_dashboard(DashboardConfig {
        token_path: token_path.clone(),
        sandbox_exe: sandbox_exe.display().to_string(),
        model: String::new(),
        dashboard_log_path: Some(custodex_dir.join("dashboard.log")),
    });

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    (rt, dashboard, custodex_dir, token, Some(sandbox_exe))
}

struct CustodexApp {
    state: AppState,
    wizard: WizardState,
    selected_instance: Option<usize>,
    active_instance: Option<usize>,
    shutdown_requested: Arc<AtomicBool>,
}

impl CustodexApp {
    fn new(state: AppState) -> Self {
        let shutdown_requested = Arc::new(AtomicBool::new(false));
        let signal_flag = Arc::clone(&shutdown_requested);
        state.rt.spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                signal_flag.store(true, Ordering::SeqCst);
            }
        });

        let selected_instance = (!state.saved_instances.is_empty()).then_some(0);
        Self {
            state,
            wizard: WizardState::default(),
            selected_instance,
            active_instance: None,
            shutdown_requested,
        }
    }

    fn start_instance(&mut self, idx: usize) {
        if self.state.is_bot_running() {
            self.state.stop_bot();
        }
        self.state.start_bot_for_instance(Some(idx));
        self.active_instance = Some(idx);
    }

    fn stop_all(&mut self) {
        self.state.stop_bot();
        self.active_instance = None;
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        let status = self.state.get_service_status();
        let (input_tokens, output_tokens) = self.state.get_token_usage();

        ui.horizontal(|ui| {
            let (dot, text, color) = match &status {
                dashboard::ServiceStatus::Stopped => ("○", "Stopped", egui::Color32::GRAY),
                dashboard::ServiceStatus::Starting => ("◐", "Starting", egui::Color32::YELLOW),
                dashboard::ServiceStatus::Running => ("●", "In Progress", egui::Color32::from_rgb(166, 227, 161)),
                dashboard::ServiceStatus::Error(_) => ("●", "Error", egui::Color32::from_rgb(243, 139, 168)),
            };
            ui.label(egui::RichText::new(dot).color(color));
            ui.label(egui::RichText::new(text).color(color));

            ui.separator();

            match &status {
                dashboard::ServiceStatus::Stopped | dashboard::ServiceStatus::Error(_) => {
                    if ui.button("Start Active").clicked() {
                        if let Some(idx) = self.active_instance.or(self.selected_instance) {
                            self.start_instance(idx);
                        }
                    }
                }
                _ => {
                    if ui.button("Stop All").clicked() {
                        self.stop_all();
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{input_tokens} in / {output_tokens} out"))
                        .color(egui::Color32::from_rgb(166, 173, 200)),
                );
                if let Some(idx) = self
                    .active_instance
                    .or(self.selected_instance)
                    .and_then(|idx| self.state.saved_instances.get(idx))
                {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!("Active: {}", idx.id))
                            .color(egui::Color32::from_rgb(166, 173, 200)),
                    );
                }
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
                let is_active = self.active_instance == Some(idx);

                let resp = ui.group(|ui| {
                    ui.horizontal(|ui| {
                        let dot_color = if is_active {
                            egui::Color32::from_rgb(166, 227, 161)
                        } else if is_selected {
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
                    if is_active {
                        ui.label(
                            egui::RichText::new("In Progress/Active")
                                .color(egui::Color32::from_rgb(166, 227, 161))
                                .small(),
                        );
                    }
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

        let Some(inst) = self.state.saved_instances.get(idx).cloned() else {
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
            ui.label("State:");
            let text = if self.active_instance == Some(idx) && self.state.is_bot_running() {
                egui::RichText::new("In Progress").color(egui::Color32::from_rgb(166, 227, 161))
            } else if self.active_instance == Some(idx) {
                egui::RichText::new("Active").color(egui::Color32::from_rgb(137, 180, 250))
            } else {
                egui::RichText::new("Inactive").color(egui::Color32::GRAY)
            };
            ui.label(text);
        });
        ui.horizontal(|ui| {
            ui.label("Description:");
            ui.label(inst.template.description());
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Set Active").clicked() {
                self.active_instance = Some(idx);
            }

            let can_start = !self.state.is_bot_running() || self.active_instance != Some(idx);
            ui.add_enabled_ui(can_start, |ui| {
                if ui.button("Start This Instance").clicked() {
                    self.start_instance(idx);
                }
            });

            let can_stop = self.state.is_bot_running() && self.active_instance == Some(idx);
            ui.add_enabled_ui(can_stop, |ui| {
                if ui.button("Stop This Instance").clicked() {
                    self.stop_all();
                }
            });
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
        if self.shutdown_requested.swap(false, Ordering::SeqCst) {
            self.stop_all();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

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
            let idx = self.state.saved_instances.len().saturating_sub(1);
            self.selected_instance = Some(idx);
            self.active_instance = Some(idx);
        }
    }
}
