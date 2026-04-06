mod instance;

pub use instance::BotInstance;
pub use instance::InstanceManager;
pub use instance::InstanceManagerConfig;

// Re-export codex types that downstream crates need.
pub use codex_core::CodexThread;
pub use codex_protocol::protocol::{EventMsg, Op, ReviewDecision};
pub use codex_protocol::user_input::UserInput;
