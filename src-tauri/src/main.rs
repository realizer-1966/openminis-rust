// OpenMinis Rust — Android 전용 AI 에이전트
// 메인 진입점 (향후 Tauri Mobile host)

use anyhow::Result;
use tracing_subscriber::EnvFilter;

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

fn main() -> Result<()> {
    // 로깅 초기화
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env()
            .add_directive("openminis=info".parse().unwrap()))
        .init();

    tracing::info!("OpenMinis Rust — starting up");

    // 설정 레지스트리 초기화
    let mut config = ConfigRegistry::new();
    register_builtins(&mut config);
    tracing::info!("Registered {} config fields", config.list().len());

    // 툴 디스패처 초기화
    let mut dispatcher = ToolDispatcher::new();
    dispatcher.register(Box::new(FileReadTool));
    dispatcher.register(Box::new(FileWriteTool));
    dispatcher.register(Box::new(FileEditTool));
    dispatcher.register(Box::new(ShellExecuteTool));
    dispatcher.register(Box::new(ReadImageTool));
    dispatcher.register(Box::new(MemoryWriteTool));
    dispatcher.register(Box::new(MemoryGetTool));
    dispatcher.register(Box::new(BrowserUseTool));
    tracing::info!("Registered {} tools", dispatcher.definitions().len());

    // bashism 감지기
    let bashism = BashismDetector::new();
    let test_cmd = "arr=(1 2 3); echo ${arr[@]}";
    let detected = bashism.detect(test_cmd);
    tracing::info!("Bashism detector test: '{}' → {} issues", test_cmd, detected.len());

    // minis:// URL 해석 테스트
    let url = "minis://workspace/report.csv";
    let path = minis_minis_url::UrlResolver::to_linux_path(url)?;
    tracing::info!("URL test: {} → {}", url, path.display());

    tracing::info!("OpenMinis Rust — initialization complete");
    tracing::info!("Ready for Tauri Mobile integration (Phase 10)");

    Ok(())
}