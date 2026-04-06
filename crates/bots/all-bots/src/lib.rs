use std::path::Path;

use eframe::egui;
use serde::{Deserialize, Serialize};

/// All available bot templates.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TemplateKind {
    CodeAgent,
    SimpleChat,
    AutoApprove,
}

impl TemplateKind {
    pub const ALL: &[TemplateKind] = &[
        TemplateKind::CodeAgent,
        TemplateKind::SimpleChat,
        TemplateKind::AutoApprove,
    ];

    pub fn name(&self) -> &str {
        match self {
            Self::CodeAgent => "code-agent",
            Self::SimpleChat => "simple-chat",
            Self::AutoApprove => "auto-approve",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::CodeAgent => "Full code agent with tool execution and approval flow",
            Self::SimpleChat => "Text conversation only, no tool execution",
            Self::AutoApprove => "Code agent that auto-approves all tool calls",
        }
    }
}

/// Template-specific config, one variant per template.
#[derive(Clone, Serialize, Deserialize)]
pub enum TemplateConfig {
    CodeAgent(CodeAgentConfig),
    SimpleChat(SimpleChatConfig),
    AutoApprove(AutoApproveConfig),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CodeAgentConfig {
    pub work_dir: String,
    pub model: String,
    pub auto_approve: bool,
}

impl Default for CodeAgentConfig {
    fn default() -> Self {
        Self {
            work_dir: ".local/home/code-agent".to_string(),
            model: String::new(),
            auto_approve: false,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SimpleChatConfig {
    pub model: String,
}

impl Default for SimpleChatConfig {
    fn default() -> Self {
        Self {
            model: String::new(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AutoApproveConfig {
    pub work_dir: String,
    pub model: String,
}

impl Default for AutoApproveConfig {
    fn default() -> Self {
        Self {
            work_dir: ".local/home/auto-approve".to_string(),
            model: String::new(),
        }
    }
}

impl TemplateConfig {
    pub fn new(kind: &TemplateKind) -> Self {
        match kind {
            TemplateKind::CodeAgent => Self::CodeAgent(CodeAgentConfig::default()),
            TemplateKind::SimpleChat => Self::SimpleChat(SimpleChatConfig::default()),
            TemplateKind::AutoApprove => Self::AutoApprove(AutoApproveConfig::default()),
        }
    }

    pub fn step_count(&self) -> usize {
        match self {
            Self::CodeAgent(_) => 2,
            Self::SimpleChat(_) => 1,
            Self::AutoApprove(_) => 1,
        }
    }

    pub fn render_step(&mut self, ui: &mut egui::Ui, step: usize) -> bool {
        match self {
            Self::CodeAgent(cfg) => render_code_agent_step(ui, cfg, step),
            Self::SimpleChat(cfg) => render_simple_chat_step(ui, cfg, step),
            Self::AutoApprove(cfg) => render_auto_approve_step(ui, cfg, step),
        }
    }
}

fn render_code_agent_step(ui: &mut egui::Ui, cfg: &mut CodeAgentConfig, step: usize) -> bool {
    match step {
        0 => {
            ui.heading("Working Directory");
            ui.add_space(8.0);
            ui.label("Path where the agent operates:");
            ui.text_edit_singleline(&mut cfg.work_dir);
            ui.add_space(4.0);
            ui.label("Model (leave empty for default):");
            ui.text_edit_singleline(&mut cfg.model);
            !cfg.work_dir.is_empty()
        }
        1 => {
            ui.heading("Approval Policy");
            ui.add_space(8.0);
            ui.radio_value(&mut cfg.auto_approve, false, "Ask for approval (default)");
            ui.radio_value(&mut cfg.auto_approve, true, "Auto-approve all commands");
            true
        }
        _ => true,
    }
}

fn render_simple_chat_step(ui: &mut egui::Ui, cfg: &mut SimpleChatConfig, step: usize) -> bool {
    match step {
        0 => {
            ui.heading("Model");
            ui.add_space(8.0);
            ui.label("Model (leave empty for default):");
            ui.text_edit_singleline(&mut cfg.model);
            true
        }
        _ => true,
    }
}

fn render_auto_approve_step(
    ui: &mut egui::Ui,
    cfg: &mut AutoApproveConfig,
    step: usize,
) -> bool {
    match step {
        0 => {
            ui.heading("Auto-Approve Agent");
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Warning: this bot auto-approves ALL tool calls.")
                    .color(egui::Color32::YELLOW),
            );
            ui.add_space(8.0);
            ui.label("Working directory:");
            ui.text_edit_singleline(&mut cfg.work_dir);
            ui.add_space(4.0);
            ui.label("Model (leave empty for default):");
            ui.text_edit_singleline(&mut cfg.model);
            !cfg.work_dir.is_empty()
        }
        _ => true,
    }
}

/// Wizard state for creating a new instance.
pub struct WizardState {
    pub active: bool,
    pub selected_template: Option<TemplateKind>,
    pub config: Option<TemplateConfig>,
    pub step: usize,
}

impl Default for WizardState {
    fn default() -> Self {
        Self {
            active: false,
            selected_template: None,
            config: None,
            step: 0,
        }
    }
}

impl WizardState {
    pub fn open(&mut self) {
        self.active = true;
        self.selected_template = None;
        self.config = None;
        self.step = 0;
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn render(&mut self, ctx: &egui::Context) -> Option<TemplateConfig> {
        if !self.active {
            return None;
        }

        let mut result = None;

        egui::Window::new("New Instance")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                if self.selected_template.is_none() {
                    ui.heading("Select Bot Template");
                    ui.add_space(8.0);

                    let mut selected = None;
                    for kind in TemplateKind::ALL {
                        ui.horizontal(|ui| {
                            if ui.button(kind.name()).clicked() {
                                selected = Some(kind.clone());
                            }
                            ui.label(kind.description());
                        });
                    }

                    if let Some(kind) = selected {
                        self.config = Some(TemplateConfig::new(&kind));
                        self.selected_template = Some(kind);
                        self.step = 0;
                    }

                    ui.add_space(8.0);
                    if ui.button("Cancel").clicked() {
                        self.close();
                    }
                } else if let Some(config) = &mut self.config {
                    let total_steps = config.step_count();
                    let valid = config.render_step(ui, self.step);

                    ui.add_space(12.0);
                    ui.horizontal(|ui| {
                        if self.step > 0 && ui.button("← Back").clicked() {
                            self.step -= 1;
                        }
                        if ui.button("Cancel").clicked() {
                            self.close();
                        }
                        if self.step + 1 < total_steps {
                            ui.add_enabled_ui(valid, |ui| {
                                if ui.button("Next →").clicked() {
                                    self.step += 1;
                                }
                            });
                        } else {
                            ui.add_enabled_ui(valid, |ui| {
                                if ui.button("Launch").clicked() {
                                    result = self.config.take();
                                    self.close();
                                }
                            });
                        }
                    });
                }
            });

        result
    }
}

/// A saved instance that can be persisted to .custodex/ and restored on startup.
#[derive(Clone, Serialize, Deserialize)]
pub struct SavedInstance {
    pub id: String,
    pub template: TemplateKind,
    pub config: TemplateConfig,
}

pub fn save_instances(custodex_dir: &Path, instances: &[SavedInstance]) -> std::io::Result<()> {
    let path = custodex_dir.join("instances.json");
    let json = serde_json::to_string_pretty(instances)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

pub fn load_instances(custodex_dir: &Path) -> Vec<SavedInstance> {
    let path = custodex_dir.join("instances.json");
    let Ok(json) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    serde_json::from_str(&json).unwrap_or_default()
}
