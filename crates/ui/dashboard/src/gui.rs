use std::future::Future;
use std::time::Instant;

use eframe::egui;

use crate::{ServiceStatus, SharedDashboard};

/// Function type for launching the bot. Caller captures all needed state in the closure.
pub type BotLauncher =
    Box<dyn Fn() -> std::pin::Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

/// Reusable dashboard panels. The app crate owns the eframe::App and calls these.
pub struct DashboardPanels {
    pub rt: tokio::runtime::Runtime,
    pub dashboard: SharedDashboard,
    pub bot_launcher: BotLauncher,
    pub bot_handle: Option<tokio::task::JoinHandle<()>>,
    pub startup_time: Instant,
    pub log_auto_scroll: bool,
}

impl DashboardPanels {
    pub fn new(
        rt: tokio::runtime::Runtime,
        dashboard: SharedDashboard,
        bot_launcher: BotLauncher,
    ) -> Self {
        Self {
            rt,
            dashboard,
            bot_launcher,
            bot_handle: None,
            startup_time: Instant::now(),
            log_auto_scroll: true,
        }
    }

    pub fn start_bot(&mut self) {
        if self.bot_handle.is_some() {
            return;
        }
        {
            let mut d = self.dashboard.lock().unwrap();
            d.service_status = ServiceStatus::Starting;
            d.push_log(None, "Starting bot...".to_string());
        }
        let fut = (self.bot_launcher)();
        self.bot_handle = Some(self.rt.spawn(fut));
    }

    pub fn stop_bot(&mut self) {
        if let Some(handle) = self.bot_handle.take() {
            handle.abort();
            let mut d = self.dashboard.lock().unwrap();
            d.service_status = ServiceStatus::Stopped;
            d.push_log(None, "Bot stopped by user.".to_string());
        }
    }

    pub fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        let d = self.dashboard.lock().unwrap();
        let service_status = d.service_status.clone();
        let instance_count = d.instances.len();
        let input_tokens = d.input_tokens;
        let output_tokens = d.output_tokens;
        drop(d);

        ui.horizontal(|ui| {
            let (status_text, status_color) = match &service_status {
                ServiceStatus::Stopped => ("Stopped", egui::Color32::GRAY),
                ServiceStatus::Starting => ("Starting...", egui::Color32::YELLOW),
                ServiceStatus::Running => ("Running", egui::Color32::GREEN),
                ServiceStatus::Error(e) => {
                    ui.label(egui::RichText::new(format!("Error: {e}")).color(egui::Color32::RED));
                    ("Error", egui::Color32::RED)
                }
            };
            ui.label(egui::RichText::new(format!("Service: {status_text}")).color(status_color));

            ui.separator();

            match &service_status {
                ServiceStatus::Stopped | ServiceStatus::Error(_) => {
                    if ui.button("Start").clicked() {
                        self.start_bot();
                    }
                }
                ServiceStatus::Running | ServiceStatus::Starting => {
                    if ui.button("Stop").clicked() {
                        self.stop_bot();
                    }
                }
            }

            ui.separator();
            ui.label(format!("Instances: {instance_count}"));
            ui.separator();
            ui.label(format!("Tokens: {} in / {} out", input_tokens, output_tokens));
        });
    }

    /// Render the instances panel. Returns true if "+ New" was clicked.
    pub fn render_instances_panel(&self, ui: &mut egui::Ui) -> bool {
        let d = self.dashboard.lock().unwrap();
        let instances: Vec<_> = d
            .instances
            .values()
            .map(|s| (s.chat_id, s.message_count, s.last_activity))
            .collect();
        drop(d);

        ui.heading("Instances");
        ui.separator();

        if instances.is_empty() {
            ui.label("No active instances.");
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (chat_id, msg_count, last_activity) in &instances {
                    let ago = last_activity.elapsed().as_secs();
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(format!("Chat {chat_id}")).strong());
                        ui.label(format!("Messages: {msg_count}"));
                        ui.label(format!("Last: {ago}s ago"));
                    });
                }
            });
        }

        ui.add_space(8.0);
        ui.button("+ New Instance").clicked()
    }

    pub fn render_config_panel(&self, ui: &mut egui::Ui) {
        let d = self.dashboard.lock().unwrap();
        let config_token_path = d.config.token_path.clone();
        let config_sandbox = d.config.sandbox_exe.clone();
        let config_model = d.config.model.clone();
        drop(d);

        ui.heading("Configuration");
        ui.separator();

        egui::Grid::new("config_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label("Token file:");
                ui.label(&config_token_path);
                ui.end_row();

                ui.label("Sandbox exe:");
                ui.label(&config_sandbox);
                ui.end_row();

                ui.label("Model:");
                ui.label(if config_model.is_empty() {
                    "(default)"
                } else {
                    &config_model
                });
                ui.end_row();
            });
    }

    pub fn render_event_log(&mut self, ui: &mut egui::Ui) {
        let d = self.dashboard.lock().unwrap();
        let log_entries: Vec<_> = d
            .log
            .iter()
            .map(|e| {
                (
                    e.timestamp.duration_since(self.startup_time).as_secs_f64(),
                    e.chat_id,
                    e.message.clone(),
                )
            })
            .collect();
        drop(d);

        ui.horizontal(|ui| {
            ui.heading("Event Log");
            ui.separator();
            ui.checkbox(&mut self.log_auto_scroll, "Auto-scroll");
            if ui.button("Clear").clicked() {
                let mut d = self.dashboard.lock().unwrap();
                d.log.clear();
            }
        });
        ui.separator();

        let row_height = ui.text_style_height(&egui::TextStyle::Monospace) + 2.0;
        let num_rows = log_entries.len();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(self.log_auto_scroll)
            .show_rows(ui, row_height, num_rows, |ui, row_range| {
                for i in row_range {
                    let (secs, chat_id, msg) = &log_entries[i];
                    let chat_str = chat_id
                        .map(|id| format!("[{id}]"))
                        .unwrap_or_default();
                    ui.label(
                        egui::RichText::new(format!("{secs:>8.1}s {chat_str:>14} {msg}"))
                            .monospace(),
                    );
                }
            });
    }
}
