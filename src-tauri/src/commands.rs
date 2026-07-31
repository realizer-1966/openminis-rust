// Tauri IPC 명령 — WebView에서 Rust 코어로 호출되는 함수
// 원본: src-tauri/src/commands.rs
//
// #[tauri::command] 매크로로 WebView JS에서 호출 가능

use std::sync::Arc;
use serde_json::Value;
use tauri::State;

use minis_core::agent_loop::{AgentLoop, AgentLoopConfig, AgentEvent};
use minis_core::tool_dispatch::ToolDispatcher;

/// 에이전트 상태 — Tauri managed state
pub struct AgentState {
    pub dispatcher: Arc<ToolDispatcher>,
    pub agent_loop: AgentLoop,
}

/// 메시지 전송 — WebView에서 invoke("send_message", { message, model, provider })
#[tauri::command]
pub async fn send_message(
    state: State<'_, AgentState>,
    message: String,
    model: String,
    provider_type: String,
    api_key: String,
) -> Result<String, String> {
    // TODO: 프로바이더 생성 → 에이전트 루프 실행
    // 현재는 placeholder
    Ok(format!("[Agent received: {}]", message))
}

/// 세션 목록 조회
#[tauri::command]
pub async fn list_sessions(
) -> Result<Vec<SessionSummary>, String> {
    // TODO: storage crate에서 세션 목록 조회
    Ok(vec![])
}

/// 세션 생성
#[tauri::command]
pub async fn create_session(
    title: String,
    provider_id: String,
    model_id: String,
) -> Result<String, String> {
    use minis_core::session::Session;
    let session = Session::new(&provider_id, &model_id);
    // TODO: storage에 저장
    Ok(session.id)
}

/// 설정 조회
#[tauri::command]
pub async fn get_config(
    path: String,
) -> Result<Value, String> {
    // TODO: config registry에서 조회
    Ok(Value::Null)
}

/// 설정 변경
#[tauri::command]
pub async fn set_config(
    path: String,
    value: Value,
) -> Result<(), String> {
    // TODO: config registry에 저장
    Ok(())
}

/// 셸 명령 실행 (디버그용)
#[tauri::command]
pub async fn run_shell(
    command: String,
) -> Result<String, String> {
    use minis_sandbox::shell::PersistentShell;
    use std::time::Duration;
    let shell = PersistentShell::new();
    let result = shell.execute(&command, Duration::from_secs(30))
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("exit={} stdout={} stderr={}", result.exit_code, result.stdout, result.stderr))
}

/// 등록된 툴 목록
#[tauri::command]
pub async fn list_tools(
    state: State<'_, AgentState>,
) -> Result<Vec<Value>, String> {
    let defs = state.dispatcher.definitions();
    Ok(defs.into_iter().map(|d| serde_json::json!({
        "name": d.name,
        "description": d.description,
    })).collect())
}

#[derive(serde::Serialize)]
pub struct SessionSummary {
    id: String,
    title: String,
}