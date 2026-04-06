pub mod bot;

use std::path::PathBuf;

use dashboard::SharedDashboard;

pub const TEMPLATE_NAME: &str = "code-agent";

/// Create a bot launcher for the code-agent template.
pub fn make_launcher(
    token: String,
    dashboard: SharedDashboard,
    work_dir: PathBuf,
    custodex_dir: PathBuf,
    sandbox_exe: Option<PathBuf>,
) -> dashboard::gui::BotLauncher {
    Box::new(move || {
        let token = token.clone();
        let dash = dashboard.clone();
        let wd = work_dir.clone();
        let cd = custodex_dir.clone();
        let se = sandbox_exe.clone();
        Box::pin(bot::run_bot(token, dash, wd, cd, se))
    })
}
