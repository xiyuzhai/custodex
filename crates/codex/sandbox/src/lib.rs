use std::path::PathBuf;

/// Path to the codex-linux-sandbox binary, built from the pinned codex fork
/// at compile time.
pub fn sandbox_exe() -> PathBuf {
    PathBuf::from(env!("CODEX_SANDBOX_EXE"))
}
