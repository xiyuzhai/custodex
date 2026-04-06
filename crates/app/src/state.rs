use std::path::PathBuf;

use all_bots::{
    AutoApproveConfig, CodeAgentConfig, SavedInstance, SimpleChatConfig, TemplateConfig,
    TemplateKind, load_instances, save_instances,
};
use dashboard::gui::BotLauncher;
use dashboard::SharedDashboard;

pub struct AppState {
    pub rt: tokio::runtime::Runtime,
    pub dashboard: SharedDashboard,
    pub bot_launcher: BotLauncher,
    pub bot_handle: Option<tokio::task::JoinHandle<()>>,
    pub saved_instances: Vec<SavedInstance>,
    pub custodex_dir: PathBuf,
    pub log_filter_chat: Option<i64>,
    pub log_filter_text: String,
}

impl AppState {
    pub fn new(
        rt: tokio::runtime::Runtime,
        dashboard: SharedDashboard,
        bot_launcher: BotLauncher,
        custodex_dir: PathBuf,
    ) -> Self {
        let mut saved_instances = load_instances(&custodex_dir);
        if saved_instances.is_empty() {
            // Seed with one instance per template on first run
            saved_instances = seed_default_instances();
            let _ = save_instances(&custodex_dir, &saved_instances);
            let mut d = dashboard.lock().unwrap();
            d.push_log(
                None,
                format!("Created {} default instance(s).", saved_instances.len()),
            );
        } else {
            let mut d = dashboard.lock().unwrap();
            d.push_log(
                None,
                format!("Restored {} saved instance(s).", saved_instances.len()),
            );
        }
        Self {
            rt,
            dashboard,
            bot_launcher,
            bot_handle: None,
            saved_instances,
            custodex_dir,
            log_filter_chat: None,
            log_filter_text: String::new(),
        }
    }

    pub fn start_bot(&mut self) {
        if self.bot_handle.is_some() {
            return;
        }
        {
            let mut d = self.dashboard.lock().unwrap();
            d.service_status = dashboard::ServiceStatus::Starting;
            d.push_log(None, "Starting bot...".to_string());
        }
        let fut = (self.bot_launcher)();
        self.bot_handle = Some(self.rt.spawn(fut));
    }

    pub fn stop_bot(&mut self) {
        if let Some(handle) = self.bot_handle.take() {
            handle.abort();
            let mut d = self.dashboard.lock().unwrap();
            d.service_status = dashboard::ServiceStatus::Stopped;
            d.push_log(None, "Bot stopped by user.".to_string());
        }
    }

    pub fn is_bot_running(&self) -> bool {
        self.bot_handle.is_some()
    }

    pub fn add_instance(&mut self, config: TemplateConfig) {
        let template = match &config {
            TemplateConfig::CodeAgent(_) => TemplateKind::CodeAgent,
            TemplateConfig::SimpleChat(_) => TemplateKind::SimpleChat,
            TemplateConfig::AutoApprove(_) => TemplateKind::AutoApprove,
        };
        let id = format!("{}-{}", template.name(), self.saved_instances.len());
        let saved = SavedInstance {
            id: id.clone(),
            template,
            config,
        };
        self.saved_instances.push(saved);

        if let Err(e) = save_instances(&self.custodex_dir, &self.saved_instances) {
            let mut d = self.dashboard.lock().unwrap();
            d.push_log(None, format!("Failed to save instances: {e}"));
        } else {
            let mut d = self.dashboard.lock().unwrap();
            d.push_log(None, format!("Instance {id} created and saved."));
        }
    }

    pub fn set_log_filter_chat(&mut self, chat_id: Option<i64>) {
        self.log_filter_chat = chat_id;
    }

    pub fn get_service_status(&self) -> dashboard::ServiceStatus {
        self.dashboard.lock().unwrap().service_status.clone()
    }

    pub fn get_instance_count(&self) -> usize {
        self.dashboard.lock().unwrap().instances.len()
    }

    pub fn get_token_usage(&self) -> (u64, u64) {
        let d = self.dashboard.lock().unwrap();
        (d.input_tokens, d.output_tokens)
    }

    pub fn get_log_entries(&self) -> Vec<(Option<i64>, String)> {
        let d = self.dashboard.lock().unwrap();
        d.log
            .iter()
            .filter(|e| {
                if let Some(fc) = self.log_filter_chat {
                    if e.chat_id != Some(fc) {
                        return false;
                    }
                }
                if !self.log_filter_text.is_empty()
                    && !e.message.contains(self.log_filter_text.as_str())
                {
                    return false;
                }
                true
            })
            .map(|e| (e.chat_id, e.message.clone()))
            .collect()
    }
}

fn seed_default_instances() -> Vec<SavedInstance> {
    vec![
        SavedInstance {
            id: "code-agent-0".to_string(),
            template: TemplateKind::CodeAgent,
            config: TemplateConfig::CodeAgent(CodeAgentConfig::default()),
        },
        SavedInstance {
            id: "simple-chat-0".to_string(),
            template: TemplateKind::SimpleChat,
            config: TemplateConfig::SimpleChat(SimpleChatConfig::default()),
        },
        SavedInstance {
            id: "auto-approve-0".to_string(),
            template: TemplateKind::AutoApprove,
            config: TemplateConfig::AutoApprove(AutoApproveConfig::default()),
        },
    ]
}
