// OpenMinis Rust — 로컬 테스트 서버
// 브라우저에서 http://localhost:8765 접속하여 에이전트 기능 테스트

use std::sync::Arc;
use std::path::PathBuf;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use minis_core::tool_dispatch::ToolDispatcher;
use minis_core::tools::file_read::FileReadTool;
use minis_core::tools::file_write::FileWriteTool;
use minis_core::tools::file_edit::FileEditTool;
use minis_core::tools::shell_execute::ShellExecuteTool;
use minis_core::tools::read_image::ReadImageTool;
use minis_core::tools::memory::{MemoryWriteTool, MemoryGetTool};
use minis_core::tools::browser_use::BrowserUseTool;

use minis_sandbox::shell::PersistentShell;
use minis_sandbox::BashismDetector;
use minis_minis_url::UrlResolver;
use minis_minis_url::session_mount::SessionMount;
use minis_config::ConfigRegistry;
use minis_config::builtins::register_builtins;

const HTML: &str = include_str!("../../frontend/index.html");

pub async fn run_server() -> std::io::Result<()> {
    // 툴 디스패처
    let mut dispatcher = ToolDispatcher::new();
    dispatcher.register(Box::new(FileReadTool));
    dispatcher.register(Box::new(FileWriteTool));
    dispatcher.register(Box::new(FileEditTool));
    dispatcher.register(Box::new(ShellExecuteTool::new(Arc::new(PersistentShell::new()))));
    dispatcher.register(Box::new(ReadImageTool));
    dispatcher.register(Box::new(MemoryWriteTool));
    dispatcher.register(Box::new(MemoryGetTool));
    dispatcher.register(Box::new(BrowserUseTool));
    let dispatcher = Arc::new(dispatcher);

    // 세션 마운트
    let session_root = PathBuf::from("/var/minis/workspace/openminis-rust/.sessions");
    std::fs::create_dir_all(&session_root).ok();
    let mount = Arc::new(SessionMount::new(session_root));
    mount.ensure_namespaces().ok();

    // 설정
    let mut config = ConfigRegistry::new();
    register_builtins(&mut config);

    let bashism = Arc::new(BashismDetector::new());

    println!("\n╔══════════════════════════════════════════╗");
    println!("║   OpenMinis Rust — 로컬 테스트 서버      ║");
    println!("╠══════════════════════════════════════════╣");
    println!("║  http://localhost:8765                   ║");
    println!("╚══════════════════════════════════════════╝\n");
    println!("툴: {}개 | bashism 규칙: {}개", dispatcher.definitions().len(), 35);
    println!("Ctrl+C로 종료\n");

    let listener = TcpListener::bind("0.0.0.0:8765").await?;
    println!("서버 시작: http://localhost:8765");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let dispatcher = dispatcher.clone();
        let mount = mount.clone();
        let bashism = bashism.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            let n = socket.read(&mut buf).await.unwrap_or(0);
            if n == 0 { return; }
            let raw = String::from_utf8_lossy(&buf[..n]).to_string();

            // 요청 파싱
            let (method, path, body) = parse_request(&raw);

            let cors = "Access-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n";

            if method == "OPTIONS" {
                let resp = format!("HTTP/1.1 204 No Content\r\n{}\r\n", cors);
                let _ = socket.write_all(resp.as_bytes()).await;
                return;
            }

            let (status, body_str, content_type) = route(&method, &path, &body, &dispatcher, &mount, &bashism).await;

            let response = format!(
                "HTTP/1.1 {}\r\n{}Content-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                status, cors, content_type, body_str.len(), body_str
            );
            let _ = socket.write_all(response.as_bytes()).await;
        });
    }
}

fn parse_request(raw: &str) -> (String, String, String) {
    let mut parts = raw.splitn(3, "\r\n");
    let first_line = parts.next().unwrap_or("GET / HTTP/1.1");
    let rest = parts.next().unwrap_or("");

    let fp: Vec<&str> = first_line.split_whitespace().collect();
    let method = fp.get(0).unwrap_or(&"GET").to_string();
    let path = fp.get(1).unwrap_or(&"/").to_string();

    // body는 \r\n\r\n 이후
    let body = if let Some(idx) = raw.find("\r\n\r\n") {
        raw[idx + 4..].trim_end_matches('\0').to_string()
    } else {
        String::new()
    };

    (method, path, body)
}

async fn route(
    method: &str,
    path: &str,
    body: &str,
    dispatcher: &ToolDispatcher,
    mount: &SessionMount,
    bashism: &BashismDetector,
) -> (String, String, &'static str) {
    match (method, path) {
        ("GET", "/") => ("200 OK".to_string(), HTML.to_string(), "text/html; charset=utf-8"),

        ("GET", "/api/tools") => {
            let tools: Vec<serde_json::Value> = dispatcher.definitions().iter().map(|t| serde_json::json!({
                "name": t.name,
                "description": t.description,
            })).collect();
            ("200 OK".to_string(), serde_json::to_string_pretty(&serde_json::json!({"tools": tools, "count": tools.len()})).unwrap(), "application/json")
        }

        ("GET", "/api/urls") => {
            let urls = mount.list_minis_urls();
            ("200 OK".to_string(), serde_json::to_string_pretty(&serde_json::json!({"urls": urls, "count": urls.len()})).unwrap(), "application/json")
        }

        ("POST", "/api/bashism") => {
            if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(body) {
                let command = cmd["command"].as_str().unwrap_or("");
                let detected = bashism.detect(command);
                let results: Vec<serde_json::Value> = detected.iter().map(|r| serde_json::json!({
                    "name": r.name,
                    "fix_hint": r.fix_hint,
                })).collect();
                ("200 OK".to_string(), serde_json::to_string(&serde_json::json!({"command": command, "has_bashisms": !results.is_empty(), "rules": results})).unwrap(), "application/json")
            } else {
                ("400 Bad Request".to_string(), r#"{"error":"invalid JSON"}"#.into(), "application/json")
            }
        }

        ("POST", "/api/shell") => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
                let command = v["command"].as_str().unwrap_or("");
                let shell = PersistentShell::new();
                match shell.execute(command, Duration::from_secs(10)).await {
                    Ok(result) => ("200 OK".to_string(), serde_json::json!({
                        "exit_code": result.exit_code,
                        "stdout": result.stdout,
                        "stderr": result.stderr,
                        "timed_out": result.timed_out,
                        "new_urls": result.new_minis_urls,
                    }).to_string(), "application/json"),
                    Err(e) => ("500 Internal".to_string(), serde_json::json!({"error": e.to_string()}).to_string(), "application/json"),
                }
            } else {
                ("400 Bad Request".to_string(), r#"{"error":"invalid JSON"}"#.into(), "application/json")
            }
        }

        ("POST", "/api/url") => {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(body) {
                let url = v["url"].as_str().unwrap_or("");
                match UrlResolver::to_linux_path(url) {
                    Ok(p) => ("200 OK".to_string(), serde_json::json!({"url": url, "linux_path": p.to_string_lossy().to_string()}).to_string(), "application/json"),
                    Err(e) => ("400 Bad Request".to_string(), serde_json::json!({"error": e.to_string()}).to_string(), "application/json"),
                }
            } else {
                ("400 Bad Request".to_string(), r#"{"error":"invalid JSON"}"#.into(), "application/json")
            }
        }

        ("GET", "/api/status") => {
            ("200 OK".to_string(), serde_json::json!({
                "status": "running",
                "tools_count": dispatcher.definitions().len(),
                "minis_root": "/var/minis/",
            }).to_string(), "application/json")
        }

        _ => ("404 Not Found".to_string(), r#"{"error":"not found"}"#.into(), "application/json"),
    }
}