/// Opaque identifier for a conversation (one `CodexThread` per `CodexConversationId`).
///
/// Channel adapters (Telegram, CLI, etc.) map their own ids into this.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
pub struct CodexConversationId(i64);

impl CodexConversationId {
    pub const fn new(id: i64) -> Self {
        Self(id)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

impl From<i64> for CodexConversationId {
    fn from(v: i64) -> Self {
        Self(v)
    }
}

impl std::fmt::Display for CodexConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
