use std::path::PathBuf;

use all_bots::{SavedInstance, TemplateConfig, TemplateKind, WizardState, save_instances, load_instances};
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
        custodex_dir.clone(),
        Some(sandbox_exe),
    );

    let panels = DashboardPanels::new(rt, dashboard, bot_launcher);

    // Load saved instances
    let saved = load_instances(&custodex_dir);

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "custodex",
        options,
        Box::new(move |_cc| {
            Ok(Box::new(CustodexApp::new(panels, saved, custodex_dir)) as Box<dyn eframe::App>)
        }),
    )
    .expect("eframe failed");
}

struct CustodexApp {
    panels: DashboardPanels,
    wizard: WizardState,
    saved_instances: Vec<SavedInstance>,
    custodex_dir: PathBuf,
}

impl CustodexApp {
    fn new(panels: DashboardPanels, saved_instances: Vec<SavedInstance>, custodex_dir: PathBuf) -> Self {
        if !saved_instances.is_empty() {
            let mut d = panels.dashboard.lock().unwrap();
            d.push_log(
                None,
                format!("Restored {} saved instance(s).", saved_instances.len()),
            );
        }
        Self {
            panels,
            wizard: WizardState::default(),
            saved_instances,
            custodex_dir,
        }
    }

    fn add_instance(&mut self, config: TemplateConfig) {
        let template = match &config {
            TemplateConfig::CodeAgent(_) => TemplateKind::CodeAgent,
        };
        let id = format!("{}-{}", template.name(), self.saved_instances.len());
        let saved = SavedInstance {
            id: id.clone(),
            template,
            config,
        };
        self.saved_instances.push(saved);

        // Persist
        if let Err(e) = save_instances(&self.custodex_dir, &self.saved_instances) {
            let mut d = self.panels.dashboard.lock().unwrap();
            d.push_log(None, format!("Failed to save instances: {e}"));
        } else {
            let mut d = self.panels.dashboard.lock().unwrap();
            d.push_log(None, format!("Instance {id} created and saved."));
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

                // Show saved instances
                if !self.saved_instances.is_empty() {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.label(egui::RichText::new("Saved").strong());
                    for inst in &self.saved_instances {
                        ui.group(|ui| {
                            ui.label(&inst.id);
                            ui.label(inst.template.name());
                        });
                    }
                }

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
        if let Some(config) = self.wizard.render(ctx) {
            self.add_instance(config);
        }
    }
}
