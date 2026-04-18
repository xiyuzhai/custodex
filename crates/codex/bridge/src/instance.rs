use std::path::PathBuf;
use std::sync::Arc;

use codex_core::config::{Config, ConfigOverrides};
use codex_core::CodexThread;
use codex_core::NewThread;
use codex_core::ThreadManager;
use codex_exec_server::EnvironmentManager;
use codex_login::AuthCredentialsStoreMode;
use codex_login::AuthManager;
use codex_models_manager::collaboration_mode_presets::CollaborationModesConfig;
use codex_protocol::protocol::{Op, SessionSource};
use codex_protocol::user_input::UserInput;
use dashmap::DashMap;
use tokio::sync::Mutex;

use crate::conversation_id::CodexConversationId;

/// Configuration for creating a CodexConversationManager.
pub struct CodexConversationManagerConfig {
    /// Working directory for bot instances.
    pub work_dir: PathBuf,
    /// Path to the codex-linux-sandbox binary.
    pub sandbox_exe: Option<PathBuf>,
    /// Persistent state directory (.custodex/).
    pub custodex_dir: PathBuf,
}

/// A single codex instance bound to a chat. Wraps a CodexThread.
pub struct CodexConversation {
    pub thread: Arc<CodexThread>,
}

/// Manages bot instances (one per chat). Wraps codex ThreadManager.
pub struct CodexConversationManager {
    thread_mgr: Arc<ThreadManager>,
    instances: DashMap<CodexConversationId, Arc<Mutex<CodexConversation>>>,
    work_dir: PathBuf,
    sandbox_exe: Option<PathBuf>,
}

impl CodexConversationManager {
    pub async fn new(im_config: CodexConversationManagerConfig) -> Self {
        let sandbox_exe = im_config.sandbox_exe.clone();
        let overrides = ConfigOverrides {
            codex_linux_sandbox_exe: sandbox_exe.clone(),
            cwd: Some(im_config.work_dir.clone()),
            ..Default::default()
        };
        let config = Config::load_with_cli_overrides_and_harness_overrides(vec![], overrides)
            .await
            .expect("failed to load codex config");

        let auth_manager = AuthManager::shared(
            config.codex_home.clone(),
            /*enable_codex_api_key_env*/ true,
            AuthCredentialsStoreMode::Auto,
        );

        let environment_manager = Arc::new(EnvironmentManager::new(/*exec_server_url*/ None));

        let thread_mgr = Arc::new(ThreadManager::new(
            &config,
            auth_manager,
            SessionSource::Custom("telegram".to_string()),
            CollaborationModesConfig {
                default_mode_request_user_input: false,
            },
            environment_manager,
        ));

        Self {
            thread_mgr,
            instances: DashMap::new(),
            work_dir: im_config.work_dir,
            sandbox_exe,
        }
    }

    /// The working directory for this manager's instances.
    pub fn work_dir(&self) -> &PathBuf {
        &self.work_dir
    }

    /// Get or create a codex instance for the given conversation.
    pub async fn get_or_create(&self, id: CodexConversationId) -> Arc<Mutex<CodexConversation>> {
        if let Some(inst) = self.instances.get(&id) {
            return inst.clone();
        }

        let thread_config = Config::load_with_cli_overrides_and_harness_overrides(
            vec![],
            ConfigOverrides {
                codex_linux_sandbox_exe: self.sandbox_exe.clone(),
                cwd: Some(self.work_dir.clone()),
                ..Default::default()
            },
        )
            .await
            .expect("failed to load config for new thread");

        let NewThread {
            thread_id: _,
            thread,
            session_configured: _,
        } = self
            .thread_mgr
            .start_thread(thread_config)
            .await
            .expect("failed to start codex thread");

        let inst = Arc::new(Mutex::new(CodexConversation { thread }));
        self.instances.insert(id, Arc::clone(&inst));
        inst
    }

    /// Submit a text message to the instance for a given conversation.
    pub async fn submit_text(&self, id: CodexConversationId, text: &str) -> Arc<CodexThread> {
        let inst = self.get_or_create(id).await;
        let thread = {
            let i = inst.lock().await;
            Arc::clone(&i.thread)
        };

        thread
            .submit(Op::UserInput {
                items: vec![UserInput::Text {
                    text: text.to_string(),
                    text_elements: vec![],
                }],
                final_output_json_schema: None,
            })
            .await
            .expect("failed to submit user input");

        thread
    }

    /// Submit an approval decision.
    pub async fn submit_approval(
        &self,
        id: CodexConversationId,
        op: Op,
    ) -> Result<(), codex_protocol::error::CodexErr> {
        let inst = self.get_or_create(id).await;
        let thread = {
            let i = inst.lock().await;
            Arc::clone(&i.thread)
        };
        thread.submit(op).await.map(|_| ())
    }
}
