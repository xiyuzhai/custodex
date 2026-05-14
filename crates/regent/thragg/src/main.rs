use std::io::{self, Read, Write};

use anyhow::{Context, Result};
use thragg::{Decision, StopHookInput};

#[tokio::main]
async fn main() -> Result<()> {
    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .context("read stop hook input from stdin")?;

    let input: StopHookInput =
        serde_json::from_str(&buf).context("parse stop hook input as JSON")?;

    // Re-entrancy guard: if the regent's own injected reply triggered another
    // stop, do not loop — let it through.
    let decision = if input.stop_hook_active {
        Decision::AllowStop
    } else {
        decide(&input).await?
    };

    let out = decision.into_stop_hook_output();
    let json = serde_json::to_string(&out)?;
    let mut stdout = io::stdout().lock();
    stdout.write_all(json.as_bytes())?;
    stdout.write_all(b"\n")?;
    Ok(())
}

/// Ask a codex chat to judge whether the agent should stop here.
///
/// TODO: wire codex-bridge. For now, allow the stop so the binary is
/// runnable end-to-end and the JSON contract is exercised.
async fn decide(_input: &StopHookInput) -> Result<Decision> {
    Ok(Decision::AllowStop)
}
