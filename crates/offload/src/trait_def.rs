// OffloadHandler trait — 각 Android 기능이 구현
// 원본: sandbox/offload/ (25개 핸들러)
//
// Rust 쪽: 요청 파싱, 권한 게이트, 응답 포맷팅
// JNI 쪽: 실제 Android API 호출 (ContentResolver, AlarmManager, etc.)

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Offload 실행 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OffloadResult {
    pub success: bool,
    pub data: Value,
    pub error: Option<String>,
    /// exit code (126 = permission denied, 0 = success, 2 = bad args)
    pub exit_code: i32,
}

impl OffloadResult {
    pub fn ok(data: Value) -> Self {
        Self { success: true, data, error: None, exit_code: 0 }
    }

    pub fn error(message: &str, exit_code: i32) -> Self {
        Self {
            success: false,
            data: Value::Null,
            error: Some(message.into()),
            exit_code,
        }
    }

    pub fn permission_denied(display_name: &str) -> Self {
        Self::error(
            &format!("permission_denied: Agent is not allowed to use {}. Open Settings → Permissions to change.", display_name),
            126,
        )
    }

    pub fn bad_args(message: &str) -> Self {
        Self::error(message, 2)
    }
}

/// 권한 상태 — tri-state 게이트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionState {
    /// 에이전트가 자유롭게 사용 가능
    Allowed,
    /// 사용자에게 한 번만 묻기
    AskOnce,
    /// 사용 불가
    NotAllowed,
}

/// OffloadHandler trait — 각 Android 기능이 구현
#[async_trait]
pub trait OffloadHandler: Send + Sync {
    /// 핸들러 이름 (e.g. "calendar", "contacts", "alarm")
    fn name(&self) -> &str;

    /// 사용자에게 표시할 이름
    fn display_name(&self) -> &str;

    /// 요청 실행 — args는 JSON
    /// 실제 Android API 호출은 JNI 콜백으로 위임
    async fn execute(&self, args: Value) -> Result<OffloadResult>;

    /// 권한 확인 — 기본 구현은 Allowed
    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }

    /// 도움말 텍스트
    fn help(&self) -> &str {
        ""
    }
}