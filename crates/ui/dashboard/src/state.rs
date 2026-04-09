use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use std::time::Instant;

const MAX_LOG_ENTRIES: usize = 2000;

pub type SharedDashboard = std::sync::Arc<std::sync::Mutex<Dashboard>>;

pub fn new_dashboard(config: DashboardConfig) -> SharedDashboard {
    std::sync::Arc::new(std::sync::Mutex::new(Dashboard::new(config)))
}

pub struct Dashboard {
    pub log: VecDeque<LogEntry>,
    pub instances: HashMap<i64, InstanceInfo>,
    pub service_status: ServiceStatus,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub config: DashboardConfig,
}

impl Dashboard {
    fn new(config: DashboardConfig) -> Self {
        Self {
            log: VecDeque::new(),
            instances: HashMap::new(),
            service_status: ServiceStatus::Stopped,
            input_tokens: 0,
            output_tokens: 0,
            config,
        }
    }

    pub fn push_log(&mut self, chat_id: Option<i64>, message: String) {
        if let Some(path) = &self.config.dashboard_log_path {
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                let chat = chat_id.map(|id| format!("[{id}] ")).unwrap_or_default();
                let _ = writeln!(file, "{chat}{message}");
            }
        }
        self.log.push_back(LogEntry {
            timestamp: Instant::now(),
            chat_id,
            message,
        });
        while self.log.len() > MAX_LOG_ENTRIES {
            self.log.pop_front();
        }
    }

    pub fn update_instance_activity(&mut self, chat_id: i64) {
        let info = self.instances.entry(chat_id).or_insert(InstanceInfo {
            chat_id,
            message_count: 0,
            last_activity: Instant::now(),
        });
        info.message_count += 1;
        info.last_activity = Instant::now();
    }
}

pub struct LogEntry {
    pub timestamp: Instant,
    pub chat_id: Option<i64>,
    pub message: String,
}

pub struct InstanceInfo {
    pub chat_id: i64,
    pub message_count: u32,
    pub last_activity: Instant,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

pub struct DashboardConfig {
    pub token_path: String,
    pub sandbox_exe: String,
    pub model: String,
    pub dashboard_log_path: Option<PathBuf>,
}
