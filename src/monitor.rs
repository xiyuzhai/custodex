use std::collections::{HashMap, VecDeque};
use std::time::Instant;

const MAX_LOG_ENTRIES: usize = 2000;

pub type SharedMonitor = std::sync::Arc<std::sync::Mutex<MonitorState>>;

pub fn new_shared_monitor(config: MonitorConfig) -> SharedMonitor {
    std::sync::Arc::new(std::sync::Mutex::new(MonitorState::new(config)))
}

pub struct MonitorState {
    pub log: VecDeque<LogEntry>,
    pub sessions: HashMap<i64, SessionInfo>,
    pub bot_status: BotStatus,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub config: MonitorConfig,
}

impl MonitorState {
    fn new(config: MonitorConfig) -> Self {
        Self {
            log: VecDeque::new(),
            sessions: HashMap::new(),
            bot_status: BotStatus::Stopped,
            input_tokens: 0,
            output_tokens: 0,
            config,
        }
    }

    pub fn push_log(&mut self, chat_id: Option<i64>, message: String) {
        self.log.push_back(LogEntry {
            timestamp: Instant::now(),
            chat_id,
            message,
        });
        while self.log.len() > MAX_LOG_ENTRIES {
            self.log.pop_front();
        }
    }

    pub fn update_session_activity(&mut self, chat_id: i64) {
        let info = self.sessions.entry(chat_id).or_insert(SessionInfo {
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

pub struct SessionInfo {
    pub chat_id: i64,
    pub message_count: u32,
    pub last_activity: Instant,
}

#[derive(Clone, PartialEq)]
pub enum BotStatus {
    Stopped,
    Starting,
    Running,
    Error(String),
}

pub struct MonitorConfig {
    pub token_path: String,
    pub sandbox_exe: String,
    pub model: String,
}
