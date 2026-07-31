// OpenMinis Rust — Android 전용 AI 에이전트
// 메인 진입점 (Tauri Mobile host)

mod android;
mod commands;

use std::sync::Arc;
use anyhow::Result;
use tracing_subscriber::EnvFilter;

use minis_core::agent_loop::{AgentLoop, AgentLoopConfig};
use minis_core::tool_dispatch::ToolDispatcher;
use minis_core::tools::file_read::FileReadTool;
use minis_core::tools::file_write::FileWriteTool;
use minis_core::tools::file_edit::FileEditTool;
use minis_core::tools::shell_execute::ShellExecuteTool;
use minis_core::tools::read_image::ReadImageTool;
use minis_core::tools::memory::{MemoryWriteTool, MemoryGetTool};
use minis_core::tools::browser_use::BrowserUseTool;

use minis_config::ConfigRegistry;
use minis_config::builtins::register_builtins;

use minis_sandbox::BashismDetector;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("openminis=info".parse().unwrap()))
        .init();

    tracing::info!("OpenMinis Rust — starting up");

    // 설정 레지스트리
    let mut config = ConfigRegistry::new();
    register_builtins(&mut config);
    tracing::info!("Registered {} config fields", config.list().len());

    // 툴 디스패처
    let mut dispatcher = ToolDispatcher::new();
    dispatcher.register(Box::new(FileReadTool));
    dispatcher.register(Box::new(FileWriteTool));
    dispatcher.register(Box::new(FileEditTool));
    dispatcher.register(Box::new(ShellExecuteTool::default()));
    dispatcher.register(Box::new(ReadImageTool));
    dispatcher.register(Box::new(MemoryWriteTool));
    dispatcher.register(Box::new(MemoryGetTool));
    dispatcher.register(Box::new(BrowserUseTool));
    let dispatcher = Arc::new(dispatcher);
    tracing::info!("Registered {} tools", dispatcher.definitions().len());

    // bashism 감지
    let bashism = BashismDetector::new();
    let test_cmd = "arr=(1 2 3); echo ${arr[@]}";
    let detected = bashism.detect(test_cmd);
    tracing::info!("Bashism detector: '{}' → {} issues", test_cmd, detected.len());

    // minis:// URL 해석
    let url = "minis://workspace/report.csv";
    let path = minis_minis_url::UrlResolver::to_linux_path(url)?;
    tracing::info!("URL test: {} → {}", url, path.display());

    // 에이전트 루프
    let agent_loop = AgentLoop::new(AgentLoopConfig::default(), dispatcher.clone());
    let tools = agent_loop.dispatcher().llm_definitions();
    tracing::info!("Agent loop ready with {} tool definitions", tools.len());

    // TODO: Tauri 앱 초기화
    // tauri::Builder::default()
    //     .manage(commands::AgentState { dispatcher, agent_loop })
    //     .invoke_handler(tauri::generate_handler![
    //         commands::send_message,
    //         commands::list_sessions,
    //         commands::create_session,
    //         commands::get_config,
    //         commands::set_config,
    //         commands::run_shell,
    //         commands::list_tools,
    //     ])
    //     .run(tauri::generate_context!())
    //     .expect("error while running tauri application");

    tracing::info!("OpenMinis Rust — initialization complete");
    tracing::info!("Ready for Tauri Mobile integration");
    tracing::info!("Use with: ANTHROPIC_API_KEY=... cargo run");

    Ok(())
}