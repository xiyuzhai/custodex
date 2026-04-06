use std::time::Instant;

use eframe::egui;

use crate::monitor::{BotStatus, SharedMonitor};

pub struct ControlPanel {
    rt: tokio::runtime::Runtime,
    token: String,
    monitor: SharedMonitor,
    bot_handle: Option<tokio::task::JoinHandle<()>>,
    startup_time: Instant,
    log_auto_scroll: bool,
}

impl ControlPanel {
    pub fn new(rt: tokio::runtime::Runtime, token: String, monitor: SharedMonitor) -> Self {
        Self {
            rt,
            token,
            monitor,
            bot_handle: None,
            startup_time: Instant::now(),
            log_auto_scroll: true,
        }
    }

    fn start_bot(&mut self) {
        if self.bot_handle.is_some() {
            return;
        }
        {
            let mut m = self.monitor.lock().unwrap();
            m.bot_status = BotStatus::Starting;
            m.push_log(None, "Starting bot...".to_string());
        }
        let token = self.token.clone();
        let monitor = self.monitor.clone();
        self.bot_handle = Some(self.rt.spawn(crate::bot::run_bot(token, monitor)));
    }

    fn stop_bot(&mut self) {
        if let Some(handle) = self.bot_handle.take() {
            handle.abort();
            let mut m = self.monitor.lock().unwrap();
            m.bot_status = BotStatus::Stopped;
            m.push_log(None, "Bot stopped by user.".to_string());
        }
    }
}

impl eframe::App for ControlPanel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint every 200ms so we see live updates
        ctx.request_repaint_after(std::time::Duration::from_millis(200));

        let m = self.monitor.lock().unwrap();
        let bot_status = m.bot_status.clone();
        let input_tokens = m.input_tokens;
        let output_tokens = m.output_tokens;
        let session_count = m.sessions.len();
        let sessions: Vec<_> = m
            .sessions
            .values()
            .map(|s| (s.chat_id, s.message_count, s.last_activity))
            .collect();
        let log_entries: Vec<_> = m
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
        let config_token_path = m.config.token_path.clone();
        let config_sandbox = m.config.sandbox_exe.clone();
        let config_model = m.config.model.clone();
        drop(m);

        // Top panel: status bar
        egui::TopBottomPanel::top("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let (status_text, status_color) = match &bot_status {
                    BotStatus::Stopped => ("Stopped", egui::Color32::GRAY),
                    BotStatus::Starting => ("Starting...", egui::Color32::YELLOW),
                    BotStatus::Running => ("Running", egui::Color32::GREEN),
                    BotStatus::Error(e) => {
                        ui.label(
                            egui::RichText::new(format!("Error: {e}"))
                                .color(egui::Color32::RED),
                        );
                        ("Error", egui::Color32::RED)
                    }
                };
                ui.label(egui::RichText::new(format!("Bot: {status_text}")).color(status_color));

                ui.separator();

                match &bot_status {
                    BotStatus::Stopped | BotStatus::Error(_) => {
                        if ui.button("Start").clicked() {
                            self.start_bot();
                        }
                    }
                    BotStatus::Running | BotStatus::Starting => {
                        if ui.button("Stop").clicked() {
                            self.stop_bot();
                        }
                    }
                }

                ui.separator();
                ui.label(format!("Sessions: {session_count}"));
                ui.separator();
                ui.label(format!(
                    "Tokens: {} in / {} out",
                    input_tokens, output_tokens
                ));
            });
        });

        // Left panel: sessions
        egui::SidePanel::left("sessions_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Sessions");
                ui.separator();

                if sessions.is_empty() {
                    ui.label("No active sessions.");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (chat_id, msg_count, last_activity) in &sessions {
                            let ago = last_activity.elapsed().as_secs();
                            ui.group(|ui| {
                                ui.label(
                                    egui::RichText::new(format!("Chat {chat_id}")).strong(),
                                );
                                ui.label(format!("Messages: {msg_count}"));
                                ui.label(format!("Last: {ago}s ago"));
                            });
                        }
                    });
                }
            });

        // Right panel: config
        egui::SidePanel::right("config_panel")
            .default_width(250.0)
            .show(ctx, |ui| {
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
            });

        // Central panel: event log
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Event Log");
                ui.separator();
                ui.checkbox(&mut self.log_auto_scroll, "Auto-scroll");
                if ui.button("Clear").clicked() {
                    let mut m = self.monitor.lock().unwrap();
                    m.log.clear();
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
        });
    }
}
