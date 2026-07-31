// 알림 offload — 시스템 알림 전송, 조회, 삭제
// 원본: NotificationOffloadHandler.kt

use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use crate::trait_def::{OffloadHandler, OffloadResult, PermissionState};

pub struct NotificationHandler {
    jni_callback: Option<Box<dyn Fn(&str, &Value) -> Result<Value> + Send + Sync>>,
}

impl NotificationHandler {
    pub fn new() -> Self { Self { jni_callback: None } }
    pub fn with_jni<F>(callback: F) -> Self
    where F: Fn(&str, &Value) -> Result<Value> + Send + Sync + 'static {
        Self { jni_callback: Some(Box::new(callback)) }
    }

    fn call_jni(&self, sub: &str, args: &Value) -> Result<Value> {
        if let Some(cb) = &self.jni_callback { cb(sub, args) }
        else { Ok(json!({"error": "jni_callback not configured"})) }
    }
}

const HELP: &str = r#"android-notification — send, list, clear system notifications.

Usage:
  android-notification send --title T [--body B]
  android-notification clear
  android-notification list [--max N]
"#;

#[async_trait]
impl OffloadHandler for NotificationHandler {
    fn name(&self) -> &str { "notification" }
    fn display_name(&self) -> &str { "android-notification" }
    fn help(&self) -> &str { HELP }

    async fn execute(&self, args: Value) -> Result<OffloadResult> {
        let sub = args.get("subcommand")
            .and_then(|v| v.as_str())
            .unwrap_or("list");

        if sub == "send" {
            if args.get("title").and_then(|v| v.as_str()).is_none() {
                return Ok(OffloadResult::bad_args("send: --title is required"));
            }
        }

        let result = self.call_jni(sub, &args)?;
        if result.get("error").is_some() {
            Ok(OffloadResult { success: false, data: result.clone(), error: result.get("error").and_then(|v| v.as_str()).map(|s| s.to_string()), exit_code: 1 })
        } else {
            Ok(OffloadResult::ok(result))
        }
    }

    fn check_permission(&self, _session_id: Option<&str>) -> PermissionState {
        PermissionState::Allowed
    }
}

impl Default for NotificationHandler { fn default() -> Self { Self::new() } }