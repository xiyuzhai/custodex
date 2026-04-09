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
    pub bot_handle: Option<tokio::task::JoinHandle<()>>,
    pub saved_instances: Vec<SavedInstance>,
    pub custodex_dir: PathBuf,
    pub token: String,
    pub sandbox_exe: Option<PathBuf>,
    pub launcher_override: Option<BotLauncher>,
    pub log_filter_chat: Option<i64>,
    pub log_filter_text: String,
}

impl AppState {
    pub fn new(
        rt: tokio::runtime::Runtime,
        dashboard: SharedDashboard,
        custodex_dir: PathBuf,
        token: String,
        sandbox_exe: Option<PathBuf>,
    ) -> Self {
        Self::new_inner(rt, dashboard, custodex_dir, token, sandbox_exe, None)
    }

    pub fn new_with_launcher(
        rt: tokio::runtime::Runtime,
        dashboard: SharedDashboard,
        bot_launcher: BotLauncher,
        custodex_dir: PathBuf,
    ) -> Self {
        Self::new_inner(
            rt,
            dashboard,
            custodex_dir,
            String::new(),
            None,
            Some(bot_launcher),
        )
    }

    fn new_inner(
        rt: tokio::runtime::Runtime,
        dashboard: SharedDashboard,
        custodex_dir: PathBuf,
        token: String,
        sandbox_exe: Option<PathBuf>,
        launcher_override: Option<BotLauncher>,
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
            bot_handle: None,
            saved_instances,
            custodex_dir,
            token,
            sandbox_exe,
            launcher_override,
            log_filter_chat: None,
            log_filter_text: String::new(),
        }
    }

    pub fn start_bot(&mut self) {
        self.start_bot_for_instance(None);
    }

    pub fn start_bot_for_instance(&mut self, selected_instance: Option<usize>) {
        if self.bot_handle.is_some() {
            return;
        }
        let instance = self.instance_to_launch(selected_instance);
        let work_dir = self.work_dir_for_instance(&instance);
        if let Err(e) = std::fs::create_dir_all(&work_dir) {
            let mut d = self.dashboard.lock().unwrap();
            d.service_status = dashboard::ServiceStatus::Error(format!(
                "Failed to create work dir {}: {e}",
                work_dir.display()
            ));
            d.push_log(None, format!("Failed to create work dir {}: {e}", work_dir.display()));
            return;
        }
        {
            let mut d = self.dashboard.lock().unwrap();
            d.service_status = dashboard::ServiceStatus::Starting;
            d.config.model = self.model_for_instance(&instance).to_string();
            d.push_log(
                None,
                format!(
                    "Starting {} with work dir {}...",
                    instance.id,
                    work_dir.display()
                ),
            );
        }
        let fut = if let Some(launcher) = &self.launcher_override {
            launcher()
        } else {
            self.launcher_for_instance(&instance, work_dir)()
        };
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

    fn instance_to_launch(&self, selected_instance: Option<usize>) -> SavedInstance {
        if let Some(idx) = selected_instance.and_then(|idx| self.saved_instances.get(idx)) {
            return idx.clone();
        }
        self.saved_instances
            .first()
            .cloned()
            .expect("expected at least one saved instance")
    }

    fn work_dir_for_instance(&self, instance: &SavedInstance) -> PathBuf {
        match &instance.config {
            TemplateConfig::CodeAgent(cfg) => PathBuf::from(&cfg.work_dir),
            TemplateConfig::AutoApprove(cfg) => PathBuf::from(&cfg.work_dir),
            TemplateConfig::SimpleChat(_) => PathBuf::from(format!(
                ".local/home/{}",
                simple_chat::TEMPLATE_NAME
            )),
        }
    }

    fn model_for_instance<'a>(&self, instance: &'a SavedInstance) -> &'a str {
        match &instance.config {
            TemplateConfig::CodeAgent(cfg) => &cfg.model,
            TemplateConfig::SimpleChat(cfg) => &cfg.model,
            TemplateConfig::AutoApprove(cfg) => &cfg.model,
        }
    }

    fn launcher_for_instance(
        &self,
        instance: &SavedInstance,
        work_dir: PathBuf,
    ) -> dashboard::gui::BotLauncher {
        match &instance.config {
            TemplateConfig::CodeAgent(_) => code_agent::make_launcher(
                self.token.clone(),
                self.dashboard.clone(),
                work_dir,
                self.custodex_dir.clone(),
                self.sandbox_exe.clone(),
            ),
            TemplateConfig::SimpleChat(_) => simple_chat::make_launcher(
                self.token.clone(),
                self.dashboard.clone(),
                work_dir,
                self.custodex_dir.clone(),
                self.sandbox_exe.clone(),
            ),
            TemplateConfig::AutoApprove(_) => auto_approve::make_launcher(
                self.token.clone(),
                self.dashboard.clone(),
                work_dir,
                self.custodex_dir.clone(),
                self.sandbox_exe.clone(),
            ),
        }
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
