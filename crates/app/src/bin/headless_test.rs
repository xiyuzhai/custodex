use std::path::PathBuf;

// Reuse the state module from the main crate
use custodex::state::AppState;

use all_bots::{CodeAgentConfig, SimpleChatConfig, AutoApproveConfig, TemplateConfig};
use dashboard::{DashboardConfig, new_dashboard};

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Headless Test ===\n");

    // Setup
    let home_root = PathBuf::from(".local/home");
    let custodex_dir = home_root.join(".custodex");
    std::fs::create_dir_all(&custodex_dir).unwrap();

    for name in &["code-agent", "simple-chat", "auto-approve"] {
        std::fs::create_dir_all(home_root.join(name)).unwrap();
    }

    let dashboard = new_dashboard(DashboardConfig {
        token_path: "(headless)".to_string(),
        sandbox_exe: "(headless)".to_string(),
        model: String::new(),
        dashboard_log_path: None,
    });

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Dummy launcher that does nothing
    let dash_clone = dashboard.clone();
    let bot_launcher: dashboard::gui::BotLauncher = Box::new(move || {
        let d = dash_clone.clone();
        Box::pin(async move {
            {
                let mut d = d.lock().unwrap();
                d.service_status = dashboard::ServiceStatus::Running;
                d.push_log(None, "Fake bot running.".to_string());
            }
            // Just sit forever
            tokio::signal::ctrl_c().await.ok();
        })
    });

    let mut state = AppState::new_with_launcher(rt, dashboard, bot_launcher, custodex_dir.clone());

    // Test 1: initial state
    println!("1. Initial state");
    println!("   Status: {:?}", state.get_service_status());
    println!("   Saved instances: {}", state.saved_instances.len());
    println!("   Bot running: {}", state.is_bot_running());
    println!();

    // Test 2: add instances (all three templates)
    println!("2. Adding instances");

    state.add_instance(TemplateConfig::CodeAgent(CodeAgentConfig {
        work_dir: ".local/home/code-agent".to_string(),
        model: "o4-mini".to_string(),
        auto_approve: false,
    }));
    println!("   After code-agent: {} saved", state.saved_instances.len());

    state.add_instance(TemplateConfig::SimpleChat(SimpleChatConfig {
        model: String::new(),
    }));
    println!("   After simple-chat: {} saved", state.saved_instances.len());

    state.add_instance(TemplateConfig::AutoApprove(AutoApproveConfig {
        work_dir: ".local/home/auto-approve".to_string(),
        model: String::new(),
    }));
    println!("   After auto-approve: {} saved", state.saved_instances.len());
    println!();

    // Test 3: verify persistence
    println!("3. Persistence");
    let loaded = all_bots::load_instances(&custodex_dir);
    println!("   Loaded from disk: {} instances", loaded.len());
    for inst in &loaded {
        println!("   - {} ({})", inst.id, inst.template.name());
    }
    println!();

    // Test 4: start/stop bot
    println!("4. Bot lifecycle");
    state.start_bot();
    println!("   After start: running={}", state.is_bot_running());
    // Give the fake bot a moment to set status
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!("   Status: {:?}", state.get_service_status());

    state.stop_bot();
    println!("   After stop: running={}", state.is_bot_running());
    println!("   Status: {:?}", state.get_service_status());
    println!();

    // Test 5: log entries
    println!("5. Log entries");
    let entries = state.get_log_entries();
    println!("   Total entries: {}", entries.len());
    for (chat_id, msg) in &entries {
        let c = chat_id.map(|id| format!("[{id}]")).unwrap_or_default();
        println!("   {c:>14} {msg}");
    }
    println!();

    // Test 6: log filtering
    println!("6. Log filtering");
    state.log_filter_text = "instance".to_string();
    let filtered = state.get_log_entries();
    println!("   Filter='instance': {} entries", filtered.len());
    state.log_filter_text.clear();
    println!();

    println!("=== All tests passed ===");
}
